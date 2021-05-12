use super::super::heatmap::Heatmap;
use super::utils::centered_rect;
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

impl Heatmap {
    pub fn render<B>(&self, f: &mut Frame<B>, rect: Rect)
    where
        B: Backend,
    {
        // these should be determined by the terminal size
        let width = 5;
        let height = 3;

        // TODO probably a better way to do this
        let top = Row::new(vec![
            Cell::from("").style(Style::default().bg(self.cells[0])),
            Cell::from("").style(Style::default().bg(self.cells[1])),
            Cell::from("").style(Style::default().bg(self.cells[2])),
        ])
        .height(height);
        let middle = Row::new(vec![
            Cell::from("").style(Style::default().bg(self.cells[3])),
            Cell::from("").style(Style::default().bg(self.cells[4])),
            Cell::from("").style(Style::default().bg(self.cells[5])),
        ])
        .height(height);
        let bottom = Row::new(vec![
            Cell::from("").style(Style::default().bg(self.cells[6])),
            Cell::from("").style(Style::default().bg(self.cells[7])),
            Cell::from("").style(Style::default().bg(self.cells[8])),
        ])
        .height(height);

        let widths = [
            Constraint::Length(width),
            Constraint::Length(width),
            Constraint::Length(width),
        ];
        let t = Table::new(vec![top, middle, bottom])
            .block(Block::default().borders(Borders::NONE).title("heatmap"))
            .widths(&widths)
            .column_spacing(0);

        // TODO the size of the centered rect needs to be dynamic
        let area = centered_rect(15, 15, rect);
        f.render_widget(t, area);
    }
}
