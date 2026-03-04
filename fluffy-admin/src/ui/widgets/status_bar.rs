use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::app::AppState;
use fluffy_ui::theme;

/// The top status bar.
pub struct StatusBarWidget<'a> {
    pub state: &'a AppState,
}

impl<'a> Widget for StatusBarWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let target = match &self.state.alter_target {
            Some(tag) => {
                // Find hostname for this tag
                let hostname = self
                    .state
                    .clients
                    .values()
                    .find(|c| c.tag == *tag)
                    .map(|c| c.hostname.as_str())
                    .unwrap_or("?");
                format!("{} [{}]", tag, hostname)
            }
            None => "local".to_string(),
        };

        let client_count = self.state.clients.len();
        let time = chrono::Local::now().format("%H:%M:%S").to_string();

        let mut spans = vec![
            Span::styled(" FLUFFY ", theme::status_bar_style()),
            Span::styled("  │  ", theme::status_bar_style()),
            Span::styled(format!("Target: {}", target), theme::status_bar_style()),
            Span::styled("  │  ", theme::status_bar_style()),
            Span::styled(format!("{} online", client_count), theme::status_bar_style()),
            Span::styled("  │  ", theme::status_bar_style()),
            Span::styled(time, theme::status_bar_style()),
        ];

        // Pad the rest
        let content_len: usize = spans.iter().map(|s| s.content.len()).sum();
        if area.width as usize > content_len {
            let pad = " ".repeat(area.width as usize - content_len);
            spans.push(Span::styled(pad, theme::status_bar_style()));
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line).block(Block::default().borders(Borders::NONE));

        Widget::render(paragraph, area, buf);
    }
}
