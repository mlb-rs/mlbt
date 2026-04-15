use crate::components::constants::register_teams;
use crate::components::stats::table::{StatType, TeamOrPlayer};
use crate::state::cache::{NetworkCache, PRUNE_INTERVAL};
use crate::state::messages::{NetworkRequest, NetworkResponse, RefreshableRequest};
use chrono::{Datelike, NaiveDate};
use log::{debug, error, warn};
use mlbt_api::client::{ApiResult, MLBApi, MLBApiBuilder, StatGroup};
use mlbt_api::season::{SeasonInfo, game_type_for_date};
use mlbt_api::team::RosterType;
use mlbt_api::teams::SportId;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::mpsc;

const DEBOUNCE_DELAY: Duration = Duration::from_millis(250);
const SPINNER_CHARS: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
pub const ERROR_CHAR: char = '!';

#[derive(Debug, Copy, Clone)]
pub struct LoadingState {
    pub is_loading: bool,
    pub spinner_char: char,
}

impl Default for LoadingState {
    fn default() -> Self {
        Self {
            is_loading: false,
            spinner_char: ' ',
        }
    }
}

pub struct NetworkWorker {
    client: MLBApi,
    requests: mpsc::Receiver<RefreshableRequest>,
    responses: mpsc::Sender<NetworkResponse>,
    is_loading: Arc<AtomicBool>,
    /// Cached season info, keyed by year.
    season_info: Option<(i32, SeasonInfo)>,
    cache: NetworkCache,
}

impl NetworkWorker {
    pub fn new(
        requests: mpsc::Receiver<RefreshableRequest>,
        responses: mpsc::Sender<NetworkResponse>,
    ) -> Self {
        Self {
            client: MLBApiBuilder::default().build().unwrap(),
            requests,
            responses,
            is_loading: Arc::new(AtomicBool::new(false)),
            season_info: None,
            cache: NetworkCache::new(),
        }
    }

    pub async fn run(mut self) {
        let debounce_sleep = tokio::time::sleep(DEBOUNCE_DELAY);
        tokio::pin!(debounce_sleep);

        let mut debounce_game_id: Option<u64> = None;
        let mut prune_interval = tokio::time::interval(PRUNE_INTERVAL);
        prune_interval.tick().await; // consume the immediate first tick

        let mut overflow: Vec<RefreshableRequest> = Vec::new();

        loop {
            tokio::select! {
                msg = self.requests.recv() => {
                    let Some(mut refreshable) = msg else { break };

                    // Coalesce: if this is a debounced GameData, drain queued ones
                    if let NetworkRequest::GameData { game_id } = refreshable.request
                        && !refreshable.force_refresh
                    {
                        let latest = self.drain_game_data(game_id, &mut overflow);
                        refreshable.request = NetworkRequest::GameData { game_id: latest };
                    }

                    // The primary request cancels any pending debounce when it's not a new
                    // debounceable GameData. Overflow items don't as they arrived earlier in time.
                    let is_debounceable_game =
                        matches!(refreshable.request, NetworkRequest::GameData { .. })
                            && !refreshable.force_refresh;
                    if !is_debounceable_game {
                        debounce_game_id = None;
                    }

                    self.handle_request(refreshable, &mut debounce_game_id, debounce_sleep.as_mut()).await;

                    // Process any non-GameData requests collected during draining
                    for req in overflow.drain(..) {
                        self.handle_request(req, &mut debounce_game_id, debounce_sleep.as_mut()).await;
                    }
                }
                () = &mut debounce_sleep, if debounce_game_id.is_some() => {
                    // Drain any last-moment queued GameData requests
                    let game_id = self.drain_game_data(debounce_game_id.take().unwrap(), &mut overflow);
                    self.fetch_and_send(NetworkRequest::GameData { game_id }).await;

                    for req in overflow.drain(..) {
                        self.handle_request(req, &mut debounce_game_id, debounce_sleep.as_mut()).await;
                    }
                }
                _ = prune_interval.tick() => {
                    self.cache.prune();
                }
            }
        }
    }

