use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pet {
    pub id: Uuid,
    pub species_id: u16,
    pub nickname: Option<String>,
    pub shiny: bool,
    pub stats: PetStats,
    pub exp: u32,
    pub level: u8,
    pub status: PetStatus,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetStats {
    /// 0.0 (starving) – 100.0 (full)
    pub hunger: f32,
    /// 0.0 – 100.0
    pub happiness: f32,
    /// 0.0 (asleep) – 100.0 (fully rested)
    pub energy: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PetStatus {
    Egg { hatch_steps: u32 },
    Alive,
    /// happiness hit 0 — needs medicine
    Fainted,
}

impl Pet {
    pub fn new_egg(species_id: u16, shiny: bool, hatch_steps: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            species_id,
            nickname: None,
            shiny,
            stats: PetStats {
                hunger: 80.0,
                happiness: 70.0,
                energy: 100.0,
            },
            exp: 0,
            level: 1,
            status: PetStatus::Egg { hatch_steps },
            last_updated: Utc::now(),
        }
    }

    pub fn display_name(&self) -> String {
        self.nickname
            .clone()
            .unwrap_or_else(|| crate::state::species::name(self.species_id))
    }

    pub fn is_alive(&self) -> bool {
        matches!(self.status, PetStatus::Alive)
    }

    pub fn is_egg(&self) -> bool {
        matches!(self.status, PetStatus::Egg { .. })
    }
}

impl PetStats {
    pub fn clamp(&mut self) {
        self.hunger = self.hunger.clamp(0.0, 100.0);
        self.happiness = self.happiness.clamp(0.0, 100.0);
        self.energy = self.energy.clamp(0.0, 100.0);
    }
}
