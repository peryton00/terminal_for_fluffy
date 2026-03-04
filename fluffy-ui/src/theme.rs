use ratatui::style::{Color, Modifier, Style};

// ─── Brand Colors ───────────────────────────────────────────
pub const BRAND_COLOR: Color = Color::Cyan;
pub const SUCCESS_COLOR: Color = Color::Green;
pub const ERROR_COLOR: Color = Color::Red;
pub const WARNING_COLOR: Color = Color::Yellow;
pub const LOCAL_TAG_COLOR: Color = Color::Magenta;
pub const CLIENT_TAG_COLOR: Color = Color::Yellow;
pub const DIM_COLOR: Color = Color::Gray;
pub const TEXT_COLOR: Color = Color::White;
pub const BG_COLOR: Color = Color::Reset;
pub const BORDER_COLOR: Color = Color::Cyan;
pub const HIGHLIGHT_COLOR: Color = Color::LightCyan;

// ─── Style Helpers ──────────────────────────────────────────

/// Default text style
pub fn text_style() -> Style {
    Style::default().fg(TEXT_COLOR)
}

/// Bold text style
pub fn bold_style() -> Style {
    Style::default()
        .fg(TEXT_COLOR)
        .add_modifier(Modifier::BOLD)
}

/// Brand colored text
pub fn brand_style() -> Style {
    Style::default()
        .fg(BRAND_COLOR)
        .add_modifier(Modifier::BOLD)
}

/// Success text
pub fn success_style() -> Style {
    Style::default().fg(SUCCESS_COLOR)
}

/// Error text
pub fn error_style() -> Style {
    Style::default().fg(ERROR_COLOR)
}

/// Warning text
pub fn warning_style() -> Style {
    Style::default().fg(WARNING_COLOR)
}

/// Dimmed text
pub fn dim_style() -> Style {
    Style::default().fg(DIM_COLOR)
}

/// Local tag style
pub fn local_tag_style() -> Style {
    Style::default()
        .fg(LOCAL_TAG_COLOR)
        .add_modifier(Modifier::BOLD)
}

/// Client tag style
pub fn client_tag_style() -> Style {
    Style::default()
        .fg(CLIENT_TAG_COLOR)
        .add_modifier(Modifier::BOLD)
}

/// Border style
pub fn border_style() -> Style {
    Style::default().fg(BORDER_COLOR)
}

/// Highlighted / selected item
pub fn highlight_style() -> Style {
    Style::default()
        .fg(HIGHLIGHT_COLOR)
        .add_modifier(Modifier::BOLD)
}

/// Status bar style
pub fn status_bar_style() -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(BRAND_COLOR)
        .add_modifier(Modifier::BOLD)
}
