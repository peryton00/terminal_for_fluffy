use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

use crate::theme;

/// A reusable popup/overlay notification widget.
pub struct FluffyNotification<'a> {
    title: &'a str,
    content: Text<'a>,
    scroll: u16,
}

impl<'a> FluffyNotification<'a> {
    pub fn new(title: &'a str, content: Text<'a>) -> Self {
        Self {
            title,
            content,
            scroll: 0,
        }
    }

    pub fn scroll(mut self, offset: u16) -> Self {
        self.scroll = offset;
        self
    }
}

impl<'a> Widget for FluffyNotification<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the area first (so the popup is opaque)
        Clear.render(area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme::brand_style())
            .title(self.title)
            .title_style(theme::brand_style());

        let paragraph = Paragraph::new(self.content)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));

        Widget::render(paragraph, area, buf);
    }
}
