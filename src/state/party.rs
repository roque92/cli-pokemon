use serde::{Deserialize, Serialize};

use super::{item::Inventory, pet::Pet};
use crate::error::PetcliError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Party {
    pub slots: [Option<Pet>; 6],
    /// Index of the currently displayed pet (0–5)
    pub active: usize,
    pub inventory: Inventory,
    pub wallet: u32,
}

impl Party {
    pub fn new() -> Self {
        Self {
            slots: Default::default(),
            active: 0,
            inventory: Inventory::new(),
            wallet: 200,
        }
    }

    pub fn active_pet(&self) -> Option<&Pet> {
        self.slots[self.active].as_ref()
    }

    pub fn active_pet_mut(&mut self) -> Option<&mut Pet> {
        self.slots[self.active].as_mut()
    }

    pub fn add_pet(&mut self, pet: Pet) -> Result<usize, PetcliError> {
        for (i, slot) in self.slots.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(pet);
                return Ok(i);
            }
        }
        Err(PetcliError::PartyFull)
    }

    pub fn count(&self) -> usize {
        self.slots.iter().filter(|s| s.is_some()).count()
    }

    pub fn cycle_active(&mut self) {
        let count = self.slots.len();
        for i in 1..=count {
            let next = (self.active + i) % count;
            if self.slots[next].is_some() {
                self.active = next;
                return;
            }
        }
    }
}
