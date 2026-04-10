use rand::Rng;

use crate::sprite::sheet::SpriteSheet;
use crate::state::pet::PetStats;

use super::state_machine::{AnimEvent, AnimState};

pub struct AnimTicker {
    pub current_state: AnimState,
    pub frame_index: usize,
    /// Accumulated ms within the current frame
    elapsed_ms: u32,
    /// How many full frame cycles have played in this state
    state_cycles: u32,
    /// Cycles required before natural transition is eligible
    min_cycles: u32,
    pending_event: Option<AnimEvent>,
}

impl AnimTicker {
    pub fn new() -> Self {
        Self {
            current_state: AnimState::Idle,
            frame_index: 0,
            elapsed_ms: 0,
            state_cycles: 0,
            min_cycles: 1,
            pending_event: None,
        }
    }

    pub fn push_event(&mut self, event: AnimEvent) {
        if self.current_state.is_interruptible() {
            self.pending_event = Some(event);
        }
    }

    /// Advance the animation by delta_ms milliseconds.
    /// Returns the current frame index into the sheet.
    pub fn tick(
        &mut self,
        delta_ms: u32,
        sheet: &SpriteSheet,
        stats: &PetStats,
        rng: &mut impl Rng,
    ) -> usize {
        // Handle pending event
        if let Some(event) = self.pending_event.take() {
            self.transition_to(AnimState::from_event(event), 1);
        }

        let frames = sheet.idle_frames();
        if frames.is_empty() {
            return 0;
        }

        self.elapsed_ms += delta_ms;
        let frame_delay = frames[self.frame_index].delay_ms;

        if self.elapsed_ms >= frame_delay {
            self.elapsed_ms -= frame_delay;
            self.frame_index += 1;

            if self.frame_index >= frames.len() {
                self.frame_index = 0;
                self.state_cycles += 1;

                // Natural transition after min_cycles completed
                if self.state_cycles >= self.min_cycles {
                    let next = AnimState::next_natural(self.current_state, stats, rng);
                    if next != self.current_state {
                        self.transition_to(next, self.default_min_cycles(next));
                    } else {
                        self.state_cycles = 0;
                    }
                }
            }
        }

        self.frame_index.min(frames.len().saturating_sub(1))
    }

    fn transition_to(&mut self, state: AnimState, min_cycles: u32) {
        self.current_state = state;
        self.frame_index = 0;
        self.elapsed_ms = 0;
        self.state_cycles = 0;
        self.min_cycles = min_cycles;
    }

    fn default_min_cycles(&self, state: AnimState) -> u32 {
        match state {
            AnimState::Eat => 2,
            AnimState::HappyBounce => 2,
            AnimState::Hatch => 3,
            AnimState::Sleep => 4,
            _ => 1,
        }
    }
}
