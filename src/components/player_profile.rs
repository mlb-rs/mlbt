use crate::components::constants::lookup_team;
use crate::components::standings::Team;
use crate::components::util::{OptionDisplayExt, OptionMapDisplayExt, format_date};
use mlbt_api::player::PersonFull;
use mlbt_api::stats::{Split, Stat, StatSplit};
use tui::layout::Constraint;
use tui::prelude::{Modifier, Style};
use tui::text::Line;
use tui::widgets::{Cell, Row};

const STAT_COL_WIDTH: u16 = 6;

const HITTING_HEADERS: &[&str] = &[
    "G", "AB", "AVG", "OBP", "SLG", "OPS", "R", "H", "2B", "3B", "HR", "RBI", "BB", "SO", "SB",
    "CS",
];
const PITCHING_HEADERS: &[&str] = &[
    "W", "L", "ERA", "G", "GS", "SV", "IP", "H", "R", "ER", "HR", "BB", "SO", "WHIP",
];
const GAME_LOG_HITTING_HEADERS: &[&str] = &[
    "Date", "", "Opp", "AB", "R", "H", "2B", "3B", "HR", "RBI", "BB", "SO", "SB", "CS", "AVG",
];
const GAME_LOG_PITCHING_HEADERS: &[&str] = &[
    "Date", "", "Opp", "IP", "H", "R", "ER", "HR", "BB", "SO", "ERA",
];
const GAME_LOG_PREFIX_WIDTHS: &[Constraint] = &[
    Constraint::Length(11), // date
    Constraint::Length(2),  // W/L
    Constraint::Length(8),  // opp (@ CHC)
];

/// Player profile data for display. All fields are resolved from the API's optional fields with
/// sensible defaults.
pub struct PlayerProfile {
    pub id: u64,
    pub name: String,
    pub number: String,
    pub team: Team,
    pub position: String,
    pub bats: String,
    pub throws: String,
    pub height: String,
    pub weight: String,
    pub age: String,
    pub birth_date: String,
    pub birthplace: String,
    pub draft_year: String,
    pub mlb_debut: String,
    pub stats: Vec<Stat>,
}

impl PlayerProfile {
    pub fn from_person(person: PersonFull) -> Self {
        let team_name = person
            .current_team
            .as_ref()
            .map(|t| t.name.as_str())
            .unwrap_or_default();

        let birthplace = [
            person.birth_city.as_deref(),
            person.birth_state_province.as_deref(),
            person.birth_country.as_deref(),
        ]
        .iter()
        .filter_map(|s| *s)
        .collect::<Vec<_>>()
        .join(", ");

        Self {
            id: person.id,
            name: person.full_name,
            number: person.primary_number.display_or("--"),
            team: lookup_team(team_name),
            position: person
                .primary_position
                .as_ref()
                .map(|p| &p.abbreviation)
                .display_or("-"),
            bats: person.bat_side.as_ref().map(|s| &s.code).display_or("-"),
            throws: person.pitch_hand.as_ref().map(|s| &s.code).display_or("-"),
            height: person.height.display_or("-"),
            weight: person.weight.map_display_or(|w| format!("{w}lb"), ""),
            age: person.current_age.display_or("-"),
            birth_date: person.birth_date.map_display_or(|d| format_date(d), "---"),
            birthplace,
            // TODO fetch draft details (round, pick, team, college) from /draft endpoint
            draft_year: person.draft_year.display_or("---"),
            mlb_debut: person
                .mlb_debut_date
                .map_display_or(|d| format_date(d), "---"),
            stats: person.stats,
        }
    }

