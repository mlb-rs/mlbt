use crate::app::MenuItem;

pub const HEADER: &[&str; 2] = &["Description", "Key"];

/// Total row count of the docs table.
pub const DOCS_LEN: usize = GENERAL_DOCS.len()
    + SCOREBOARD_DOCS.len()
    + GAMEDAY_DOCS.len()
    + STATS_DOCS.len()
    + STANDINGS_DOCS.len()
    + TEAM_PAGE_DOCS.len()
    + PLAYER_PROFILE_DOCS.len();

const GENERAL_DOCS: &[&[&str; 2]; 9] = &[
    &["Exit help", "Esc"],
    &["Move down", "j/↓"],
    &["Move up", "k/↑"],
    &["Page down", "Shift + j/↓"],
    &["Page up", "Shift + k/↑"],
    &["Quit", "q"],
    &["Full screen", "f"],
    &["Focus settings", "Tab"],
    &["Change setting", "Enter"],
];
const SCOREBOARD_DOCS: &[&[&str; 2]; 9] = &[
    &["Scoreboard", "1"],
    &["Move down", "j/↓"],
    &["Move up", "k/↑"],
    &["View game in Gameday", "Enter"],
    &["Select date", ":"],
    &["Switch boxscore team", "h/a"],
    &["Scroll boxscore down", "Shift + j/↓"],
    &["Scroll boxscore up", "Shift + k/↑"],
    &["Toggle win probability", "w"],
];
const GAMEDAY_DOCS: &[&[&str; 2]; 12] = &[
    &["Gameday", "2"],
    &["Toggle game info", "i"],
    &["Toggle pitches", "p"],
    &["Toggle boxscore", "b"],
    &["Switch boxscore team", "h/a"],
    &["Scroll boxscore down", "Shift + j/↓"],
    &["Scroll boxscore up", "Shift + k/↑"],
    &["Toggle win probability", "w"],
    &["Move down at bat", "j/↓"],
    &["Move up at bat", "k/↑"],
    &["Go to live at bat", "l"],
    &["Go to first at bat", "s"],
];
const STATS_DOCS: &[&[&str; 2]; 16] = &[
    &["Stats", "3"],
    &["Switch hitting/pitching", "h/p"],
    &["Switch team/player", "t/l"],
    &["Switch pane", "←/→/Tab"],
    &["Move down", "j/↓"],
    &["Move up", "k/↑"],
    &["Page down", "Shift + j/↓"],
    &["Page up", "Shift + k/↑"],
    &["View player/team", "Enter"],
    &["Select date", ":"],
    &["Search", " "],
    &[" Fuzzy search", "Ctrl + f"],
    &["Options", " "],
    &[" Toggle stat", "Enter"],
    &[" Sort by stat", "s"],
    &[" Toggle options pane", "o"],
];
const STANDINGS_DOCS: &[&[&str; 2]; 6] = &[
    &["Standings", "4"],
    &["Move down", "j/↓"],
    &["Move up", "k/↑"],
    &["View team", "Enter"],
    &["Select date", ":"],
    &["Toggle division/league", "l"],
];
const TEAM_PAGE_DOCS: &[&[&str; 2]; 10] = &[
    &["Team Page", " "],
    &[" Switch section", "←/→/Tab"],
    &[" Move down", "j/↓"],
    &[" Move up", "k/↑"],
    &[" Page down", "Shift + j/↓"],
    &[" Page up", "Shift + k/↑"],
    &[" Toggle calendar", "c"],
    &[" Toggle roster type", "r"],
    &[" View player", "Enter"],
    &[" Close team page", "Esc"],
];
const PLAYER_PROFILE_DOCS: &[&[&str; 2]; 7] = &[
    &["Player Profile", " "],
    &[" Toggle category", "s"],
    &[" Scroll down", "j/↓"],
    &[" Scroll up", "k/↑"],
    &[" Page down", "Shift + j/↓"],
    &[" Page up", "Shift + k/↑"],
    &[" Close profile", "Esc"],
];

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum RowType {
    Header,
    SubHeader,
    Row,
}

/// A docs row classified for styling. `text` is already formatted into a single column string so
/// the UI layer can drop it straight into a `Row`.
pub struct HelpRow {
    pub row_type: RowType,
    pub text: Vec<String>,
}

/// Classify and format a raw `[description, key]` entry into a `HelpRow`. Entries whose key parses
/// as a digit are tab headers, entries with a blank key are subsection headers, and everything else
/// is a normal row.
pub fn format_row(r: &[&str; 2]) -> HelpRow {
    let row_type = if r[1].parse::<u8>().is_ok() {
        RowType::Header
    } else if r[1] == " " {
        RowType::SubHeader
    } else {
        RowType::Row
    };
    HelpRow {
        row_type,
        text: vec![format!("{:30}{:15}", r[0], r[1])],
    }
}

/// Build the docs so that the order is: general, active tab, other tabs. Team Page and Player
/// Profile docs are inserted once: after Stats when Stats is active, otherwise after Standings.
pub fn build_docs(active_tab: MenuItem) -> Vec<&'static [&'static str; 2]> {
    let mut docs = GENERAL_DOCS.to_vec();

    match active_tab {
        MenuItem::Gameday => {
            docs.extend_from_slice(GAMEDAY_DOCS);
            docs.extend_from_slice(SCOREBOARD_DOCS);
            docs.extend_from_slice(STATS_DOCS);
            docs.extend_from_slice(STANDINGS_DOCS);
            docs.extend_from_slice(TEAM_PAGE_DOCS);
            docs.extend_from_slice(PLAYER_PROFILE_DOCS);
        }
        MenuItem::Stats => {
            docs.extend_from_slice(STATS_DOCS);
            docs.extend_from_slice(TEAM_PAGE_DOCS);
            docs.extend_from_slice(PLAYER_PROFILE_DOCS);
            docs.extend_from_slice(SCOREBOARD_DOCS);
            docs.extend_from_slice(GAMEDAY_DOCS);
            docs.extend_from_slice(STANDINGS_DOCS);
        }
        MenuItem::Standings => {
            docs.extend_from_slice(STANDINGS_DOCS);
            docs.extend_from_slice(TEAM_PAGE_DOCS);
            docs.extend_from_slice(PLAYER_PROFILE_DOCS);
            docs.extend_from_slice(SCOREBOARD_DOCS);
            docs.extend_from_slice(GAMEDAY_DOCS);
            docs.extend_from_slice(STATS_DOCS);
        }
        // everything else uses the default order
        _ => {
            docs.extend_from_slice(SCOREBOARD_DOCS);
            docs.extend_from_slice(GAMEDAY_DOCS);
            docs.extend_from_slice(STATS_DOCS);
            docs.extend_from_slice(STANDINGS_DOCS);
            docs.extend_from_slice(TEAM_PAGE_DOCS);
            docs.extend_from_slice(PLAYER_PROFILE_DOCS);
        }
    }

    docs
}
