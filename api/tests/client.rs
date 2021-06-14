use mlb_api::client::MLBApiBuilder;
use mockito::mock;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standings() {
        let client = MLBApiBuilder::default().build().unwrap();

        let _m = mock("GET", "v1/standings?leagueId=103,104")
            .with_status(200)
            .with_header("content-type", "application/json;charset=UTF-8")
            .with_body_from_file("./tests/responses/standings.json")
            .create();

        let resp = client.get_standings();
        println!("{:?}", resp);
    }
}
