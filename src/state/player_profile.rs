use crate::components::stats::player_profile::PlayerProfile;
use crate::state::messages::NetworkRequest;
use mlbt_api::client::StatGroup;
use mlbt_api::player::PeopleResponse;
use mlbt_api::season::GameType;
use tui::widgets::ScrollbarState;

/// State for a single Player Profile view.
pub struct PlayerProfileState {
    pub profile: PlayerProfile,
    pub stat_group: StatGroup,
    pub game_type: GameType,
    pub season_year: i32,
    pub scroll_offset: u16,
    pub scroll_state: ScrollbarState,
    pub content_height: u16,
    pub viewport_height: u16,
}

impl PlayerProfileState {
    /// Create from an api response. Returns None if the response has no player data.
    pub fn from_response(
        data: PeopleResponse,
        stat_group: StatGroup,
        game_type: GameType,
        season_year: i32,
    ) -> Option<Self> {
        // only one player was requested so there should only be one person in the response vec.
        let person = data.people.into_iter().next()?;
        Some(Self {
            profile: PlayerProfile::from_person(person),
            stat_group,
            game_type,
            season_year,
            scroll_offset: 0,
            scroll_state: ScrollbarState::default(),
            content_height: 0,
            viewport_height: 0,
        })
    }

    pub fn game_type_toggle_request(&self, date: chrono::NaiveDate) -> NetworkRequest {
        let game_type = match self.game_type {
            GameType::RegularSeason => GameType::SpringTraining,
            GameType::SpringTraining => GameType::RegularSeason,
        };
        NetworkRequest::PlayerProfile {
            player_id: self.profile.id,
            group: self.stat_group,
            date,
            game_type,
        }
    }

    pub fn scroll_down(&mut self) {
        let max = self.content_height.saturating_sub(self.viewport_height);
        if self.scroll_offset < max {
            self.scroll_offset += 1;
            self.scroll_state = self.scroll_state.position(self.scroll_offset as usize);
        }
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
        self.scroll_state = self.scroll_state.position(self.scroll_offset as usize);
    }

    pub fn page_down(&mut self) {
        let max = self.content_height.saturating_sub(self.viewport_height);
        self.scroll_offset = (self.scroll_offset + self.viewport_height).min(max);
        self.scroll_state = self.scroll_state.position(self.scroll_offset as usize);
    }

    pub fn page_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(self.viewport_height);
        self.scroll_state = self.scroll_state.position(self.scroll_offset as usize);
    }

    /// Calculate the height of each section for layout.
    pub fn section_heights(&self) -> [u16; 5] {
        let bio_height = self.profile.bio.len() as u16;
        let splits = &self.profile.splits;

        let season_height = if !splits.season.is_empty() {
            splits.season.len() as u16 + 2 // title + header + rows
        } else {
            2 // title + "No data"
        };

        let recent_splits_height = if splits.recent_splits.iter().any(|s| s.stat.is_some()) {
            5 // title + header + 3 rows
        } else {
            0
        };

        let career_height = if !splits.year_by_year.is_empty() {
            splits.year_by_year.len() as u16 + splits.career.len() as u16 + 2
        } else {
            0
        };

        let game_log_rows = splits.game_log.len();
        let game_log_height = if game_log_rows > 0 {
            game_log_rows as u16 + 2
        } else {
            0
        };

        [
            bio_height + 1, // +1 for blank line below section
            season_height + 1,
            recent_splits_height + 1,
            career_height + 1,
            game_log_height,
        ]
    }

    pub fn sync_scrollbar(&mut self) {
        if self.content_height > self.viewport_height {
            self.scroll_state = self
                .scroll_state
                .content_length(self.content_height as usize)
                .position(self.scroll_offset as usize);
        }
    }
}
