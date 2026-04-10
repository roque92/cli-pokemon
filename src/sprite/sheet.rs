use std::collections::HashMap;

use crate::animation::state_machine::AnimState;

use super::gif::DecodedFrame;

#[derive(Debug)]
pub struct SpriteSheet {
    pub species_id: u16,
    pub shiny: bool,
    /// Frames keyed by animation state. At minimum, Idle must be populated.
    frames: HashMap<AnimState, Vec<DecodedFrame>>,
}

impl SpriteSheet {
    /// Build a sheet from raw decoded frames. For Phase 1, all states share
    /// the same frames (the animated GIF contains idle + walk together).
    pub fn from_frames(species_id: u16, shiny: bool, raw_frames: Vec<DecodedFrame>) -> Self {
        let mut frames = HashMap::new();

        // Gen 5 animated GIFs contain a walking cycle. We split them:
        // - Frames 0..N/2 → Idle
        // - Frames N/2..N → WalkLeft / WalkRight (same frames, direction flipped by renderer)
        // For Phase 1 we just assign all frames to all locomotion states.
        let all: Vec<DecodedFrame> = raw_frames.clone();
        frames.insert(AnimState::Idle, all.clone());
        frames.insert(AnimState::WalkLeft, all.clone());
        frames.insert(AnimState::WalkRight, all.clone());

        // Slower/gentler loop for sleep (use first frame only)
        if let Some(first) = raw_frames.first().cloned() {
            frames.insert(AnimState::Sleep, vec![DecodedFrame { delay_ms: 500, ..first }]);
        }

        // Eat and HappyBounce: same as Idle for now (Phase 2 will use distinct sprites)
        frames.insert(AnimState::Eat, raw_frames.clone());
        frames.insert(AnimState::HappyBounce, raw_frames);

        Self { species_id, shiny, frames }
    }

    /// Get frames for a given state, falling back to Idle if unavailable.
    pub fn frames_for(&self, state: AnimState) -> &[DecodedFrame] {
        self.frames
            .get(&state)
            .or_else(|| self.frames.get(&AnimState::Idle))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}
