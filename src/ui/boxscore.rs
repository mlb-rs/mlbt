use crate::components::boxscore::Boxscore;
use crate::state::app_state::HomeOrAway;
use tui::prelude::*;
use tui::widgets::{Block, Paragraph, Row, Table, Wrap};

const BATTER_WIDTHS: [Constraint; 9] = [
    Constraint::Length(25), // player name
    Constraint::Length(4),  // ab
    Constraint::Length(4),  // r
    Constraint::Length(4),  // h
    Constraint::Length(4),  // rbi
    Constraint::Length(4),  // bb
    Constraint::Length(4),  // k
    Constraint::Length(4),  // lob
    Constraint::Length(5),  // avg
];

const PITCHER_WIDTHS: [Constraint; 9] = [
    Constraint::Length(25), // pitcher name
    Constraint::Length(5),  // ip
    Constraint::Length(4),  // h
    Constraint::Length(4),  // r
    Constraint::Length(4),  // er
    Constraint::Length(4),  // bb
    Constraint::Length(4),  // k
    Constraint::Length(4),  // hr
    Constraint::Length(5),  // era
];

pub struct TeamBatterBoxscoreWidget<'a> {
    pub active: HomeOrAway,
    pub boxscore: &'a Boxscore,
}

impl Widget for TeamBatterBoxscoreWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let batting = self.boxscore.to_batting_table_rows(self.active);
        let batting_notes = self
            .boxscore
            .get_batting_notes(self.active)
            .iter()
            .filter_map(|n| n.to_line());
        let notes_paragraph = Paragraph::new(batting_notes.collect::<Vec<_>>())
            .block(Block::default())
            .wrap(Wrap { trim: true });
        let pitching = self.boxscore.to_pitching_table_rows(self.active);

        let [boxscore, note, pitchers, game_notes] = Layout::vertical([
            Constraint::Length(batting.len() as u16 + 1), // +1 for header
            Constraint::Length(notes_paragraph.line_count(area.width) as u16),
            Constraint::Length(pitching.len() as u16 + 1), // +1 for header
            Constraint::Fill(1),
        ])
        .spacing(1)
        .areas(area);

        Widget::render(
            Table::new(batting.into_iter().map(Row::new), BATTER_WIDTHS)
                .column_spacing(0)
                .header(
                    Row::new(self.boxscore.get_batting_header().iter().copied())
                        .bold()
                        .underlined(),
                ),
            boxscore,
            buf,
        );

        Widget::render(notes_paragraph, note, buf);

        Widget::render(
            Table::new(pitching.into_iter().map(Row::new), PITCHER_WIDTHS)
                .column_spacing(0)
                .header(
                    Row::new(self.boxscore.get_pitching_header().iter().copied())
                        .bold()
                        .underlined(),
                ),
            pitchers,
            buf,
        );

        Widget::render(
            Paragraph::new(self.boxscore.get_game_notes())
                .block(Block::default())
                .wrap(Wrap { trim: true }),
            game_notes,
            buf,
        );
    }
}
