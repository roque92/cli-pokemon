use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};

use super::app::TuiEvent;

pub fn map_event(event: Event) -> Option<TuiEvent> {
    match event {
        Event::Key(KeyEvent { code, kind, .. }) => {
            // Ignore key release events (Kitty keyboard protocol sends them)
            if kind == KeyEventKind::Release {
                return None;
            }
            match code {
                KeyCode::Char('q') | KeyCode::Char('Q') => Some(TuiEvent::Quit),
                KeyCode::Esc => Some(TuiEvent::Quit),
                KeyCode::Char('f') | KeyCode::Char('F') => Some(TuiEvent::Feed),
                KeyCode::Char('p') | KeyCode::Char('P') => Some(TuiEvent::Pet),
                KeyCode::Char('u') | KeyCode::Char('U') => Some(TuiEvent::UseItem),
                KeyCode::Char('s') | KeyCode::Char('S') => Some(TuiEvent::OpenShop),
                KeyCode::Tab => Some(TuiEvent::CycleParty),
                // Shop navigation (also usable outside shop for up/down if needed)
                KeyCode::Up | KeyCode::Char('k') => Some(TuiEvent::ShopUp),
                KeyCode::Down | KeyCode::Char('j') => Some(TuiEvent::ShopDown),
                KeyCode::Enter | KeyCode::Char('y') => Some(TuiEvent::ShopBuy),
                KeyCode::Char('n') => Some(TuiEvent::ShopCancel),
                _ => None,
            }
        }
        Event::Resize(_, _) => Some(TuiEvent::Resize),
        _ => None,
    }
}
