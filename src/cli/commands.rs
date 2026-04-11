use anyhow::{bail, Result};

use crate::{
    error::PetcliError,
    state::{
        party::Party,
        pet::{Pet, PetStatus},
        pokedex::POKEMON,
    },
};

/// Set which Pokemon is displayed (by name or Pokedex number).
/// Replaces the entire party with one Pokemon.
pub fn set_pokemon(party: &mut Party, input: &str, shiny: bool) -> Result<()> {
    let species_id = resolve_species(input)?;
    let name = pokemon_name(species_id);

    // Clear all slots and put this one in slot 0
    party.slots = Default::default();
    party.slots[0] = Some(Pet::new_alive(species_id, shiny));
    party.active = 0;

    println!(
        "Set to {}{} — run `petcli` to watch it walk!",
        name,
        if shiny { " ✦ (shiny)" } else { "" }
    );
    Ok(())
}

/// Show all Pokemon currently in the party with their slot numbers.
pub fn status(party: &Party) {
    let alive: Vec<_> = party.slots.iter().enumerate()
        .filter_map(|(i, s)| s.as_ref().map(|p| (i, p)))
        .filter(|(_, p)| matches!(p.status, PetStatus::Alive))
        .collect();

    if alive.is_empty() {
        println!("No Pokemon walking. Run: petcli set <name>  or  petcli pokemon");
        return;
    }

    println!("Party ({}/6):", alive.len());
    for (slot_idx, (_, pet)) in alive.iter().enumerate() {
        let shiny = if pet.shiny { " ✦" } else { "" };
        println!(
            "  {}. {}{}",
            slot_idx + 1,
            pet.display_name(),
            shiny,
        );
    }
    println!();
    println!("Use `petcli remove <n>` to remove by number.");
}

/// Add a Pokemon to the next free slot (up to 6 total).
pub fn add_pokemon(party: &mut Party, input: &str, shiny: bool) -> Result<()> {
    let species_id = resolve_species(input)?;
    let name = pokemon_name(species_id);

    party.add_pet(Pet::new_alive(species_id, shiny)).map_err(|e| match e {
        PetcliError::PartyFull => anyhow::anyhow!(
            "Already have 6 Pokemon walking. Use `petcli set <name>` to replace, or `petcli remove <n>` to free a slot."
        ),
        e => e.into(),
    })?;

    println!("Added {}{}!", name, if shiny { " ✦ (shiny)" } else { "" });
    Ok(())
}

/// Remove a Pokemon by its displayed party number (1-based, as shown in status).
pub fn remove_pokemon(party: &mut Party, number: usize) -> Result<()> {
    // Collect alive slots in display order (same order as `status`)
    let alive_slots: Vec<usize> = party.slots.iter().enumerate()
        .filter_map(|(i, s)| s.as_ref().map(|p| (i, p)))
        .filter(|(_, p)| matches!(p.status, PetStatus::Alive))
        .map(|(i, _)| i)
        .collect();

    if number == 0 || number > alive_slots.len() {
        bail!(
            "Invalid number {}. You have {} Pokemon (use `petcli status` to see them).",
            number,
            alive_slots.len()
        );
    }

    let slot_idx = alive_slots[number - 1];
    let name = party.slots[slot_idx]
        .as_ref()
        .map(|p| p.display_name())
        .unwrap_or_default();

    party.slots[slot_idx] = None;

    // Keep active pointing at a valid slot
    if party.active == slot_idx {
        party.active = party.slots.iter().position(|s| s.is_some()).unwrap_or(0);
    }

    println!("Removed {}.", name);
    Ok(())
}

/// Wipe the entire party — removes all Pokemon from the saved state.
pub fn reset_party(party: &mut Party) {
    party.slots = Default::default();
    party.active = 0;
    println!("Party cleared. All Pokemon removed from your terminal.");
    println!("Run `petcli pokemon` to add new ones.");
}

/// Print all 493 Pokemon in a compact table.
pub fn list_pokemon() {
    let gen_ranges = [
        ("Gen 1 (Kanto)",   1u16..=151u16),
        ("Gen 2 (Johto)",   152..=251),
        ("Gen 3 (Hoenn)",   252..=386),
        ("Gen 4 (Sinnoh)",  387..=493),
    ];

    for (label, range) in &gen_ranges {
        println!("── {} ─────────────────────────────", label);
        const COLS: usize = 4;
        let entries: Vec<(u16, &str)> = POKEMON.iter()
            .filter(|(id, _)| range.contains(id))
            .map(|(id, name)| (*id, *name))
            .collect();

        for row in entries.chunks(COLS) {
            let cells: Vec<String> = row.iter()
                .map(|(id, name)| format!("{id:>3}. {name:<12}"))
                .collect();
            println!("  {}", cells.join("  "));
        }
        println!();
    }

    println!("Usage:");
    println!("  petcli pokemon          — open TUI picker with live preview");
    println!("  petcli add <name/num>   — add directly by name or number");
}

// ── Internal ──────────────────────────────────────────────────────────────────

fn pokemon_name(species_id: u16) -> &'static str {
    POKEMON.iter()
        .find(|(id, _)| *id == species_id)
        .map(|(_, name)| *name)
        .unwrap_or("Unknown")
}

pub fn resolve_species(input: &str) -> Result<u16> {
    let trimmed = input.trim();

    // Try as a Pokedex number first
    if let Ok(n) = trimmed.parse::<u16>() {
        if POKEMON.iter().any(|(id, _)| *id == n) {
            return Ok(n);
        }
        bail!("Pokedex number {} not found (valid range: 1–493).", n);
    }

    // Try by name (case-insensitive, also match normalized forms)
    let query = trimmed.to_lowercase();
    for &(id, name) in POKEMON {
        if name.to_lowercase() == query {
            return Ok(id);
        }
    }

    // Partial match fallback
    let matches: Vec<_> = POKEMON.iter()
        .filter(|(_, name)| name.to_lowercase().starts_with(&query))
        .collect();

    match matches.len() {
        1 => Ok(matches[0].0),
        0 => bail!(
            "Unknown Pokemon {:?}. Run `petcli pokemon` to browse all 493.",
            input
        ),
        _ => {
            let names: Vec<_> = matches.iter().take(5).map(|(_, n)| *n).collect();
            bail!(
                "Ambiguous name {:?} — did you mean: {}{}?",
                input,
                names.join(", "),
                if matches.len() > 5 { ", ..." } else { "" }
            )
        }
    }
}
