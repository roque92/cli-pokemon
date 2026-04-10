use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemStack {
    pub kind: ItemKind,
    pub qty: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub stacks: Vec<ItemStack>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            stacks: vec![
                ItemStack { kind: ItemKind::Food(FoodKind::OranBerry), qty: 3 },
                ItemStack { kind: ItemKind::Toy(ToyKind::Ball), qty: 1 },
            ],
        }
    }

    pub fn has(&self, kind: &ItemKind) -> bool {
        self.stacks.iter().any(|s| &s.kind == kind && s.qty > 0)
    }

    pub fn consume(&mut self, kind: &ItemKind) -> bool {
        if let Some(stack) = self.stacks.iter_mut().find(|s| &s.kind == kind) {
            if stack.qty > 0 {
                stack.qty -= 1;
                return true;
            }
        }
        false
    }

    pub fn add(&mut self, kind: ItemKind, qty: u32) {
        if let Some(stack) = self.stacks.iter_mut().find(|s| s.kind == kind) {
            stack.qty += qty;
        } else {
            self.stacks.push(ItemStack { kind, qty });
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemKind {
    Food(FoodKind),
    Toy(ToyKind),
    Medicine(MedKind),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FoodKind {
    OranBerry,
    SitrusBerry,
    PokePuff,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToyKind {
    Ball,
    Ribbon,
    Mirror,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MedKind {
    Potion,
    FullRestore,
    Revive,
}

pub struct StatDelta {
    pub hunger: f32,
    pub happiness: f32,
    pub energy: f32,
}

impl ItemKind {
    pub fn stat_delta(&self) -> StatDelta {
        match self {
            ItemKind::Food(FoodKind::OranBerry) => StatDelta { hunger: 20.0, happiness: 5.0, energy: 0.0 },
            ItemKind::Food(FoodKind::SitrusBerry) => StatDelta { hunger: 35.0, happiness: 10.0, energy: 0.0 },
            ItemKind::Food(FoodKind::PokePuff) => StatDelta { hunger: 15.0, happiness: 25.0, energy: 5.0 },
            ItemKind::Toy(ToyKind::Ball) => StatDelta { hunger: 0.0, happiness: 15.0, energy: -10.0 },
            ItemKind::Toy(ToyKind::Ribbon) => StatDelta { hunger: 0.0, happiness: 20.0, energy: -5.0 },
            ItemKind::Toy(ToyKind::Mirror) => StatDelta { hunger: 0.0, happiness: 10.0, energy: -3.0 },
            ItemKind::Medicine(MedKind::Potion) => StatDelta { hunger: 0.0, happiness: 15.0, energy: 20.0 },
            ItemKind::Medicine(MedKind::FullRestore) => StatDelta { hunger: 50.0, happiness: 50.0, energy: 100.0 },
            ItemKind::Medicine(MedKind::Revive) => StatDelta { hunger: 30.0, happiness: 30.0, energy: 50.0 },
        }
    }

    pub fn shop_price(&self) -> u32 {
        match self {
            ItemKind::Food(FoodKind::OranBerry) => 50,
            ItemKind::Food(FoodKind::SitrusBerry) => 100,
            ItemKind::Food(FoodKind::PokePuff) => 150,
            ItemKind::Toy(ToyKind::Ball) => 80,
            ItemKind::Toy(ToyKind::Ribbon) => 120,
            ItemKind::Toy(ToyKind::Mirror) => 90,
            ItemKind::Medicine(MedKind::Potion) => 200,
            ItemKind::Medicine(MedKind::FullRestore) => 500,
            ItemKind::Medicine(MedKind::Revive) => 350,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ItemKind::Food(FoodKind::OranBerry) => "Oran Berry",
            ItemKind::Food(FoodKind::SitrusBerry) => "Sitrus Berry",
            ItemKind::Food(FoodKind::PokePuff) => "Poke Puff",
            ItemKind::Toy(ToyKind::Ball) => "Poke Ball",
            ItemKind::Toy(ToyKind::Ribbon) => "Ribbon",
            ItemKind::Toy(ToyKind::Mirror) => "Mirror",
            ItemKind::Medicine(MedKind::Potion) => "Potion",
            ItemKind::Medicine(MedKind::FullRestore) => "Full Restore",
            ItemKind::Medicine(MedKind::Revive) => "Revive",
        }
    }
}
