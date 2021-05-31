use crate::matchup::Matchup;
use crate::ui::layout::LayoutAreas;
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Row, Table},
    Frame,
};

impl Matchup {
    pub fn render<B>(&self, f: &mut Frame<B>, rect: Rect)
    where
        B: Backend,
    {
        let chunks = LayoutAreas::for_info(rect);

        let t = Table::new(self.to_table().iter().map(|row| Row::new(row.clone())))
            .widths(&[Constraint::Length(12), Constraint::Length(25)])
            .column_spacing(1)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::NONE));

        f.render_widget(t, chunks[0]);
    }
}
