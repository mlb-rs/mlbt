use crate::components::game::live_game::GameState;
use crate::symbols::Symbols;
use crate::theme::Theme;
use tui::prelude::*;
use tui::widgets::{Block, Borders, Padding, Paragraph};

pub struct MatchupWidget<'a> {
    pub game: &'a GameState,
    pub selected_at_bat: Option<u8>,
    pub symbols: &'a Symbols,
}

impl Widget for MatchupWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (at_bat, is_current) = self
            .game
            .get_at_bat_by_index_or_current(self.selected_at_bat);

        let [matchup, on_deck] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        let [away, scoreboard, home] = Layout::horizontal([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .horizontal_margin(2)
        .areas(matchup);

        Widget::render(
            Paragraph::new(at_bat.matchup.format_team_lines(
                self.game.away_team.team_name,
                self.game.away_team.abbreviation,
                self.game.away_abs_challenges,
                false,
                is_current,
                &self.game.players,
                self.symbols,
            ))
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .padding(Padding::new(0, 0, 1, 0)),
            ),
            away,
            buf,
        );
        let mut scoreboard_lines = at_bat.matchup.format_scoreboard_lines(self.symbols);
        // Append weather line if available, colored by temperature and wind
        if let Some(weather) = &self.game.weather
            && let (Some(condition), Some(temp)) = (&weather.condition, &weather.temp)
        {
            let icon = self.symbols.weather_icon(condition);
            let temp_val: u8 = temp.parse().unwrap_or(70);
            let temp_color = Theme::temp_color(temp_val);

            let mut spans = vec![];
            if !icon.is_empty() {
                spans.push(Span::styled(
                    format!("{icon} "),
                    Style::default().fg(temp_color),
                ));
            }
            spans.push(Span::styled(
                format!("{temp}°F"),
                Style::default().fg(temp_color),
            ));

            if let Some(wind) = &weather.wind {
                let mph: u8 = wind
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let wind_color = Theme::wind_color(mph);
                let formatted = self.symbols.format_wind(wind);
                spans.push(Span::styled(
                    format!("  {formatted}"),
                    Style::default().fg(wind_color),
                ));
            }

            scoreboard_lines.push(Line::from(spans));
        }
        Widget::render(
            Paragraph::new(scoreboard_lines)
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::BOTTOM)
                        .padding(Padding::new(0, 0, 1, 0)),
                ),
            scoreboard,
            buf,
        );
        Widget::render(
            Paragraph::new(at_bat.matchup.format_team_lines(
                self.game.home_team.team_name,
                self.game.home_team.abbreviation,
                self.game.home_abs_challenges,
                true,
                is_current,
                &self.game.players,
                self.symbols,
            ))
            .alignment(Alignment::Right)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .padding(Padding::new(0, 0, 1, 0)),
            ),
            home,
            buf,
        );

        // only display on deck if it's the current at bat
        if is_current {
            // split into three chunks so that the center line is always exactly in the center
            let [od, split, ih] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(3),
                Constraint::Fill(1),
            ])
            .areas(on_deck);

            if let (Some(on_deck), Some(in_hole)) =
                (self.game.format_on_deck(), self.game.format_in_hole())
            {
                Widget::render(
                    Paragraph::new(on_deck)
                        .alignment(Alignment::Right)
                        .block(Block::default().padding(Padding::new(2, 0, 0, 0))),
                    od,
                    buf,
                );
                Widget::render(
                    Paragraph::new(" | ".to_string())
                        .alignment(Alignment::Center)
                        .block(Block::default().padding(Padding::new(0, 0, 0, 0))),
                    split,
                    buf,
                );
                Widget::render(
                    Paragraph::new(in_hole)
                        .alignment(Alignment::Left)
                        .block(Block::default().padding(Padding::new(0, 2, 0, 0))),
                    ih,
                    buf,
                );
            }
        }
    }
}
