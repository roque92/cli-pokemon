use rand::SeedableRng;
use rand::rngs::SmallRng;
use ratatui::layout::Rect;
use std::time::{Duration, Instant};

use crate::{
    animation::{
        state_machine::AnimEvent,
        ticker::AnimTicker,
        walk::WalkState,
    },
    renderer::{kitty::KittyRenderer, unicode::UnicodeRenderer, RendererKind},
    sim::interactions,
    sprite::sheet::SpriteSheet,
    state::{item::ItemKind, party::Party},
    tui::widgets::shop_overlay::ShopState,
};

pub struct Notification {
    pub message: String,
    pub expires: Instant,
}

impl Notification {
    pub fn new(msg: impl Into<String>, secs: u64) -> Self {
        Self { message: msg.into(), expires: Instant::now() + Duration::from_secs(secs) }
    }
    pub fn is_expired(&self) -> bool { Instant::now() >= self.expires }
}

pub struct App {
    pub party: Party,
    pub renderer_kind: RendererKind,

    // Animation
    pub ticker: AnimTicker,
    pub walk: Option<WalkState>,
    pub rng: SmallRng,

    // Sprites
    pub sheet: Option<SpriteSheet>,
    pub unicode_renderer: Option<UnicodeRenderer>,
    pub kitty: KittyRenderer,

    pub play_bounds: Rect,
    pub notification: Option<Notification>,
    pub shop: Option<ShopState>,   // Some = shop is open
    pub compact: bool,
    pub should_quit: bool,
    pub loading_sprite: bool,
}

impl App {
    pub fn new(party: Party, renderer_kind: RendererKind) -> Self {
        Self {
            party,
            renderer_kind,
            ticker: AnimTicker::new(),
            walk: None,
            rng: SmallRng::from_entropy(),
            sheet: None,
            unicode_renderer: None,
            kitty: KittyRenderer::new(),
            play_bounds: Rect::default(),
            notification: None,
            shop: None,
            compact: false,
            should_quit: false,
            loading_sprite: false,
        }
    }

    pub fn handle_event(&mut self, event: TuiEvent) {
        // Shop is modal — capture all events when open
        if self.shop.is_some() {
            self.handle_shop_event(event);
            return;
        }

        match event {
            TuiEvent::Quit => self.should_quit = true,

            TuiEvent::Feed => match interactions::feed(&mut self.party) {
                Ok(ev) => { self.ticker.push_event(AnimEvent::Feed); self.handle_sim_event(ev); }
                Err(e) => self.notify(e.to_string(), 3),
            },

            TuiEvent::Pet => match interactions::pet_pokemon(&mut self.party) {
                Ok(ev) => { self.ticker.push_event(AnimEvent::Pet); self.handle_sim_event(ev); }
                Err(e) => self.notify(e.to_string(), 3),
            },

            TuiEvent::UseItem => {
                // Use the first available non-medicine item, or first medicine if pet is fainted
                let kind = self.best_item_to_use();
                if let Some(kind) = kind {
                    match interactions::use_item(&mut self.party, &kind) {
                        Ok(ev) => { self.ticker.push_event(AnimEvent::Feed); self.handle_sim_event(ev); }
                        Err(e) => self.notify(e.to_string(), 3),
                    }
                } else {
                    self.notify("No items in inventory! Open shop with [s]".to_string(), 3);
                }
            }

            TuiEvent::OpenShop => self.shop = Some(ShopState::new()),

            TuiEvent::CycleParty => {
                self.party.cycle_active();
                self.reset_sprite();
            }

            TuiEvent::Resize => {
                if self.renderer_kind == RendererKind::Kitty {
                    self.kitty.on_resize();
                }
            }
            // Shop events are consumed above when shop is open; ignore here
            _ => {}
        }
    }

