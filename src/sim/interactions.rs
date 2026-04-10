use crate::state::{
    item::ItemKind,
    party::Party,
    pet::PetStatus,
};
use crate::error::PetcliError;

#[derive(Debug)]
pub enum SimEvent {
    None,
    Hatched { species_name: String },
    Evolved { from: String, to: String },
    LevelUp { level: u8 },
}

const EXP_PER_LEVEL: u32 = 100;

pub fn feed(party: &mut Party) -> Result<SimEvent, PetcliError> {
    let pet = party.active_pet_mut().ok_or(PetcliError::NoPet)?;

    if let PetStatus::Egg { .. } = &pet.status {
        let hatched = super::egg::step(pet);
        if hatched {
            let name = crate::state::species::name(pet.species_id);
            return Ok(SimEvent::Hatched { species_name: name });
        }
        return Ok(SimEvent::None);
    }

    pet.stats.hunger = (pet.stats.hunger + 20.0).min(100.0);
    pet.stats.happiness = (pet.stats.happiness + 3.0).min(100.0);
    pet.stats.clamp();
    party.wallet += 2;

    Ok(check_level_and_evolution(party))
}

pub fn pet_pokemon(party: &mut Party) -> Result<SimEvent, PetcliError> {
    let pet = party.active_pet_mut().ok_or(PetcliError::NoPet)?;

    if let PetStatus::Egg { .. } = &pet.status {
        let hatched = super::egg::step(pet);
        if hatched {
            let name = crate::state::species::name(pet.species_id);
            return Ok(SimEvent::Hatched { species_name: name });
        }
        return Ok(SimEvent::None);
    }

    pet.stats.happiness = (pet.stats.happiness + 15.0).min(100.0);
    pet.exp += 10;
    pet.stats.clamp();
    party.wallet += 5;

    Ok(check_level_and_evolution(party))
}

pub fn use_item(party: &mut Party, kind: &ItemKind) -> Result<SimEvent, PetcliError> {
    if !party.inventory.has(kind) {
        return Err(PetcliError::NoPet); // reuse as "item not available"
    }
    party.inventory.consume(kind);

    let delta = kind.stat_delta();
    let pet = party.active_pet_mut().ok_or(PetcliError::NoPet)?;
    pet.stats.hunger = (pet.stats.hunger + delta.hunger).min(100.0);
    pet.stats.happiness = (pet.stats.happiness + delta.happiness).min(100.0);
    pet.stats.energy = (pet.stats.energy + delta.energy).min(100.0);
    pet.stats.clamp();

    // Revive fixes fainted
    if matches!(kind, ItemKind::Medicine(_)) {
        if matches!(pet.status, PetStatus::Fainted) {
            pet.status = PetStatus::Alive;
        }
    }

    Ok(check_level_and_evolution(party))
}

/// After any stat change: check for level-up then evolution.
fn check_level_and_evolution(party: &mut Party) -> SimEvent {
    let pet = match party.active_pet_mut() {
        Some(p) if p.is_alive() => p,
        _ => return SimEvent::None,
    };

    // Level up
    let threshold = pet.level as u32 * EXP_PER_LEVEL;
    if pet.exp >= threshold && pet.level < 100 {
        pet.level += 1;
        let new_level = pet.level;

        // Check evolution after level-up
        if let Some(into_id) = super::evolution::check(pet) {
            let from = pet.display_name();
            let to = super::evolution::apply(pet, into_id);
            return SimEvent::Evolved { from, to };
        }

        return SimEvent::LevelUp { level: new_level };
    }

    // Check happiness-based evolution (no level-up needed)
    if let Some(into_id) = super::evolution::check(pet) {
        let from = pet.display_name();
        let to = super::evolution::apply(pet, into_id);
        return SimEvent::Evolved { from, to };
    }

    SimEvent::None
}
