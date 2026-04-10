use super::RendererKind;

/// Detect the best renderer for the current terminal environment.
/// Priority: Kitty > Unicode art
pub fn detect() -> RendererKind {
    // Kitty: check authoritative env var before anything else
    if std::env::var("KITTY_WINDOW_ID").is_ok() {
        return RendererKind::Kitty;
    }

    // WezTerm: supports Kitty graphics protocol natively
    if let Ok(prog) = std::env::var("TERM_PROGRAM") {
        if prog == "WezTerm" {
            return RendererKind::Kitty;
        }
    }
    if std::env::var("WEZTERM_PANE").is_ok() {
        return RendererKind::Kitty;
    }

    // Ghostty
    if std::env::var("GHOSTTY_RESOURCES_DIR").is_ok() {
        return RendererKind::Kitty;
    }
    if let Ok(prog) = std::env::var("TERM_PROGRAM") {
        if prog == "ghostty" {
            return RendererKind::Kitty;
        }
    }

    // Fallback: Unicode half-block art (works everywhere including Alacritty)
    RendererKind::Unicode
}
