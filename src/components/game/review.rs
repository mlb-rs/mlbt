use crate::components::game::live_game::{PlayerId, PlayerMap};
use crate::components::standings::Team;
use tui::prelude::{Span, Style};

#[derive(Debug)]
pub struct ReviewDetails {
    pub is_overturned: bool,
    pub in_progress: bool,
    // pub review_type: String, // TODO not sure what this is yet
    pub challenge_team_id: u16,
    pub player_id: PlayerId,
}

impl From<&mlb_api::plays::ReviewDetails> for ReviewDetails {
    fn from(review: &mlb_api::plays::ReviewDetails) -> Self {
        Self {
            is_overturned: review.is_overturned,
            in_progress: review.in_progress,
            // review_type: review.review_type.clone(),
            challenge_team_id: review.challenge_team_id,
            player_id: review.player.as_ref().map(|p| p.id).unwrap_or_default(),
        }
    }
}

impl ReviewDetails {
    fn team_abbreviation<'a>(&self, home_team: &'a Team, away_team: &'a Team) -> Option<&'a str> {
        if self.challenge_team_id == home_team.id {
            Some(home_team.abbreviation)
        } else if self.challenge_team_id == away_team.id {
            Some(away_team.abbreviation)
        } else {
            None
        }
    }

    fn player_name(&self, player_map: &PlayerMap) -> Option<String> {
        player_map.get(&self.player_id).map(|p| p.last_name.clone())
    }

    fn player_full_name(&self, player_map: &PlayerMap) -> Option<String> {
        player_map
            .get(&self.player_id)
            .map(|p| format!("{} {}", p.first_name, p.last_name))
    }

    /// Returns the completed review status: " | Overturned [Ruth]" or " | Upheld [Ruth]"
    pub fn format_status_spans(&self, player_map: &PlayerMap) -> Option<Vec<Span<'static>>> {
        if self.in_progress {
            return None;
        }

        let status = if self.is_overturned {
            "Overturned"
        } else {
            "Upheld"
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
        if !self.in_progress {
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
