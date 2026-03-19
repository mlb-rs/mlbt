use crate::components::player_profile::PlayerProfile;
use mlbt_api::client::StatGroup;
use mlbt_api::player::PeopleResponse;
use mlbt_api::season::GameType;

/// State for a single Player Profile view.
pub struct PlayerProfileState {
    pub profile: PlayerProfile,
    pub stat_group: StatGroup,
    pub game_type: GameType,
    pub season_year: i32,
    pub scroll_offset: u16,
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
            content_height: 0,
            viewport_height: 0,
        })
    }

    pub fn toggle_game_type(&mut self) {
        self.game_type = match self.game_type {
            GameType::RegularSeason => GameType::SpringTraining,
            GameType::SpringTraining => GameType::RegularSeason,
        };
    }

    pub fn scroll_down(&mut self) {
        let max = self.content_height.saturating_sub(self.viewport_height);
        if self.scroll_offset < max {
            self.scroll_offset += 1;
        }
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn page_down(&mut self) {
        let max = self.content_height.saturating_sub(self.viewport_height);
        self.scroll_offset = (self.scroll_offset + self.viewport_height).min(max);
    }

    pub fn page_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(self.viewport_height);
    }
}
