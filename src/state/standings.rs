use crate::components::constants::{DIVISIONS, lookup_team_by_id};
use crate::components::date_selector::DateSelector;
use crate::components::standings::{Division, Standing, Team};
use crate::state::team_page::TeamPageState;
use chrono::NaiveDate;
use chrono_tz::Tz;
use mlbt_api::player::PeopleResponse;
use mlbt_api::schedule::ScheduleResponse;
use mlbt_api::season::GameType;
use mlbt_api::standings::StandingsResponse;
use mlbt_api::team::{RosterResponse, RosterType, TransactionsResponse};
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;
use tui::widgets::TableState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    ByDivision,
    Overall,
    WildCard,
}

/// Stores the state for rendering the standings. The `standings` field is a nested Vec to make
/// displaying by division easier.
pub struct StandingsState {
    pub state: TableState,
    pub favorite_team: Option<Team>,
    pub standings: Vec<Division>,
    pub league_standings: Vec<Standing>,
    /// The wild card race, grouped by league.
    pub wild_card_standings: Vec<Division>,
    pub team_ids: Vec<u16>,
    pub date_selector: DateSelector,
    pub view_mode: ViewMode,
    /// Used to skip selecting division names in the table.
    division_row_indices: HashSet<usize>,
    pub team_page: Option<TeamPageState>,
}

/// Map a division id to its league id. Pre-1969 standings have no divisions and are already grouped
/// by league, and historical teams predating divisions have an id of `0`.
fn league_id(division_id: u16) -> u16 {
    match division_id {
        200..=202 => 103,
        203..=205 => 104,
        id => id,
    }
}

impl Default for StandingsState {
    fn default() -> Self {
        Self {
            state: TableState::default(),
            standings: Division::create_divisions(),
            league_standings: vec![],
            wild_card_standings: vec![],
            // populated by `generate_ids`, which also records which rows are group headers
            team_ids: vec![],
            date_selector: DateSelector::default(),
            view_mode: ViewMode::ByDivision,
            division_row_indices: HashSet::new(),
            favorite_team: None,
            team_page: None,
        }
    }
}

impl StandingsState {
    /// Update the data from the API.
    pub fn update(&mut self, standings: &StandingsResponse) {
        self.standings = Division::create_table(standings, self.favorite_team);
        self.league_standings = self.get_teams_by_record();
        self.wild_card_standings = self.get_wild_card_teams();
        self.team_ids = self.generate_ids();
        self.reset_selection();
    }

    pub fn reset_selection(&mut self) {
        let idx = self
            .favorite_team
            .and_then(|team| self.favorite_team_index(team))
            .or_else(|| self.first_selectable_index());
        self.state.select(idx);
    }

    /// Reapply the favorite team ordering/highlight to already loaded standings data.
    pub fn apply_favorite_team(&mut self, favorite_team: Option<Team>) {
        self.favorite_team = favorite_team;

        if !self
            .standings
            .iter()
            .any(|division| !division.standings.is_empty())
        {
            return;
        }

        Division::sort_by_favorite(&mut self.standings, favorite_team);
        self.league_standings = self.get_teams_by_record();
        self.wild_card_standings = self.get_wild_card_teams();
        self.team_ids = self.generate_ids();
        self.reset_selection();
    }

    /// Set the date from the validated input string from the date picker.
    pub fn set_date_from_valid_input(&mut self, date: NaiveDate) {
        self.date_selector.set_date_from_valid_input(date);
    }

    /// Set the date using Left/Right arrow keys to move a single day at a time.
    pub fn set_date_with_arrows(&mut self, forward: bool) -> NaiveDate {
        self.date_selector.set_date_with_arrows(forward)
    }

