use chrono::{Datelike, NaiveDate};
use serde::Deserialize;

/// If the seasons API fails, conservatively assume spring training ends before March 20.
/// This avoids using spring training params after the regular season has started.
const SPRING_TRAINING_FALLBACK_MONTH: u32 = 3;
const SPRING_TRAINING_FALLBACK_DAY: u32 = 20;

/// Whether the date falls in spring training or the regular season.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GameType {
    SpringTraining,
    RegularSeason,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SeasonsResponse {
    pub seasons: Vec<SeasonInfo>,
}

/// Season date boundaries fetched from the MLB seasons API.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeasonInfo {
    pub regular_season_start_date: NaiveDate,
}

/// Determine the game type for a given date.
/// Uses SeasonInfo if available, otherwise falls back to a conservative heuristic.
pub fn game_type_for_date(date: NaiveDate, season_info: Option<&SeasonInfo>) -> GameType {
    match season_info {
        Some(info) => {
            if date < info.regular_season_start_date {
                GameType::SpringTraining
            } else {
                GameType::RegularSeason
            }
        }
        None => {
            let cutoff = NaiveDate::from_ymd_opt(
                date.year(),
                SPRING_TRAINING_FALLBACK_MONTH,
                SPRING_TRAINING_FALLBACK_DAY,
            );
            match cutoff {
                Some(c) if date < c => GameType::SpringTraining,
                _ => GameType::RegularSeason,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_training_with_season_info() {
        let info = SeasonInfo {
            regular_season_start_date: NaiveDate::from_ymd_opt(2026, 3, 25).unwrap(),
        };
        let spring = NaiveDate::from_ymd_opt(2026, 3, 10).unwrap();
        let regular = NaiveDate::from_ymd_opt(2026, 3, 25).unwrap();
        let mid_season = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();

        assert_eq!(
            game_type_for_date(spring, Some(&info)),
            GameType::SpringTraining
        );
        assert_eq!(
            game_type_for_date(regular, Some(&info)),
            GameType::RegularSeason
        );
        assert_eq!(
            game_type_for_date(mid_season, Some(&info)),
            GameType::RegularSeason
        );
    }

    #[test]
    fn test_spring_training_fallback_without_season_info() {
        let before_cutoff = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
        let after_cutoff = NaiveDate::from_ymd_opt(2026, 3, 25).unwrap();
        let on_cutoff = NaiveDate::from_ymd_opt(2026, 3, 20).unwrap();

        assert_eq!(
            game_type_for_date(before_cutoff, None),
            GameType::SpringTraining
        );
        assert_eq!(
            game_type_for_date(after_cutoff, None),
            GameType::RegularSeason
        );
        assert_eq!(game_type_for_date(on_cutoff, None), GameType::RegularSeason);
    }
}
