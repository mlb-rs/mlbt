use crate::components::game::live_game::GameState;
use mlbt_api::schedule::AbstractGameState;

#[derive(Default)]
pub struct GamedayState {
    pub panels: GamedayPanels,
    pub game: GameState,
    pub scoring_plays_only: bool,
    /// The at bat index (map key) that is currently selected, or `None` when live.
    selected_at_bat: Option<u8>,
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
            self.selected_at_bat = None;
            self.game.reset();
            self.game.game_id = new_id;
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

    /// Go to "live" at bat by deselecting the current at bat.
    pub fn live(&mut self) {
        self.selected_at_bat = None;
    }

    /// Go to the start of the game.
    pub fn start(&mut self) {
        self.selected_at_bat = self.navigable_at_bats().first().copied();
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

    /// When enabling scoring plays only, move the selection onto a scoring play so the cursor is
    /// visible. Keeps a selection that is already a scoring play, snaps a non-scoring selection to
    /// the closest scoring play, and selects the most recent scoring play when live.
    fn snap_to_scoring_play(&mut self) {
        let scoring = self.navigable_at_bats();
        self.selected_at_bat = match self.selected_at_bat {
            Some(selected) if scoring.contains(&selected) => Some(selected),
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

    /// Build a state whose at bats have the given indexes, marking the listed indexes as scoring
    /// plays.
    fn state_with(indexes: &[u8], scoring: &[u8]) -> GamedayState {
        let mut state = GamedayState::default();
        for &index in indexes {
            let mut at_bat = AtBat {
                index,
                ..Default::default()
            };
            at_bat.play_result.is_scoring_play = scoring.contains(&index);
            state.game.at_bats.insert(index, at_bat);
        }
        state
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
