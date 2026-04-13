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
const PRUNE_INTERVAL: Duration = Duration::from_hours(2);

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
    last_prune: Instant,
}

impl NetworkCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            game_states: HashMap::new(),
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

    /// Default TTL for a cache key.
    fn default_ttl(key: &CacheKey) -> Duration {
        match key {
            CacheKey::GameData { .. } => Duration::from_secs(10),
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

    /// Insert a response with the default TTL for its key type. If the response is a schedule,
    /// automatically updates game state tracking and invalidates affected caches.
    pub fn insert(&mut self, key: CacheKey, response: NetworkResponse) {
        let date = match &key {
            CacheKey::Schedule { date } => Some(*date),
            _ => None,
        };
        let ttl = Self::default_ttl(&key);
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
        let removed = before - self.entries.len();
        debug!("pruned {removed} stale cache entries");
    }

    /// Update tracked game states from a schedule response. When any game transitions to Final:
    /// - Extends that game's cached data TTL to 1 hour (data is now static)
    /// - Invalidates standings and stats caches for the request date only
    fn update_game_states(&mut self, date: NaiveDate, schedule: &ScheduleResponse) {
        let mut any_went_final = false;

        for date_entry in &schedule.dates {
            let Some(games) = &date_entry.games else {
                continue;
            };
            for game in games {
                let Some(new_state) = &game.status.abstract_game_state else {
                    continue;
                };
                let game_id = game.game_pk;
                let was_final = matches!(
                    self.game_states.get(&game_id),
                    Some((AbstractGameState::Final, _))
                );
                let is_final = matches!(new_state, AbstractGameState::Final);

                if is_final && !was_final {
                    debug!("game {game_id} went Final, extending cache TTL");
                    if let Some(entry) = self.entries.get_mut(&CacheKey::GameData { game_id }) {
                        entry.ttl = FINAL_GAME_TTL;
                        entry.fetched_at = Instant::now();
                    }
                    any_went_final = true;
                }

                self.game_states
                    .insert(game_id, (*new_state, Instant::now()));
            }
        }

        if any_went_final {
            debug!("invalidating standings and stats for {date} after game(s) went Final");
            self.entries.retain(|key, _| match key {
                CacheKey::Standings { date: d } | CacheKey::Stats { date: d, .. } => *d != date,
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
                stat_type: StatType {
                    group: StatGroup::Hitting,
                    team_player: crate::components::stats::table::TeamOrPlayer::Player,
                },
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
                    stat_type: StatType {
                        group: StatGroup::Hitting,
                        team_player: crate::components::stats::table::TeamOrPlayer::Player,
                    },
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
    fn unknown_to_final_counts_as_transition() {
        let mut cache = NetworkCache::new();
        let date = NaiveDate::from_ymd_opt(2026, 4, 13).unwrap();

        cache.insert(
            CacheKey::Standings { date },
            NetworkResponse::StandingsLoaded {
                standings: Arc::new(mlbt_api::standings::StandingsResponse::default()),
            },
        );

        // Game appears for the first time as Final (no prior state tracked)
        let schedule = make_schedule(TEST_DATE, vec![(456, AbstractGameState::Final)]);
        cache.update_game_states(test_date(), &schedule);

        assert!(cache.get(&CacheKey::Standings { date }).is_none());
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

        // Insert a fresh entry
        cache.insert(CacheKey::GameData { game_id: 222 }, game_data_response());
        cache
            .game_states
            .insert(222, (AbstractGameState::Live, Instant::now()));

        cache.prune();

        assert!(
            !cache
                .entries
                .contains_key(&CacheKey::GameData { game_id: 111 })
        );
        assert!(!cache.game_states.contains_key(&111));
        assert!(
            cache
                .entries
                .contains_key(&CacheKey::GameData { game_id: 222 })
        );
        assert!(cache.game_states.contains_key(&222));
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
    fn final_on_one_date_does_not_invalidate_other_dates() {
        let mut cache = NetworkCache::new();
        let apr13 = NaiveDate::from_ymd_opt(2026, 4, 13).unwrap();
        let apr10 = NaiveDate::from_ymd_opt(2026, 4, 10).unwrap();

        // Cache standings for both dates
        cache.insert(
            CacheKey::Standings { date: apr13 },
            NetworkResponse::StandingsLoaded {
                standings: Arc::new(mlbt_api::standings::StandingsResponse::default()),
            },
        );
        cache.insert(
            CacheKey::Standings { date: apr10 },
            NetworkResponse::StandingsLoaded {
                standings: Arc::new(mlbt_api::standings::StandingsResponse::default()),
            },
        );

        // Game on Apr 13 goes Final
        let schedule = make_schedule(TEST_DATE, vec![(123, AbstractGameState::Live)]);
        cache.update_game_states(test_date(), &schedule);
        let schedule = make_schedule(TEST_DATE, vec![(123, AbstractGameState::Final)]);
        cache.update_game_states(test_date(), &schedule);

        // Apr 13 standings invalidated, Apr 10 standings preserved
        assert!(cache.get(&CacheKey::Standings { date: apr13 }).is_none());
        assert!(cache.get(&CacheKey::Standings { date: apr10 }).is_some());
    }
}
