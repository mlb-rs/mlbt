use crate::components::constants::lookup_team;
use crate::state::player_profile::PlayerProfileState;
use chrono::NaiveDate;
use mlbt_api::season::GameType;
use mlbt_api::stats::{Split, StatSplit};
use tui::prelude::*;
use tui::widgets::{Block, BorderType, Borders, Cell, Padding, Paragraph, Row, Table};

/// Format "YYYY-MM-DD" as "M/D/YYYY", or return the original string if parsing fails.
fn format_date(s: &str) -> String {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map(|d| d.format("%-m/%-d/%Y").to_string())
        .unwrap_or_else(|_| s.to_string())
}

const STAT_COL_WIDTH: u16 = 6;

fn section_title_style() -> Style {
    Style::default().bg(Color::Blue).fg(Color::Black)
}

pub struct PlayerProfileWidget<'a> {
    pub state: &'a mut PlayerProfileState,
}

impl<'a> Widget for PlayerProfileWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let bio = &self.state.data;
        let number = bio.primary_number.as_deref().unwrap_or("--");
        let team_abbrev = bio
            .current_team
            .as_ref()
            .map(|t| lookup_team(&t.name).abbreviation.to_string())
            .unwrap_or_default();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .padding(Padding::new(1, 1, 0, 0))
            .title(Line::from(vec![
                Span::styled(
                    format!(" #{number} {} ", bio.full_name),
                    Style::default().fg(Color::Black).bg(Color::Blue),
                ),
                Span::styled(
                    format!(" {team_abbrev} "),
                    Style::default().fg(Color::Black).bg(Color::Cyan),
                ),
            ]));
        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        // Calculate section heights
        let bio_height = 4;

        let season_splits = self
            .state
            .find_stat_group("season")
            .map(|s| s.splits.len())
            .unwrap_or(0);
        // With data: title + header + rows. Empty: title + "No data"
        let season_height = if season_splits > 0 {
            season_splits as u16 + 2
        } else {
            2
        };

        let yby_splits = self
            .state
            .find_stat_group("yearByYear")
            .map(|s| s.splits.len())
            .unwrap_or(0);
        let career_splits = self
            .state
            .find_stat_group("career")
            .map(|s| s.splits.len())
            .unwrap_or(0);
        let career_height = if yby_splits > 0 {
            yby_splits as u16 + career_splits as u16 + 2 // career_splits adds the totals row
        } else {
            0
        };

        let constraints = [
            Constraint::Length(bio_height + 1), // +1 for blank line between each section
            Constraint::Length(season_height + 1),
            Constraint::Length(career_height + 1),
            Constraint::Fill(1), // game log gets remaining space
        ];
        let [bio_area, season_area, career_area, gamelog_area] =
            Layout::vertical(constraints).areas(inner);

        self.render_bio(bio_area, buf);

        let season_title = format!("{} Season", self.state.season_year);
        let season_splits = self
            .state
            .find_stat_group("season")
            .map(|s| s.splits.as_slice())
            .unwrap_or(&[]);
        self.render_section(&season_title, season_splits, false, season_area, buf);

        if let Some(yby) = self.state.find_stat_group("yearByYear") {
            let career_totals = self.state.find_stat_group("career").map(|c| &c.splits);
            self.render_section_with_totals(
                "Career Stats",
                &yby.splits,
                career_totals,
                career_area,
                buf,
            );
        }

        if let Some(game_log) = self.state.find_stat_group("gameLog") {
            self.render_game_log(&game_log.splits, gamelog_area, buf);
        }
    }
}

