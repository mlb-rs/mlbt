use crate::components::constants::{lookup_team, lookup_team_by_id};
use crate::components::standings::Team;
use crate::components::stats::splits::StatSplits;
use crate::components::util::{OptionDisplayExt, OptionMapDisplayExt, format_date};
use mlbt_api::player::PersonFull;
use mlbt_api::stats::{Split, StatSplit};
use tui::layout::Constraint;
use tui::prelude::{Line, Modifier, Style};
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
    /// True when the player's current team is a minor-league affiliate.
    pub is_minor_league: bool,
    pub bio: Vec<Line<'static>>,
    pub splits: StatSplits,
}

impl PlayerProfile {
    pub fn from_person(person: PersonFull) -> Self {
        let (team, is_minor_league) = Self::resolve_team(&person);
        let bio = Self::bio_lines(&person);

        Self {
            id: person.id,
            name: person.full_name,
            number: person.primary_number.display_or("--"),
            team,
            is_minor_league,
            bio,
            splits: StatSplits::from_stats(person.stats),
        }
    }

    /// Resolve the player's team. If their current team is a minor-league affiliate, look up the
    /// MLB parent team instead.
    fn resolve_team(person: &PersonFull) -> (Team, bool) {
        let current = person.current_team.as_ref();
        let team_name = current.map(|t| t.name.as_str()).unwrap_or_default();
        let team = lookup_team(team_name);

        // if lookup succeeded, this is an MLB team
        if team.id != 0 {
            return (team, false);
        }

        // for an unknown team, check if it has a parent org id (which should be a MLB team)
        if let Some(parent_id) = current.and_then(|t| t.parent_org_id)
            && let Some(parent) = lookup_team_by_id(parent_id)
        {
            return (parent, true);
        }

        // otherwise fallback to the default team
        (team, false)
    }

    /// Extract the player info and format it into lines to be rendered as a paragraph.
    fn bio_lines(person: &PersonFull) -> Vec<Line<'static>> {
        let position = person
            .primary_position
            .as_ref()
            .map(|p| &p.abbreviation)
            .display_or("-");
        let bats = person.bat_side.as_ref().map(|s| &s.code).display_or("-");
        let throws = person.pitch_hand.as_ref().map(|s| &s.code).display_or("-");

        let height = person.height.display_or("-");
        let weight = person.weight.map_display_or(|w| format!("{w}lb"), "");
        let age = person.current_age.display_or("-");

        let birth_date = person.birth_date.map_display_or(|d| format_date(d), "---");
        let birthplace = [
            person.birth_city.as_deref(),
            person.birth_state_province.as_deref(),
            person.birth_country.as_deref(),
        ]
        .iter()
        .filter_map(|s| *s)
        .collect::<Vec<_>>()
        .join(", ");

        // TODO fetch draft details (round, pick, team, college) from /draft endpoint
        let draft_year = person.draft_year.display_or("---");
        let mlb_debut = person
            .mlb_debut_date
            .map_display_or(|d| format_date(d), "---");

        let mut bio = vec![
            format!("{position} | {bats}/{throws} | {height} {weight} | Age: {age}").into(),
            format!("Born: {birth_date} in {birthplace}").into(),
            format!("Drafted: {draft_year}").into(),
            format!("MLB Debut: {mlb_debut}").into(),
        ];

        // TODO fetch IL info from the api with hydration=rosterEntries
        if let Some(active) = person.active {
            let status = if active { "Active" } else { "Inactive" };
            bio.push(format!("Status: {status}").into());
        };

