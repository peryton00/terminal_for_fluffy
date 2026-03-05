use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

use fluffy_ui::theme;

/// The help overlay popup.
pub struct HelpOverlayWidget {
    pub scroll: u16,
}

impl Widget for HelpOverlayWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the area behind the popup
        Clear.render(area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme::brand_style())
            .title(" Fluffy Terminal — Help (ESC to close) ")
            .title_style(theme::brand_style());

        let mut lines = Vec::new();

        lines.push(Line::from(Span::styled("  ═══════════════════════════════════════", theme::brand_style())));
        lines.push(Line::from(Span::styled("  ADMIN COMMANDS (run locally)", theme::brand_style())));
        lines.push(Line::from(Span::styled("  ─────────────────────────────────────", theme::brand_style())));
        lines.push(Line::from(""));

        for cmd in crate::repl::get_admin_commands() {
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<19}", cmd.usage), theme::highlight_style()),
                Span::styled(" → ", theme::dim_style()),
                Span::styled(cmd.description, theme::text_style()),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("  ═══════════════════════════════════════", theme::brand_style())));
        lines.push(Line::from(Span::styled("  CLIENT COMMANDS (sent to target)", theme::brand_style())));
        lines.push(Line::from(Span::styled("  ─────────────────────────────────────", theme::brand_style())));
        lines.push(Line::from(""));

        for cmd in shared::commands::get_client_commands() {
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<19}", cmd.usage), theme::highlight_style()),
                Span::styled(" → ", theme::dim_style()),
                Span::styled(cmd.description, theme::text_style()),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("  ═══════════════════════════════════════", theme::brand_style())));
        lines.push(Line::from(Span::styled("  SHORTCUTS", theme::brand_style())));
        lines.push(Line::from(Span::styled("  ─────────────────────────────────────", theme::brand_style())));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled("  Ctrl+C              ", theme::highlight_style()), Span::styled(" → ", theme::dim_style()), Span::styled("Exit", theme::text_style())]));
        lines.push(Line::from(vec![Span::styled("  ESC                 ", theme::highlight_style()), Span::styled(" → ", theme::dim_style()), Span::styled("Close help overlay", theme::text_style())]));
        lines.push(Line::from(vec![Span::styled("  ↑/↓                 ", theme::highlight_style()), Span::styled(" → ", theme::dim_style()), Span::styled("Scroll output / History", theme::text_style())]));
        lines.push(Line::from(vec![Span::styled("  Shift + ↑/↓         ", theme::highlight_style()), Span::styled(" → ", theme::dim_style()), Span::styled("Scroll output manually", theme::text_style())]));
        lines.push(Line::from(vec![Span::styled("  Page Up/Down        ", theme::highlight_style()), Span::styled(" → ", theme::dim_style()), Span::styled("Scroll output (fast)", theme::text_style())]));
        lines.push(Line::from(""));

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));

        Widget::render(paragraph, area, buf);
    }
}


