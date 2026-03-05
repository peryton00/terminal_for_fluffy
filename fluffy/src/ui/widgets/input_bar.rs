use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::app::AppState;
use fluffy_ui::theme;

/// The bottom input bar widget.
pub struct InputBarWidget<'a> {
    pub state: &'a AppState,
}

impl<'a> Widget for InputBarWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let prompt = self.state.prompt();
        let alter_indicator = if self.state.alter_target.is_some() {
            theme::client_tag_style()
        } else {
            theme::brand_style()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme::border_style());

        let line = Line::from(vec![
            Span::styled(&prompt, alter_indicator),
            Span::styled(&self.state.input_buffer, theme::text_style()),
        ]);

        let paragraph = Paragraph::new(line).block(block);
        Widget::render(paragraph, area, buf);
    }
}

/// Get the cursor position for the input bar (x, y relative to terminal).
pub fn cursor_position(state: &AppState, input_area: Rect) -> (u16, u16) {
    let prompt_len = state.prompt().len() as u16;
    let cursor_x = input_area.x + 1 + prompt_len + state.cursor_pos as u16;
    let cursor_y = input_area.y + 1;
    (cursor_x.min(input_area.x + input_area.width - 2), cursor_y)
}
