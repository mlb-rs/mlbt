use crate::components::boxscore::{Boxscore, Note};
use crate::components::game::live_game::PlayerId;
use crate::components::game::player::Player;
use crate::state::app_state::HomeOrAway;
use crate::state::app_state::HomeOrAway::{Away, Home};
use mlb_api::live::LiveResponse;
use std::collections::HashMap;
use tui::prelude::*;
use tui::widgets::{Block, Cell, Paragraph, ScrollbarState, Wrap};

#[derive(Default)]
pub struct BoxscoreState {
    pub team: HomeOrAway,
    pub boxscore: Boxscore,
    pub scroll: usize,
    pub scroll_state: ScrollbarState,
    pub cache: BoxscoreRenderCache,
    pub max_scroll: usize,
}

#[derive(Default)]
pub struct BoxscoreRenderCache {
    // Cache paragraphs for length calculation
    pub home_batting_notes_paragraph: Option<Paragraph<'static>>,
    pub away_batting_notes_paragraph: Option<Paragraph<'static>>,
    pub game_notes_paragraph: Option<Paragraph<'static>>,
    // Cached table heights
    pub home_batting_height: u16,
    pub away_batting_height: u16,
    pub home_pitching_height: u16,
    pub away_pitching_height: u16,
    // Cached paragraph heights
    pub home_notes_height: u16,
    pub away_notes_height: u16,
    pub game_notes_height: u16,
    pub home_total_content_height: u16,
    pub away_total_content_height: u16,
    // Track last viewport width to know when to recalculate
    pub last_viewport_width: u16,
}

impl BoxscoreState {
    pub fn update(&mut self, live_game: &LiveResponse, players: &HashMap<PlayerId, Player>) {
        self.boxscore = Boxscore::from_live_data(live_game, players);
        self.update_cache();
    }

    pub fn reset_scroll(&mut self) {
        self.scroll = 0;
        self.scroll_state = ScrollbarState::default();
    }

    fn update_cache(&mut self) {
        // cache full paragraphs to be used later to calculate height based on wrapped line length
        self.cache.home_batting_notes_paragraph =
            Self::build_paragraph(self.boxscore.get_batting_notes(Home));
        self.cache.away_batting_notes_paragraph =
            Self::build_paragraph(self.boxscore.get_batting_notes(Away));
        self.cache.game_notes_paragraph = Self::build_paragraph(self.boxscore.get_game_notes());

        // calculate static table heights
        self.cache.home_batting_height = self.boxscore.count_batting_table_rows(Home) as u16 + 1; // +1 for header
        self.cache.away_batting_height = self.boxscore.count_batting_table_rows(Away) as u16 + 1; // +1 for header
        self.cache.home_pitching_height = self.boxscore.count_pitching_table_rows(Home) as u16 + 1; // +1 for header
        self.cache.away_pitching_height = self.boxscore.count_pitching_table_rows(Away) as u16 + 1; // +1 for header

        // Reset viewport width to force recalculation of wrapped content heights
        self.cache.last_viewport_width = 0;
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

    fn calculate_heights_for_width(&mut self, viewport_width: u16) {
        if self.cache.last_viewport_width == viewport_width {
            // already calculated for this width
            return;
        }

        // calculate wrapped text heights for current viewport width
        self.cache.home_notes_height = self
            .cache
            .home_batting_notes_paragraph
            .as_ref()
            .map(|p| p.line_count(viewport_width) as u16)
            .unwrap_or(0);

        self.cache.away_notes_height = self
            .cache
            .away_batting_notes_paragraph
            .as_ref()
            .map(|p| p.line_count(viewport_width) as u16)
            .unwrap_or(0);

        self.cache.game_notes_height = self
            .cache
            .game_notes_paragraph
            .as_ref()
            .map(|p| p.line_count(viewport_width) as u16)
            .unwrap_or(0);

        // calculate total content height for each team
        self.cache.home_total_content_height = self.cache.home_batting_height
            + self.cache.home_notes_height
            + self.cache.home_pitching_height
            + self.cache.game_notes_height
            + 3; // +3 for spacing

        self.cache.away_total_content_height = self.cache.away_batting_height
            + self.cache.away_notes_height
            + self.cache.away_pitching_height
            + self.cache.game_notes_height
            + 3; // +3 for spacing

        self.cache.last_viewport_width = viewport_width;
    }

    pub fn update_scroll_state_for_viewport(&mut self, viewport_height: u16, viewport_width: u16) {
        // use the height to determine height after text wrapping
        self.calculate_heights_for_width(viewport_width);

        let total_content_height = match self.team {
            Home => self.cache.home_total_content_height,
            Away => self.cache.away_total_content_height,
        };

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

    pub fn get_batting_rows(&self, team: HomeOrAway) -> Vec<Vec<Cell>> {
        self.boxscore.to_batting_table_rows(team)
    }

    pub fn get_pitching_rows(&self, team: HomeOrAway) -> Vec<Vec<Cell>> {
        self.boxscore.to_pitching_table_rows(team)
    }

    pub fn get_batting_notes_paragraph(&self, team: HomeOrAway) -> Option<&Paragraph<'static>> {
        match team {
            Home => self.cache.home_batting_notes_paragraph.as_ref(),
            Away => self.cache.away_batting_notes_paragraph.as_ref(),
        }
    }

    pub fn get_game_notes_paragraph(&self) -> Option<&Paragraph<'static>> {
        self.cache.game_notes_paragraph.as_ref()
    }

    pub fn get_content_heights(&self, team: HomeOrAway) -> (u16, u16, u16, u16) {
        let (batting_height, pitching_height, notes_height) = match team {
            Home => (
                self.cache.home_batting_height,
                self.cache.home_pitching_height,
                self.cache.home_notes_height,
            ),
            Away => (
                self.cache.away_batting_height,
                self.cache.away_pitching_height,
                self.cache.away_notes_height,
            ),
        };

        (
            batting_height,
            notes_height,
            pitching_height,
            self.cache.game_notes_height,
        )
    }
}