        bio
    }

    fn split_to_cells(split: &Split, show_year: bool) -> Vec<Cell<'_>> {
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
                    s.avg.as_str().into(),
                    s.obp.as_str().into(),
                    s.slg.as_str().into(),
                    s.ops.as_str().into(),
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
                    s.era.as_str().into(),
                    s.games_played.to_string().into(),
                    s.games_started.to_string().into(),
                    s.saves.to_string().into(),
                    s.innings_pitched.as_str().into(),
                    s.hits.to_string().into(),
                    s.runs.to_string().into(),
                    s.earned_runs.to_string().into(),
                    s.home_runs.to_string().into(),
                    s.base_on_balls.to_string().into(),
                    s.strike_outs.to_string().into(),
                    s.whip.as_str().into(),
                ]);
            }
        }
        cells
    }

    fn game_log_cells(split: &Split) -> Vec<Cell<'_>> {
        let date = split.date.map_display_or(|d| format_date(d), "");
        let opp = split
            .opponent
            .map_display_or(|o| lookup_team(&o.name).abbreviation, "---");
        let prefix = if split.is_home == Some(true) {
            "vs"
        } else {
            "@"
        };
        // whether the team won/lost, not the pitcher's game decision
        let result = match split.is_win {
            Some(true) => "W",
            Some(false) => "L",
            None => "-",
        };

        let mut cells = vec![date.into(), result.into(), format!("{prefix} {opp}").into()];

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
                    s.avg.as_str().into(),
                ]);
            }
            StatSplit::Pitching(s) => {
                cells.extend([
                    s.innings_pitched.as_str().into(),
                    s.hits.to_string().into(),
                    s.runs.to_string().into(),
                    s.earned_runs.to_string().into(),
                    s.home_runs.to_string().into(),
                    s.base_on_balls.to_string().into(),
                    s.strike_outs.to_string().into(),
                    s.era.as_str().into(),
                ]);
            }
        }
        cells
    }

    pub fn career_total_cells(split: &Split) -> Vec<Cell<'_>> {
        let mut cells = vec!["".into(), "TOT".into()];
        cells.extend(Self::split_to_cells(split, false));
        cells
    }

    /// Build header row, column widths, and data rows for a stat table.
    pub fn build_stat_rows(
        splits: &[Split],
        show_year: bool,
    ) -> Option<(Row<'_>, Vec<Constraint>, Vec<Row<'_>>)> {
        let first = splits.first()?;
        let headers = if matches!(&first.stat, StatSplit::Hitting(_)) {
            HITTING_HEADERS
        } else {
            PITCHING_HEADERS
        };

        let mut names = Vec::with_capacity(headers.len() + 2);
        let mut widths = Vec::with_capacity(headers.len() + 2);

        if show_year {
            names.extend_from_slice(&["Year", "Team"]);
            widths.extend([Constraint::Length(6), Constraint::Length(5)]);
        }

        names.extend_from_slice(headers);
        widths.resize(names.len(), Constraint::Length(STAT_COL_WIDTH));

        let header =
            Row::new(names).style(Style::default().bold().add_modifier(Modifier::UNDERLINED));

        let rows = splits
            .iter()
            .map(|split| Row::new(Self::split_to_cells(split, show_year)))
            .collect();

        Some((header, widths, rows))
    }

    /// Build header row, column widths, and data rows for the game log table.
    pub fn build_game_log_rows(
        splits: &[Split],
    ) -> Option<(Row<'_>, Vec<Constraint>, Vec<Row<'_>>)> {
        let first = splits.first()?;
        let headers = if matches!(&first.stat, StatSplit::Hitting(_)) {
            GAME_LOG_HITTING_HEADERS
        } else {
            GAME_LOG_PITCHING_HEADERS
        };
        let mut widths = Vec::with_capacity(headers.len());
        widths.extend_from_slice(GAME_LOG_PREFIX_WIDTHS);
        widths.resize(headers.len(), Constraint::Length(STAT_COL_WIDTH));

        let header = Row::new(headers.to_vec())
            .style(Style::default().bold().add_modifier(Modifier::UNDERLINED));

        let rows = splits
            .iter()
            .rev()
            .map(|split| Row::new(Self::game_log_cells(split)))
            .collect();

        Some((header, widths, rows))
    }
}
