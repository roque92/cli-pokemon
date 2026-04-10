use super::gif::DecodedFrame;

/// Holds idle and walk animation frames for a Pokemon species.
#[derive(Debug)]
pub struct SpriteSheet {
    pub species_id: u16,
    pub shiny: bool,
    /// Frames for the idle animation
    idle: Vec<DecodedFrame>,
    /// Frames for the walk animation (empty = use idle for both)
    walk: Vec<DecodedFrame>,
}

impl SpriteSheet {
    pub fn new(
        species_id: u16,
        shiny: bool,
        idle: Vec<DecodedFrame>,
        walk: Vec<DecodedFrame>,
    ) -> Self {
        Self { species_id, shiny, idle, walk }
    }

    /// Build from a single animation source — use same frames for idle and walk.
    pub fn from_single(species_id: u16, shiny: bool, frames: Vec<DecodedFrame>) -> Self {
        let walk = frames.clone();
        Self { species_id, shiny, idle: frames, walk }
    }

    pub fn idle_frames(&self) -> &[DecodedFrame] {
        &self.idle
    }

    pub fn walk_frames(&self) -> &[DecodedFrame] {
        if self.walk.is_empty() { &self.idle } else { &self.walk }
    }

    /// Pixel dimensions from first idle frame (default 48×48).
    pub fn pixel_size(&self) -> (u32, u32) {
        self.idle
            .first()
            .map(|f| (f.pixels.width(), f.pixels.height()))
            .unwrap_or((48, 48))
    }
}
