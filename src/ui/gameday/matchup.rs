use crate::components::game::live_game::GameStateV2;
use crate::components::game::matchup::Matchup;
use tui::prelude::*;
use tui::widgets::{Block, Borders, Paragraph};

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
        // TODO get rid of this
        let v1_matchup = Matchup::from_v2(
            matchup_v2,
            self.game.home_team,
            self.game.away_team,
            is_current,
        );

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .horizontal_margin(2)
            .vertical_margin(1)
            .constraints(
                [
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                ]
                .as_ref(),
            )
            .split(area)
            .to_vec();
        // let chunks = LayoutAreas::for_info(area);
        Widget::render(
            Paragraph::new(v1_matchup.to_table_away())
                .alignment(Alignment::Left)
                .block(Block::default().borders(Borders::BOTTOM)),
            chunks[0],
            buf,
        );
        Widget::render(
            Paragraph::new(v1_matchup.to_at_bat())
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::BOTTOM)),
            chunks[1],
            buf,
        );
        Widget::render(
            Paragraph::new(v1_matchup.to_table_home())
                .alignment(Alignment::Right)
                .block(Block::default().borders(Borders::BOTTOM)),
            chunks[2],
            buf,
        );
    }
}
