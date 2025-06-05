use crate::components::live_game::GameStateV2;

#[derive(Default)]
pub struct GamedayState {
    pub panels: GamedayPanels,
    selected_at_bat: Option<usize>,
    pub game: GameStateV2,
}

impl GamedayState {
    pub fn selected_at_bat(&self) -> Option<u8> {
        self.selected_at_bat.map(|i| i as u8)
    }

    pub fn current_game_id(&self) -> u64 {
        self.game.game_id
    }

    pub fn set_current_game_id(&mut self, game_id: u64) {
        self.game.game_id = game_id;
    }
}

/// Store which panels should be rendered in the Gameday tab.
#[derive(Debug, Copy, Clone)]
pub struct GamedayPanels {
    pub info: bool,
    pub at_bat: bool,
    pub boxscore: bool,
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
        }
    }
}
