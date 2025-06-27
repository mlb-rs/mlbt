use chrono::NaiveDate;
use mlb_api::client::MLBApi;
use mlb_api::client::MLBApiBuilder;
use mlb_api::client::StatGroup;
use mockito::{Matcher, ServerGuard};
use std::time::Duration;

async fn generate_mock_client() -> (MLBApi, ServerGuard) {
    let server = mockito::Server::new_async().await;
    let base_url = server.url();
    let formatted_url = if base_url.ends_with('/') {
        base_url
    } else {
        format!("{base_url}/")
    };

    let client = MLBApiBuilder::default()
        .base_url(&formatted_url)
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    (client, server)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Datelike, Local};

    /// Test the schedule for the All Star Game 2021
    #[tokio::test]
    async fn test_schedule_all_star_game() {
        let (client, mut server) = generate_mock_client().await;

        let m = server
            .mock("GET", "/v1/schedule?sportId=1&date=2021-07-13")
            .with_status(200)
            .with_header("content-type", "application/json;charset=UTF-8")
            .with_body_from_file("./tests/responses/all-star-game.json")
            .create();

        let date = NaiveDate::from_ymd_opt(2021, 7, 13).unwrap();
        let resp = client.get_schedule_date(date).await.unwrap();
        m.assert(); // assert mock was called
        assert_eq!(resp.total_games, 1);
    }

    #[tokio::test]
    async fn test_standings() {
        let (client, mut server) = generate_mock_client().await;
        let date: NaiveDate = NaiveDate::from_ymd_opt(2021, 6, 10).unwrap();

        let m = server
            .mock(
                "GET",
                "/v1/standings?sportId=1&season=2021&date=2021-06-10&leagueId=103,104",
            )
            .with_status(200)
            .with_header("content-type", "application/json;charset=UTF-8")
            .with_body_from_file("./tests/responses/standings.json")
            .create();

        let resp = client.get_standings(date).await.unwrap();
        m.assert(); // assert mock was called
        assert_ne!(resp.records.len(), 0);
    }

    #[tokio::test]
    async fn test_live() {
        let (client, mut server) = generate_mock_client().await;

        let game_id = 777687;
        let url = format!("/v1.1/game/{game_id}/feed/live?language=en");
        let m = server
            .mock("GET", Matcher::Exact(url))
            .with_status(200)
            .with_header("content-type", "application/json;charset=UTF-8")
            .with_body_from_file("./tests/responses/live.json")
            .create();

        let resp = client.get_live_data(game_id).await.unwrap();
        m.assert(); // assert mock was called
        assert_eq!(resp.game_pk, game_id);
    }

    #[tokio::test]
    async fn test_team_stats() {
        let (client, mut server) = generate_mock_client().await;

        let local: DateTime<Local> = Local::now();
        for group in [StatGroup::Hitting, StatGroup::Pitching] {
            let url = format!(
                "/v1/teams/stats?sportId=1&stats=season&season={}&group={}",
                local.year(),
                group
            );

            let m = server
                .mock("GET", Matcher::Exact(url))
                .with_status(200)
                .with_header("content-type", "application/json;charset=UTF-8")
                .with_body_from_file(format!("./tests/responses/team-stats-{group}.json"))
                .create();

            let resp = client.get_team_stats(group).await.unwrap();
            m.assert(); // assert mock was called
            assert_ne!(resp.stats.len(), 0);
            assert_eq!(resp.stats[0].group.display_name, group.to_string());
        }
    }

    #[tokio::test]
    async fn test_team_stats_on_date() {
        let (client, mut server) = generate_mock_client().await;

        let date: NaiveDate = NaiveDate::from_ymd_opt(2025, 5, 20).unwrap();
        for group in [StatGroup::Hitting, StatGroup::Pitching] {
            let url = format!(
                "/v1/teams/stats?sportId=1&stats=byDateRange&season={}&endDate={}&group={}",
                date.year(),
                date.format("%Y-%m-%d"),
                group
            );

            let m = server
                .mock("GET", Matcher::Exact(url))
                .with_status(200)
                .with_header("content-type", "application/json;charset=UTF-8")
                .with_body_from_file(format!("./tests/responses/team-stats-{group}-date.json"))
                .create();

            let resp = client.get_team_stats_on_date(group, date).await.unwrap();
            m.assert(); // assert mock was called
            assert_ne!(resp.stats.len(), 0);
            assert_eq!(resp.stats[0].group.display_name, group.to_string());
        }
    }

    #[tokio::test]
    async fn test_player_stats() {
        let (client, mut server) = generate_mock_client().await;

        let local: DateTime<Local> = Local::now();
        for group in [StatGroup::Hitting, StatGroup::Pitching] {
            let url = format!(
                "/v1/stats?sportId=1&stats=season&season={}&group={}&limit=300",
                local.year(),
                group
            );

            let m = server
                .mock("GET", Matcher::Exact(url))
                .with_status(200)
                .with_header("content-type", "application/json;charset=UTF-8")
                .with_body_from_file(format!("./tests/responses/player-stats-{group}.json"))
                .create();

            let resp = client.get_player_stats(group).await.unwrap();
            m.assert(); // assert mock was called
            assert_ne!(resp.stats.len(), 0);
            assert_eq!(resp.stats[0].group.display_name, group.to_string());
        }
    }

    #[tokio::test]
    async fn test_player_stats_on_date() {
        let (client, mut server) = generate_mock_client().await;
        let date: NaiveDate = NaiveDate::from_ymd_opt(2025, 5, 20).unwrap();

        for group in [StatGroup::Hitting, StatGroup::Pitching] {
            let url = format!(
                "/v1/stats?sportId=1&stats=byDateRange&season={}&endDate={}&group={}&limit=300",
                date.year(),
                date.format("%Y-%m-%d"),
                group
            );

            let m = server
                .mock("GET", Matcher::Exact(url))
                .with_status(200)
                .with_header("content-type", "application/json;charset=UTF-8")
                .with_body_from_file(format!("./tests/responses/player-stats-{group}-date.json"))
                .create();

            let resp = client.get_player_stats_on_date(group, date).await.unwrap();
            m.assert(); // assert mock was called
            assert_ne!(resp.stats.len(), 0);
            assert_eq!(resp.stats[0].group.display_name, group.to_string());
        }
    }
}