    fn handle_shop_event(&mut self, event: TuiEvent) {
        match event {
            TuiEvent::Quit | TuiEvent::ShopCancel => {
                if self.shop.as_ref().map(|s| s.confirm).unwrap_or(false) {
                    if let Some(s) = self.shop.as_mut() { s.confirm = false; }
                } else {
                    self.shop = None;
                }
            }
            TuiEvent::ShopUp => { if let Some(s) = self.shop.as_mut() { s.up(); } }
            TuiEvent::ShopDown => { if let Some(s) = self.shop.as_mut() { s.down(); } }
            TuiEvent::ShopBuy => {
                let (confirm, item, price) = match self.shop.as_ref() {
                    Some(s) => (s.confirm, s.selected_item().clone(), s.selected_item().shop_price()),
                    None => return,
                };
                if confirm {
                    if self.party.wallet >= price {
                        self.party.wallet -= price;
                        self.party.inventory.add(item.clone(), 1);
                        let msg = format!("Bought {}!", item.display_name());
                        self.notify(msg, 2);
                    } else {
                        self.notify("Not enough coins!".to_string(), 2);
                    }
                    if let Some(s) = self.shop.as_mut() { s.confirm = false; }
                } else {
                    if let Some(s) = self.shop.as_mut() { s.confirm = true; }
                }
            }
            _ => {}
        }
    }

    fn best_item_to_use(&self) -> Option<ItemKind> {
        let pet = self.party.active_pet()?;
        // If fainted, try medicine first
        if matches!(pet.status, crate::state::pet::PetStatus::Fainted) {
            for stack in &self.party.inventory.stacks {
                if matches!(stack.kind, ItemKind::Medicine(_)) && stack.qty > 0 {
                    return Some(stack.kind.clone());
                }
            }
        }
        // Otherwise use first available food or toy
        for stack in &self.party.inventory.stacks {
            if stack.qty > 0 {
                return Some(stack.kind.clone());
            }
        }
        None
    }

    fn handle_sim_event(&mut self, event: crate::sim::interactions::SimEvent) {
        use crate::sim::interactions::SimEvent;
        match event {
            SimEvent::None => {}
            SimEvent::Hatched { species_name } => {
                self.notify(format!("Your egg hatched into {}!", species_name), 5);
                self.ticker.push_event(AnimEvent::Hatch);
                self.reset_sprite();
            }
            SimEvent::Evolved { from, to } => {
                self.notify(format!("{} evolved into {}!", from, to), 6);
                self.ticker.push_event(AnimEvent::Pet);
                self.reset_sprite();
            }
            SimEvent::LevelUp { level } => {
                self.notify(format!("Level up! Now Lv.{}!", level), 3);
            }
        }
    }

    pub fn on_sprite_loaded(&mut self, sheet: SpriteSheet) {
        if self.renderer_kind == RendererKind::Unicode {
            self.unicode_renderer = Some(UnicodeRenderer::from_sheet(&sheet));
        }
        self.sheet = Some(sheet);
        self.loading_sprite = false;
        if self.play_bounds.width > 0 && self.walk.is_none() {
            self.walk = Some(WalkState::new(self.play_bounds, &mut self.rng));
        }
    }

    pub fn set_play_bounds(&mut self, bounds: Rect) {
        if self.play_bounds != bounds {
            self.play_bounds = bounds;
            if self.sheet.is_some() && self.walk.is_none() {
                self.walk = Some(WalkState::new(bounds, &mut self.rng));
            }
        }
    }

    fn reset_sprite(&mut self) {
        self.sheet = None;
        self.unicode_renderer = None;
        self.ticker = AnimTicker::new();
        self.walk = None;
        self.loading_sprite = false;
    }

    pub fn notify(&mut self, msg: impl Into<String>, secs: u64) {
        self.notification = Some(Notification::new(msg, secs));
    }

    pub fn tick_notifications(&mut self) {
        if let Some(n) = &self.notification {
            if n.is_expired() { self.notification = None; }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TuiEvent {
    Quit,
    Feed,
    Pet,
    UseItem,
    OpenShop,
    ShopUp,
    ShopDown,
    ShopBuy,
    ShopCancel,
    CycleParty,
    Resize,
}
