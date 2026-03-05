use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use crate::app::{AppState, OutputLine};
use fluffy_ui::theme;

/// The main output panel showing command outputs AND the input prompt.
pub struct OutputPanelWidget<'a> {
    pub state: &'a AppState,
    pub title: &'a str,
    pub lines: &'a [OutputLine],
}

impl<'a> Widget for OutputPanelWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .title(self.title)
            .title_style(theme::brand_style());

        let content_width = area.width.saturating_sub(2); // Account for borders
        let mut total_visual_rows = 0u16;

        // Build output lines
        let mut all_lines: Vec<Line> = self
            .lines
            .iter()
            .map(|line| {
                let time_str = line.timestamp.format("%H:%M:%S").to_string();
                let tag_str = format!("[{}] ", line.tag);
                let content = &line.text;

                // Calculate how many rows this specific line takes
                let total_len = time_str.len() + 1 + tag_str.len() + content.len();
                let rows = if content_width > 0 {
                    (total_len as u16 + content_width - 1) / content_width
                } else {
                    1
                };
                total_visual_rows += rows;

                let tag_style = if line.tag == "local" {
                    theme::local_tag_style()
                } else if line.tag == "system" {
                    theme::brand_style()
                } else {
                    theme::client_tag_style()
                };

                Line::from(vec![
                    Span::styled(format!("{} ", time_str), theme::dim_style()),
                    Span::styled(tag_str, tag_style),
                    Span::styled(content, ratatui::style::Style::default().fg(line.color)),
                ])
            })
            .collect();

        // Add the prompt + input as the last line (like a real terminal)
        let prompt = self.state.prompt();
        let alter_indicator = if self.state.alter_target.is_some() {
            theme::client_tag_style()
        } else {
            theme::brand_style()
        };

        let prompt_line = Line::from(vec![
            Span::styled(&prompt, alter_indicator),
            Span::styled(&self.state.input_buffer, theme::text_style()),
        ]);

        // Account for the prompt line's visual rows
        let prompt_len = prompt.len() + self.state.input_buffer.len();
        let prompt_rows = if content_width > 0 {
            (prompt_len as u16 + content_width - 1) / content_width
        } else {
            1
        };
        total_visual_rows += prompt_rows;

        all_lines.push(prompt_line);

        // Available height inside the bordered block
        let view_height = area.height.saturating_sub(2);

        // Calculate the maximum meaningful scroll value
        let max_scroll = total_visual_rows.saturating_sub(view_height);

        // If scroll_offset is u16::MAX, we want Sticky-Bottom (always show latest + prompt)
        let scroll_val = if self.state.scroll_offset == u16::MAX {
            max_scroll
        } else {
            // Manual scroll mode: clamp so we never scroll past the content
            self.state.scroll_offset.min(max_scroll)
        };

        let paragraph = Paragraph::new(all_lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((scroll_val, 0));

        Widget::render(paragraph, area, buf);
    }
}

/// Get the cursor position for the inline prompt (inside the output panel).
pub fn cursor_position(state: &AppState, output_area: Rect) -> (u16, u16) {
    let prompt_len = state.prompt().len() as u16;
    let cursor_x = output_area.x + 1 + prompt_len + state.cursor_pos as u16;
    // Cursor is on the last visible row inside the bordered area
    let cursor_y = output_area.y + output_area.height.saturating_sub(2);
    (cursor_x.min(output_area.x + output_area.width - 2), cursor_y)
}

