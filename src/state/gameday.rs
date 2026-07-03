use crate::components::game::live_game::GameState;
use mlbt_api::live::LiveResponse;
use mlbt_api::schedule::AbstractGameState;
use mlbt_api::win_probability::WinProbabilityResponse;
use tui::widgets::ScrollbarState;

#[derive(Default)]
pub struct GamedayState {
    pub panels: GamedayPanels,
    pub game: GameState,
    pub scoring_plays_only: bool,
    /// The at bat index (map key) that is currently selected, or `None` when live.
    selected_at_bat: Option<u8>,
    /// Snap to a scoring play when the next update arrives. Set when switching games with the
    /// scoring play filter on, since the new game's at bats load asynchronously.
    snap_pending: bool,
    pub plays_scroll_offset: u16,
    pub plays_scroll_state: ScrollbarState,
}

impl GamedayState {
    pub fn selected_at_bat(&self) -> Option<u8> {
        self.selected_at_bat
    }

    pub fn current_game_id(&self) -> u64 {
        self.game.game_id
    }

    pub fn is_final(&self) -> bool {
        matches!(
            self.game.abstract_game_state,
            Some(AbstractGameState::Final)
        )
    }

    pub fn reset(&mut self, game_id: Option<u64>) {
        let new_id = game_id.unwrap_or(0);

        if self.game.game_id != new_id {
            self.on_game_changing();
            self.game.reset();
            self.game.game_id = new_id;
        }
    }

    /// Update with latest data from the API, snapping to a scoring play if a game switch left one
    /// pending. Detects a game switch here too since `GameData` responses from directly navigating
    /// games (rather than a schedule refresh) arrive without a prior call to `reset`.
    pub fn update(&mut self, live_data: &LiveResponse, win_probability: &WinProbabilityResponse) {
        if self.game.game_id != live_data.game_pk {
            self.on_game_changing();
        }
        self.game.update(live_data, win_probability);
        self.snap_if_pending();
    }

    /// Clear the selection and, if the scoring play filter is on, mark a snap as pending once the
    /// new game's at bats load.
    fn on_game_changing(&mut self) {
        self.selected_at_bat = None;
        self.snap_pending = self.scoring_plays_only;
    }

    fn snap_if_pending(&mut self) {
        if std::mem::take(&mut self.snap_pending) && self.scoring_plays_only {
            self.snap_to_scoring_play();
        }
    }

    /// The at bat indexes that can be navigated to, in game order. When `scoring_plays_only` is
    /// enabled this is restricted to scoring plays, so scrolling skips everything else.
    fn navigable_at_bats(&self) -> Vec<u8> {
        self.game
            .at_bats
            .values()
            .filter(|at_bat| !self.scoring_plays_only || at_bat.play_result.is_scoring_play)
            .map(|at_bat| at_bat.index)
            .collect()
    }

    /// The navigable at bat indexes and the position of the current selection within them. Returns
    /// `None` when there is nothing to navigate. The inner `None` means nothing is selected yet.
    fn navigable_selection(&self) -> Option<(Vec<u8>, Option<usize>)> {
        let indexes = self.navigable_at_bats();
        if indexes.is_empty() {
            return None;
        }
        let pos = self
            .selected_at_bat
            .and_then(|selected| indexes.iter().position(|&i| i == selected));
        Some((indexes, pos))
    }

    pub fn next_at_bat(&mut self) {
        let Some((indexes, pos)) = self.navigable_selection() else {
            return;
        };
        let next = match pos {
            Some(p) if p >= indexes.len() - 1 => 0,
            Some(p) => p + 1,
            None => 0,
        };
        self.selected_at_bat = Some(indexes[next]);
    }

    pub fn previous_at_bat(&mut self) {
        let Some((indexes, pos)) = self.navigable_selection() else {
            return;
        };
        let prev = match pos {
            None | Some(0) => indexes.len() - 1,
            Some(p) => p - 1,
        };
        self.selected_at_bat = Some(indexes[prev]);
    }

    /// Go to "live" at bat by deselecting the current at bat. When the scoring play filter is
    /// enabled, "live" means the most recent scoring play instead, matching what enabling the
    /// filter while live already selects.
    pub fn live(&mut self) {
        self.selected_at_bat = None;
        if self.scoring_plays_only {
            self.snap_to_scoring_play();
        }
    }

    /// Go to the start of the game.
    pub fn start(&mut self) {
        if let Some(first) = self.navigable_at_bats().first() {
            self.selected_at_bat = Some(*first);
        }
    }

    pub fn toggle_info(&mut self) {
        self.panels.info = !self.panels.info;
    }

    pub fn toggle_at_bat(&mut self) {
        self.panels.at_bat = !self.panels.at_bat;
    }

