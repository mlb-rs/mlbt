use crate::components::constants::{lookup_team, lookup_team_by_id};
use crate::components::standings::Team;
use crate::components::stats::splits::{RecentSplit, RecentStats, StatSplits};
use crate::components::team_colors;
use crate::components::util::{
    DimColor, OptionDisplayExt, OptionMapDisplayExt, avg_color, era_color, format_date,
    obp_color, slg_color, ops_color, whip_color,
};
use crate::symbols::Symbols;
use mlbt_api::player::PersonFull;
use mlbt_api::stats::{Split, StatSplit};
use tui::layout::Constraint;
use tui::prelude::{Line, Modifier, Style, Stylize};
use tui::style::Color;
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
const SPLITS_HITTING_HEADERS: &[&str] = &[
    "Duration", "AB", "R", "H", "HR", "RBI", "BB", "SO", "SB", "AVG", "OBP", "SLG",
];
const SPLITS_PITCHING_HEADERS: &[&str] = &[
    "Duration", "W", "L", "ERA", "G", "GS", "SV", "IP", "H", "ER", "BB", "SO", "WHIP",
];
const SPLITS_DURATION_WIDTH: u16 = 16;

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

        let draft_year = person.draft_year.display_or("---");
        let mut draft = format!("Drafted: {draft_year}");
        // find draft details for the player's draft year, if any
        if let Some(info) = person
            .drafts
            .as_deref()
            .and_then(|drafts| drafts.iter().find(|d| d.year == draft_year))
        {
            draft.push_str(&format!(
                ", {}, Round: {}, Overall Pick: {}",
                info.team.name, info.pick_round, info.pick_number
            ));
        }

        let mlb_debut = person
            .mlb_debut_date
            .map_display_or(|d| format_date(d), "---");

        let mut bio = vec![
            format!("{position} | {bats}/{throws} | {height} {weight} | Age: {age}").into(),
            format!("Born: {birth_date} in {birthplace}").into(),
            draft.into(),
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
                    Cell::from(s.games_played.to_string()).fg(s.games_played.dim_or(Color::White)),
                    Cell::from(s.at_bats.to_string()).fg(s.at_bats.dim_or(Color::White)),
                    Cell::from(s.avg.as_str())
                        .fg(avg_color(s.avg.as_str()).unwrap_or(Color::White)),
                    Cell::from(s.obp.as_str()).fg(obp_color(s.obp.as_str()).unwrap_or(Color::White)),
                    Cell::from(s.slg.as_str()).fg(slg_color(s.slg.as_str()).unwrap_or(Color::White)),
                    Cell::from(s.ops.as_str()).fg(ops_color(s.ops.as_str()).unwrap_or(Color::White)),
                    Cell::from(s.runs.to_string()).fg(s.runs.dim_or(Color::White)),
                    Cell::from(s.hits.to_string()).fg(s.hits.dim_or(Color::White)),
                    Cell::from(s.doubles.to_string()).fg(s.doubles.dim_or(Color::White)),
                    Cell::from(s.triples.to_string()).fg(s.triples.dim_or(Color::White)),
                    Cell::from(s.home_runs.to_string()).fg(s.home_runs.dim_or(Color::White)),
                    Cell::from(s.rbi.to_string()).fg(s.rbi.dim_or(Color::White)),
                    Cell::from(s.base_on_balls.to_string())
                        .fg(s.base_on_balls.dim_or(Color::White)),
                    Cell::from(s.strike_outs.to_string()).fg(s.strike_outs.dim_or(Color::White)),
                    Cell::from(s.stolen_bases.to_string()).fg(s.stolen_bases.dim_or(Color::White)),
                    Cell::from(s.caught_stealing.to_string())
                        .fg(s.caught_stealing.dim_or(Color::White)),
                ]);
            }
            StatSplit::Pitching(s) => {
                cells.extend([
                    Cell::from(s.wins.to_string()).fg(s.wins.dim_or(Color::White)),
                    Cell::from(s.losses.to_string()).fg(s.losses.dim_or(Color::White)),
                    Cell::from(s.era.as_str())
                        .fg(era_color(s.era.as_str()).unwrap_or(Color::White)),
                    Cell::from(s.games_played.to_string()).fg(s.games_played.dim_or(Color::White)),
                    Cell::from(s.games_started.to_string())
                        .fg(s.games_started.dim_or(Color::White)),
                    Cell::from(s.saves.to_string()).fg(s.saves.dim_or(Color::White)),
                    s.innings_pitched.as_str().into(),
                    Cell::from(s.hits.to_string()).fg(s.hits.dim_or(Color::White)),
                    Cell::from(s.runs.to_string()).fg(s.runs.dim_or(Color::White)),
                    Cell::from(s.earned_runs.to_string()).fg(s.earned_runs.dim_or(Color::White)),
                    Cell::from(s.home_runs.to_string()).fg(s.home_runs.dim_or(Color::White)),
                    Cell::from(s.base_on_balls.to_string())
                        .fg(s.base_on_balls.dim_or(Color::White)),
                    Cell::from(s.strike_outs.to_string()).fg(s.strike_outs.dim_or(Color::White)),
                    Cell::from(s.whip.as_str()).fg(whip_color(s.whip.as_str()).unwrap_or(Color::White)),
                ]);
            }
        }
        cells
    }

    fn game_log_cells<'a>(split: &'a Split, symbols: &Symbols) -> Vec<Cell<'a>> {
        let date = split.date.map_display_or(|d| format_date(d), "");
        let prefix = if split.is_home == Some(true) { "vs" } else { "@" };

        let opp_team = split
            .opponent
            .as_ref()
            .map(|o| lookup_team(&o.name));
        let opp_abbr = opp_team.map(|t| t.abbreviation).unwrap_or("---");

        // whether the team won/lost, not the pitcher's game decision
        let result_cell = match split.is_win {
            Some(true) => Cell::from("W").fg(Color::Green),
            Some(false) => Cell::from("L").fg(Color::Red),
            None => Cell::from("-"),
        };

        let opp_cell = if symbols.team_colors() {
            let color = team_colors::get(opp_abbr, false).unwrap_or(Color::White);
            Cell::from(format!("{prefix} {opp_abbr}")).fg(color)
        } else {
            Cell::from(format!("{prefix} {opp_abbr}"))
        };

        let mut cells = vec![date.into(), result_cell, opp_cell];

        match &split.stat {
            StatSplit::Hitting(s) => {
                cells.extend([
                    Cell::from(s.at_bats.to_string()).fg(s.at_bats.dim_or(Color::White)),
                    Cell::from(s.runs.to_string()).fg(s.runs.dim_or(Color::White)),
                    Cell::from(s.hits.to_string()).fg(s.hits.dim_or(Color::White)),
                    Cell::from(s.doubles.to_string()).fg(s.doubles.dim_or(Color::White)),
                    Cell::from(s.triples.to_string()).fg(s.triples.dim_or(Color::White)),
                    Cell::from(s.home_runs.to_string()).fg(s.home_runs.dim_or(Color::White)),
                    Cell::from(s.rbi.to_string()).fg(s.rbi.dim_or(Color::White)),
                    Cell::from(s.base_on_balls.to_string())
                        .fg(s.base_on_balls.dim_or(Color::White)),
                    Cell::from(s.strike_outs.to_string()).fg(s.strike_outs.dim_or(Color::White)),
                    Cell::from(s.stolen_bases.to_string()).fg(s.stolen_bases.dim_or(Color::White)),
                    Cell::from(s.caught_stealing.to_string())
                        .fg(s.caught_stealing.dim_or(Color::White)),
                    Cell::from(s.avg.as_str())
                        .fg(avg_color(s.avg.as_str()).unwrap_or(Color::White)),
                ]);
            }
            StatSplit::Pitching(s) => {
                cells.extend([
                    s.innings_pitched.as_str().into(),
                    Cell::from(s.hits.to_string()).fg(s.hits.dim_or(Color::White)),
                    Cell::from(s.runs.to_string()).fg(s.runs.dim_or(Color::White)),
                    Cell::from(s.earned_runs.to_string()).fg(s.earned_runs.dim_or(Color::White)),
                    Cell::from(s.home_runs.to_string()).fg(s.home_runs.dim_or(Color::White)),
                    Cell::from(s.base_on_balls.to_string())
                        .fg(s.base_on_balls.dim_or(Color::White)),
                    Cell::from(s.strike_outs.to_string()).fg(s.strike_outs.dim_or(Color::White)),
                    Cell::from(s.era.as_str())
                        .fg(era_color(s.era.as_str()).unwrap_or(Color::White)),
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
    pub fn build_game_log_rows<'a>(
        splits: &'a [Split],
        symbols: &Symbols,
    ) -> Option<(Row<'a>, Vec<Constraint>, Vec<Row<'a>>)> {
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
            .map(|split| Row::new(Self::game_log_cells(split, symbols)))
            .collect();

        Some((header, widths, rows))
    }

    /// Build header row, column widths, and data rows for the recent splits table.
    pub fn build_splits_rows(
        recent_splits: &[RecentSplit],
        is_hitting: bool,
    ) -> Option<(Row<'_>, Vec<Constraint>, Vec<Row<'_>>)> {
        if !recent_splits.iter().any(|s| s.stat.is_some()) {
            return None;
        }

        let headers = if is_hitting {
            SPLITS_HITTING_HEADERS
        } else {
            SPLITS_PITCHING_HEADERS
        };

        let mut widths = vec![Constraint::Length(SPLITS_DURATION_WIDTH)];
        widths.resize(headers.len(), Constraint::Length(STAT_COL_WIDTH));

        let header = Row::new(headers.to_vec())
            .style(Style::default().bold().add_modifier(Modifier::UNDERLINED));

        let rows = recent_splits
            .iter()
            .map(|split| {
                let mut cells: Vec<Cell> = vec![split.label.into()];
                match &split.stat {
                    Some(RecentStats::Hitting(s)) => {
                        cells.extend([
                            Cell::from(s.ab.to_string()).fg(s.ab.dim_or(Color::White)),
                            Cell::from(s.r.to_string()).fg(s.r.dim_or(Color::White)),
                            Cell::from(s.h.to_string()).fg(s.h.dim_or(Color::White)),
                            Cell::from(s.hr.to_string()).fg(s.hr.dim_or(Color::White)),
                            Cell::from(s.rbi.to_string()).fg(s.rbi.dim_or(Color::White)),
                            Cell::from(s.bb.to_string()).fg(s.bb.dim_or(Color::White)),
                            Cell::from(s.so.to_string()).fg(s.so.dim_or(Color::White)),
                            Cell::from(s.sb.to_string()).fg(s.sb.dim_or(Color::White)),
                            Cell::from(s.avg.as_str())
                                .fg(avg_color(s.avg.as_str()).unwrap_or(Color::White)),
                            Cell::from(s.obp.as_str()).fg(obp_color(s.obp.as_str()).unwrap_or(Color::White)),
                            Cell::from(s.slg.as_str()).fg(slg_color(s.slg.as_str()).unwrap_or(Color::White)),
                        ]);
                    }
                    Some(RecentStats::Pitching(s)) => {
                        cells.extend([
                            Cell::from(s.w.to_string()).fg(s.w.dim_or(Color::White)),
                            Cell::from(s.l.to_string()).fg(s.l.dim_or(Color::White)),
                            Cell::from(s.era.as_str())
                                .fg(era_color(s.era.as_str()).unwrap_or(Color::White)),
                            Cell::from(s.g.to_string()).fg(s.g.dim_or(Color::White)),
                            Cell::from(s.gs.to_string()).fg(s.gs.dim_or(Color::White)),
                            Cell::from(s.sv.to_string()).fg(s.sv.dim_or(Color::White)),
                            s.ip.as_str().into(),
                            Cell::from(s.h.to_string()).fg(s.h.dim_or(Color::White)),
                            Cell::from(s.er.to_string()).fg(s.er.dim_or(Color::White)),
                            Cell::from(s.bb.to_string()).fg(s.bb.dim_or(Color::White)),
                            Cell::from(s.so.to_string()).fg(s.so.dim_or(Color::White)),
                            Cell::from(s.whip.as_str()).fg(whip_color(s.whip.as_str()).unwrap_or(Color::White)),
                        ]);
                    }
                    None => {
                        let dashes: &[&str] = if is_hitting {
                            &["-", "-", "-", "-", "-", "-", "-", "-", "---", "---", "---"]
                        } else {
                            &[
                                "-", "-", "---", "-", "-", "-", "---", "-", "-", "-", "-", "---",
                            ]
                        };
                        cells.extend(dashes.iter().map(|&s| Cell::from(s)));
                    }
                }
                Row::new(cells)
            })
            .collect();

        Some((header, widths, rows))
    }
}
