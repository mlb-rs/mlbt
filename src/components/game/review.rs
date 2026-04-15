use crate::components::game::live_game::{PlayerId, PlayerMap};
use crate::components::standings::Team;
use tui::prelude::{Span, Style};

#[derive(Debug)]
pub struct ReviewDetails {
    pub is_overturned: Option<bool>,
    pub in_progress: Option<bool>,
    // pub review_type: String, // TODO not sure what this is yet
    /// Team that initiated the review. If None, usually means it was a crew chief review.
    pub challenge_team_id: Option<u16>,
    /// Player that initiated the review, if any.
    pub player_id: Option<PlayerId>,
}

impl From<&mlbt_api::plays::ReviewDetails> for ReviewDetails {
    fn from(review: &mlbt_api::plays::ReviewDetails) -> Self {
        Self {
            is_overturned: review.is_overturned,
            in_progress: review.in_progress,
            // review_type: review.review_type.clone(),
            challenge_team_id: review.challenge_team_id,
            player_id: review.player.as_ref().map(|p| p.id),
        }
    }
}

impl ReviewDetails {
    fn team_abbreviation<'a>(&self, home_team: &'a Team, away_team: &'a Team) -> Option<&'a str> {
        self.challenge_team_id.and_then(|id| {
            if id == home_team.id {
                Some(home_team.abbreviation)
            } else if id == away_team.id {
                Some(away_team.abbreviation)
            } else {
                None
            }
        })
    }

    fn player_name(&self, player_map: &PlayerMap) -> Option<String> {
        self.player_id
            .and_then(|id| player_map.get(&id).map(|p| p.last_name.clone()))
    }

    fn player_full_name(&self, player_map: &PlayerMap) -> Option<String> {
        self.player_id.and_then(|id| {
            player_map
                .get(&id)
                .map(|p| format!("{} {}", p.first_name, p.last_name))
        })
    }

    /// Returns the completed review status: " | Overturned [Ruth]", " | Upheld [Ruth]", or
    /// " | Challenged [Ruth]" when the outcome is unknown.
    pub fn format_status_spans(&self, player_map: &PlayerMap) -> Option<Vec<Span<'static>>> {
        if self.in_progress.unwrap_or(true) {
            return None;
        }

        let status = match self.is_overturned {
            Some(true) => "Overturned",
            Some(false) => "Upheld",
            None => "Challenged",
        };

        let mut spans = vec![Span::raw(format!(" | {status}"))];

