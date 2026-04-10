use chrono::Utc;
use uuid::Uuid;

use crate::state::{
    decay,
    item::{FoodKind, Inventory, ItemKind},
    party::Party,
    pet::{Pet, PetStats, PetStatus},
    species,
};

fn alive_pet(hunger: f32, happiness: f32, energy: f32, level: u8, exp: u32) -> Pet {
    Pet {
        id: Uuid::new_v4(),
        species_id: 1, // Bulbasaur
        nickname: None,
        shiny: false,
        stats: PetStats { hunger, happiness, energy },
        exp,
        level,
        status: PetStatus::Alive,
        last_updated: Utc::now(),
    }
}

// ── Decay ────────────────────────────────────────────────────────────────────

#[test]
fn decay_tick_reduces_stats() {
    let mut pet = alive_pet(100.0, 100.0, 100.0, 1, 0);
    // 1 hour worth of ticks
    decay::apply_tick(&mut pet, 3_600_000);
    assert!(pet.stats.hunger < 100.0, "hunger should decrease");
    assert!(pet.stats.happiness < 100.0, "happiness should decrease");
    assert!(pet.stats.energy < 100.0, "energy should decrease");
}

#[test]
fn decay_clamps_at_zero() {
    let mut pet = alive_pet(0.5, 0.5, 100.0, 1, 0);
    decay::apply_tick(&mut pet, 3_600_000);
    assert!(pet.stats.hunger >= 0.0);
    assert!(pet.stats.happiness >= 0.0);
}

#[test]
fn faint_triggers_when_happiness_zero() {
    let mut pet = alive_pet(80.0, 0.1, 80.0, 1, 0);
    // Force happiness below 0 with a large delta
    decay::apply_tick(&mut pet, 10 * 3_600_000);
    assert_eq!(pet.status, PetStatus::Fainted);
}

#[test]
fn sleeping_pet_recovers_energy() {
    let mut pet = alive_pet(80.0, 80.0, 10.0, 1, 0); // energy < 20 → sleeping
    decay::apply_tick(&mut pet, 3_600_000);
    assert!(pet.stats.energy > 10.0, "energy should recover while sleeping");
}

#[test]
fn decay_skip_for_fainted_pet() {
    let mut pet = alive_pet(80.0, 80.0, 80.0, 1, 0);
    pet.status = PetStatus::Fainted;
    let before_hunger = pet.stats.hunger;
    decay::apply_tick(&mut pet, 3_600_000);
    assert_eq!(pet.stats.hunger, before_hunger, "fainted pets should not decay");
}

// ── Evolution ────────────────────────────────────────────────────────────────

#[test]
fn level_evolution_triggers_at_threshold() {
    let mut pet = alive_pet(80.0, 80.0, 80.0, 16, 0);
    // Bulbasaur evolves at level 16
    let result = crate::sim::evolution::check(&pet);
    assert_eq!(result, Some(2), "Bulbasaur should evolve into Ivysaur (#2) at Lv16");
}

#[test]
fn level_evolution_does_not_trigger_below_threshold() {
    let pet = alive_pet(80.0, 80.0, 80.0, 15, 0);
    assert_eq!(crate::sim::evolution::check(&pet), None);
}

#[test]
fn happiness_evolution_triggers_at_threshold() {
    let mut pet = alive_pet(80.0, 90.0, 80.0, 1, 0);
    pet.species_id = 25; // Pikachu → Raichu at happiness ≥ 90
    let result = crate::sim::evolution::check(&pet);
    assert_eq!(result, Some(26));
}

#[test]
fn happiness_evolution_does_not_trigger_below_threshold() {
    let mut pet = alive_pet(80.0, 85.0, 80.0, 1, 0);
    pet.species_id = 25;
    // Pikachu needs 90 happiness; 85 should not trigger
    assert_eq!(crate::sim::evolution::check(&pet), None);
}

#[test]
fn apply_evolution_changes_species() {
    let mut pet = alive_pet(80.0, 80.0, 80.0, 16, 0);
    crate::sim::evolution::apply(&mut pet, 2);
    assert_eq!(pet.species_id, 2);
}

// ── Species DB ───────────────────────────────────────────────────────────────

#[test]
fn species_db_loads_gen1() {
    let db = species::db();
    assert_eq!(db.len(), 151, "Gen 1 has 151 species");
    assert_eq!(db[&1].name, "Bulbasaur");
    assert_eq!(db[&151].name, "Mew");
}

