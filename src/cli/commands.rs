use anyhow::{bail, Result};

use crate::{
    error::PetcliError,
    state::{
        party::Party,
        pet::{Pet, PetStatus},
        species,
    },
};

/// Set which Pokemon is displayed (by name or Pokedex number).
pub fn set_pokemon(party: &mut Party, input: &str, shiny: bool) -> Result<()> {
    let species_id = resolve_species(input)?;
    let name = species::name(species_id);

    // Replace active slot (or first slot if empty)
    let slot = if party.slots.iter().any(|s| s.is_none()) || party.count() == 0 {
        party.active
    } else {
        party.active
    };

    party.slots[slot] = Some(Pet::new_alive(species_id, shiny));
    party.active = slot;

    println!(
        "Set to {}{} — run `petcli` to watch it walk!",
        name,
        if shiny { " ✦ (shiny)" } else { "" }
    );
    Ok(())
}

pub fn status(party: &Party) {
    let Some(pet) = party.active_pet() else {
        println!("No Pokemon set. Run: petcli set <name-or-number>");
        return;
    };

    let shiny = if pet.shiny { " ✦" } else { "" };
    let state = match &pet.status {
        PetStatus::Alive  => "alive".to_string(),
        PetStatus::Egg { .. } => "egg".to_string(),
        PetStatus::Fainted    => "fainted".to_string(),
    };
    println!(
        "{}{} — {}",
        pet.display_name(),
        shiny,
        state
    );
}

/// Add a Pokemon to the next free slot (up to 6 total).
pub fn add_pokemon(party: &mut Party, input: &str, shiny: bool) -> Result<()> {
    let species_id = resolve_species(input)?;
    let name = species::name(species_id);

    party.add_pet(Pet::new_alive(species_id, shiny)).map_err(|e| match e {
        PetcliError::PartyFull => anyhow::anyhow!(
            "Already have 6 Pokemon walking. Use `petcli set <name>` to replace one."
        ),
        e => e.into(),
    })?;

    println!(
        "Added {}{} — restart with `petcli` to see them all!",
        name,
        if shiny { " ✦ (shiny)" } else { "" }
    );
    Ok(())
}

/// Print all Gen 1 Pokemon in a compact table.
pub fn list_pokemon() {
    println!("── Available Pokemon (Gen 1) ────────────────────────────────");
    const COLS: usize = 3;
    let names: Vec<(u16, String)> = (1u16..=151)
        .map(|id| (id, species::name(id)))
        .collect();

    for row in names.chunks(COLS) {
        let cells: Vec<String> = row
            .iter()
            .map(|(id, name)| format!("{id:>3}. {name:<13}"))
            .collect();
        println!("  {}", cells.join("  "));
    }
    println!();
    println!("Use: petcli set <name-or-number>   — replace current pet");
    println!("     petcli add <name-or-number>   — add alongside current pets");
}

fn resolve_species(input: &str) -> Result<u16> {
    // Try as a Pokedex number first
    if let Ok(n) = input.trim().parse::<u16>() {
        if n >= 1 && n <= 151 {
            return Ok(n);
        }
        bail!("Pokedex number must be 1–151 (Gen 1 only for now)");
    }

    // Try by name (case-insensitive)
    let query = input.trim().to_lowercase();
    for id in 1u16..=151 {
        if species::name(id).to_lowercase() == query {
            return Ok(id);
        }
    }

    bail!(
        "Unknown Pokemon {:?}. Use a name (e.g. \"pikachu\") or number (1–151).",
        input
    )
}