    pub fn bio_lines(&self) -> Vec<Line<'_>> {
        vec![
            Line::from(format!(
                "{} | {}/{} | {} {} | Age: {}",
                self.position, self.bats, self.throws, self.height, self.weight, self.age
            )),
            Line::from(format!("Born: {} in {}", self.birth_date, self.birthplace)),
            Line::from(format!("Drafted: {}", self.draft_year)),
            Line::from(format!("MLB Debut: {}", self.mlb_debut)),
        ]
    }

    /// Find a specific stat group by type name, e.g. "season" or "yearByYear".
    pub fn find_stat_group(&self, type_name: &str) -> Option<&Stat> {
        self.stats
            .iter()
            .find(|s| s.stat_type.display_name == type_name)
    }

    pub fn split_to_cells(split: &Split, show_year: bool) -> Vec<Cell<'static>> {
        let mut cells = Vec::new();

        if show_year {
            cells.push(split.season.display_or("").into());
            cells.push(
                split
                    .team
                    .map_display_or(|t| lookup_team(&t.name).abbreviation, "---")
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

    pub fn game_log_cells(split: &Split) -> Vec<Cell<'static>> {
        let date = split.date.map_display_or(|d| format_date(d), "");
        let opp = split
            .opponent
            .map_display_or(|o| lookup_team(&o.name).abbreviation, "---");
        let prefix = if split.is_home == Some(true) {
            "vs"
        } else {
            "@"
        };
        let result = if split.is_win == Some(true) { "W" } else { "L" };

        let mut cells: Vec<Cell> = vec![
            date.into(),
            result.to_string().into(),
            format!("{prefix} {opp}").into(),
        ];

        match &split.stat {
            StatSplit::Hitting(s) => {
                cells.extend([
                    s.at_bats.to_string().into(),
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
                    s.avg.clone().into(),
                ]);
            }
            StatSplit::Pitching(s) => {
                cells.extend([
                    s.innings_pitched.clone().into(),
                    s.hits.to_string().into(),
                    s.runs.to_string().into(),
                    s.earned_runs.to_string().into(),
                    s.home_runs.to_string().into(),
                    s.base_on_balls.to_string().into(),
                    s.strike_outs.to_string().into(),
                    s.era.clone().into(),
                ]);
            }
        }
        cells
    }

    pub fn career_total_cells(split: &Split) -> Vec<Cell<'static>> {
        let mut cells: Vec<Cell> = vec!["".into(), "TOT".into()];
        cells.extend(Self::split_to_cells(split, false));
        cells
    }

    /// Build header row, column widths, and data rows for a stat table.
    pub fn build_stat_rows(
        splits: &[Split],
        show_year: bool,
    ) -> (Row<'static>, Vec<Constraint>, Vec<Row<'static>>) {
        let is_hitting = matches!(&splits[0].stat, StatSplit::Hitting(_));

        let mut names = if is_hitting {
            HITTING_HEADERS.to_vec()
        } else {
            PITCHING_HEADERS.to_vec()
        };
        let mut widths = vec![Constraint::Length(STAT_COL_WIDTH); names.len()];

        if show_year {
            names.splice(0..0, vec!["Year", "Team"]);
            widths.splice(0..0, vec![Constraint::Length(6), Constraint::Length(5)]);
        }

        let header =
            Row::new(names).style(Style::default().bold().add_modifier(Modifier::UNDERLINED));

        let rows: Vec<Row> = splits
            .iter()
            .map(|split| Row::new(Self::split_to_cells(split, show_year)))
            .collect();

        (header, widths, rows)
    }

    /// Build header row, column widths, and data rows for the game log table.
    pub fn build_game_log_rows(
        splits: &[Split],
    ) -> (Row<'static>, Vec<Constraint>, Vec<Row<'static>>) {
        let recent: Vec<&Split> = splits.iter().rev().take(15).collect();
        let is_hitting = matches!(&recent[0].stat, StatSplit::Hitting(_));

        let headers = if is_hitting {
            GAME_LOG_HITTING_HEADERS.to_vec()
        } else {
            GAME_LOG_PITCHING_HEADERS.to_vec()
        };
        let stat_col_count = headers.len() - 3; // subtract prefix columns
        let mut widths: Vec<Constraint> = GAME_LOG_PREFIX_WIDTHS.to_vec();
        widths.extend(vec![Constraint::Length(STAT_COL_WIDTH); stat_col_count]);

        let header =
            Row::new(headers).style(Style::default().bold().add_modifier(Modifier::UNDERLINED));

        let rows: Vec<Row> = recent
            .iter()
            .map(|split| Row::new(Self::game_log_cells(split)))
            .collect();

        (header, widths, rows)
    }
}