    /// Drain queued `GameData` requests from the channel and return the latest game_id.
    /// Non-`GameData` requests encountered during draining are collected for later processing.
    /// This coalesces across intervening non-game requests, which can reorder them.
    fn drain_game_data(&mut self, mut game_id: u64, overflow: &mut Vec<RefreshableRequest>) -> u64 {
        while let Ok(next) = self.requests.try_recv() {
            match next.request {
                NetworkRequest::GameData {
                    game_id: new_game_id,
                } if !next.force_refresh => {
                    game_id = new_game_id;
                }
                _ => overflow.push(next),
            }
        }
        game_id
    }

    async fn handle_request(
        &mut self,
        refreshable: RefreshableRequest,
        debounce_game_id: &mut Option<u64>,
        mut debounce_sleep: Pin<&mut tokio::time::Sleep>,
    ) {
        let request = refreshable.request;
        let cache_key = NetworkCache::key_for(&request);

        // Force refresh invalidates so the cache check below misses
        if refreshable.force_refresh
            && let Some(key) = cache_key.as_ref()
        {
            self.cache.invalidate(key);
        }

        // Check cache before making API calls
        if let Some(key) = cache_key.as_ref()
            && let Some(cached) = self.cache.get(key)
        {
            let _ = self.responses.send(cached).await;
            return;
        }

        // Debounce game data requests (unless force refresh)
        if let NetworkRequest::GameData { game_id } = request
            && !refreshable.force_refresh
        {
            debug!("debouncing game data request for {game_id}");
            *debounce_game_id = Some(game_id);
            debounce_sleep
                .as_mut()
                .reset(tokio::time::Instant::now() + DEBOUNCE_DELAY);
            return;
        }

        self.fetch_and_send(request).await;
    }

    async fn fetch_and_send(&mut self, request: NetworkRequest) {
        let cache_key = NetworkCache::key_for(&request);

        self.start_loading_animation().await;
        let result = match request {
            NetworkRequest::Initialize => self.handle_initialize().await,
            NetworkRequest::Schedule { date } => self.handle_load_schedule(date).await,
            NetworkRequest::GameData { game_id } => self.handle_load_game_data(game_id).await,
            NetworkRequest::Standings { date } => self.handle_load_standings(date).await,
            NetworkRequest::Stats { date, stat_type } => {
                self.handle_load_stats(date, stat_type).await
            }
            NetworkRequest::PlayerProfile {
                player_id,
                group,
                date,
                game_type,
            } => {
                self.handle_load_player_profile(player_id, group, date, game_type)
                    .await
            }
            NetworkRequest::TeamPage { team_id, date } => {
                self.handle_load_team_page(team_id, date).await
            }
            NetworkRequest::TeamRoster {
                team_id,
                season,
                roster_type,
            } => {
                self.handle_load_team_roster(team_id, season, roster_type)
                    .await
            }
        };
        debug!("request complete");
        self.stop_loading_animation(result.is_ok()).await;

        // Cache successful responses
        if let Ok(ref response) = result
            && let Some(key) = cache_key
        {
            self.cache.insert(key, response.clone());
        }

        let response = result.unwrap_or_else(|err| NetworkResponse::Error { message: err.log() });
        if let Err(e) = self.responses.send(response).await {
            error!("Failed to send network response: {e}");
        }
    }

    async fn handle_load_schedule(&self, date: NaiveDate) -> ApiResult<NetworkResponse> {
        debug!("loading schedule for {date}");
        let schedule = self.client.get_schedule_date(date).await?;
        Ok(NetworkResponse::ScheduleLoaded {
            schedule: Arc::new(schedule),
        })
    }

    async fn handle_load_game_data(&self, game_id: u64) -> ApiResult<NetworkResponse> {
        debug!("loading game data for {game_id}");
        let (game, wp) = tokio::join!(
            self.client.get_live_data(game_id),
            self.client.get_win_probability(game_id),
        );
        Ok(NetworkResponse::GameDataLoaded {
            game: Arc::new(game?),
            win_probability: Arc::new(wp?),
        })
    }

    async fn handle_load_standings(&mut self, date: NaiveDate) -> ApiResult<NetworkResponse> {
        debug!("loading standings for {date}");
        self.ensure_season_info(date).await;
        let game_type = game_type_for_date(date, self.cached_season_info());
        let standings = self.client.get_standings(date, game_type).await?;
        Ok(NetworkResponse::StandingsLoaded {
            standings: Arc::new(standings),
        })
    }

