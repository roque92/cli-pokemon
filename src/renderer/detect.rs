use super::RendererKind;

/// Detect the best renderer for the current terminal environment.
/// Priority: Kitty > iTerm2 > Unicode art
pub fn detect() -> RendererKind {
    // Allow manual override via env var
    if let Ok(val) = std::env::var("PETCLI_RENDERER") {
        match val.to_lowercase().as_str() {
            "kitty"   => return RendererKind::Kitty,
            "iterm2"  => return RendererKind::Iterm2,
            "unicode" => return RendererKind::Unicode,
            _         => {}
        }
    }

    // Kitty: check authoritative env var before anything else
    if std::env::var("KITTY_WINDOW_ID").is_ok() {
        return RendererKind::Kitty;
    }

    // WezTerm: supports Kitty graphics protocol natively
    if std::env::var("WEZTERM_PANE").is_ok() {
        return RendererKind::Kitty;
    }
    if let Ok(prog) = std::env::var("TERM_PROGRAM") {
        if prog == "WezTerm" {
            return RendererKind::Kitty;
        }
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

    // iTerm2 (macOS) and terminals that support the iTerm2 inline image protocol
    // Warp also sets TERM_PROGRAM=WarpTerminal and supports iTerm2 images
    if let Ok(prog) = std::env::var("TERM_PROGRAM") {
        match prog.as_str() {
            "iTerm.app" | "WarpTerminal" | "Tabby" => return RendererKind::Iterm2,
            _ => {}
        }
    }
    // Warp detection fallback
    if std::env::var("WARP_IS_LOCAL_SHELL_SESSION").is_ok() {
        return RendererKind::Iterm2;
    }

    // Fallback: Unicode half-block art (works everywhere)
    RendererKind::Unicode
}
