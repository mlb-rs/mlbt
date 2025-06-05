use mlb_api::live::LiveResponse;

use crate::state::app_state::HomeOrAway;
use tui::style::{Color, Modifier, Style};
use tui::text::Span;
use tui::widgets::Cell;

/// LineScore is used for the two line summary of a game. It shows each teams runs per inning and
/// their total runs, hits, and errors.
#[derive(Debug)]
pub struct LineScore {
    pub header: Vec<String>,
    pub away: LineScoreLine,
    pub home: LineScoreLine,
    pub mini: bool,
}

/// LineScoreLine stores the high level game information for a single team.
#[derive(Default, Debug)]
pub struct LineScoreLine {
    pub team: HomeOrAway,
    #[allow(dead_code)]
    pub name: String,
    pub abbreviation: String,
    pub runs: u8,
    pub hits: u8,
    pub errors: u8,
    pub inning_score: Vec<u8>,
}

impl Default for LineScore {
    fn default() -> Self {
        LineScore {
            header: LineScoreLine::create_header_vec(0),
            away: LineScoreLine {
                team: HomeOrAway::Away,
                name: "away".to_string(),
                abbreviation: "A".to_string(),
                ..Default::default()
            },
            home: LineScoreLine {
                team: HomeOrAway::Home,
                name: "home".to_string(),
                abbreviation: "H".to_string(),
                ..Default::default()
            },
            mini: true,
        }
    }
}

impl LineScore {
    pub fn from_live_data(live_game: &LiveResponse) -> Self {
        let home = LineScoreLine::from_live_data(live_game, true);
        let away = LineScoreLine::from_live_data(live_game, false);
        let played = live_game.live_data.linescore.current_inning.unwrap_or(0);
        let header = LineScoreLine::create_header_vec(played);
        LineScore {
            header,
            away,
            home,
            mini: true,
        }
    }
}

impl LineScoreLine {
    pub fn from_live_data(live_game: &LiveResponse, home: bool) -> Self {
        let (name, team) = match home {
            true => (&live_game.game_data.teams.home, HomeOrAway::Home),
            false => (&live_game.game_data.teams.away, HomeOrAway::Away),
        };
        let mut line = LineScoreLine {
            team,
            name: name.team_name.to_string(),
            abbreviation: name.abbreviation.to_string(),
            ..Default::default()
        };
        for inning in &live_game.live_data.linescore.innings {
            let inning = match home {
                true => &inning.home,
                false => &inning.away,
            };
            let hr = inning.runs.unwrap_or(0);
            line.inning_score.push(hr);
            line.runs += hr;
            line.hits += inning.hits;
            line.errors += inning.errors;
        }
        line
    }

    pub fn create_score_vec(&self, active: HomeOrAway) -> Vec<Cell> {
        let mut row = vec![];
        // Display a blue background if the team is active
        let team = match active == self.team {
            true => Span::styled(
                self.abbreviation.clone(),
                Style::default().fg(Color::Black).bg(Color::Blue),
            ),
            false => Span::raw(self.abbreviation.clone()),
        };
        row.push(Cell::from(team));

        let scores = self
            .inning_score
            .iter()
            .map(|s| Cell::from(s.to_string()))
            .collect::<Vec<_>>();
        row.extend(scores);

        // fill out the rest of the innings if needed
        while row.len() <= 9 {
            row.push(Cell::from("-"))
        }

        // add the runs, hits, and errors to the end
        row.push(Cell::from(Span::styled(
            self.runs.to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        )));
        row.push(Cell::from(self.hits.to_string()));
        row.push(Cell::from(self.errors.to_string()));
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
        let nine = LineScoreLine::create_header_vec(i);
        assert_eq!(nine, good_nine);
    }

    // test extra innings
    let good_10: Vec<String> = vec![
        "", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "R", "H", "E",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let ten = LineScoreLine::create_header_vec(10);
    assert_eq!(ten, good_10);

    let good_11: Vec<String> = vec![
        "", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "R", "H", "E",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let eleven = LineScoreLine::create_header_vec(11);
    assert_eq!(eleven, good_11);
}