    async fn handle_load_stats(
        &mut self,
        date: NaiveDate,
        stat_type: StatType,
    ) -> ApiResult<NetworkResponse> {
        debug!("loading {stat_type:?} stats for {date}");
        self.ensure_season_info(date).await;
        let game_type = game_type_for_date(date, self.cached_season_info());
        let StatType { team_player, group } = stat_type;
        let stats = match team_player {
            TeamOrPlayer::Team => {
                self.client
                    .get_team_stats_on_date(group, date, game_type)
                    .await
            }
            TeamOrPlayer::Player => {
                self.client
                    .get_player_stats_on_date(group, date, game_type)
                    .await
            }
        }?;
        Ok(NetworkResponse::StatsLoaded {
            stats: Arc::new(stats),
        })
    }

    async fn handle_load_player_profile(
        &self,
        player_id: u64,
        group: StatGroup,
        date: NaiveDate,
        game_type: mlbt_api::season::GameType,
    ) -> ApiResult<NetworkResponse> {
        debug!("loading player profile for {player_id} ({group}) on {date}");
        let data = self
            .client
            .get_player_profile(player_id, group, date.year(), game_type)
            .await?;
        Ok(NetworkResponse::PlayerProfileLoaded {
            data: Arc::new(data),
            game_type,
        })
    }

    async fn handle_load_team_page(
        &self,
        team_id: u16,
        date: NaiveDate,
    ) -> ApiResult<NetworkResponse> {
        debug!("loading team page for team {team_id} on {date}");
        let (schedule, roster, transactions) = tokio::try_join!(
            self.client.get_team_schedule(team_id, date.year()),
            self.client
                .get_team_roster(team_id, date.year(), RosterType::Active),
            self.client
                .get_team_transactions(team_id, date - chrono::Duration::days(30), date),
        )?;
        Ok(NetworkResponse::TeamPageLoaded {
            team_id,
            date,
            schedule: Arc::new(schedule),
            roster: Arc::new(roster),
            transactions: Arc::new(transactions),
        })
    }

    async fn handle_load_team_roster(
        &self,
        team_id: u16,
        season: i32,
        roster_type: RosterType,
    ) -> ApiResult<NetworkResponse> {
        debug!("loading {roster_type} roster for team {team_id} in {season} season");
        let roster = self
            .client
            .get_team_roster(team_id, season, roster_type)
            .await?;
        Ok(NetworkResponse::TeamRosterLoaded {
            team_id,
            roster: Arc::new(roster),
            roster_type,
        })
    }

    /// Best-effort initialization that always returns Ok so the app can proceed even if the API
    /// calls fails.
    /// - Fetch teams from the API to populate the dynamic team cache.
    async fn handle_initialize(&self) -> ApiResult<NetworkResponse> {
        match self
            .client
            .get_teams(&[SportId::Mlb, SportId::International])
            .await
        {
            Ok(response) => {
                debug!("loaded {} teams from API", response.teams.len());
                register_teams(response.teams);
            }
            Err(e) => warn!("Failed to fetch teams, using static fallback: {}", e.log()),
        }
        Ok(NetworkResponse::Initialized)
    }

    fn cached_season_info(&self) -> Option<&SeasonInfo> {
        self.season_info.as_ref().map(|(_, info)| info)
    }

    /// Lazily fetch and cache season info, re-fetching if the year changes.
    async fn ensure_season_info(&mut self, date: NaiveDate) {
        let year = date.year();
        if let Some((cached_year, _)) = &self.season_info
            && *cached_year == year
        {
            return;
        }
        match self.client.get_season_info(year).await {
            Ok(Some(info)) => self.season_info = Some((year, info)),
            Ok(None) => self.season_info = None,
            Err(e) => {
                warn!("Failed to fetch season info, using fallback: {}", e.log());
                self.season_info = None;
            }
        }
    }