    /// Cycle through the division, league, and wild card views.
    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::ByDivision => ViewMode::Overall,
            ViewMode::Overall => ViewMode::WildCard,
            ViewMode::WildCard => ViewMode::ByDivision,
        };
        self.team_ids = self.generate_ids();
        self.reset_selection();
    }

    /// The wild card race: teams not leading their division, grouped by league and ordered by the
    /// rank the API already computed, which resolves tiebreakers for us. Only division leaders lack
    /// a rank, so seasons before the wild card era yield an empty view.
    fn get_wild_card_teams(&self) -> Vec<Division> {
        let mut leagues: BTreeMap<u16, Vec<Standing>> = BTreeMap::new();
        for division in &self.standings {
            leagues.entry(league_id(division.id)).or_default().extend(
                division
                    .standings
                    .iter()
                    .filter(|s| s.wild_card_rank.is_some())
                    .cloned(),
            );
        }

        let mut leagues: Vec<Division> = leagues
            .into_iter()
            .filter(|(_, standings)| !standings.is_empty())
            .map(|(id, mut standings)| {
                standings.sort_by_key(|s| s.wild_card_rank);
                Division {
                    name: DIVISIONS.get(&id).unwrap_or(&"Unknown").to_string(),
                    id,
                    standings,
                }
            })
            .collect();

        // show the favorite team's league first, matching how the division view is ordered
        if let Some(team) = self.favorite_team {
            let favorite = league_id(team.division_id);
            leagues.sort_by_key(|league| league.id != favorite);
        }

        leagues
    }

    /// Get all teams sorted by record (for overall view)
    fn get_teams_by_record(&self) -> Vec<Standing> {
        let mut teams: Vec<Standing> = self
            .standings
            .iter()
            .flat_map(|division| division.standings.iter())
            .cloned()
            .collect();

        teams.sort_by(|a, b| {
            // Sort by wins descending, then losses ascending
            b.wins.cmp(&a.wins).then(a.losses.cmp(&b.losses))
        });

        teams
    }

    fn generate_ids(&mut self) -> Vec<u16> {
        self.division_row_indices.clear(); // clear previous indices in case they change, e.g. historical standings

        match self.view_mode {
            ViewMode::ByDivision | ViewMode::WildCard => {
                let groups = if self.view_mode == ViewMode::WildCard {
                    &self.wild_card_standings
                } else {
                    &self.standings
                };
                let mut ids = Vec::with_capacity(36); // 30 teams, 6 divisions
                let mut count = 0;
                for division in groups {
                    ids.push(division.id);
                    self.division_row_indices.insert(count);
                    for team in &division.standings {
                        ids.push(team.team.id);
                    }
                    count += 1 + division.standings.len();
                }
                ids
            }
            ViewMode::Overall => {
                // For overall view, just collect team IDs without divisions
                self.league_standings
                    .iter()
                    .map(|standing| standing.team.id)
                    .collect()
            }
        }
    }

    /// The row index of the favorite team, or `None` when it isn't in the current view.
    fn favorite_team_index(&self, team: Team) -> Option<usize> {
        match self.view_mode {
            ViewMode::ByDivision => {
                // Find team position including division headers
                let mut current_idx = 0;
                for division in &self.standings {
                    current_idx += 1; // Skip division header
                    for standing in &division.standings {
                        if standing.team.id == team.id {
                            return Some(current_idx);
                        }
                        current_idx += 1;
                    }
                }
                None
            }
            // group ids never collide with team ids, so the row ids are enough to find the team.
            // A division leader isn't in the wild card race, so it may not be there at all.
            ViewMode::WildCard => self.team_ids.iter().position(|&id| id == team.id),
            ViewMode::Overall => {
                // Find team position in sorted list
                self.league_standings
                    .iter()
                    .position(|standing| standing.team.id == team.id)
            }
        }
    }

    /// The first row that isn't a group header, or `None` when there's nothing selectable.
    fn first_selectable_index(&self) -> Option<usize> {
        (0..self.team_ids.len()).find(|i| !self.skip_division(*i))
    }

    /// Check if any team has been eliminated from their division.
    fn any_team_eliminated(&self) -> bool {
        self.standings
            .iter()
            .flat_map(|division| &division.standings)
            .any(|standing| standing.clinch.division_eliminated)
    }

    /// The view mode that should show an elimination # column, or `None` to leave the column off.
    pub fn elimination_column(&self) -> Option<ViewMode> {
        // overall mixes both leagues, so a division number means nothing there
        if self.view_mode == ViewMode::Overall || !self.any_team_eliminated() {
            return None;
        }

        Some(self.view_mode)
    }

    pub fn has_team_page(&self) -> bool {
        self.team_page.is_some()
    }

    /// Close the top layer overlay.
    pub fn close_overlay(&mut self) {
        if let Some(tp) = &mut self.team_page {
            if tp.player_profile.is_some() {
                tp.player_profile = None;
            } else {
                self.team_page = None;
            }
        }
    }

    pub fn update_team_page(
        &mut self,
        team_id: u16,
        date: NaiveDate,
        schedule: &ScheduleResponse,
        roster: &RosterResponse,
        transactions: &TransactionsResponse,
        tz: Tz,
    ) {
        let team = lookup_team_by_id(team_id).unwrap_or_default();
        self.team_page = Some(TeamPageState::from_response(
            team,
            date,
            schedule,
            roster,
            transactions,
            tz,
        ));
    }

    pub fn update_team_roster(
        &mut self,
        team_id: u16,
        roster: &RosterResponse,
        roster_type: RosterType,
    ) {
        if let Some(tp) = &mut self.team_page
            && tp.team.id == team_id
        {
            tp.update_roster(roster, roster_type);
        }
    }

    pub fn update_team_player_profile(&mut self, data: Arc<PeopleResponse>, game_type: GameType) {
        if let Some(tp) = &mut self.team_page {
            tp.update_player_profile(data, game_type);
        }
    }

    pub fn get_selected(&self) -> Option<u16> {
        let selected = self.state.selected()?;
        if self.skip_division(selected) {
            return None;
        }
        self.team_ids.get(selected).copied()
    }

    fn skip_division(&self, index: usize) -> bool {
        // the overall view is the only one without group header rows
        self.view_mode != ViewMode::Overall && self.division_row_indices.contains(&index)
    }

    fn move_forward(&self, current: usize) -> usize {
        let len = self.team_ids.len();
        if current >= len - 1 { 0 } else { current + 1 }
    }

    fn move_backward(&self, current: usize) -> usize {
        let len = self.team_ids.len();
        if current == 0 { len - 1 } else { current - 1 }
    }

    pub fn next(&mut self) {
        let len = self.team_ids.len();
        if len == 0 {
            return;
        }

        let start = self.state.selected().unwrap_or(0);
        let mut i = self.move_forward(start);

        if self.skip_division(i) {
            i = self.move_forward(i);
        }

        self.state.select(Some(i));

        // Reset offset when wrapping to beginning
        if i < start {
            self.state = TableState::default();
            self.state.select(Some(i));
        }
    }

    pub fn previous(&mut self) {
        let len = self.team_ids.len();
        if len == 0 {
            return;
        }

        let start = self.state.selected().unwrap_or(0);
        let mut i = self.move_backward(start);

        if self.skip_division(i) {
            i = self.move_backward(i);
        }

        self.state.select(Some(i));

        // Reset offset when wrapping to end
        if i > start {
            self.state = TableState::default();
            self.state.select(Some(i));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn standing(team_id: u16, wins: u8, losses: u8) -> Standing {
        Standing {
            team: lookup_team_by_id(team_id).unwrap(),
            wins,
            losses,
            ..Standing::default()
        }
    }

    #[test]
    fn apply_favorite_team_reorders_divisions_and_selects_team() {
        let mut state = StandingsState {
            state: TableState::default(),
            favorite_team: None,
            standings: vec![
                Division {
                    id: 200,
                    name: "AL West".to_string(),
                    standings: vec![standing(108, 10, 5)],
                },
                Division {
                    id: 201,
                    name: "AL East".to_string(),
                    standings: vec![standing(147, 11, 4)],
                },
                Division {
                    id: 205,
                    name: "NL Central".to_string(),
                    standings: vec![standing(112, 9, 6)],
                },
            ],
            league_standings: vec![],
            wild_card_standings: vec![],
            team_ids: vec![],
            date_selector: DateSelector::default(),
            view_mode: ViewMode::ByDivision,
            division_row_indices: HashSet::new(),
            team_page: None,
        };

        state.apply_favorite_team(lookup_team_by_id(147));

        assert_eq!(
            state
                .standings
                .iter()
                .map(|division| division.id)
                .collect::<Vec<_>>(),
            vec![201, 202, 200, 203, 204, 205]
                .into_iter()
                .filter(|id| [200, 201, 205].contains(id))
                .collect::<Vec<_>>()
        );
        assert_eq!(state.get_selected(), Some(147));
    }

    /// Division leaders, which the API gives no wild card rank, plus ranked contenders.
    fn wild_card_state() -> StandingsState {
        let leader = |id| standing(id, 20, 5);
        let contender = |id, rank| Standing {
            wild_card_rank: Some(rank),
            ..standing(id, 10, 10)
        };

        StandingsState {
            standings: vec![
                Division {
                    id: 201,
                    name: "AL East".to_string(),
                    standings: vec![leader(147), contender(111, 2)],
                },
                Division {
                    id: 202,
                    name: "AL Central".to_string(),
                    standings: vec![leader(145), contender(114, 1)],
                },
                Division {
                    id: 205,
                    name: "NL Central".to_string(),
                    standings: vec![leader(158), contender(112, 1)],
                },
            ],
            ..StandingsState::default()
        }
    }

    #[test]
    fn wild_card_view_groups_by_league_without_division_leaders() {
        let mut state = wild_card_state();
        state.apply_favorite_team(None);

        // leaders dropped, AL and NL split, each ordered by the API's wild card rank
        assert_eq!(
            state
                .wild_card_standings
                .iter()
                .map(|l| (l.id, l.standings.iter().map(|s| s.team.id).collect()))
                .collect::<Vec<(u16, Vec<u16>)>>(),
            vec![(103, vec![114, 111]), (104, vec![112])]
        );

        // an NL favorite team puts the NL first
        state.apply_favorite_team(lookup_team_by_id(112));
        assert_eq!(
            state
                .wild_card_standings
                .iter()
                .map(|l| l.id)
                .collect::<Vec<u16>>(),
            vec![104, 103]
        );

        // and an AL favorite team puts the AL first
        state.apply_favorite_team(lookup_team_by_id(111));
        assert_eq!(
            state
                .wild_card_standings
                .iter()
                .map(|l| l.id)
                .collect::<Vec<u16>>(),
            vec![103, 104]
        );

        state.view_mode = ViewMode::WildCard;
        state.apply_favorite_team(None);

        // the league name rows aren't selectable
        assert_eq!(state.get_selected(), Some(114));
        state.next();
        assert_eq!(state.get_selected(), Some(111));
        state.next();
        assert_eq!(state.get_selected(), Some(112));
    }

    #[test]
    fn group_headers_are_never_selected() {
        // nothing is selectable before the first standings response
        let mut state = StandingsState::default();
        state.reset_selection();
        assert_eq!(state.get_selected(), None);

        // the placeholder divisions have no teams, so cycling the view before data loads leaves
        // nothing but headers, and navigating them stays inert
        state.toggle_view_mode(); // Overall
        state.toggle_view_mode(); // WildCard
        state.toggle_view_mode(); // back to ByDivision, now with ids generated
        assert_eq!(state.team_ids, vec![200, 201, 202, 203, 204, 205]);
        assert_eq!(state.get_selected(), None);
        state.next();
        assert_eq!(state.get_selected(), None);
    }

    #[test]
    fn navigation_only_rests_on_teams() {
        let mut state = wild_card_state();
        state.apply_favorite_team(None);

        // one full cycle through the division view, wrapping back to the top
        let visited: Vec<Option<u16>> = (0..state.team_ids.len())
            .map(|_| {
                let id = state.get_selected();
                state.next();
                id
            })
            .collect();

        // every division header is skipped, and the wrap lands back on the first team
        assert_eq!(
            visited,
            [147, 111, 145, 114, 158, 112, 147, 111, 145].map(Some)
        );
    }

    #[test]
    fn a_division_leader_favorite_falls_back_to_the_first_wild_card_team() {
        let mut state = wild_card_state();
        state.view_mode = ViewMode::WildCard;
        state.apply_favorite_team(lookup_team_by_id(147)); // AL East leader, not in the race

        // its league sorts first, so the fallback is that league's top ranked contender
        assert_eq!(state.team_ids, vec![103, 114, 111, 104, 112]);
        assert_eq!(state.get_selected(), Some(114));
    }

    #[test]
    fn elimination_column_waits_until_a_team_is_out_of_the_division_race() {
        let mut state = wild_card_state();
        state.apply_favorite_team(None);

        // early season, everyone still has a live number
        assert_eq!(state.elimination_column(), None);

        // the first elimination lands weeks before the first clinch
        state.standings[0].standings[0].clinch.division_eliminated = true;
        assert_eq!(state.elimination_column(), Some(ViewMode::ByDivision));

        state.view_mode = ViewMode::Overall;
        assert_eq!(state.elimination_column(), None);
    }

    #[test]
    fn a_season_without_wild_card_ranks_has_nothing_to_select() {
        let mut state = StandingsState {
            standings: vec![Division {
                id: 201,
                name: "AL East".to_string(),
                standings: vec![standing(147, 20, 5), standing(111, 10, 10)],
            }],
            view_mode: ViewMode::WildCard,
            ..StandingsState::default()
        };
        state.apply_favorite_team(None);

        assert!(state.wild_card_standings.is_empty());
        assert!(state.team_ids.is_empty());
        assert_eq!(state.get_selected(), None);

        // navigation is inert rather than trapped on header rows
        state.next();
        state.previous();
        assert_eq!(state.get_selected(), None);
    }
}
