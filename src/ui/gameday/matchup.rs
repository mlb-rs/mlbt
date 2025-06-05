use crate::components::game::matchup::Matchup;
use crate::ui::layout::LayoutAreas;

use crate::components::game::live_game::GameStateV2;
use tui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Row, Table, Widget},
};

pub struct MatchupWidget<'a> {
    pub game: &'a GameStateV2,
    pub selected_at_bat: Option<u8>,
}

impl Widget for MatchupWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (game, is_current) = self
            .game
            .get_at_bat_by_index_or_current(self.selected_at_bat);
        let matchup_v2 = &game.matchup;
        let summary = self.game.get_summary();
        // TODO get rid of this
        let v1_matchup = Matchup::from_v2(matchup_v2, summary, is_current);

        let chunks = LayoutAreas::for_info(area);
        Widget::render(
            Table::new(
                v1_matchup.to_table().into_iter().map(Row::new),
                [Constraint::Length(12), Constraint::Length(25)],
            )
            .column_spacing(1)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::NONE)),
            chunks[0],
            buf,
        );
    }
}
