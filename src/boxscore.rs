use mlb_api::live::Linescore;

/// TableInning is used to store the game state for a single team. It is meant
/// to be used to fill out the boxscore table.
#[derive(Default, Debug)]
pub struct BoxScore {
    pub home: BoxScoreLine,
    pub away: BoxScoreLine,
    pub header: Vec<String>,
}

impl BoxScore {
    pub fn new(linescore: &Linescore) -> Self {
        let (home, away) = BoxScore::generate_boxscore_info(&linescore);
        let played = linescore.current_inning.unwrap_or(0);
        let header = BoxScoreLine::create_header_vec(played);
        BoxScore { home, away, header }
    }

    fn generate_boxscore_info(linescore: &Linescore) -> (BoxScoreLine, BoxScoreLine) {
        // TODO save team name with inning
        let mut home = BoxScoreLine {
            home: true,
            ..Default::default()
        };
        let mut away = BoxScoreLine {
            home: false,
            ..Default::default()
        };
        for inning in &linescore.innings {
            // println!("{:?}", inn);
            let hr = inning.home.runs.unwrap_or(0);
            home.inning_score.push(hr);
            home.runs += hr;
            home.hits += inning.home.hits;
            home.errors += inning.home.errors;

            let ar = inning.away.runs.unwrap_or(0);
            away.inning_score.push(ar);
            away.runs += ar;
            away.hits += inning.away.hits;
            away.errors += inning.away.errors;
        }
        (home, away)
    }
}
/// TableInning is used to store the game state for a single team. It is meant
/// to be used to fill out the boxscore table.
#[derive(Default, Debug)]
pub struct BoxScoreLine {
    pub home: bool,
    pub runs: u8,
    pub hits: u8,
    pub errors: u8,
    pub inning_score: Vec<u8>,
}

impl BoxScoreLine {
    pub fn create_score_vec(&self) -> Vec<String> {
        // TODO replace with actual team name
        let team = match self.home {
            true => "Home".to_string(),
            false => "Away".to_string(),
        };

        let mut row = vec![team];
        let scores = self
            .inning_score
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        row.extend(scores);

        // fill out the rest of the innings if needed
        while row.len() <= 9 {
            row.push("-".to_string())
        }

        // add the runs, hits, and errors to the end
        row.push(self.runs.to_string());
        row.push(self.hits.to_string());
        row.push(self.errors.to_string());
        row
    }

    /// Generate the header row to be used in the box score.
    /// e.g. ["", "1", "2", "3", "4", "5", "6", "7", "8", "9", "R", "H", "E"]
    pub fn create_header_vec(played: u8) -> Vec<String> {
        let mut header_row: Vec<String> = (0..10).map(|i| i.to_string()).collect();
        header_row[0] = "".to_string();
        // Add any extra innings
        for i in 10..played + 1 {
            header_row.push(i.to_string());
        }
        header_row.push("R".to_string());
        header_row.push("H".to_string());
        header_row.push("E".to_string());
        header_row
    }
}

#[test]
fn test_create_header_row() {
    let good_nine: Vec<String> = vec![
        "", "1", "2", "3", "4", "5", "6", "7", "8", "9", "R", "H", "E",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    for i in 0..10 {
        let nine = BoxScoreLine::create_header_vec(i);
        assert_eq!(nine, good_nine);
    }

    // test extra innings
    let good_10: Vec<String> = vec![
        "", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "R", "H", "E",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let ten = BoxScoreLine::create_header_vec(10);
    assert_eq!(ten, good_10);

    let good_11: Vec<String> = vec![
        "", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "R", "H", "E",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let eleven = BoxScoreLine::create_header_vec(11);
    assert_eq!(eleven, good_11);
}
