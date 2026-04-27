use crate::components::stats::table::StatType;
use crate::state::messages::{NetworkRequest, NetworkResponse};
use chrono::NaiveDate;
use log::debug;
use mlbt_api::schedule::{AbstractGameState, ScheduleResponse};
use mlbt_api::team::RosterType;
use std::collections::HashMap;
use std::time::{Duration, Instant};

const FINAL_GAME_TTL: Duration = Duration::from_hours(1);
const PRUNE_AGE: Duration = Duration::from_hours(48);
pub const PRUNE_INTERVAL: Duration = Duration::from_hours(2);

/// Cache key identifying a unique API request. PlayerProfile is excluded because it requires
/// owned data for transformation (StatSplits stores Vec<Split> directly).
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum CacheKey {
    Schedule {
        date: NaiveDate,
    },
    GameData {
        game_id: u64,
    },
    Standings {
        date: NaiveDate,
    },
    Stats {
        date: NaiveDate,
        stat_type: StatType,
    },
    TeamPage {
        team_id: u16,
        date: NaiveDate,
    },
    TeamRoster {
        team_id: u16,
        season: i32,
        roster_type: RosterType,
    },
}

struct CacheEntry {
    response: NetworkResponse,
    fetched_at: Instant,
    ttl: Duration,
}

impl CacheEntry {
    fn is_fresh(&self) -> bool {
        self.fetched_at.elapsed() < self.ttl
    }
}

pub struct NetworkCache {
    entries: HashMap<CacheKey, CacheEntry>,
    /// Tracks the last known abstract game state per game_id, used to detect transitions to Final.
    /// The `Instant` records when the state was last observed from a schedule response.
    game_states: HashMap<u64, (AbstractGameState, Instant)>,
    /// Schedule dates this process has observed with at least one non `Final` game.
    /// Used to distinguish live observations (invalidate on Final) from historical browsing
    /// (no invalidation, since current caches already reflect the result).
    mutable_dates: HashMap<NaiveDate, Instant>,
    last_prune: Instant,
}

