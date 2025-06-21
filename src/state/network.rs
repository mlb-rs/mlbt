use crate::components::stats::{StatType, TeamOrPlayer};
use crate::{NetworkRequest, NetworkResponse};
use chrono::NaiveDate;
use log::{debug, error};
use mlb_api::client::{ApiResult, MLBApi, MLBApiBuilder};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::mpsc;

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
    requests: mpsc::Receiver<NetworkRequest>,
    responses: mpsc::Sender<NetworkResponse>,
    is_loading: Arc<AtomicBool>,
}

impl NetworkWorker {
    pub fn new(
        requests: mpsc::Receiver<NetworkRequest>,
        responses: mpsc::Sender<NetworkResponse>,
    ) -> Self {
        Self {
            client: MLBApiBuilder::default().build().unwrap(),
            requests,
            responses,
            is_loading: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn run(mut self) {
        while let Some(request) = self.requests.recv().await {
            self.start_loading_animation().await;
            let result = match request {
                NetworkRequest::Schedule { date } => self.handle_load_schedule(date).await,
                NetworkRequest::GameData { game_id } => self.handle_load_game_data(game_id).await,
                NetworkRequest::Standings { date } => self.handle_load_standings(date).await,
                NetworkRequest::Stats { date, stat_type } => {
                    self.handle_load_stats(date, stat_type).await
                }
            };
            debug!("request complete");
            self.stop_loading_animation(result.is_ok()).await;

            let response =
                result.unwrap_or_else(|err| NetworkResponse::Error { message: err.log() });
            if let Err(e) = self.responses.send(response).await {
                error!("Failed to send network response: {e}");
                break;
            }
        }
    }

    async fn handle_load_schedule(&self, date: NaiveDate) -> ApiResult<NetworkResponse> {
        debug!("loading schedule for {date}");
        let schedule = self.client.get_schedule_date(date).await?;
        Ok(NetworkResponse::ScheduleLoaded { schedule })
    }

    async fn handle_load_game_data(&self, game_id: u64) -> ApiResult<NetworkResponse> {
        debug!("loading game data for {game_id}");
        let (game, wp) = tokio::join!(
            self.client.get_live_data(game_id),
            self.client.get_win_probability(game_id),
        );
        Ok(NetworkResponse::GameDataLoaded {
            game: Box::new(game?),
            win_probability: wp?,
        })
    }

    async fn handle_load_standings(&self, date: NaiveDate) -> ApiResult<NetworkResponse> {
        debug!("loading standings for {date}");
        let standings = self.client.get_standings(date).await?;
        Ok(NetworkResponse::StandingsLoaded { standings })
    }

    async fn handle_load_stats(
        &self,
        date: NaiveDate,
        stat_type: StatType,
    ) -> ApiResult<NetworkResponse> {
        debug!("loading {stat_type:?} stats for {date}");
        let StatType { team_player, group } = stat_type;
        let stats = match team_player {
            TeamOrPlayer::Team => self.client.get_team_stats_on_date(group, date).await,
            TeamOrPlayer::Player => self.client.get_player_stats_on_date(group, date).await,
        }?;
        Ok(NetworkResponse::StatsLoaded { stats })
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