        if let Some(player) = self.player_name(player_map) {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(format!("[{player}]"), Style::default().bold()));
        }
        spans.push(Span::raw(" "));

        Some(spans)
    }

    /// Returns the in progress review status: "Pitch challenged by Babe Ruth [NYY]"
    pub fn format_in_progress_spans(
        &self,
        home_team: &Team,
        away_team: &Team,
        player_map: &PlayerMap,
    ) -> Option<Vec<Span<'static>>> {
        if !self.in_progress.unwrap_or(false) {
            return None;
        }

        let mut spans = vec![Span::raw("Pitch challenged")];

        if let Some(player) = self.player_full_name(player_map) {
            spans.push(Span::raw(format!(" by {player}")));
        }

        if let Some(team) = self.team_abbreviation(home_team, away_team) {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(format!("[{team}]"), Style::default().bold()));
        }

        Some(spans)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::game::player::Player;
    use std::collections::HashMap;

    const HOME: Team = Team {
        id: 147,
        division_id: 0,
        name: "NYY",
        team_name: "Yankees",
        abbreviation: "NYY",
    };
    const AWAY: Team = Team {
        id: 111,
        division_id: 0,
        name: "BOS",
        team_name: "Red Sox",
        abbreviation: "BOS",
    };

    fn players() -> PlayerMap {
        HashMap::from([(
            1,
            Player {
                id: 1,
                first_name: "Babe".to_owned(),
                last_name: "Ruth".to_owned(),
                ..Default::default()
            },
        )])
    }

    fn spans_text(spans: &[Span<'_>]) -> String {
        spans.iter().map(|s| s.content.as_ref()).collect()
    }

    fn review(
        overturned: bool,
        in_progress: bool,
        team_id: u16,
        player_id: Option<u64>,
    ) -> ReviewDetails {
        ReviewDetails {
            is_overturned: Some(overturned),
            in_progress: Some(in_progress),
            challenge_team_id: Some(team_id),
            player_id,
        }
    }

    #[test]
    fn status_returns_none_when_in_progress() {
        assert!(
            review(false, true, 147, Some(1))
                .format_status_spans(&players())
                .is_none()
        );
    }

    #[test]
    fn status_overturned_with_player() {
        let text = spans_text(
            &review(true, false, 147, Some(1))
                .format_status_spans(&players())
                .unwrap(),
        );
        assert!(text.contains("Overturned") && text.contains("[Ruth]"));
    }

    #[test]
    fn status_upheld_with_player() {
        let text = spans_text(
            &review(false, false, 147, Some(1))
                .format_status_spans(&players())
                .unwrap(),
        );
        assert!(text.contains("Upheld") && !text.contains("Overturned"));
    }

    #[test]
    fn status_without_player() {
        let text = spans_text(
            &review(true, false, 147, None)
                .format_status_spans(&HashMap::new())
                .unwrap(),
        );
        assert!(text.contains("Overturned") && !text.contains("["));
    }

    #[test]
    fn status_unknown_outcome_falls_back_to_challenged() {
        let review = ReviewDetails {
            is_overturned: None,
            in_progress: Some(false),
            challenge_team_id: Some(147),
            player_id: Some(1),
        };
        let text = spans_text(&review.format_status_spans(&players()).unwrap());
        assert!(text.contains("Challenged") && text.contains("[Ruth]"));
    }

    #[test]
    fn status_returns_none_when_state_unknown() {
        let review = ReviewDetails {
            is_overturned: Some(true),
            in_progress: None,
            challenge_team_id: Some(147),
            player_id: Some(1),
        };
        assert!(review.format_status_spans(&players()).is_none());
    }

    #[test]
    fn in_progress_returns_none_when_state_unknown() {
        let review = ReviewDetails {
            is_overturned: Some(true),
            in_progress: None,
            challenge_team_id: Some(147),
            player_id: Some(1),
        };
        assert!(
            review
                .format_in_progress_spans(&HOME, &AWAY, &players())
                .is_none()
        );
    }

    #[test]
    fn in_progress_returns_none_when_completed() {
        assert!(
            review(true, false, 147, Some(1))
                .format_in_progress_spans(&HOME, &AWAY, &HashMap::new())
                .is_none()
        );
    }

    #[test]
    fn in_progress_home_team() {
        let text = spans_text(
            &review(false, true, 147, Some(1))
                .format_in_progress_spans(&HOME, &AWAY, &players())
                .unwrap(),
        );
        assert!(
            text.contains("Pitch challenged")
                && text.contains("by Babe Ruth")
                && text.contains("[NYY]")
        );
    }

    #[test]
    fn in_progress_away_team() {
        let text = spans_text(
            &review(false, true, 111, Some(1))
                .format_in_progress_spans(&HOME, &AWAY, &players())
                .unwrap(),
        );
        assert!(text.contains("[BOS]"));
    }

    #[test]
    fn in_progress_unknown_team() {
        let text = spans_text(
            &review(false, true, 999, Some(1))
                .format_in_progress_spans(&HOME, &AWAY, &players())
                .unwrap(),
        );
        assert!(text.contains("by Babe Ruth") && !text.contains("["));
    }

    #[test]
    fn in_progress_no_player() {
        let text = spans_text(
            &review(false, true, 147, None)
                .format_in_progress_spans(&HOME, &AWAY, &HashMap::new())
                .unwrap(),
        );
        assert_eq!(text, "Pitch challenged [NYY]");
    }
}
