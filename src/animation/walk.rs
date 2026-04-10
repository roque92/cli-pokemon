use rand::Rng;

/// 2-D bouncing walker — pet moves freely around the full terminal,
/// bouncing off all four edges. Direction determines sprite flip.
pub struct Walker {
    pub x: f32,
    pub y: f32,
    dx: f32, // columns per tick (positive = right)
    dy: f32, // rows per tick    (positive = down)
    sprite_cols: u16,
    sprite_rows: u16,
    term_cols: u16,
    term_rows: u16,
    /// Ticks until next random direction nudge
    nudge_ticks: u32,
}

impl Walker {
    pub fn new(
        term_cols: u16,
        term_rows: u16,
        sprite_cols: u16,
        sprite_rows: u16,
        rng: &mut impl Rng,
    ) -> Self {
        let x = rng.gen_range(0.0..(term_cols.saturating_sub(sprite_cols) as f32).max(1.0));
        let y = rng.gen_range(0.0..(term_rows.saturating_sub(sprite_rows + 1) as f32).max(1.0));
        let angle: f32 = rng.gen_range(0.0..std::f32::consts::TAU);
        let speed: f32 = rng.gen_range(0.3..0.8);
        Self {
            x,
            y,
            dx: angle.cos() * speed,
            dy: angle.sin() * speed * 0.5, // rows are ~2× taller than cols, so halve Y speed
            sprite_cols,
            sprite_rows,
            term_cols,
            term_rows,
            nudge_ticks: rng.gen_range(60u32..200),
        }
    }

    pub fn tick(&mut self, rng: &mut impl Rng) {
        self.x += self.dx;
        self.y += self.dy;

        let max_x = self.term_cols.saturating_sub(self.sprite_cols) as f32;
        let max_y = self.term_rows.saturating_sub(self.sprite_rows + 1) as f32; // +1 = hint row

        // Bounce X
        if self.x <= 0.0 {
            self.x = 0.0;
            self.dx = self.dx.abs();
        } else if self.x >= max_x {
            self.x = max_x;
            self.dx = -self.dx.abs();
        }

        // Bounce Y
        if self.y <= 0.0 {
            self.y = 0.0;
            self.dy = self.dy.abs();
        } else if self.y >= max_y {
            self.y = max_y;
            self.dy = -self.dy.abs();
        }

        // Occasional random direction nudge
        if self.nudge_ticks == 0 {
            let angle: f32 = rng.gen_range(0.0..std::f32::consts::TAU);
            let speed: f32 = rng.gen_range(0.3..0.8);
            self.dx = angle.cos() * speed;
            self.dy = angle.sin() * speed * 0.5;
            self.nudge_ticks = rng.gen_range(60u32..200);
        } else {
            self.nudge_ticks -= 1;
        }
    }

    pub fn col(&self) -> u16 { self.x as u16 }
    pub fn row(&self) -> u16 { self.y as u16 }

    /// True when moving right (sprite faces right; left = flip)
    pub fn facing_left(&self) -> bool { self.dx < 0.0 }

    /// Always "walking" in free-bounce mode
    pub fn is_walking(&self) -> bool { true }

    pub fn resize(&mut self, term_cols: u16, term_rows: u16) {
        self.term_cols = term_cols;
        self.term_rows = term_rows;
        let max_x = term_cols.saturating_sub(self.sprite_cols) as f32;
        let max_y = term_rows.saturating_sub(self.sprite_rows + 1) as f32;
        self.x = self.x.clamp(0.0, max_x);
        self.y = self.y.clamp(0.0, max_y);
    }

    pub fn set_sprite_size(&mut self, cols: u16, rows: u16) {
        self.sprite_cols = cols;
        self.sprite_rows = rows;
    }
}
