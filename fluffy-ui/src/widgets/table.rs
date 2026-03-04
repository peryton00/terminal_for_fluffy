use ratatui::{
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Row, Table as RatatuiTable, Widget},
    buffer::Buffer,
};

use crate::theme;

/// A reusable styled table widget for Fluffy.
pub struct FluffyTable<'a> {
    title: &'a str,
    headers: Vec<&'a str>,
    rows: Vec<Vec<String>>,
    widths: Vec<ratatui::layout::Constraint>,
}

impl<'a> FluffyTable<'a> {
    pub fn new(title: &'a str, headers: Vec<&'a str>, widths: Vec<ratatui::layout::Constraint>) -> Self {
        Self {
            title,
            headers,
            rows: Vec::new(),
            widths,
        }
    }

    pub fn rows(mut self, rows: Vec<Vec<String>>) -> Self {
        self.rows = rows;
        self
    }
}

impl<'a> Widget for FluffyTable<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let header_cells: Vec<ratatui::text::Text> = self
            .headers
            .iter()
            .map(|h| ratatui::text::Text::from(Line::styled(h.to_string(), theme::brand_style())))
            .collect();
        let header = Row::new(header_cells).style(theme::bold_style()).height(1);

        let data_rows: Vec<Row> = self
            .rows
            .iter()
            .map(|row| {
                let cells: Vec<ratatui::text::Text> = row
                    .iter()
                    .map(|c| ratatui::text::Text::from(c.as_str()))
                    .collect();
                Row::new(cells).style(theme::text_style())
            })
            .collect();

        let table = RatatuiTable::new(data_rows, &self.widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(theme::border_style())
                    .title(self.title)
                    .title_style(theme::brand_style()),
            )
            .style(Style::default());

        Widget::render(table, area, buf);
    }
}
