use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Create the main application layout with:
/// - Top: status bar (3 rows)
/// - Middle: main content area (variable)
/// - Bottom: input bar (3 rows)
pub fn main_layout(area: Rect) -> (Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // status bar
            Constraint::Min(5),    // main content
            Constraint::Length(3), // input bar
        ])
        .split(area);
    (chunks[0], chunks[1], chunks[2])
}

/// Split the main content area into:
/// - Left: client list (25%)
/// - Right: output panel (75%)
pub fn content_layout(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(75),
        ])
        .split(area);
    (chunks[0], chunks[1])
}

/// Create a centered popup rect of given percentage size.
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
