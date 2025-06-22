use crate::components::game::live_game::GameState;

#[derive(Default)]
pub struct GamedayState {
    pub panels: GamedayPanels,
    pub game: GameState,
    selected_at_bat: Option<usize>,
}

impl GamedayState {
    pub fn selected_at_bat(&self) -> Option<u8> {
        self.selected_at_bat.map(|i| i as u8)
    }

    pub fn current_game_id(&self) -> u64 {
        self.game.game_id
    }

    pub fn reset(&mut self, game_id: Option<u64>) {
        let new_id = game_id.unwrap_or(0);

        if self.game.game_id != new_id {
            self.selected_at_bat = None;
            self.game.reset();
            self.game.game_id = new_id;
        }
    }

    pub fn next_at_bat(&mut self) {
        let count = self.game.count_events();
        if count == 0 {
            return;
        }
        let i = match self.selected_at_bat {
            Some(i) if i >= count - 1 => 0,
            Some(i) => i + 1,
            None => 0,
        };

        self.selected_at_bat = Some(i);
    }

    pub fn previous_at_bat(&mut self) {
        let count = self.game.count_events();
        if count == 0 {
            return;
        }
        let i = match self.selected_at_bat {
            None => count - 1,
            Some(0) => count - 1,
            Some(i) => i - 1,
        };
        self.selected_at_bat = Some(i);
    }

    /// Go to "live" at bat by deselecting the current at bat.
    pub fn live(&mut self) {
        self.selected_at_bat = None;
    }

    /// Go to the start of the game.
    pub fn start(&mut self) {
        self.selected_at_bat = Some(0);
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