impl NetworkCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            game_states: HashMap::new(),
            mutable_dates: HashMap::new(),
            last_prune: Instant::now(),
        }
    }

    /// Map a network request to a cache key. Returns `None` for uncacheable requests (Initialize, PlayerProfile).
    pub fn key_for(request: &NetworkRequest) -> Option<CacheKey> {
        match request {
            NetworkRequest::Schedule { date } => Some(CacheKey::Schedule { date: *date }),
            NetworkRequest::GameData { game_id } => Some(CacheKey::GameData { game_id: *game_id }),
            NetworkRequest::Standings { date } => Some(CacheKey::Standings { date: *date }),
            NetworkRequest::Stats { date, stat_type } => Some(CacheKey::Stats {
                date: *date,
                stat_type: *stat_type,
            }),
            NetworkRequest::TeamPage { team_id, date } => Some(CacheKey::TeamPage {
                team_id: *team_id,
                date: *date,
            }),
            NetworkRequest::TeamRoster {
                team_id,
                season,
                roster_type,
            } => Some(CacheKey::TeamRoster {
                team_id: *team_id,
                season: *season,
                roster_type: *roster_type,
            }),
            // Not cached: Initialize is one-shot, PlayerProfile consumes owned data
            NetworkRequest::Initialize | NetworkRequest::PlayerProfile { .. } => None,
        }
    }

    /// TTL for a cache key. Final games get an extended TTL since their data is static.
    fn ttl_for(&self, key: &CacheKey) -> Duration {
        match key {
            CacheKey::GameData { game_id } => {
                if self.is_final_game(*game_id) {
                    FINAL_GAME_TTL
                } else {
                    Duration::from_secs(10)
                }
            }
            CacheKey::Schedule { .. } => Duration::from_secs(30),
            CacheKey::Standings { .. } => Duration::from_secs(1800),
            CacheKey::Stats { .. } => Duration::from_secs(1800),
            CacheKey::TeamPage { .. } => Duration::from_secs(600),
            CacheKey::TeamRoster { .. } => Duration::from_secs(1800),
        }
    }

    /// Get a cached response if it exists and is fresh.
    pub fn get(&self, key: &CacheKey) -> Option<NetworkResponse> {
        let entry = self.entries.get(key)?;
        if entry.is_fresh() {
            debug!("cache hit for {key:?}");
            Some(entry.response.clone())
        } else {
            debug!("cache expired for {key:?}");
            None
        }
    }

    fn has_fresh_mutable_date(&self, date: NaiveDate) -> bool {
        matches!(
            self.mutable_dates.get(&date),
            Some(observed_at) if observed_at.elapsed() < PRUNE_AGE
        )
    }

    /// Insert a response with the default TTL for its key type. If the response is a schedule,
    /// automatically updates game state tracking and invalidates affected caches.
    pub fn insert(&mut self, key: CacheKey, response: NetworkResponse) {
        let date = match &key {
            CacheKey::Schedule { date } => Some(*date),
            _ => None,
        };
        let ttl = self.ttl_for(&key);
        self.entries.insert(
            key,
            CacheEntry {
                response: response.clone(),
                fetched_at: Instant::now(),
                ttl,
            },
        );
        if let Some(date) = date
            && let NetworkResponse::ScheduleLoaded { schedule } = &response
        {
            self.update_game_states(date, schedule);
        }
    }

    /// Remove a cache entry, forcing the next request to fetch from the API.
    pub fn invalidate(&mut self, key: &CacheKey) {
        self.entries.remove(key);
    }

    /// Remove cache entries and game states older than `PRUNE_AGE`.
    /// Only runs once per `PRUNE_INTERVAL` to avoid unnecessary work.
    pub fn prune(&mut self) {
        if self.last_prune.elapsed() < PRUNE_INTERVAL {
            return;
        }
        self.last_prune = Instant::now();

        let before = self.entries.len();
        self.entries
            .retain(|_, entry| entry.fetched_at.elapsed() < PRUNE_AGE);
        self.game_states
            .retain(|_, (_, observed_at)| observed_at.elapsed() < PRUNE_AGE);
        self.mutable_dates
            .retain(|_, observed_at| observed_at.elapsed() < PRUNE_AGE);
        let removed = before - self.entries.len();
        debug!("pruned {removed} stale cache entries");
    }

    /// True if the game's last observed abstract state is `Final`.
    pub fn is_final_game(&self, game_id: u64) -> bool {
        matches!(
            self.game_states.get(&game_id),
            Some((AbstractGameState::Final, _))
        )
    }

    /// Update tracked game states from a schedule response. When a game we previously observed
    /// transitions to Final:
    /// - Extends that game's cached data TTL (data is now static)
    /// - Invalidates standings/stats caches for dates >= schedule_date (cumulative data stale)
    /// - Invalidates team page caches for the teams involved (season wide schedules stale)
    ///
    /// If we've never observed the game or the schedule_date in a non-final state (e.g. user
    /// browses an old date for the first time and all games there are already Final), we record
    /// the state but don't invalidate, there's no evidence our current caches are stale.
    fn update_game_states(&mut self, schedule_date: NaiveDate, schedule: &ScheduleResponse) {
        let mut invalidate = false;
        let mut affected_team_ids: Vec<u16> = Vec::new();
        let mut saw_non_final = false;

        for date_entry in &schedule.dates {
            let Some(games) = &date_entry.games else {
                continue;
            };
            for game in games {
                let Some(new_state) = &game.status.abstract_game_state else {
                    continue;
                };
                let game_id = game.game_pk;
                let prior_state = self.game_states.get(&game_id).map(|(s, _)| *s);
                let was_final = matches!(prior_state, Some(AbstractGameState::Final));
                let is_final = matches!(new_state, AbstractGameState::Final);

                if !is_final {
                    saw_non_final = true;
                }

                if is_final && !was_final {
                    debug!("game {game_id} went Final, extending cache TTL");
                    if let Some(entry) = self.entries.get_mut(&CacheKey::GameData { game_id }) {
                        entry.ttl = FINAL_GAME_TTL;
                        entry.fetched_at = Instant::now();
                    }
                    // Only trust this transition as "live" if we previously saw the game
                    // non-final OR we've observed this date as mutable in this process.
                    let prior_non_final = matches!(
                        prior_state,
                        Some(AbstractGameState::Live | AbstractGameState::Preview)
                    );
                    if prior_non_final || self.has_fresh_mutable_date(schedule_date) {
                        invalidate = true;
                        affected_team_ids.push(game.teams.home.team.id);
                        affected_team_ids.push(game.teams.away.team.id);
                    }
                }

                self.game_states
                    .insert(game_id, (*new_state, Instant::now()));
            }
        }

        if saw_non_final {
            self.mutable_dates.insert(schedule_date, Instant::now());
        }

        if invalidate {
            debug!(
                "invalidating standings/stats (>= {schedule_date}) and team pages for {affected_team_ids:?}"
            );
            self.entries.retain(|key, _| match key {
                CacheKey::Standings { date: d } | CacheKey::Stats { date: d, .. } => {
                    *d < schedule_date
                }
                CacheKey::TeamPage { team_id, .. } => !affected_team_ids.contains(team_id),
                _ => true,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlbt_api::client::StatGroup;
    use mlbt_api::schedule::ScheduleResponse;
    use mlbt_api::season::GameType;
    use std::sync::Arc;

    const TEST_DATE: &str = "2026-04-13";

    fn test_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 4, 13).unwrap()
    }

    fn schedule_key() -> CacheKey {
        CacheKey::Schedule { date: test_date() }
    }

    fn schedule_response() -> NetworkResponse {
        NetworkResponse::ScheduleLoaded {
            schedule: Arc::new(ScheduleResponse::default()),
        }
    }

    #[test]
    fn get_returns_none_for_empty_cache() {
        let cache = NetworkCache::new();
        assert!(cache.get(&schedule_key()).is_none());
    }

    #[test]
    fn get_returns_cached_response() {
        let mut cache = NetworkCache::new();
        cache.insert(schedule_key(), schedule_response());
        assert!(cache.get(&schedule_key()).is_some());
    }

    #[test]
    fn get_returns_none_after_ttl_expires() {
        let mut cache = NetworkCache::new();
        let key = schedule_key();
        // Insert with fetched_at in the past so the TTL has already elapsed
        cache.entries.insert(
            key.clone(),
            CacheEntry {
                response: schedule_response(),
                fetched_at: Instant::now() - Duration::from_secs(60),
                ttl: Duration::from_secs(30),
            },
        );
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn key_for_returns_none_for_initialize() {
        assert!(NetworkCache::key_for(&NetworkRequest::Initialize).is_none());
    }

    #[test]
    fn key_for_returns_none_for_player_profile() {
        let req = NetworkRequest::PlayerProfile {
            player_id: 1,
            group: StatGroup::Hitting,
            date: NaiveDate::from_ymd_opt(2026, 4, 13).unwrap(),
            game_type: GameType::RegularSeason,
        };
        assert!(NetworkCache::key_for(&req).is_none());
    }

    fn make_schedule(date: &str, games: Vec<(u64, AbstractGameState)>) -> ScheduleResponse {
        use mlbt_api::schedule::{Dates, Game, Status};
        ScheduleResponse {
            dates: vec![Dates {
                date: Some(date.to_string()),
                games: Some(
                    games
                        .into_iter()
                        .map(|(id, state)| Game {
                            game_pk: id,
                            status: Status {
                                abstract_game_state: Some(state),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .collect(),
                ),
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    fn game_data_response() -> NetworkResponse {
        use mlbt_api::live::LiveResponse;
        use mlbt_api::win_probability::WinProbabilityResponse;
        NetworkResponse::GameDataLoaded {
            game: Arc::new(LiveResponse::default()),
            win_probability: Arc::new(WinProbabilityResponse::default()),
        }
    }

    #[test]
    fn final_game_extends_cache_ttl() {
        let mut cache = NetworkCache::new();
        let game_id = 123;

        // Cache game data with default TTL
        cache.insert(CacheKey::GameData { game_id }, game_data_response());
        assert_eq!(
            cache.entries[&CacheKey::GameData { game_id }].ttl,
            Duration::from_secs(10)
        );

        // First schedule: game is Live
        let schedule = make_schedule(TEST_DATE, vec![(game_id, AbstractGameState::Live)]);
        cache.update_game_states(test_date(), &schedule);
        assert_eq!(
            cache.entries[&CacheKey::GameData { game_id }].ttl,
            Duration::from_secs(10)
        );

        // Game goes Final
        let schedule = make_schedule(TEST_DATE, vec![(game_id, AbstractGameState::Final)]);
        cache.update_game_states(test_date(), &schedule);
        assert_eq!(
            cache.entries[&CacheKey::GameData { game_id }].ttl,
            Duration::from_secs(3600)
        );
    }

    #[test]
    fn final_game_invalidates_standings_and_stats() {
        let mut cache = NetworkCache::new();
        let date = NaiveDate::from_ymd_opt(2026, 4, 13).unwrap();

        // Cache standings and stats
        cache.insert(
            CacheKey::Standings { date },
            NetworkResponse::StandingsLoaded {
                standings: Arc::new(mlbt_api::standings::StandingsResponse::default()),
            },
        );
        cache.insert(
            CacheKey::Stats {
                date,
                stat_type: StatType::default(),
            },
            NetworkResponse::StatsLoaded {
                stats: Arc::new(mlbt_api::stats::StatsResponse::default()),
            },
        );
        assert!(cache.get(&CacheKey::Standings { date }).is_some());

        // Game goes from Live to Final
        let schedule = make_schedule(TEST_DATE, vec![(123, AbstractGameState::Live)]);
        cache.update_game_states(test_date(), &schedule);
        let schedule = make_schedule(TEST_DATE, vec![(123, AbstractGameState::Final)]);
        cache.update_game_states(test_date(), &schedule);

        // Standings and stats should be invalidated
        assert!(cache.get(&CacheKey::Standings { date }).is_none());
        assert!(
            cache
                .get(&CacheKey::Stats {
                    date,
                    stat_type: StatType::default(),
                })
                .is_none()
        );
    }

    #[test]
    fn already_final_game_does_not_re_invalidate() {
        let mut cache = NetworkCache::new();
        let date = NaiveDate::from_ymd_opt(2026, 4, 13).unwrap();

        // Game already Final
        let schedule = make_schedule(TEST_DATE, vec![(123, AbstractGameState::Final)]);
        cache.update_game_states(test_date(), &schedule);

        // Cache standings after the game went Final
        cache.insert(
            CacheKey::Standings { date },
            NetworkResponse::StandingsLoaded {
                standings: Arc::new(mlbt_api::standings::StandingsResponse::default()),
            },
        );

        // Same schedule again, game is still Final, not a new transition
        cache.update_game_states(test_date(), &schedule);
        assert!(cache.get(&CacheKey::Standings { date }).is_some());
    }

    #[test]
    fn unknown_to_final_does_not_invalidate_without_mutable_observation() {
        let mut cache = NetworkCache::new();
        let date = NaiveDate::from_ymd_opt(2026, 4, 13).unwrap();

        cache.insert(
            CacheKey::Standings { date },
            NetworkResponse::StandingsLoaded {
                standings: Arc::new(mlbt_api::standings::StandingsResponse::default()),
            },
        );

        // Game appears for the first time as Final. No prior state and no mutable date observed.
        // This is likely historical browsing so current caches are not assumed stale.
        let schedule = make_schedule(TEST_DATE, vec![(456, AbstractGameState::Final)]);
        cache.update_game_states(test_date(), &schedule);

        assert!(cache.get(&CacheKey::Standings { date }).is_some());
    }

    #[test]
    fn final_ttl_expires_from_transition_time_not_fetch_time() {
        let mut cache = NetworkCache::new();
        let game_id = 789;

        // Insert game data with fetched_at in the past
        cache.entries.insert(
            CacheKey::GameData { game_id },
            CacheEntry {
                response: game_data_response(),
                fetched_at: Instant::now() - Duration::from_secs(600),
                ttl: Duration::from_secs(10),
            },
        );

        // Game goes Final — should reset fetched_at to now
        let schedule = make_schedule(TEST_DATE, vec![(game_id, AbstractGameState::Final)]);
        cache.update_game_states(test_date(), &schedule);

        let entry = &cache.entries[&CacheKey::GameData { game_id }];
        assert_eq!(entry.ttl, FINAL_GAME_TTL);
        assert!(entry.fetched_at.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn prune_removes_old_entries_and_old_game_states() {
        let mut cache = NetworkCache::new();
        // Force prune to run immediately
        cache.last_prune = Instant::now() - PRUNE_INTERVAL;

        // Insert an entry far in the past
        cache.entries.insert(
            CacheKey::GameData { game_id: 111 },
            CacheEntry {
                response: game_data_response(),
                fetched_at: Instant::now() - PRUNE_AGE - Duration::from_secs(1),
                ttl: Duration::from_hours(1),
            },
        );
        cache
            .game_states
            .insert(111, (AbstractGameState::Final, Instant::now() - PRUNE_AGE));
        cache
            .mutable_dates
            .insert(test_date(), Instant::now() - PRUNE_AGE);

        // Insert a fresh entry
        cache.insert(CacheKey::GameData { game_id: 222 }, game_data_response());
        cache
            .game_states
            .insert(222, (AbstractGameState::Live, Instant::now()));
        cache.mutable_dates.insert(
            NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
            Instant::now(),
        );

        cache.prune();

        assert!(
            !cache
                .entries
                .contains_key(&CacheKey::GameData { game_id: 111 })
        );
        assert!(!cache.game_states.contains_key(&111));
        assert!(!cache.mutable_dates.contains_key(&test_date()));
        assert!(
            cache
                .entries
                .contains_key(&CacheKey::GameData { game_id: 222 })
        );
        assert!(cache.game_states.contains_key(&222));
        assert!(
            cache
                .mutable_dates
                .contains_key(&NaiveDate::from_ymd_opt(2026, 4, 14).unwrap())
        );
    }

    #[test]
    fn prune_skips_when_interval_not_elapsed() {
        let mut cache = NetworkCache::new();

        // Insert an old entry
        cache.entries.insert(
            CacheKey::GameData { game_id: 111 },
            CacheEntry {
                response: game_data_response(),
                fetched_at: Instant::now() - PRUNE_AGE - Duration::from_secs(1),
                ttl: Duration::from_hours(1),
            },
        );

        // Prune should be a no-op since last_prune is recent
        cache.prune();
        assert!(
            cache
                .entries
                .contains_key(&CacheKey::GameData { game_id: 111 })
        );
    }

    #[test]
    fn unobserved_historical_final_does_not_invalidate() {
        let mut cache = NetworkCache::new();
        let past = NaiveDate::from_ymd_opt(2026, 4, 10).unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 4, 13).unwrap();

        cache.insert(
            CacheKey::Standings { date: today },
            NetworkResponse::StandingsLoaded {
                standings: Arc::new(mlbt_api::standings::StandingsResponse::default()),
            },
        );

        // User browses back to a past date so all games there are already Final.
        // We've never observed game 123 nor the past date in a non final state.
        let schedule = make_schedule("2026-04-10", vec![(123, AbstractGameState::Final)]);
        cache.update_game_states(past, &schedule);

        // No evidence the cached standings are stale.
        assert!(cache.get(&CacheKey::Standings { date: today }).is_some());
    }

    #[test]
    fn stale_mutable_date_does_not_authorize_invalidation() {
        let mut cache = NetworkCache::new();
        let past = NaiveDate::from_ymd_opt(2026, 4, 10).unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 4, 13).unwrap();

        cache.insert(
            CacheKey::Standings { date: today },
            NetworkResponse::StandingsLoaded {
                standings: Arc::new(mlbt_api::standings::StandingsResponse::default()),
            },
        );

        cache
            .mutable_dates
            .insert(past, Instant::now() - PRUNE_AGE - Duration::from_secs(1));

        let schedule = make_schedule("2026-04-10", vec![(123, AbstractGameState::Final)]);
        cache.update_game_states(past, &schedule);

        assert!(cache.get(&CacheKey::Standings { date: today }).is_some());
    }

    #[test]
    fn observed_non_final_then_final_invalidates() {
        let mut cache = NetworkCache::new();
        let date = test_date();

        cache.insert(
            CacheKey::Standings { date },
            NetworkResponse::StandingsLoaded {
                standings: Arc::new(mlbt_api::standings::StandingsResponse::default()),
            },
        );

        // Observe the game as Live first (establishes that we watched this date mutable)
        let schedule = make_schedule(TEST_DATE, vec![(123, AbstractGameState::Live)]);
        cache.update_game_states(date, &schedule);

        // Then observe the transition to Final.  This is a real signal
        let schedule = make_schedule(TEST_DATE, vec![(123, AbstractGameState::Final)]);
        cache.update_game_states(date, &schedule);

        assert!(cache.get(&CacheKey::Standings { date }).is_none());
    }

    #[test]
    fn final_invalidates_from_transition_date_onward() {
        let mut cache = NetworkCache::new();
        let apr10 = NaiveDate::from_ymd_opt(2026, 4, 10).unwrap();
        let apr13 = NaiveDate::from_ymd_opt(2026, 4, 13).unwrap();
        let apr14 = NaiveDate::from_ymd_opt(2026, 4, 14).unwrap();

        for date in [apr10, apr13, apr14] {
            cache.insert(
                CacheKey::Standings { date },
                NetworkResponse::StandingsLoaded {
                    standings: Arc::new(mlbt_api::standings::StandingsResponse::default()),
                },
            );
        }

        // Game on Apr 13 goes Final
        let schedule = make_schedule(TEST_DATE, vec![(123, AbstractGameState::Live)]);
        cache.update_game_states(test_date(), &schedule);
        let schedule = make_schedule(TEST_DATE, vec![(123, AbstractGameState::Final)]);
        cache.update_game_states(test_date(), &schedule);

        // Earlier date preserved, transition date and later invalidated
        assert!(cache.get(&CacheKey::Standings { date: apr10 }).is_some());
        assert!(cache.get(&CacheKey::Standings { date: apr13 }).is_none());
        assert!(cache.get(&CacheKey::Standings { date: apr14 }).is_none());
    }

    #[test]
    fn final_invalidates_team_page_for_involved_teams() {
        use mlbt_api::schedule::{Dates, Game, IdNameLink, Status, TeamInfo, Teams};
        let mut cache = NetworkCache::new();
        let date = test_date();

        // Cache team pages for teams 111 (home), 222 (away), and 333 (uninvolved)
        for team_id in [111, 222, 333] {
            cache.insert(
                CacheKey::TeamPage { team_id, date },
                NetworkResponse::TeamPageLoaded {
                    team_id,
                    date,
                    schedule: Arc::new(mlbt_api::schedule::ScheduleResponse::default()),
                    roster: Arc::new(mlbt_api::team::RosterResponse::default()),
                    transactions: Arc::new(mlbt_api::team::TransactionsResponse::default()),
                },
            );
        }

        // Build a schedule with a Live game between teams 111 and 222
        let make_game = |state| Game {
            game_pk: 1,
            status: Status {
                abstract_game_state: Some(state),
                ..Default::default()
            },
            teams: Teams {
                home: TeamInfo {
                    team: IdNameLink {
                        id: 111,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                away: TeamInfo {
                    team: IdNameLink {
                        id: 222,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            ..Default::default()
        };
        let schedule = ScheduleResponse {
            dates: vec![Dates {
                date: Some(TEST_DATE.to_string()),
                games: Some(vec![make_game(AbstractGameState::Live)]),
                ..Default::default()
            }],
            ..Default::default()
        };
        cache.update_game_states(date, &schedule);

        // Now the game goes Final
        let schedule = ScheduleResponse {
            dates: vec![Dates {
                date: Some(TEST_DATE.to_string()),
                games: Some(vec![make_game(AbstractGameState::Final)]),
                ..Default::default()
            }],
            ..Default::default()
        };
        cache.update_game_states(date, &schedule);

        // Involved teams invalidated, uninvolved preserved
        assert!(
            cache
                .get(&CacheKey::TeamPage { team_id: 111, date })
                .is_none()
        );
        assert!(
            cache
                .get(&CacheKey::TeamPage { team_id: 222, date })
                .is_none()
        );
        assert!(
            cache
                .get(&CacheKey::TeamPage { team_id: 333, date })
                .is_some()
        );
    }

    #[test]
    fn is_final_game_reflects_last_observed_state() {
        let mut cache = NetworkCache::new();
        assert!(!cache.is_final_game(42));

        let schedule = make_schedule(TEST_DATE, vec![(42, AbstractGameState::Live)]);
        cache.update_game_states(test_date(), &schedule);
        assert!(!cache.is_final_game(42));

        let schedule = make_schedule(TEST_DATE, vec![(42, AbstractGameState::Final)]);
        cache.update_game_states(test_date(), &schedule);
        assert!(cache.is_final_game(42));
    }
}
