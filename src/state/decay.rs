use chrono::Utc;

use super::pet::{Pet, PetStatus};

/// Stat loss rates per real-time hour
const HUNGER_DECAY_PER_HOUR: f32 = 12.0;
const HAPPINESS_DECAY_PER_HOUR: f32 = 8.0;
const ENERGY_DECAY_PER_HOUR: f32 = 5.0;

/// Energy recovers while sleeping (when energy < 20)
const ENERGY_SLEEP_RECOVERY_PER_HOUR: f32 = 30.0;

/// Apply time-based decay to a pet based on elapsed real time since last_updated.
/// Called on load and periodically during the game loop.
pub fn apply(pet: &mut Pet) {
    if !pet.is_alive() {
        pet.last_updated = Utc::now();
        return;
    }

    let now = Utc::now();
    let elapsed_secs = (now - pet.last_updated).num_seconds().max(0) as f32;
    let elapsed_hours = elapsed_secs / 3600.0;

    pet.stats.hunger -= HUNGER_DECAY_PER_HOUR * elapsed_hours;
    pet.stats.happiness -= HAPPINESS_DECAY_PER_HOUR * elapsed_hours;

    if pet.stats.energy < 20.0 {
        // Sleeping: energy recovers
        pet.stats.energy += ENERGY_SLEEP_RECOVERY_PER_HOUR * elapsed_hours;
    } else {
        pet.stats.energy -= ENERGY_DECAY_PER_HOUR * elapsed_hours;
    }

    pet.stats.clamp();

    // Faint if happiness hits 0
    if pet.stats.happiness <= 0.0 {
        pet.status = PetStatus::Fainted;
    }

    pet.last_updated = now;
}

/// Apply a single tick's worth of decay (called every game loop tick).
pub fn apply_tick(pet: &mut Pet, delta_ms: u32) {
    if !pet.is_alive() {
        return;
    }

    let elapsed_hours = delta_ms as f32 / 3_600_000.0;

    pet.stats.hunger -= HUNGER_DECAY_PER_HOUR * elapsed_hours;
    pet.stats.happiness -= HAPPINESS_DECAY_PER_HOUR * elapsed_hours;

    if pet.stats.energy < 20.0 {
        pet.stats.energy += ENERGY_SLEEP_RECOVERY_PER_HOUR * elapsed_hours;
    } else {
        pet.stats.energy -= ENERGY_DECAY_PER_HOUR * elapsed_hours;
    }

    pet.stats.clamp();

    if pet.stats.happiness <= 0.0 {
        pet.status = PetStatus::Fainted;
    }
}
