use crate::state::{
    pet::{Pet, PetStatus},
    species::{self, EvolutionCondition},
};

/// Check if a pet meets its evolution condition.
/// Returns the new species_id if evolution should trigger, None otherwise.
pub fn check(pet: &Pet) -> Option<u16> {
    if !matches!(pet.status, PetStatus::Alive) {
        return None;
    }

    let species = species::get(pet.species_id)?;
    let condition = species.evolution.as_ref()?;

    match condition {
        EvolutionCondition::Level { into_id, min_level } => {
            if pet.level >= *min_level {
                return Some(*into_id);
            }
        }
        EvolutionCondition::Happiness { into_id, min_happiness } => {
            if pet.stats.happiness >= *min_happiness {
                return Some(*into_id);
            }
        }
    }

    None
}

/// Apply evolution: update species_id, reset relevant tracking, keep stats.
/// Returns the new species name for display.
pub fn apply(pet: &mut Pet, into_id: u16) -> String {
    pet.species_id = into_id;
    // Boost happiness on evolution
    pet.stats.happiness = (pet.stats.happiness + 10.0).min(100.0);
    species::name(into_id)
}
