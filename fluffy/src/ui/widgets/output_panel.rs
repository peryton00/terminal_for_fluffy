use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use crate::app::{AppState, OutputLine};
use fluffy_ui::theme;
use unicode_width::UnicodeWidthStr;

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
        let mut total_visual_rows = 0u32;

        // Build output lines
        let mut all_lines: Vec<Line> = self
            .lines
            .iter()
            .map(|line| {
                let time_str = line.timestamp.format("%H:%M:%S").to_string();
                let tag_str = format!("[{}] ", line.tag);
                // Replace tabs with 4 spaces for width calculation and rendering
                let content = line.text.replace('\t', "    ");

                // Calculate visual width instead of byte length
                let visual_width = time_str.width() + 1 + tag_str.width() + content.width();
                
                let rows = if content_width > 0 {
                    (visual_width as u32 + content_width as u32 - 1).max(content_width as u32) / content_width as u32
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

        // Add the prompt + input as the last line
        let prompt = self.state.prompt();
        let input = self.state.input_buffer.replace('\t', "    ");
        
        let alter_indicator = if self.state.alter_target.is_some() {
            theme::client_tag_style()
        } else {
            theme::brand_style()
        };

        let prompt_line = Line::from(vec![
            Span::styled(&prompt, alter_indicator),
            Span::styled(&input, theme::text_style()),
        ]);

        // Visual rows for the prompt (minimum 1)
        let visual_prompt_width = prompt.width() + input.width();
        let prompt_rows = if content_width > 0 {
            (visual_prompt_width as u32 + content_width as u32 - 1).max(content_width as u32) / content_width as u32
        } else {
            1
        };
        total_visual_rows += prompt_rows;

        all_lines.push(prompt_line);

        // Available height inside the bordered block
        let view_height = area.height.saturating_sub(2);

        // Calculate the maximum meaningful scroll value
        let max_scroll = total_visual_rows.saturating_sub(view_height as u32);

        // Sticky-Bottom logic
        let scroll_val = if self.state.scroll_offset == u32::MAX {
            max_scroll
        } else {
            self.state.scroll_offset.min(max_scroll)
        };

        let paragraph = Paragraph::new(all_lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((scroll_val as u16, 0));

        Widget::render(paragraph, area, buf);
    }
}

/// Get the cursor position for the inline prompt (inside the output panel).
pub fn cursor_position(state: &AppState, output_area: Rect) -> (u16, u16) {
    let content_width = output_area.width.saturating_sub(2); // borders
    if content_width == 0 {
        return (output_area.x + 1, output_area.y + 1);
    }

    let lines = match state.mode {
        crate::app::TerminalMode::Admin => &state.output_lines[..],
        crate::app::TerminalMode::Client => &state.client_output[..],
    };

    // Count total visual rows for all output lines (identical to render)
    let mut total_visual_rows = 0u32;
    for line in lines {
        let time_str = line.timestamp.format("%H:%M:%S").to_string();
        let tag_str = format!("[{}] ", line.tag);
        let content = line.text.replace('\t', "    ");
        
        let visual_width = time_str.width() + 1 + tag_str.width() + content.width();
        let rows = (visual_width as u32 + content_width as u32 - 1).max(content_width as u32) / content_width as u32;
        total_visual_rows += rows;
    }

    // The prompt line's visual rows
    let prompt = state.prompt();
    let input = state.input_buffer.replace('\t', "    ");
    
    let visual_prompt_width = prompt.width() + input.width();
    let prompt_rows = (visual_prompt_width as u32 + content_width as u32 - 1).max(content_width as u32) / content_width as u32;
    total_visual_rows += prompt_rows;

    let prompt_start_row = total_visual_rows - prompt_rows;

    // Cursor position in the input buffer (byte offset -> char width offset)
    // For simplicity, we assume one char = one width unit here if it's ascii, 
    // but the robust way is to measure width of the string up to cursor_pos.
    let input_up_to_cursor = state.input_buffer.chars().take(state.cursor_pos).collect::<String>().replace('\t', "    ");
    let cursor_char_offset = prompt.width() as u32 + input_up_to_cursor.width() as u32;

    // Which visual row/column
    let cursor_row_in_prompt = cursor_char_offset / content_width as u32;
    let cursor_col = cursor_char_offset % content_width as u32;

    let cursor_abs_row = prompt_start_row + cursor_row_in_prompt;

    let view_height = output_area.height.saturating_sub(2);
    let max_scroll = total_visual_rows.saturating_sub(view_height as u32);
    let scroll_val = if state.scroll_offset == u32::MAX {
        max_scroll
    } else {
        state.scroll_offset.min(max_scroll)
    };

    let visible_row = cursor_abs_row as i64 - scroll_val as i64;

    if visible_row < 0 || visible_row >= view_height as i64 {
        (0, 0) // Hidden
    } else {
        let cx = output_area.x + 1 + cursor_col as u16;
        let cy = output_area.y + 1 + visible_row as u16;
        (cx, cy)
    }
}

