use crate::components::stats::{StatType, TeamOrPlayer};
use crate::messages::{NetworkRequest, NetworkResponse};
use chrono::NaiveDate;
use mlb_api::client::{MLBApi, MLBApiBuilder};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::mpsc;

const SPINNER_CHARS: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

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
            let response = match request {
                NetworkRequest::Schedule { date } => self.handle_load_schedule(date).await,
                NetworkRequest::GameData { game_id } => self.handle_load_game_data(game_id).await,
                NetworkRequest::Standings { date } => self.handle_load_standings(date).await,
                NetworkRequest::Stats { date, stat_type } => {
                    self.handle_load_stats(date, stat_type).await
                }
            };
            self.stop_loading_animation().await;

            if let Some(response) = response {
                if let Err(e) = self.responses.send(response).await {
                    eprintln!("Failed to send network response: {}", e);
                    break;
                }
            }
        }
    }

    async fn handle_load_schedule(&self, date: NaiveDate) -> Option<NetworkResponse> {
        let schedule = self.client.get_schedule_date(date).await;
        Some(NetworkResponse::ScheduleLoaded { schedule })
    }

    async fn handle_load_game_data(&self, game_id: u64) -> Option<NetworkResponse> {
        let game = self.client.get_live_data(game_id).await;
        Some(NetworkResponse::GameDataLoaded {
            game: Box::new(game),
        })
    }

    async fn handle_load_standings(&self, date: NaiveDate) -> Option<NetworkResponse> {
        let standings = self.client.get_standings(date).await;
        Some(NetworkResponse::StandingsLoaded { standings })
    }

    async fn handle_load_stats(
        &self,
        date: NaiveDate,
        stat_type: StatType,
    ) -> Option<NetworkResponse> {
        let StatType { team_player, group } = stat_type;
        let stats = match team_player {
            TeamOrPlayer::Team => self.client.get_team_stats_on_date(group, date).await,
            TeamOrPlayer::Player => self.client.get_player_stats_on_date(group, date).await,
        };
        Some(NetworkResponse::StatsLoaded { stats })
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

    async fn stop_loading_animation(&self) {
        self.is_loading.store(false, Ordering::Relaxed);

        // Add a small delay to ensure the animation task stops
        tokio::time::sleep(Duration::from_millis(15)).await;

        let _ = self
            .responses
            .send(NetworkResponse::LoadingStateChanged {
                loading_state: LoadingState::default(),
            })
            .await;
    }
}
