use anyhow::Result;
use rand::Rng;

use crate::{
    error::PetcliError,
    sim::interactions::{self, SimEvent},
    state::{
        party::Party,
        persist,
        pet::{Pet, PetStatus},
        species::{self, GEN1_RANGE},
    },
};

pub fn catch(party: &mut Party, force_shiny: bool, shiny_rate: u32) -> Result<()> {
    let mut rng = rand::thread_rng();
    let species_id: u16 = rng.gen_range(GEN1_RANGE);
    let shiny = force_shiny || rng.gen_ratio(1, shiny_rate.max(1));

    let species = species::get(species_id);
    let egg_cycles = species.map(|s| s.egg_cycles as u32 * 5).unwrap_or(20);
    let name = species::name(species_id);

    let pet = Pet::new_egg(species_id, shiny, egg_cycles);
    let slot = party.add_pet(pet)?;

    println!(
        "A wild egg appeared! It contains {} (slot {}){}",
        name,
        slot + 1,
        if shiny { " ✦ Shiny!" } else { "" }
    );
    println!("Interact with it to hatch!");

    persist::save(party)?;
    Ok(())
}

pub fn status(party: &Party) {
    if party.count() == 0 {
        println!("Your party is empty. Use `petcli catch` to get a Pokemon!");
        return;
    }

    println!("── Party ─────────────────────────────────────────");
    for (i, slot) in party.slots.iter().enumerate() {
        let Some(pet) = slot else { continue };
        let active = if i == party.active { "▶ " } else { "  " };
        let shiny = if pet.shiny { " ✦" } else { "" };
        let status = match &pet.status {
            PetStatus::Egg { hatch_steps } => format!("Egg ({} steps left)", hatch_steps),
            PetStatus::Alive => "Alive".to_string(),
            PetStatus::Fainted => "Fainted — needs medicine!".to_string(),
        };
        println!(
            "{}[{}] {}{} Lv.{} | {} | Hunger: {:.0} | Happy: {:.0} | Energy: {:.0}",
            active,
            i + 1,
            pet.display_name(),
            shiny,
            pet.level,
            status,
            pet.stats.hunger,
            pet.stats.happiness,
            pet.stats.energy,
        );
    }
    println!("Wallet: {} coins", party.wallet);
}

pub fn feed(party: &mut Party) -> Result<()> {
    match interactions::feed(party) {
        Ok(ev) => {
            let name = party.active_pet().map(|p| p.display_name()).unwrap_or_default();
            println!("{} munches happily!", name);
            print_sim_event(ev);
        }
        Err(PetcliError::NoPet) => println!("No active Pokemon! Use `petcli catch` first."),
        Err(e) => return Err(e.into()),
    }
    persist::save(party)?;
    Ok(())
}

pub fn pet_cmd(party: &mut Party) -> Result<()> {
    match interactions::pet_pokemon(party) {
        Ok(ev) => {
            let name = party.active_pet().map(|p| p.display_name()).unwrap_or_default();
            println!("{} loves the attention!", name);
            print_sim_event(ev);
        }
        Err(PetcliError::NoPet) => println!("No active Pokemon! Use `petcli catch` first."),
        Err(e) => return Err(e.into()),
    }
    persist::save(party)?;
    Ok(())
}

pub fn list_party(party: &Party) {
    status(party);
}

pub fn inventory(party: &Party) {
    println!("── Wallet: {} coins ─────────────────────", party.wallet);
    println!("── Inventory ────────────────────────────");
    let total: u32 = party.inventory.stacks.iter().map(|s| s.qty).sum();
    if total == 0 {
        println!("  (empty) — open the TUI and press [s] to shop");
        return;
    }
    for stack in &party.inventory.stacks {
        if stack.qty > 0 {
            println!("  {:>2}x  {}", stack.qty, stack.kind.display_name());
        }
    }
}

fn print_sim_event(ev: SimEvent) {
    match ev {
        SimEvent::None => {}
        SimEvent::Hatched { species_name } => {
            println!("🎉 Your egg hatched into {}!", species_name);
        }
        SimEvent::Evolved { from, to } => {
            println!("✨ {} evolved into {}!", from, to);
        }
        SimEvent::LevelUp { level } => {
            println!("⬆  Level up! Now Lv.{}!", level);
        }
    }
}
