use crate::state::app_settings::AppSettings;
use crate::state::app_state::AppState;
use chrono::{ParseError, Utc};
use mlb_api::live::LiveResponse;
use mlb_api::schedule::ScheduleResponse;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum MenuItem {
    #[default]
    Scoreboard,
    Gameday,
    Stats,
    Standings,
    Help,
    DatePicker,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum DebugState {
    On,
    #[default]
    Off,
}

/// A team must be either Home or Away.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum HomeOrAway {
    #[default]
    Home = 0,
    Away = 1,
}

pub struct App {
    pub settings: AppSettings,
    pub state: AppState,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            state: AppState::default(),
            settings: AppSettings::load_from_file(),
        };
        app.configure();
        app
    }

    /// Run any final configuration that might need to access multiple parts of state.
    fn configure(&mut self) {
        self.set_all_datepickers_to_today();
        self.state.standings.favorite_team = self.settings.favorite_team;
    }

    /// Sync date pickers using the correct timezone.
    fn set_all_datepickers_to_today(&mut self) {
        let today = Utc::now()
            .with_timezone(&self.settings.timezone)
            .date_naive();
        self.state.schedule.date_selector.date = today;
        self.state.stats.date_selector.date = today;
        self.state.standings.date_selector.date = today;
    }

    /// Update the schedule and return the selected game.
    /// If the schedule is empty, return None.
    pub fn update_schedule(&mut self, schedule: &ScheduleResponse) -> Option<u64> {
        self.state.schedule.update(&self.settings, schedule);

        if self.state.schedule.is_empty() {
            self.state.live_game.clear();
            None
        } else {
            self.state.schedule.get_selected_game_opt()
        }
    }

    pub fn update_live_data(&mut self, live_data: &LiveResponse) {
        // self.state.live_game.update(live_data);
        self.state.gameday.game.update(live_data);
    }

    pub fn update_tab(&mut self, next: MenuItem) {
        if self.state.active_tab != next {
            self.state.previous_tab = self.state.active_tab;
            self.state.active_tab = next;
            self.state.debug_state = DebugState::Off;
        }
        // reset selection when switching tabs but not when date picker is opened
        if next != MenuItem::DatePicker && self.state.previous_tab == MenuItem::Standings {
            self.state.standings.reset_selection();
        }
    }

    pub fn try_update_date_from_input(&mut self) -> Result<(), ParseError> {
        let valid_date = self
            .state
            .date_input
            .validate_input(self.settings.timezone)?;

        // current tab is date picker, so use previous tab to update correct date
        match self.state.previous_tab {
            MenuItem::Scoreboard => self.state.schedule.set_date_from_valid_input(valid_date),
            MenuItem::Standings => self.state.standings.set_date_from_valid_input(valid_date),
            MenuItem::Stats => self.state.stats.set_date_from_valid_input(valid_date),
            _ => (),
        }
        Ok(())
    }

    pub fn move_date_selector_by_arrow(&mut self, right_arrow: bool) {
        let date = match self.state.previous_tab {
            MenuItem::Scoreboard => Some(self.state.schedule.set_date_with_arrows(right_arrow)),
            MenuItem::Standings => Some(self.state.standings.set_date_with_arrows(right_arrow)),
            MenuItem::Stats => Some(self.state.stats.set_date_with_arrows(right_arrow)),
            _ => None,
        };
        self.state.date_input.text.clear();
        if let Some(date) = date {
            self.state.date_input.text.push_str(&date.to_string());
        }
    }

    pub fn exit_help(&mut self) {
        if self.state.active_tab == MenuItem::Help {
            self.state.active_tab = self.state.previous_tab;
        }
    }

    pub fn toggle_debug(&mut self) {
        match self.state.debug_state {
            DebugState::Off => self.state.debug_state = DebugState::On,
            DebugState::On => self.state.debug_state = DebugState::Off,
        }
    }

    pub fn toggle_full_screen(&mut self) {
        self.settings.full_screen = !self.settings.full_screen;
    }
}
