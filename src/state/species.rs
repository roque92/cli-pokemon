use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

static SPECIES_DB: OnceLock<HashMap<u16, Species>> = OnceLock::new();

const SPECIES_JSON: &str = include_str!("../../assets/data/species.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Species {
    pub id: u16,
    pub name: String,
    pub egg_cycles: u16,
    pub base_exp: u32,
    pub evolution: Option<EvolutionCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum EvolutionCondition {
    Level { into_id: u16, min_level: u8 },
    Happiness { into_id: u16, min_happiness: f32 },
}

pub fn db() -> &'static HashMap<u16, Species> {
    SPECIES_DB.get_or_init(|| {
        let list: Vec<Species> = serde_json::from_str(SPECIES_JSON)
            .expect("species.json is bundled at compile time and must be valid");
        list.into_iter().map(|s| (s.id, s)).collect()
    })
}

pub fn name(species_id: u16) -> String {
    db()
        .get(&species_id)
        .map(|s| s.name.clone())
        .unwrap_or_else(|| format!("#{}", species_id))
}

pub fn get(species_id: u16) -> Option<&'static Species> {
    db().get(&species_id)
}

/// Gen 1 Pokedex range for random egg generation
pub const GEN1_RANGE: std::ops::RangeInclusive<u16> = 1..=151;