impl PlayerProfileWidget<'_> {
    fn render_bio(&self, area: Rect, buf: &mut Buffer) {
        // Split bio area: text on left, game type selector on right
        let [bio_text_area, game_type_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(20)]).areas(area);

        self.render_game_type_selector(game_type_area, buf);

        let area = bio_text_area;
        let bio = &self.state.data;

        let position = bio
            .primary_position
            .as_ref()
            .map(|p| p.abbreviation.as_str())
            .unwrap_or("-");
        let bat = bio
            .bat_side
            .as_ref()
            .map(|s| s.code.as_str())
            .unwrap_or("-");
        let throw = bio
            .pitch_hand
            .as_ref()
            .map(|s| s.code.as_str())
            .unwrap_or("-");
        let height = bio.height.as_deref().unwrap_or("-");
        let weight = bio.weight.map(|w| format!("{w}lb")).unwrap_or_default();
        let age = bio
            .current_age
            .map(|a| format!("Age: {a}"))
            .unwrap_or_default();

        let birth_date = bio
            .birth_date
            .as_deref()
            .map(format_date)
            .unwrap_or_else(|| "---".to_string());
        let birthplace = [
            bio.birth_city.as_deref(),
            bio.birth_state_province.as_deref(),
            bio.birth_country.as_deref(),
        ]
        .iter()
        .filter_map(|s| *s)
        .collect::<Vec<_>>()
        .join(", ");

        // TODO fetch draft details (round, pick, team, college) from /draft endpoint
        let draft = bio
            .draft_year
            .map(|y| y.to_string())
            .unwrap_or_else(|| "---".to_string());

        let debut = bio
            .mlb_debut_date
            .as_deref()
            .map(format_date)
            .unwrap_or_else(|| "---".to_string());

        let text = vec![
            Line::from(format!(
                "{position} | {bat}/{throw} | {height} {weight} | {age}"
            )),
            Line::from(format!("Born: {birth_date} in {birthplace}")),
            Line::from(format!("Drafted: {draft}")),
            Line::from(format!("MLB Debut: {debut}")),
        ];
        Paragraph::new(text).render(area, buf);
    }

    fn render_game_type_selector(&self, area: Rect, buf: &mut Buffer) {
        let selected = Style::default().fg(Color::Black).bg(Color::Blue);
        let normal = Style::default().fg(Color::DarkGray);

        let (reg_style, st_style) = match self.state.game_type {
            GameType::RegularSeason => (selected, normal),
            GameType::SpringTraining => (normal, selected),
        };

        let text = vec![
            Line::from(Span::styled(" Regular Season  ", reg_style)),
            Line::from(Span::styled(" Spring Training ", st_style)),
        ];
        Paragraph::new(text)
            .alignment(Alignment::Right)
            .render(area, buf);
    }

    fn render_section(
        &self,
        title: &str,
        splits: &[Split],
        show_year: bool,
        area: Rect,
        buf: &mut Buffer,
    ) {
        if area.height < 2 {
            return;
        }
        if splits.is_empty() {
            let [title_area, msg_area] =
                Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);
            Paragraph::new(Line::from(Span::styled(
                format!(" {title} "),
                section_title_style(),
            )))
            .render(title_area, buf);
            Paragraph::new(Span::styled(
                "  No data",
                Style::default().fg(Color::DarkGray),
            ))
            .render(msg_area, buf);
            return;
        }
        let (header, widths, rows) = self.build_stat_rows(splits, show_year);
        self.render_stat_table(title, header, &widths, rows, area, buf);
    }

    fn render_section_with_totals(
        &self,
        title: &str,
        splits: &[Split],
        career: Option<&Vec<Split>>,
        area: Rect,
        buf: &mut Buffer,
    ) {
        if splits.is_empty() || area.height < 2 {
            return;
        }
        let (header, widths, mut rows) = self.build_stat_rows(splits, true);

        // Append career totals as a bold row
        if let Some(career_splits) = career {
            for split in career_splits {
                let mut cells = self.split_to_cells(split, true);
                // Replace year/team with "TOT"
                if cells.len() >= 2 {
                    cells[0] = Cell::from("");
                    cells[1] = Cell::from("TOT");
                }
                rows.push(Row::new(cells).style(Style::default().bold()));
            }
        }

        self.render_stat_table(title, header, &widths, rows, area, buf);
    }

    fn render_game_log(&self, splits: &[Split], area: Rect, buf: &mut Buffer) {
        if splits.is_empty() || area.height < 2 {
            return;
        }

        let recent: Vec<&Split> = splits.iter().rev().take(15).collect();

        let is_hitting = matches!(&recent[0].stat, StatSplit::Hitting(_));

        let prefix_widths = vec![
            Constraint::Length(11), // date
            Constraint::Length(2),  // W/L
            Constraint::Length(8),  // opp (@ LAD)
        ];
        let (header, stat_names) = if is_hitting {
            (
                vec![
                    "Date", "", "Opp", "AB", "R", "H", "2B", "3B", "HR", "RBI", "BB", "SO", "SB",
                    "CS", "AVG",
                ],
                15 - 3, // stat columns count
            )
        } else {
            (
                vec![
                    "Date", "", "Opp", "IP", "H", "R", "ER", "HR", "BB", "SO", "ERA",
                ],
                11 - 3,
            )
        };
        let mut widths = prefix_widths;
        widths.extend(vec![Constraint::Length(STAT_COL_WIDTH); stat_names]);

        let rows: Vec<Row> = recent
            .iter()
            .map(|split| {
                let date = split.date.as_deref().map(format_date).unwrap_or_default();
                let opp = split
                    .opponent
                    .as_ref()
                    .map(|o| lookup_team(&o.name).abbreviation.to_string())
                    .unwrap_or_else(|| "---".to_string());
                let prefix = if split.is_home == Some(true) {
                    "vs"
                } else {
                    "@"
                };
                let result = if split.is_win == Some(true) { "W" } else { "L" };
                let loc_opp = format!("{prefix} {opp}");

                let stat_cells: Vec<Cell> = match &split.stat {
                    StatSplit::Hitting(s) => vec![
                        Cell::from(s.at_bats.to_string()),
                        Cell::from(s.runs.to_string()),
                        Cell::from(s.hits.to_string()),
                        Cell::from(s.doubles.to_string()),
                        Cell::from(s.triples.to_string()),
                        Cell::from(s.home_runs.to_string()),
                        Cell::from(s.rbi.to_string()),
                        Cell::from(s.base_on_balls.to_string()),
                        Cell::from(s.strike_outs.to_string()),
                        Cell::from(s.stolen_bases.to_string()),
                        Cell::from(s.caught_stealing.to_string()),
                        Cell::from(s.avg.clone()),
                    ],
                    StatSplit::Pitching(s) => vec![
                        Cell::from(s.innings_pitched.clone()),
                        Cell::from(s.hits.to_string()),
                        Cell::from(s.runs.to_string()),
                        Cell::from(s.earned_runs.to_string()),
                        Cell::from(s.home_runs.to_string()),
                        Cell::from(s.base_on_balls.to_string()),
                        Cell::from(s.strike_outs.to_string()),
                        Cell::from(s.era.clone()),
                    ],
                };

                let mut cells = vec![
                    Cell::from(date.to_string()),
                    Cell::from(result.to_string()),
                    Cell::from(loc_opp),
                ];
                cells.extend(stat_cells);
                Row::new(cells)
            })
            .collect();

        let header_row = Row::new(header.iter().map(|h| Cell::from(*h)).collect::<Vec<_>>())
            .style(Style::default().bold().add_modifier(Modifier::UNDERLINED));

        let [title_area, table_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

        Paragraph::new(Line::from(Span::styled(
            " Recent Games ",
            section_title_style(),
        )))
        .render(title_area, buf);

        let table = Table::new(rows, &widths)
            .header(header_row)
            .column_spacing(0);
        Widget::render(table, table_area, buf);
    }

    fn build_stat_rows(
        &self,
        splits: &[Split],
        show_year: bool,
    ) -> (Row<'static>, Vec<Constraint>, Vec<Row<'static>>) {
        let is_hitting = matches!(&splits[0].stat, StatSplit::Hitting(_));

        let (header_names, widths) = if is_hitting {
            let mut names = vec![
                "G", "AB", "AVG", "OBP", "SLG", "OPS", "R", "H", "2B", "3B", "HR", "RBI", "BB",
                "SO", "SB", "CS",
            ];
            let mut w = vec![Constraint::Length(STAT_COL_WIDTH); names.len()];
            if show_year {
                names.insert(0, "Team");
                names.insert(0, "Year");
                w.insert(0, Constraint::Length(5));
                w.insert(0, Constraint::Length(6));
            }
            (names, w)
        } else {
            let mut names = vec![
                "W", "L", "ERA", "G", "GS", "SV", "IP", "H", "R", "ER", "HR", "BB", "SO", "WHIP",
            ];
            let mut w = vec![Constraint::Length(STAT_COL_WIDTH); names.len()];
            if show_year {
                names.insert(0, "Team");
                names.insert(0, "Year");
                w.insert(0, Constraint::Length(5));
                w.insert(0, Constraint::Length(6));
            }
            (names, w)
        };

        let header = Row::new(header_names.into_iter().map(Cell::from).collect::<Vec<_>>())
            .style(Style::default().bold().add_modifier(Modifier::UNDERLINED));

        let rows: Vec<Row> = splits
            .iter()
            .map(|split| Row::new(self.split_to_cells(split, show_year)))
            .collect();

        (header, widths, rows)
    }

    fn split_to_cells(&self, split: &Split, show_year: bool) -> Vec<Cell<'static>> {
        let mut cells = Vec::new();

        if show_year {
            cells.push(split.season.clone().unwrap_or_default().into());
            cells.push(
                split
                    .team
                    .as_ref()
                    .map(|t| lookup_team(&t.name).abbreviation.to_string())
                    .unwrap_or_else(|| "---".to_string())
                    .into(),
            );
        }

        match &split.stat {
            StatSplit::Hitting(s) => {
                cells.extend([
                    s.games_played.to_string().into(),
                    s.at_bats.to_string().into(),
                    s.avg.clone().into(),
                    s.obp.clone().into(),
                    s.slg.clone().into(),
                    s.ops.clone().into(),
                    s.runs.to_string().into(),
                    s.hits.to_string().into(),
                    s.doubles.to_string().into(),
                    s.triples.to_string().into(),
                    s.home_runs.to_string().into(),
                    s.rbi.to_string().into(),
                    s.base_on_balls.to_string().into(),
                    s.strike_outs.to_string().into(),
                    s.stolen_bases.to_string().into(),
                    s.caught_stealing.to_string().into(),
                ]);
            }
            StatSplit::Pitching(s) => {
                cells.extend([
                    s.wins.to_string().into(),
                    s.losses.to_string().into(),
                    s.era.clone().into(),
                    s.games_played.to_string().into(),
                    s.games_started.to_string().into(),
                    s.saves.to_string().into(),
                    s.innings_pitched.clone().into(),
                    s.hits.to_string().into(),
                    s.runs.to_string().into(),
                    s.earned_runs.to_string().into(),
                    s.home_runs.to_string().into(),
                    s.base_on_balls.to_string().into(),
                    s.strike_outs.to_string().into(),
                    s.whip.clone().into(),
                ]);
            }
        }
        cells
    }

    fn render_stat_table(
        &self,
        title: &str,
        header: Row<'static>,
        widths: &[Constraint],
        rows: Vec<Row<'static>>,
        area: Rect,
        buf: &mut Buffer,
    ) {
        // Render section title on the first line, table below
        let [title_area, table_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

        Paragraph::new(Line::from(Span::styled(
            format!(" {title} "),
            section_title_style(),
        )))
        .render(title_area, buf);

        let table = Table::new(rows, widths).header(header).column_spacing(0);
        Widget::render(table, table_area, buf);
    }
}
