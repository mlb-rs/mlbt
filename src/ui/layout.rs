use crate::state::gameday::GamedayPanels;
use tui::layout::{Constraint, Direction, Layout, Rect, Size};

pub struct LayoutAreas {
    pub top_bar: Vec<Rect>,
    pub main: Rect,
}

const TOP_BAR_HEIGHT: u16 = 3; // length
const MAIN_HEIGHT: u16 = 100; // percent

impl LayoutAreas {
    pub fn new(size: Size) -> Self {
        let rect = Rect::new(0, 0, size.width, size.height);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(TOP_BAR_HEIGHT),
                    Constraint::Percentage(MAIN_HEIGHT),
                ]
                .as_ref(),
            )
            .split(rect);

        LayoutAreas {
            top_bar: LayoutAreas::create_top_bar(chunks[0]),
            main: chunks[1],
        }
    }

    pub(crate) fn update(&mut self, size: Rect, full_screen: bool) {
        let constraints = match full_screen {
            true => vec![Constraint::Percentage(0), Constraint::Percentage(100)],
            false => vec![
                Constraint::Length(TOP_BAR_HEIGHT),
                Constraint::Percentage(MAIN_HEIGHT),
            ],
        };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints::<&[Constraint]>(constraints.as_ref())
            .split(size);

        self.top_bar = LayoutAreas::create_top_bar(chunks[0]);
        self.main = chunks[1];
    }

    pub fn create_top_bar(area: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
            .split(area)
            .to_vec()
    }

    /// Create two splits for displaying the line score on top and a box score below.
    pub fn for_boxscore(rect: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(2)
            .vertical_margin(1)
            .constraints(
                [
                    Constraint::Length(4),       // line score
                    Constraint::Percentage(100), // box score
                ]
                .as_ref(),
            )
            .split(rect)
            .to_vec()
    }

    /// Create two splits for displaying game info and the plays that have happened in the current
    /// inning. This is used in the `gameday` tab.
    pub fn for_info(rect: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(2)
            .vertical_margin(1)
            .constraints(
                [
                    Constraint::Length(13),      // game info
                    Constraint::Percentage(100), // inning plays
                ]
                .as_ref(),
            )
            .split(rect)
            .to_vec()
    }

    /// Create the Gameday layouts based on how many of the panels are active.
    pub fn generate_gameday_panels(active: &GamedayPanels, area: Rect) -> Vec<Rect> {
        let constraints = match active.count() {
            0 | 1 => vec![Constraint::Percentage(100)],
            2 => vec![Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)],
            3 => vec![
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ],
            _ => vec![],
        };
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints.as_slice())
            .split(area)
            .to_vec()
    }

    /// Create a centered rectangle of 4 height and 42% width.
    pub fn create_date_picker(area: Rect) -> Rect {
        let height = 4;
        let percent_width = 42;
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Ratio(1, 2),
                    Constraint::Length(height),
                    Constraint::Ratio(1, 2),
                ]
                .as_ref(),
            )
            .split(area);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_width) / 2),
                    Constraint::Percentage(percent_width),
                    Constraint::Percentage((100 - percent_width) / 2),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1]
    }
}
