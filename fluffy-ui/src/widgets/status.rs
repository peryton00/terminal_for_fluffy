use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::theme;

/// A reusable status bar widget showing key-value pairs.
pub struct FluffyStatusBar<'a> {
    items: Vec<(&'a str, String)>,
}

impl<'a> FluffyStatusBar<'a> {
    pub fn new(items: Vec<(&'a str, String)>) -> Self {
        Self { items }
    }
}

impl<'a> Widget for FluffyStatusBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut spans: Vec<Span> = Vec::new();

        spans.push(Span::styled(" FLUFFY ", theme::status_bar_style()));

        for (label, value) in &self.items {
            spans.push(Span::styled("  │  ", theme::status_bar_style()));
            spans.push(Span::styled(
                format!("{}: {}", label, value),
                theme::status_bar_style(),
            ));
        }

        // Fill the rest with the status bar background
        let content_len: usize = spans.iter().map(|s| s.content.len()).sum();
        if area.width as usize > content_len {
            let pad = " ".repeat(area.width as usize - content_len);
            spans.push(Span::styled(pad, theme::status_bar_style()));
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line).block(
            Block::default()
                .borders(Borders::NONE),
        );

        Widget::render(paragraph, area, buf);
    }
}