    pub fn toggle_boxscore(&mut self) {
        self.panels.boxscore = !self.panels.boxscore;
    }

    pub fn toggle_win_probability(&mut self) {
        self.panels.win_probability = !self.panels.win_probability;
    }

    pub fn toggle_scoring_plays_only(&mut self) {
        self.scoring_plays_only = !self.scoring_plays_only;
        if self.scoring_plays_only {
            self.snap_to_scoring_play();
        }
    }

    /// Move the selection onto a scoring play so the cursor is visible while the filter is
    /// enabled. Snaps to the closest scoring play, which keeps a selection that already scores,
    /// and selects the most recent scoring play when live. A game with no scoring plays leaves the
    /// selection untouched.
    fn snap_to_scoring_play(&mut self) {
        let scoring = self.navigable_at_bats();
        if scoring.is_empty() {
            return;
        }
        self.selected_at_bat = match self.selected_at_bat {
            Some(selected) => scoring
                .iter()
                .min_by_key(|&&i| i.abs_diff(selected))
                .copied(),
            None => scoring.last().copied(),
        };
    }
}

/// Store which panels should be rendered in the Gameday tab.
#[derive(Debug, Copy, Clone)]
pub struct GamedayPanels {
    pub info: bool,
    pub at_bat: bool,
    pub boxscore: bool,
    pub win_probability: bool,
}

impl GamedayPanels {
    /// Return the number of panels that are active.
    pub fn count(&self) -> usize {
        self.info as usize + self.at_bat as usize + self.boxscore as usize
    }
}

