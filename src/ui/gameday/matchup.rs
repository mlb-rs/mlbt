use crate::components::game::live_game::GameStateV2;
use tui::prelude::*;
use tui::widgets::{Block, Borders, Padding, Paragraph};

pub struct MatchupWidget<'a> {
    pub game: &'a GameStateV2,
    pub selected_at_bat: Option<u8>,
}

impl Widget for MatchupWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (game, is_current) = self
            .game
            .get_at_bat_by_index_or_current(self.selected_at_bat);

        let matchup_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .horizontal_margin(2)
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

        // let on_deck_chunks = Layout::default()
        //     .direction(Direction::Vertical)
        //     .constraints([Constraint::Fill(1), Constraint::Length(1)].as_ref())
        //     .split(area)
        //     .to_vec();

        Widget::render(
            Paragraph::new(game.matchup.format_away_lines(
                self.game.away_team.team_name,
                is_current,
                &self.game.players,
            ))
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .padding(Padding::new(0, 0, 1, 0)),
            ),
            matchup_chunks[0],
            buf,
        );
        Widget::render(
            Paragraph::new(game.matchup.format_scoreboard_lines())
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::BOTTOM)
                        .padding(Padding::new(0, 0, 1, 0)),
                ),
            matchup_chunks[1],
            buf,
        );
        Widget::render(
            Paragraph::new(game.matchup.format_home_lines(
                self.game.home_team.team_name,
                is_current,
                &self.game.players,
            ))
            .alignment(Alignment::Right)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .padding(Padding::new(0, 0, 1, 0)),
            ),
            matchup_chunks[2],
            buf,
        );
    }
}
