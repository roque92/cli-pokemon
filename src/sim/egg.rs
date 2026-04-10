use crate::state::pet::{Pet, PetStatus};

/// Called on each player interaction (feed, pet).
/// Returns true if the egg just hatched.
pub fn step(pet: &mut Pet) -> bool {
    if let PetStatus::Egg { ref mut hatch_steps } = pet.status {
        if *hatch_steps > 0 {
            *hatch_steps -= 1;
        }
        if *hatch_steps == 0 {
            pet.status = PetStatus::Alive;
            return true;
        }
    }
    false
}
