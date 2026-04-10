use rand::Rng;
use ratatui::layout::Rect;

use super::state_machine::AnimState;

#[derive(Debug, Clone, Copy, Default)]
pub struct TermPos {
    /// Sub-column precision for smooth movement
    pub x: f32,
    pub y: f32,
}

impl TermPos {
    pub fn col(&self) -> u16 {
        self.x as u16
    }
    pub fn row(&self) -> u16 {
        self.y as u16
    }
}

pub struct WalkState {
    pub pos: TermPos,
    pub target: TermPos,
    /// Columns per second
    pub speed: f32,
}

impl WalkState {
    pub fn new(bounds: Rect, rng: &mut impl Rng) -> Self {
        let x = bounds.x as f32 + rng.gen::<f32>() * bounds.width as f32;
        let y = bounds.y as f32 + rng.gen::<f32>() * bounds.height as f32;
        let pos = TermPos { x, y };
        let target = Self::random_target(bounds, rng);
        Self { pos, target, speed: rng.gen_range(0.5..1.5) }
    }

    /// Advance position toward target. Updates AnimState direction.
    /// Returns true if target was reached (caller should pick new target).
    pub fn step(
        &mut self,
        delta_ms: u32,
        bounds: Rect,
        anim_state: &mut AnimState,
        rng: &mut impl Rng,
    ) -> bool {
        if !matches!(anim_state, AnimState::WalkLeft | AnimState::WalkRight) {
            return false;
        }

        let dt = delta_ms as f32 / 1000.0;
        let dx = self.target.x - self.pos.x;
        let dy = self.target.y - self.pos.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < 0.5 {
            self.pick_new_target(bounds, rng);
            return true;
        }

        let step = self.speed * dt;
        self.pos.x += (dx / dist) * step;
        self.pos.y += (dy / dist) * step;

        // Clamp to bounds
        self.pos.x = self.pos.x.clamp(bounds.x as f32, (bounds.x + bounds.width).saturating_sub(1) as f32);
        self.pos.y = self.pos.y.clamp(bounds.y as f32, (bounds.y + bounds.height).saturating_sub(1) as f32);

        // Update direction
        *anim_state = if dx < 0.0 { AnimState::WalkLeft } else { AnimState::WalkRight };

        false
    }

    fn pick_new_target(&mut self, bounds: Rect, rng: &mut impl Rng) {
        self.target = Self::random_target(bounds, rng);
        self.speed = rng.gen_range(0.5..1.5);
    }

    fn random_target(bounds: Rect, rng: &mut impl Rng) -> TermPos {
        TermPos {
            x: bounds.x as f32 + rng.gen::<f32>() * bounds.width as f32,
            y: bounds.y as f32 + rng.gen::<f32>() * bounds.height as f32,
        }
    }
}
