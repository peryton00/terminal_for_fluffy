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

        let inner_height = area.height.saturating_sub(2) as usize;
        let total_lines = self.state.output_lines.len();

        // Calculate scroll position
        let scroll_offset = if total_lines > inner_height {
            // If scroll_offset is at the end, auto-scroll
            if self.state.scroll_offset >= total_lines {
                total_lines.saturating_sub(inner_height)
            } else {
                self.state.scroll_offset.min(total_lines.saturating_sub(inner_height))
            }
        } else {
            0
        };

        let lines: Vec<Line> = self
            .state
            .output_lines
            .iter()
            .map(|line| {
                let time_str = line.timestamp.format("%H:%M:%S").to_string();

                let tag_style = if line.tag == "local" {
                    theme::local_tag_style()
                } else if line.tag == "system" {
                    theme::brand_style()
                } else {
                    theme::client_tag_style()
                };

                Line::from(vec![
                    Span::styled(format!("{} ", time_str), theme::dim_style()),
                    Span::styled(format!("[{}] ", line.tag), tag_style),
                    Span::styled(&line.text, ratatui::style::Style::default().fg(line.color)),
                ])
            })
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((scroll_offset as u16, 0));

        Widget::render(paragraph, area, buf);
    }
}
