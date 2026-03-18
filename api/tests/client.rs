use chrono::NaiveDate;
use mlbt_api::client::{MLBApi, MLBApiBuilder, StatGroup};
use mlbt_api::season::GameType;
use mlbt_api::teams::SportId;
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
            .mock(
                "GET",
                "/v1/schedule?sportId=1,51&hydrate=linescore,probablePitcher,stats&date=2021-07-13",
            )
            .with_status(200)
            .with_header("content-type", "application/json;charset=UTF-8")
            .with_body_from_file("./tests/responses/all-star-game.json")
            .create();

        let date = NaiveDate::from_ymd_opt(2021, 7, 13).unwrap();
        let resp = client.get_schedule_date(date).await.unwrap();
        m.assert(); // assert mock was called
        assert_eq!(resp.total_games, 1);
    }

    /// Test that a schedule with both MLB and WBC games can be parsed.
    #[tokio::test]
    async fn test_schedule_wbc() {
        let (client, mut server) = generate_mock_client().await;

        let m = server
            .mock(
                "GET",
                "/v1/schedule?sportId=1,51&hydrate=linescore,probablePitcher,stats&date=2026-03-14",
            )
            .with_status(200)
            .with_header("content-type", "application/json;charset=UTF-8")
            .with_body_from_file("./tests/responses/schedule-wbc.json")
            .create();

        let date = NaiveDate::from_ymd_opt(2026, 3, 14).unwrap();
        let resp = client.get_schedule_date(date).await.unwrap();
        m.assert();
        assert_eq!(resp.total_games, 19);
    }

    /// Test the schedule that includes probable pitcher data.
    #[tokio::test]
    async fn test_schedule_probable_pitcher() {
        let (client, mut server) = generate_mock_client().await;

        let m = server
            .mock(
                "GET",
                "/v1/schedule?sportId=1,51&hydrate=linescore,probablePitcher,stats&date=2026-03-18",
            )
            .with_status(200)
            .with_header("content-type", "application/json;charset=UTF-8")
            .with_body_from_file("./tests/responses/schedule-probable-pitchers.json")
            .create();

        let date = NaiveDate::from_ymd_opt(2026, 3, 18).unwrap();
        let resp = client.get_schedule_date(date).await.unwrap();
        m.assert(); // assert mock was called
        assert_eq!(resp.total_games, 13);

        // Verify the probable pitcher for the first game's away team (Houston Astros)
        let first_game = &resp.dates[0].games.as_ref().unwrap()[0];
        let away_pitcher = first_game
            .teams
            .away
            .probable_pitcher
            .as_ref()
            .expect("Expected a probable pitcher for the away team");

        assert_eq!(away_pitcher.full_name, "J.P. France");
        assert_eq!(away_pitcher.stats.len(), 4);
    }

    #[tokio::test]
    async fn test_standings() {
        let (client, mut server) = generate_mock_client().await;
        let date: NaiveDate = NaiveDate::from_ymd_opt(2021, 6, 10).unwrap();

        let m = server
            .mock(
                "GET",
                "/v1/standings?sportId=1&season=2021&date=2021-06-10&leagueId=103,104&hydrate=team",
            )
            .with_status(200)
            .with_header("content-type", "application/json;charset=UTF-8")
            .with_body_from_file("./tests/responses/standings.json")
            .create();

        let resp = client
            .get_standings(date, GameType::RegularSeason)
            .await
            .unwrap();
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

            let resp = client
                .get_team_stats(group, GameType::RegularSeason)
                .await
                .unwrap();
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

            let resp = client
                .get_team_stats_on_date(group, date, GameType::RegularSeason)
                .await
                .unwrap();
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
                "/v1/stats?sportId=1&stats=season&season={}&group={}&limit=3000&sortStat={}&order=desc",
                local.year(),
                group,
                group.default_sort_stat()
            );

            let m = server
                .mock("GET", Matcher::Exact(url))
                .with_status(200)
                .with_header("content-type", "application/json;charset=UTF-8")
                .with_body_from_file(format!("./tests/responses/player-stats-{group}.json"))
                .create();

            let resp = client
                .get_player_stats(group, GameType::RegularSeason)
                .await
                .unwrap();
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
                "/v1/stats?sportId=1&stats=byDateRange&season={}&endDate={}&group={}&limit=3000&sortStat={}&order=desc",
                date.year(),
                date.format("%Y-%m-%d"),
                group,
                group.default_sort_stat()
            );

            let m = server
                .mock("GET", Matcher::Exact(url))
                .with_status(200)
                .with_header("content-type", "application/json;charset=UTF-8")
                .with_body_from_file(format!("./tests/responses/player-stats-{group}-date.json"))
                .create();

            let resp = client
                .get_player_stats_on_date(group, date, GameType::RegularSeason)
                .await
                .unwrap();
            m.assert(); // assert mock was called
            assert_ne!(resp.stats.len(), 0);
            assert_eq!(resp.stats[0].group.display_name, group.to_string());
        }
    }

    #[tokio::test]
    async fn test_standings_spring_training() {
        let (client, mut server) = generate_mock_client().await;
        let date: NaiveDate = NaiveDate::from_ymd_opt(2026, 3, 10).unwrap();

        let m = server
            .mock(
                "GET",
                "/v1/standings?sportId=1&season=2026&standingsType=springTraining&leagueId=103,104&hydrate=team",
            )
            .with_status(200)
            .with_header("content-type", "application/json;charset=UTF-8")
            .with_body_from_file("./tests/responses/standings-spring-training.json")
            .create();

        let resp = client
            .get_standings(date, GameType::SpringTraining)
            .await
            .unwrap();
        m.assert();
        assert_ne!(resp.records.len(), 0);
    }

    #[tokio::test]
    async fn test_season_info() {
        let (client, mut server) = generate_mock_client().await;

        let m = server
            .mock("GET", "/v1/seasons/2026?sportId=1")
            .with_status(200)
            .with_header("content-type", "application/json;charset=UTF-8")
            .with_body_from_file("./tests/responses/season-info.json")
            .create();

        let info = client.get_season_info(2026).await.unwrap().unwrap();
        m.assert();
        assert_eq!(
            info.regular_season_start_date,
            NaiveDate::from_ymd_opt(2026, 3, 25).unwrap()
        );
    }

    #[tokio::test]
    async fn test_team_stats_spring_training_url() {
        let (client, mut server) = generate_mock_client().await;
        let date: NaiveDate = NaiveDate::from_ymd_opt(2026, 3, 10).unwrap();

        for group in [StatGroup::Hitting, StatGroup::Pitching] {
            let url = format!(
                "/v1/teams/stats?sportId=1&stats=byDateRange&season={}&endDate={}&group={}&gameType=S",
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

            let resp = client
                .get_team_stats_on_date(group, date, GameType::SpringTraining)
                .await
                .unwrap();
            m.assert();
            assert_ne!(resp.stats.len(), 0);
        }
    }

    #[tokio::test]
    async fn test_teams() {
        let (client, mut server) = generate_mock_client().await;

        let m = server
            .mock(
                "GET",
                "/v1/teams?sportIds=1,51&fields=teams,id,name,division,teamName,abbreviation,sport",
            )
            .with_status(200)
            .with_header("content-type", "application/json;charset=UTF-8")
            .with_body_from_file("./tests/responses/teams.json")
            .create();

        let resp = client
            .get_teams(&[SportId::Mlb, SportId::International])
            .await
            .unwrap();
        m.assert();
        assert!(!resp.teams.is_empty());
        // Verify a WBC team is present
        let puerto_rico = resp.teams.iter().find(|t| t.id == 897);
        assert!(puerto_rico.is_some());
        assert_eq!(puerto_rico.unwrap().abbreviation, "PUR");
        // Verify an MLB team is present
        let athletics = resp.teams.iter().find(|t| t.id == 133);
        assert!(athletics.is_some());
        assert_eq!(athletics.unwrap().abbreviation, "ATH");
    }

    #[tokio::test]
    async fn test_player_stats_spring_training_url() {
        let (client, mut server) = generate_mock_client().await;
        let date: NaiveDate = NaiveDate::from_ymd_opt(2026, 3, 10).unwrap();

        for group in [StatGroup::Hitting, StatGroup::Pitching] {
            let url = format!(
                "/v1/stats?sportId=1&stats=season&season={}&group={}&limit=3000&sortStat={}&order=desc&gameType=S&playerPool=ALL",
                date.year(),
                group,
                group.default_sort_stat()
            );

            let m = server
                .mock("GET", Matcher::Exact(url))
                .with_status(200)
                .with_header("content-type", "application/json;charset=UTF-8")
                .with_body_from_file(format!("./tests/responses/player-stats-{group}-date.json"))
                .create();

            let resp = client
                .get_player_stats_on_date(group, date, GameType::SpringTraining)
                .await
                .unwrap();
            m.assert();
            assert_ne!(resp.stats.len(), 0);
        }
    }
}
