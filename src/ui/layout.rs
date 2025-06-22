use crate::state::gameday::GamedayPanels;
use tui::layout::{Constraint, Layout, Rect, Size};

pub struct LayoutAreas {
    pub top_bar: [Rect; 2],
    pub main: Rect,
}

const TOP_BAR_HEIGHT: u16 = 3; // length
const MAIN_HEIGHT: u16 = 100; // percent

impl LayoutAreas {
    pub fn new(size: Size) -> Self {
        let rect = Rect::new(0, 0, size.width, size.height);
        let [top, main] = Layout::vertical([
            Constraint::Length(TOP_BAR_HEIGHT),
            Constraint::Percentage(MAIN_HEIGHT),
        ])
        .margin(1)
        .areas(rect);

        LayoutAreas {
            top_bar: LayoutAreas::create_top_bar(top),
            main,
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
        let [top, main] = Layout::vertical(constraints).areas(size);

        self.top_bar = LayoutAreas::create_top_bar(top);
        self.main = main;
    }

    pub fn create_top_bar(area: Rect) -> [Rect; 2] {
        Layout::horizontal([Constraint::Percentage(90), Constraint::Percentage(10)]).areas(area)
    }

    /// Create two splits for displaying the line score on top and a box score below.
    pub fn for_boxscore(rect: Rect) -> [Rect; 2] {
        Layout::vertical([
            Constraint::Length(4), // line score
            Constraint::Fill(1),   // box score
        ])
        .horizontal_margin(2)
        .vertical_margin(1)
        .areas(rect)
    }

    /// Create two splits for displaying the current matchup and the pitches for the at bat.
    pub fn for_at_bat(rect: Rect) -> [Rect; 2] {
        Layout::vertical([
            Constraint::Length(7), // matchup + on deck
            Constraint::Fill(1),   // pitches
        ])
        .areas(rect)
    }

    /// Create two splits for displaying game info for the current inning and the recent win
    /// probability. This is used in the `gameday` tab.
    pub fn for_info(rect: Rect, show_win_probability: bool) -> Vec<Rect> {
        let constraints = match show_win_probability {
            true => vec![Constraint::Fill(1), Constraint::Length(6)],
            false => vec![Constraint::Fill(1)],
        };
        Layout::vertical(constraints)
            .horizontal_margin(2)
            .vertical_margin(1)
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
        Layout::horizontal(constraints.as_slice())
            .split(area)
            .to_vec()
    }

    /// Create a centered rectangle of 4 height and 42% width.
    pub fn create_date_picker(area: Rect) -> Rect {
        let height = 4;
        let percent_width = 42;
        let [_, popup, _] = Layout::vertical(
            [
                Constraint::Ratio(1, 2),
                Constraint::Length(height),
                Constraint::Ratio(1, 2),
            ]
            .as_ref(),
        )
        .areas(area);

        Layout::horizontal(
            [
                Constraint::Percentage((100 - percent_width) / 2),
                Constraint::Percentage(percent_width),
                Constraint::Percentage((100 - percent_width) / 2),
            ]
            .as_ref(),
        )
        .split(popup)[1]
    }
}
