use rand::Rng;

use crate::state::pet::PetStats;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimState {
    Idle,
    WalkLeft,
    WalkRight,
    Sleep,
    Eat,
    HappyBounce,
    Hatch,
}

/// Events that can force an animation state change
#[derive(Debug, Clone, Copy)]
pub enum AnimEvent {
    Feed,
    Pet,
    Hatch,
    WakeUp,
}

impl AnimState {
    /// Whether this state can be interrupted by an event or natural transition
    pub fn is_interruptible(&self) -> bool {
        matches!(self, AnimState::Idle | AnimState::WalkLeft | AnimState::WalkRight | AnimState::Sleep)
    }

    /// Pick the next state naturally (called when current state completes its cycle)
    pub fn next_natural(current: AnimState, stats: &PetStats, rng: &mut impl Rng) -> AnimState {
        match current {
            AnimState::Eat => AnimState::Idle,
            AnimState::HappyBounce => AnimState::Idle,
            AnimState::Hatch => AnimState::Idle,
            AnimState::Sleep => {
                if stats.energy >= 60.0 {
                    AnimState::Idle
                } else {
                    AnimState::Sleep
                }
            }
            _ => {
                // Low energy → sleep
                if stats.energy < 20.0 {
                    return AnimState::Sleep;
                }
                // Low hunger → mostly idle (too hungry to play)
                if stats.hunger < 15.0 {
                    return if rng.gen_bool(0.8) { AnimState::Idle } else { AnimState::WalkLeft };
                }
                // Normal: weighted random
                let roll: f32 = rng.gen();
                if roll < 0.35 {
                    AnimState::Idle
                } else if roll < 0.65 {
                    AnimState::WalkLeft
                } else {
                    AnimState::WalkRight
                }
            }
        }
    }

    /// Force a state transition from an external event
    pub fn from_event(event: AnimEvent) -> AnimState {
        match event {
            AnimEvent::Feed => AnimState::Eat,
            AnimEvent::Pet => AnimState::HappyBounce,
            AnimEvent::Hatch => AnimState::Hatch,
            AnimEvent::WakeUp => AnimState::Idle,
        }
    }
}
