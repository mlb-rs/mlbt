use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table},
    Frame,
};

use mlb_api::live::Linescore;

// The longest game in MLB history was 26 innings. There are four extra columns:
// team name, runs, hits, and errors, so having a max width of 30 for the boxscore
// seems pretty safe.
const BOXSCORE_WIDTHS: &[Constraint] = &[Constraint::Length(4); 30];

pub fn render_boxscore<B>(f: &mut Frame<B>, rect: Rect, score: &Linescore)
where
    B: Backend,
{
    let (home, away) = generate_boxscore_info(&score);

    let played = score.current_inning.unwrap_or(0);
    let header_row = TableInning::create_header_vec(played);

    // slice off the correct number of widths TODO is there a better way to do this?
    let widths: &[Constraint] = &BOXSCORE_WIDTHS[0..header_row.len()];
    // let widths: &[Constraint] = vec![Constraint::Length(4); header_row.len()].as_slice();

    let header = Row::new(header_row).height(1).bottom_margin(1).style(
        Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::Black),
    );

    let t = Table::new(vec![
        Row::new(away.create_score_vec()).bottom_margin(1),
        Row::new(home.create_score_vec()),
    ])
    .widths(widths)
    .column_spacing(1)
    .style(Style::default().fg(Color::White))
    .header(header)
    .block(Block::default().borders(Borders::ALL).title("box score"));

    f.render_widget(t, rect);
}

pub fn generate_boxscore_info(linescore: &Linescore) -> (TableInning, TableInning) {
    let mut home = TableInning {
        home: true,
        ..Default::default()
    };
    let mut away = TableInning {
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
/// TableInning is used to store the game state for a single team. It is meant
/// to be used to fill out the boxscore table.
#[derive(Default, Debug)]
pub struct TableInning {
    pub home: bool,
    pub runs: u8,
    pub hits: u8,
    pub errors: u8,
    pub inning_score: Vec<u8>,
}

impl TableInning {
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
        let nine = TableInning::create_header_vec(i);
        assert_eq!(nine, good_nine);
    }

    // test extra innings
    let good_10: Vec<String> = vec![
        "", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "R", "H", "E",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let ten = TableInning::create_header_vec(10);
    assert_eq!(ten, good_10);

    let good_11: Vec<String> = vec![
        "", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "R", "H", "E",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let eleven = TableInning::create_header_vec(11);
    assert_eq!(eleven, good_11);
}
