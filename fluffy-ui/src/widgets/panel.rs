use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Text,
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use crate::theme;

/// A reusable bordered panel widget with a title and content.
pub struct FluffyPanel<'a> {
    title: &'a str,
    content: Text<'a>,
    scroll: u16,
}

impl<'a> FluffyPanel<'a> {
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

impl<'a> Widget for FluffyPanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .title(self.title)
            .title_style(theme::brand_style());

        let paragraph = Paragraph::new(self.content)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));

        Widget::render(paragraph, area, buf);
    }
}
