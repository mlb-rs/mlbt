use crate::NetworkRequest;
use crate::app::{App, MenuItem};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;

pub struct PeriodicRefresher {
    network_requests: mpsc::Sender<NetworkRequest>,
}

impl PeriodicRefresher {
    pub fn new(network_requests: mpsc::Sender<NetworkRequest>) -> Self {
        Self { network_requests }
    }

    pub async fn run(self, app: std::sync::Arc<tokio::sync::Mutex<App>>) {
        let mut live_interval = interval(Duration::from_secs(10)); // Live data every 10s
        let mut schedule_interval = interval(Duration::from_secs(60)); // Schedule every minute
        let mut standings_interval = interval(Duration::from_secs(1800)); // Standings every 30 minutes

        loop {
            tokio::select! {
                // Live game data updates (frequent for active games)
                _ = live_interval.tick() => {
                    let (active_tab, game_id) = {
                        let app = app.lock().await;
                        (app.state.active_tab, app.state.gameday.current_game_id())
                    };

                    if active_tab == MenuItem::Gameday && game_id > 0 {
                        let _ = self.network_requests.send(NetworkRequest::GameData { game_id }).await;
                    }
                }

                // Schedule updates (moderate frequency)
                _ = schedule_interval.tick() => {
                    let (active_tab, date, game_id) = {
                        let app = app.lock().await;
                        (app.state.active_tab, app.state.schedule.date_selector.date, app.state.schedule.get_selected_game_opt().unwrap_or(0))
                    };

                    if active_tab == MenuItem::Scoreboard {
                        let _ = self.network_requests.send(NetworkRequest::Schedule { date }).await;
                        if game_id > 0 {
                            let _ = self.network_requests.send(NetworkRequest::GameData { game_id }).await;
                        }
                    }
                }

                // Standings updates (low frequency)
                _ = standings_interval.tick() => {
                    let (active_tab, date) = {
                        let app = app.lock().await;
                        (app.state.active_tab, app.state.standings.date_selector.date)
                    };

                    if active_tab == MenuItem::Standings {
                        let _ = self.network_requests.send(NetworkRequest::Standings { date }).await;
                    }
                }
            }
        }
    }
}
