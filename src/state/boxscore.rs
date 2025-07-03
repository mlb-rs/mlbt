use crate::components::boxscore::{Boxscore, Note};
use crate::components::game::live_game::PlayerId;
use crate::components::game::player::Player;
use crate::state::app_state::HomeOrAway;
use crate::state::app_state::HomeOrAway::{Away, Home};
use mlb_api::live::LiveResponse;
use std::collections::HashMap;
use tui::prelude::*;
use tui::widgets::{Block, Cell, Paragraph, ScrollbarState, Wrap};

const LAYOUT_SPACING: usize = 3;

#[derive(Default)]
pub struct BoxscoreState {
    pub active_team: HomeOrAway,
    pub boxscore: Boxscore,
    pub scroll: usize,
    pub scroll_state: ScrollbarState,
    pub cache: RenderCache,
    pub max_scroll: usize,
}

#[derive(Default)]
pub struct TeamCache {
    // Cache paragraph for length calculation based on viewport width
    pub batting_notes_paragraph: Option<Paragraph<'static>>,
    pub batting_notes_height: usize,

    // Cache static table heights
    pub batting_stats_height: usize,
    pub pitching_stats_height: usize,

    /// Total height for all the team data
    pub total_content_height: u16,
}

#[derive(Default)]
pub struct RenderCache {
    pub home_team_cache: TeamCache,
    pub away_team_cache: TeamCache,

    pub game_notes_paragraph: Option<Paragraph<'static>>,
    pub game_notes_height: usize,

    // Track last viewport width to know when to recalculate
    pub last_viewport_width: u16,
}

impl TeamCache {
    fn calculate_for_width(&mut self, viewport_width: u16) {
        if let Some(paragraph) = &self.batting_notes_paragraph {
            self.batting_notes_height = paragraph.line_count(viewport_width);
        } else {
            self.batting_notes_height = 0;
        }

        self.total_content_height = (self.batting_stats_height
            + self.batting_notes_height
            + self.pitching_stats_height
            + LAYOUT_SPACING) as u16
    }
}

impl BoxscoreState {
    pub fn set_home_active(&mut self) {
        self.active_team = Home;
    }

    pub fn set_away_active(&mut self) {
        self.active_team = Away;
    }

    pub fn update(&mut self, live_game: &LiveResponse, players: &HashMap<PlayerId, Player>) {
        self.boxscore = Boxscore::from_live_data(live_game, players);
        self.update_static_cache();
    }

    fn build_team_cache(&self, team: HomeOrAway) -> TeamCache {
        let notes = Self::build_paragraph(self.boxscore.get_batting_notes(team));
        TeamCache {
            batting_notes_paragraph: notes,
            batting_notes_height: 0,
            batting_stats_height: self.boxscore.count_batting_table_rows(team) + 1, // +1 for header
            pitching_stats_height: self.boxscore.count_pitching_table_rows(team) + 1, // +1 for header
            total_content_height: 0,
        }
    }

    fn update_static_cache(&mut self) {
        self.cache.home_team_cache = self.build_team_cache(Home);
        self.cache.away_team_cache = self.build_team_cache(Away);
        self.cache.game_notes_paragraph = Self::build_paragraph(self.boxscore.get_game_notes());
        // Reset viewport width to force recalculation of wrapped content heights
        self.cache.last_viewport_width = 0;
    }

    pub fn sync_scrollbar(&mut self, viewport_height: u16, viewport_width: u16) {
        // use the view height to determine the total number of rows after text wrapping
        let total_content_height = self.calculate_heights_for_width(viewport_width);

        if total_content_height > viewport_height {
            self.max_scroll = total_content_height.saturating_sub(viewport_height) as usize;
            self.scroll = self.scroll.min(self.max_scroll);

            self.scroll_state = self
                .scroll_state
                .content_length(total_content_height as usize)
                .position(self.scroll);
        } else {
            self.max_scroll = 0;
            self.scroll = 0;
        }
    }

    fn build_paragraph(notes: &[Note]) -> Option<Paragraph<'static>> {
        let lines: Vec<Line<'static>> = notes.iter().filter_map(|n| n.to_line()).collect();
        if lines.is_empty() {
            None
        } else {
            Some(
                Paragraph::new(lines)
                    .block(Block::default())
                    .wrap(Wrap { trim: true }),
            )
        }
    }

    /// Calculate the height of wrapped text for the current viewport width and cache the results.
    fn calculate_heights_for_width(&mut self, width: u16) -> u16 {
        if self.cache.last_viewport_width == width {
            // already calculated for this width
            return self.get_total_content_height();
        }

        self.cache.home_team_cache.calculate_for_width(width);
        self.cache.away_team_cache.calculate_for_width(width);
        if let Some(paragraph) = &self.cache.game_notes_paragraph {
            self.cache.game_notes_height = paragraph.line_count(width);
        } else {
            self.cache.game_notes_height = 0;
        }

        self.cache.last_viewport_width = width;

        self.get_total_content_height()
    }

    pub fn get_total_content_height(&self) -> u16 {
        let team_content_height = match self.active_team {
            Home => self.cache.home_team_cache.total_content_height,
            Away => self.cache.away_team_cache.total_content_height,
        };
        team_content_height + self.cache.game_notes_height as u16
    }

    pub fn get_batting_rows<'a>(
        &'a self,
        team: HomeOrAway,
    ) -> impl Iterator<Item = Vec<Cell<'a>>> + 'a {
        self.boxscore.to_batting_table_rows(team)
    }

    pub fn get_pitching_rows<'a>(
        &'a self,
        team: HomeOrAway,
    ) -> impl Iterator<Item = Vec<Cell<'a>>> + 'a {
        self.boxscore.to_pitching_table_rows(team)
    }

    pub fn get_batting_notes_paragraph(&self, team: HomeOrAway) -> Option<&Paragraph<'static>> {
        match team {
            Home => self.cache.home_team_cache.batting_notes_paragraph.as_ref(),
            Away => self.cache.away_team_cache.batting_notes_paragraph.as_ref(),
        }
    }

    pub fn get_game_notes_paragraph(&self) -> Option<&Paragraph<'static>> {
        self.cache.game_notes_paragraph.as_ref()
    }

    /// Get individual component heights to create the layout constraints.
    pub fn get_content_heights(&self, team: HomeOrAway) -> (u16, u16, u16, u16, u16) {
        let (batting_height, pitching_height, notes_height) = match team {
            Home => (
                self.cache.home_team_cache.batting_stats_height,
                self.cache.home_team_cache.pitching_stats_height,
                self.cache.home_team_cache.batting_notes_height,
            ),
            Away => (
                self.cache.away_team_cache.batting_stats_height,
                self.cache.away_team_cache.pitching_stats_height,
                self.cache.away_team_cache.batting_notes_height,
            ),
        };

        (
            batting_height as u16,
            notes_height as u16,
            pitching_height as u16,
            self.cache.game_notes_height as u16,
            self.get_total_content_height(),
        )
    }

    pub fn reset_scroll(&mut self) {
        self.scroll = 0;
        self.scroll_state = ScrollbarState::default();
    }

    pub fn scroll_down(&mut self) {
        if self.scroll < self.max_scroll {
            self.scroll += 1;
            self.scroll_state = self.scroll_state.position(self.scroll);
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll = self.scroll.saturating_sub(1);
            self.scroll_state = self.scroll_state.position(self.scroll);
        }
    }
}