impl Default for GamedayPanels {
    fn default() -> Self {
        GamedayPanels {
            info: true,
            at_bat: true,
            boxscore: false,
            win_probability: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::game::at_bat::AtBat;
    use mlbt_api::live::LiveData;
    use mlbt_api::plays::{About, Play, Plays};

    /// Build a state whose at bats have the given indexes, marking the listed indexes as scoring
    /// plays.
    fn state_with(indexes: &[u8], scoring: &[u8]) -> GamedayState {
        let mut state = GamedayState::default();
        insert_at_bats(&mut state, indexes, scoring);
        state
    }

    fn insert_at_bats(state: &mut GamedayState, indexes: &[u8], scoring: &[u8]) {
        for &index in indexes {
            let mut at_bat = AtBat {
                index,
                ..Default::default()
            };
            at_bat.play_result.is_scoring_play = scoring.contains(&index);
            state.game.at_bats.insert(index, at_bat);
        }
    }

    #[test]
    fn navigation_walks_every_play_normally() {
        let mut state = state_with(&[0, 1, 2], &[1]);

        state.next_at_bat();
        assert_eq!(state.selected_at_bat(), Some(0));
        state.next_at_bat();
        assert_eq!(state.selected_at_bat(), Some(1));
        state.previous_at_bat();
        assert_eq!(state.selected_at_bat(), Some(0));
        // wraps to the last at bat
        state.previous_at_bat();
        assert_eq!(state.selected_at_bat(), Some(2));
        state.next_at_bat();
        assert_eq!(state.selected_at_bat(), Some(0));
    }

    #[test]
    fn navigation_skips_non_scoring_plays_when_filtered() {
        let mut state = state_with(&[0, 1, 2, 3, 4], &[1, 3]);
        state.scoring_plays_only = true;

        // only scoring plays are reachable, in order
        state.next_at_bat();
        assert_eq!(state.selected_at_bat(), Some(1));
        state.next_at_bat();
        assert_eq!(state.selected_at_bat(), Some(3));
        // wraps back to the first scoring play
        state.next_at_bat();
        assert_eq!(state.selected_at_bat(), Some(1));
        state.previous_at_bat();
        assert_eq!(state.selected_at_bat(), Some(3));
    }

    #[test]
    fn start_respects_scoring_filter() {
        let mut state = state_with(&[0, 1, 2, 3], &[2]);
        state.start();
        assert_eq!(state.selected_at_bat(), Some(0));

        state.scoring_plays_only = true;
        state.start();
        assert_eq!(state.selected_at_bat(), Some(2));
    }

    #[test]
    fn live_snaps_to_latest_scoring_play_when_filtered() {
        // this is what the `1` (Scoreboard) key calls on the way out of Gameday. Without the
        // filter check, repeatedly switching to Scoreboard and back to the same, already-loaded
        // game (no game id change, so nothing re-snaps on return) would strand the selection on
        // live instead of a scoring play.
        let mut state = state_with(&[0, 1, 2, 3, 4], &[1, 3]);
        state.scoring_plays_only = true;
        state.selected_at_bat = Some(1);

        state.live();
        assert_eq!(state.selected_at_bat(), Some(3));

        // idempotent across repeated round trips
        state.live();
        assert_eq!(state.selected_at_bat(), Some(3));

        // without the filter, live still means truly live
        state.scoring_plays_only = false;
        state.live();
        assert_eq!(state.selected_at_bat(), None);
    }

    #[test]
    fn toggle_snaps_selection_to_nearest_scoring_play() {
        let mut state = state_with(&[0, 1, 2, 3, 4], &[1, 4]);

        // selecting a non scoring play then filtering snaps to the closest scoring play
        state.selected_at_bat = Some(3);
        state.toggle_scoring_plays_only();
        assert_eq!(state.selected_at_bat(), Some(4));

        // an already scoring selection is left untouched
        state.toggle_scoring_plays_only(); // off
        state.selected_at_bat = Some(1);
        state.toggle_scoring_plays_only(); // on
        assert_eq!(state.selected_at_bat(), Some(1));

        // toggling on while live selects the most recent scoring play
        state.toggle_scoring_plays_only(); // off
        state.live();
        state.toggle_scoring_plays_only(); // on
        assert_eq!(state.selected_at_bat(), Some(4));
    }

    /// Build a `LiveResponse` for the given game with at bats at the given indexes, marking the
    /// listed indexes as scoring plays. Mirrors a real `GameDataLoaded` network response.
    fn live_response(game_pk: u64, indexes: &[u8], scoring: &[u8]) -> LiveResponse {
        let all_plays = indexes
            .iter()
            .map(|&index| Play {
                about: About {
                    at_bat_index: index,
                    is_scoring_play: Some(scoring.contains(&index)),
                    ..Default::default()
                },
                ..Default::default()
            })
            .collect();
        LiveResponse {
            game_pk,
            live_data: LiveData {
                plays: Plays {
                    all_plays: Some(all_plays),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn switching_games_via_direct_navigation_snaps_to_scoring_play() {
        // this is the path taken when the user selects a different game with j/k on the
        // Scoreboard tab: `GameData` responses arrive via `update` directly, with no prior call to
        // `reset` (that only happens on a schedule refresh).
        let mut state = state_with(&[0, 1, 2], &[1]);
        state.scoring_plays_only = true;
        state.selected_at_bat = Some(1);

        let win_probability = WinProbabilityResponse::default();
        let response = live_response(6, &[0, 1, 2, 3], &[1, 3]);
        state.update(&response, &win_probability);

        assert_eq!(state.game.game_id, 6);
        assert_eq!(state.selected_at_bat(), Some(3));

        // a later update for the same game doesn't re-snap and drag the selection along
        state.previous_at_bat();
        assert_eq!(state.selected_at_bat(), Some(1));
        state.update(&response, &win_probability);
        assert_eq!(state.selected_at_bat(), Some(1));
    }

    #[test]
    fn switching_games_without_filter_just_clears_the_selection() {
        let mut state = state_with(&[0, 1, 2], &[1]);
        state.selected_at_bat = Some(1);

        let win_probability = WinProbabilityResponse::default();
        let response = live_response(6, &[0, 1, 2, 3], &[1, 3]);
        state.update(&response, &win_probability);

        assert_eq!(state.selected_at_bat(), None);
    }

    #[test]
    fn switching_games_via_schedule_refresh_snaps_to_scoring_play() {
        // this is the path taken on a schedule poll: `reset` runs first (before at bats have
        // loaded), then the `GameData` response arrives separately via `update`.
        let mut state = state_with(&[0, 1, 2], &[1]);
        state.scoring_plays_only = true;
        state.selected_at_bat = Some(1);

        state.reset(Some(6));
        assert_eq!(state.selected_at_bat(), None);

        let win_probability = WinProbabilityResponse::default();
        let response = live_response(6, &[0, 1, 2, 3], &[1, 3]);
        state.update(&response, &win_probability);
        assert_eq!(state.selected_at_bat(), Some(3));
    }

    #[test]
    fn filter_keeps_selection_without_scoring_plays() {
        let mut state = state_with(&[0, 1, 2], &[]);
        state.selected_at_bat = Some(1);

        // toggling the filter on has no scoring play to snap to
        state.toggle_scoring_plays_only();
        assert_eq!(state.selected_at_bat(), Some(1));

        // going to the start has nowhere to go either
        state.start();
        assert_eq!(state.selected_at_bat(), Some(1));
    }

    #[test]
    fn navigation_is_a_noop_without_navigable_plays() {
        // no at bats at all
        let mut empty = GamedayState::default();
        empty.next_at_bat();
        assert_eq!(empty.selected_at_bat(), None);

        // at bats exist but none score, with the filter on
        let mut none_score = state_with(&[0, 1], &[]);
        none_score.scoring_plays_only = true;
        none_score.next_at_bat();
        assert_eq!(none_score.selected_at_bat(), None);
    }
}
