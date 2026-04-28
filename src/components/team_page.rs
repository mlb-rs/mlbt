use crate::components::constants::lookup_team_by_id;
use crate::components::datetime::{
    format_game_time, format_numeric_date_or, format_short_date, format_short_date_or,
};
use crate::components::util::{OptionDisplayExt, OptionMapDisplayExt};
use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;
use mlbt_api::schedule::{AbstractGameState, ScheduleResponse};
use mlbt_api::team::{RosterResponse, TransactionsResponse};

#[derive(Debug, Clone)]
pub struct TeamGame {
    pub date: NaiveDate,
    pub date_display: String,
    pub opponent: String,
    pub time_or_score: String,
    pub start_time_utc: Option<DateTime<Utc>>,
    pub is_home: bool,
    pub is_past: bool,
}

#[derive(Debug, Clone)]
pub struct RosterRow {
    pub player_id: u64,
    pub number: String,
    pub name: String,
    pub position: String,
    pub position_group: PositionGroup,
    pub bats_throws: String,
    pub height: String,
    pub weight: String,
    pub dob: String,
    pub status: String,
    pub status_code: String,
}

#[derive(Debug, Clone)]
pub struct TransactionRow {
    pub date: String,
    pub description: String,
}

impl TeamGame {
    pub fn from_schedule(
        response: &ScheduleResponse,
        team_id: u16,
        date: NaiveDate,
        tz: Tz,
    ) -> Vec<TeamGame> {
        let mut games = Vec::new();
        for date_entry in &response.dates {
            let Some(date_games) = &date_entry.games else {
                continue;
            };
            for game in date_games {
                let is_home = game.teams.home.team.id == team_id;
                let opponent_team = if is_home {
                    &game.teams.away.team
                } else {
                    &game.teams.home.team
                };

                let abbr = lookup_team_by_id(opponent_team.id)
                    .map(|t| t.abbreviation.to_string())
                    .unwrap_or_else(|| opponent_team.name.clone());

                let opponent = if is_home {
                    format!("vs {abbr}")
                } else {
                    format!("@ {abbr}")
                };

                let game_date = game.official_date;
                let date_display = format_short_date(game_date);

                let is_final = matches!(
                    game.status.abstract_game_state,
                    Some(AbstractGameState::Final)
                );
                let is_past = is_final && game_date < date;
                let start_time_utc = if is_final { None } else { Some(game.game_date) };

                let time_or_score = if is_final {
                    let home_score = game.teams.home.score.unwrap_or(0);
                    let away_score = game.teams.away.score.unwrap_or(0);
                    let (team_score, opp_score) = if is_home {
                        (home_score, away_score)
                    } else {
                        (away_score, home_score)
                    };
                    let result = if team_score > opp_score {
                        "W"
                    } else if team_score == opp_score {
                        "T"
                    } else {
                        "L"
                    };
                    format!("{team_score}-{opp_score} {result}")
                } else {
                    start_time_utc
                        .map(|utc| format_game_time(utc, tz))
                        .unwrap_or_else(|| "TBD".to_string())
                };

                games.push(TeamGame {
                    date: game_date,
                    date_display,
                    opponent,
                    time_or_score,
                    start_time_utc,
                    is_home,
                    is_past,
                });
            }
        }
        games
    }

    pub fn refresh_time_or_score(&mut self, tz: Tz) {
        if let Some(utc) = self.start_time_utc {
            self.time_or_score = format_game_time(utc, tz);
        }
    }
}

/// Roster position grouping. Variant order determines display/sort order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PositionGroup {
    Pitcher,
    /// Shohei Ohtani
    TwoWay,
    Catcher,
    Infielder,
    Outfielder,
    Other,
}

impl PositionGroup {
    fn from_api(s: &str) -> Self {
        match s {
            "Pitcher" => Self::Pitcher,
            "Catcher" => Self::Catcher,
            "Infielder" => Self::Infielder,
            "Outfielder" => Self::Outfielder,
            "Two-Way Player" => Self::TwoWay,
            _ => Self::Other,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Pitcher => "Pitchers",
            Self::Catcher => "Catchers",
            Self::Infielder => "Infielders",
            Self::Outfielder => "Outfielders",
            Self::TwoWay => "Two-Way Players",
            Self::Other => "Other",
        }
    }

    pub fn stat_group(self) -> mlbt_api::client::StatGroup {
        match self {
            Self::Pitcher => mlbt_api::client::StatGroup::Pitching,
            _ => mlbt_api::client::StatGroup::Hitting,
        }
    }
}

impl RosterRow {
    pub fn from_roster(response: &RosterResponse) -> Vec<RosterRow> {
        let mut rows: Vec<RosterRow> = response
            .roster
            .iter()
            .map(|entry| {
                let person = &entry.person;
                let bats = person.bat_side.as_ref().map_display_or(|s| &s.code, "-");
                let throws = person.pitch_hand.as_ref().map_display_or(|s| &s.code, "-");

                RosterRow {
                    player_id: person.id,
                    number: entry.jersey_number.display_or("-"),
                    name: person.full_name.clone(),
                    position: entry.position.abbreviation.clone(),
                    position_group: PositionGroup::from_api(&entry.position.r#type),
                    bats_throws: format!("{bats}/{throws}"),
                    height: person.height.display_or("-"),
                    weight: person.weight.display_or("-"),
                    dob: format_numeric_date_or(person.birth_date, "-"),
                    status: entry.status.description.clone(),
                    status_code: entry.status.code.clone(),
                }
            })
            .collect();

        // sort by position group, then jersey number
        rows.sort_by(|a, b| {
            a.position_group.cmp(&b.position_group).then_with(|| {
                let a_num: u16 = a.number.parse().unwrap_or(u16::MAX);
                let b_num: u16 = b.number.parse().unwrap_or(u16::MAX);
                a_num.cmp(&b_num)
            })
        });

        rows
    }
}

impl TransactionRow {
    pub fn from_transactions(response: &TransactionsResponse) -> Vec<TransactionRow> {
        let mut rows: Vec<TransactionRow> = response
            .transactions
            .iter()
            .filter_map(|t| {
                let description = t.description.clone()?;
                let date = format_short_date_or(t.date, "");
                Some(TransactionRow { date, description })
            })
            .collect();
        // show recent transactions first
        rows.reverse();
        rows
    }
}
