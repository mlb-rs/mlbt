use chrono::NaiveDate;
use mlb_api::client::MLBApiBuilder;
use mlb_api::client::StatGroup;
use mockito::mock;
use mockito::Matcher;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test the schedule for the All Star Game 2021
    #[test]
    fn test_schedule_all_star_game() {
        let client = MLBApiBuilder::default().build().unwrap();

        let _m = mock("GET", "v1/schedule?sportId=1&date=2021-07-13")
            .with_status(200)
            .with_header("content-type", "application/json;charset=UTF-8")
            .with_body_from_file("./tests/responses/all-star-game.json")
            .create();

        let date = NaiveDate::from_ymd(2021, 7, 13);
        let resp = client.get_schedule_date(date);
        println!("{:?}", resp);
    }

    #[test]
    fn test_standings() {
        let client = MLBApiBuilder::default().build().unwrap();

        let _m = mock(
            "GET",
            "v1/standings?sportId=1?season=2021?date=2021-06-10?leagueId=103,104",
        )
        .with_status(200)
        .with_header("content-type", "application/json;charset=UTF-8")
        .with_body_from_file("./tests/responses/standings.json")
        .create();

        let resp = client.get_standings();
        println!("{:?}", resp);
    }

    #[test]
    fn test_live() {
        let client = MLBApiBuilder::default().build().unwrap();
        let _m = mock("GET", "v1.1/game/632386/feed/live?language=en")
            .with_status(200)
            .with_header("content-type", "application/json;charset=UTF-8")
            .with_body_from_file("./tests/responses/live.json")
            .create();

        let resp = client.get_live_data(632386);
        println!("{:?}", resp);
    }

    #[test]
    fn test_team_stats() {
        let client = MLBApiBuilder::default().build().unwrap();
        for group in vec![StatGroup::Hitting, StatGroup::Pitching] {
            let url = format!(
                "v1/teams/stats?sportId=1&stats=season&season=2021&group={}",
                group
            );

            let _m = mock("GET", Matcher::Exact(url))
                .with_status(200)
                .with_header("content-type", "application/json;charset=UTF-8")
                .with_body_from_file("./tests/responses/team-stats.json")
                .create();

            let resp = client.get_team_stats(group);
            println!("{:?}", resp);
        }
    }

    #[test]
    fn test_player_stats() {
        let client = MLBApiBuilder::default().build().unwrap();
        for group in vec![StatGroup::Hitting, StatGroup::Pitching] {
            let url = format!("v1/stats?stats=season&season=2021&group={}", group);

            let _m = mock("GET", Matcher::Exact(url))
                .with_status(200)
                .with_header("content-type", "application/json;charset=UTF-8")
                .with_body_from_file("./tests/responses/player-stats.json")
                .create();

            let resp = client.get_player_stats(group);
            println!("{:?}", resp);
        }
    }
}
