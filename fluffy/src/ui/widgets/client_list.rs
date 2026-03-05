use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Widget},
};

use crate::app::AppState;
use fluffy_ui::theme;

/// Render the connected clients list panel.
pub struct ClientListWidget<'a> {
    pub state: &'a AppState,
}

impl<'a> Widget for ClientListWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .title(" Clients ")
            .title_style(theme::brand_style());

        let mut items: Vec<ListItem> = Vec::new();

        let mut clients: Vec<_> = self.state.clients.values().collect();
        clients.sort_by(|a, b| a.tag.cmp(&b.tag));

        for client in &clients {
            let is_alter_target = self
                .state
                .alter_target
                .as_ref()
                .map(|t| t == &client.tag)
                .unwrap_or(false);

            let dot_color = theme::SUCCESS_COLOR;
            let tag_style = if is_alter_target {
                theme::highlight_style()
            } else {
                theme::client_tag_style()
            };

            let line = Line::from(vec![
                Span::styled("● ", ratatui::style::Style::default().fg(dot_color)),
                Span::styled(format!("{} ", client.tag), tag_style),
                Span::styled(&client.hostname, theme::text_style()),
            ]);

            items.push(ListItem::new(line));
        }

        if items.is_empty() {
            items.push(ListItem::new(Line::from(Span::styled(
                " No clients",
                theme::dim_style(),
            ))));
        }

        let list = List::new(items).block(block);
        Widget::render(list, area, buf);
    }
}