    async fn start_loading_animation(&self) {
        self.is_loading.store(true, Ordering::Relaxed);

        // Send initial loading state
        let mut loading_state = LoadingState {
            is_loading: true,
            spinner_char: SPINNER_CHARS[0],
        };
        let _ = self
            .responses
            .send(NetworkResponse::LoadingStateChanged { loading_state })
            .await;

        // Spawn a background task for spinner animation
        let responses = self.responses.clone();
        let is_loading = self.is_loading.clone();

        tokio::spawn(async move {
            let mut spinner_index = 1;
            // 30 FPS animation
            let mut interval = tokio::time::interval(Duration::from_millis(33));

            loop {
                interval.tick().await;

                // Check loading state before sending update and exit immediately when loading stops
                if !is_loading.load(Ordering::Relaxed) {
                    break;
                }

                let spinner_char = SPINNER_CHARS[spinner_index];
                spinner_index = (spinner_index + 1) % SPINNER_CHARS.len();
                loading_state.spinner_char = spinner_char;

                // Send spinner update to UI
                let _ = responses
                    .send(NetworkResponse::LoadingStateChanged { loading_state })
                    .await;
            }
        });
    }

    async fn stop_loading_animation(&self, is_ok: bool) {
        self.is_loading.store(false, Ordering::Relaxed);

        // Add a small delay to ensure the animation task stops
        tokio::time::sleep(Duration::from_millis(15)).await;

        let loading_state = match is_ok {
            true => LoadingState {
                is_loading: false,
                spinner_char: ' ',
            },
            false => LoadingState {
                is_loading: false,
                spinner_char: ERROR_CHAR,
            },
        };

        let _ = self
            .responses
            .send(NetworkResponse::LoadingStateChanged { loading_state })
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn worker() -> NetworkWorker {
        let (_request_tx, request_rx) = mpsc::channel(32);
        let (response_tx, _response_rx) = mpsc::channel(32);
        NetworkWorker::new(request_rx, response_tx)
    }

    fn game_request(game_id: u64) -> RefreshableRequest {
        RefreshableRequest {
            request: NetworkRequest::GameData { game_id },
            force_refresh: false,
        }
    }

    fn force_refresh_game_request(game_id: u64) -> RefreshableRequest {
        RefreshableRequest {
            request: NetworkRequest::GameData { game_id },
            force_refresh: true,
        }
    }

    fn schedule_request(date: NaiveDate) -> RefreshableRequest {
        RefreshableRequest {
            request: NetworkRequest::Schedule { date },
            force_refresh: false,
        }
    }

    #[test]
    fn drain_game_data_returns_latest_game_id_from_burst() {
        let mut worker = worker();
        let (tx, rx) = mpsc::channel(32);
        worker.requests = rx;
        let mut overflow = Vec::new();

        tx.try_send(game_request(2)).unwrap();
        tx.try_send(game_request(3)).unwrap();
        tx.try_send(game_request(4)).unwrap();

        let latest = worker.drain_game_data(1, &mut overflow);

        assert_eq!(latest, 4);
        assert!(overflow.is_empty());
    }

    #[test]
    fn drain_game_data_preserves_non_game_requests_in_overflow() {
        let mut worker = worker();
        let (tx, rx) = mpsc::channel(32);
        worker.requests = rx;
        let mut overflow = Vec::new();
        let date = NaiveDate::from_ymd_opt(2025, 4, 13).unwrap();

        tx.try_send(schedule_request(date)).unwrap();
        tx.try_send(game_request(9)).unwrap();

        let latest = worker.drain_game_data(7, &mut overflow);

        assert_eq!(latest, 9);
        assert_eq!(overflow.len(), 1);
        assert!(matches!(
            overflow[0].request,
            NetworkRequest::Schedule { date: d } if d == date
        ));
    }

    #[test]
    fn drain_game_data_does_not_coalesce_force_refresh_game_requests() {
        let mut worker = worker();
        let (tx, rx) = mpsc::channel(32);
        worker.requests = rx;
        let mut overflow = Vec::new();

        tx.try_send(force_refresh_game_request(99)).unwrap();
        tx.try_send(game_request(42)).unwrap();

        let latest = worker.drain_game_data(1, &mut overflow);

        assert_eq!(latest, 42);
        assert_eq!(overflow.len(), 1);
        assert!(matches!(
            overflow[0],
            RefreshableRequest {
                request: NetworkRequest::GameData { game_id: 99 },
                force_refresh: true,
            }
        ));
    }
}