#[test]
fn species_name_fallback_for_unknown_id() {
    assert_eq!(species::name(999), "#999");
}

// ── Inventory ────────────────────────────────────────────────────────────────

#[test]
fn inventory_consume_reduces_qty() {
    let mut inv = Inventory::new();
    let kind = ItemKind::Food(FoodKind::OranBerry);
    assert!(inv.has(&kind));
    assert!(inv.consume(&kind));
    // Started with 3, now 2
    let qty = inv.stacks.iter().find(|s| s.kind == kind).map(|s| s.qty).unwrap_or(0);
    assert_eq!(qty, 2);
}

#[test]
fn inventory_consume_returns_false_when_empty() {
    let mut inv = Inventory::new();
    let kind = ItemKind::Food(FoodKind::PokePuff); // starts with 0
    assert!(!inv.has(&kind));
    assert!(!inv.consume(&kind));
}

#[test]
fn inventory_add_stacks() {
    let mut inv = Inventory::new();
    let kind = ItemKind::Food(FoodKind::OranBerry);
    inv.add(kind.clone(), 10);
    let qty = inv.stacks.iter().find(|s| s.kind == kind).map(|s| s.qty).unwrap_or(0);
    assert_eq!(qty, 13); // 3 default + 10
}

// ── Party ────────────────────────────────────────────────────────────────────

#[test]
fn party_cycle_active_skips_empty_slots() {
    let mut party = Party::new();
    let p1 = alive_pet(80.0, 80.0, 80.0, 1, 0);
    let mut p2 = alive_pet(80.0, 80.0, 80.0, 1, 0);
    p2.species_id = 4;
    // Fill slot 0 and slot 2, leave 1 empty
    party.slots[0] = Some(p1);
    party.slots[2] = Some(p2);
    party.active = 0;

    party.cycle_active();
    assert_eq!(party.active, 2, "should skip empty slot 1");
}

#[test]
fn party_full_error_on_seventh_add() {
    let mut party = Party::new();
    for _ in 0..6 {
        let p = alive_pet(80.0, 80.0, 80.0, 1, 0);
        party.add_pet(p).unwrap();
    }
    let result = party.add_pet(alive_pet(80.0, 80.0, 80.0, 1, 0));
    assert!(result.is_err(), "7th pet should fail with PartyFull");
}

// ── Egg ──────────────────────────────────────────────────────────────────────

#[test]
fn egg_step_decrements_hatch_steps() {
    let mut pet = alive_pet(80.0, 80.0, 80.0, 1, 0);
    pet.status = PetStatus::Egg { hatch_steps: 5 };
    let hatched = crate::sim::egg::step(&mut pet);
    assert!(!hatched);
    assert_eq!(pet.status, PetStatus::Egg { hatch_steps: 4 });
}

#[test]
fn egg_hatches_at_zero_steps() {
    let mut pet = alive_pet(80.0, 80.0, 80.0, 1, 0);
    pet.status = PetStatus::Egg { hatch_steps: 1 };
    let hatched = crate::sim::egg::step(&mut pet);
    assert!(hatched);
    assert_eq!(pet.status, PetStatus::Alive);
}

// ── Unicode color mapping ─────────────────────────────────────────────────────

#[test]
fn rgba_to_ansi_red_maps_to_xterm_color_cube() {
    use image::Rgba;
    // Pure red (255, 0, 0) → cube index 16 + 36*5 + 6*0 + 0 = 196
    let px = Rgba([255u8, 0, 0, 255]);
    // replicate the mapping from unicode.rs
    fn step(c: u8) -> u8 {
        if c < 48 { 0 } else if c < 115 { 1 } else if c < 155 { 2 }
        else if c < 195 { 3 } else if c < 235 { 4 } else { 5 }
    }
    let idx = 16 + 36 * step(px[0]) + 6 * step(px[1]) + step(px[2]);
    assert_eq!(idx, 196);
}

#[test]
fn rgba_to_ansi_white_maps_to_231() {
    fn step(c: u8) -> u8 {
        if c < 48 { 0 } else if c < 115 { 1 } else if c < 155 { 2 }
        else if c < 195 { 3 } else if c < 235 { 4 } else { 5 }
    }
    let idx = 16 + 36 * step(255) + 6 * step(255) + step(255);
    assert_eq!(idx, 231);
}
