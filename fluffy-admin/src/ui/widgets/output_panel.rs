use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use crate::app::AppState;
use fluffy_ui::theme;

/// The main output panel showing command outputs.
pub struct OutputPanelWidget<'a> {
    pub state: &'a AppState,
}

impl<'a> Widget for OutputPanelWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .title(" Output ")
            .title_style(theme::brand_style());



        let mut total_visual_rows = 0u16;
        let content_width = area.width.saturating_sub(2); // Account for borders

        let lines: Vec<Line> = self
            .state
            .output_lines
            .iter()
            .map(|line| {
                let time_str = line.timestamp.format("%H:%M:%S").to_string();
                let tag_str = format!("[{}] ", line.tag);
                let content = &line.text;

                // Calculate how many rows this specific line takes
                // tag + time + text
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

        // If scroll_offset is u16::MAX, we want Sticky-Bottom
        let scroll_val = if self.state.scroll_offset == u16::MAX {
            total_visual_rows.saturating_sub(area.height.saturating_sub(2))
        } else {
            // Manual scroll mode: user has scrolled to a specific position
            // We'll treat scroll_offset as the top line to show
            self.state.scroll_offset
        };

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((scroll_val, 0));

        Widget::render(paragraph, area, buf);
    }
}
