use tui::layout::{Constraint, Direction, Layout, Rect};

pub struct LayoutAreas {
    pub top_bar: Vec<Rect>,
    pub main: Rect,
}

const TOP_BAR_HEIGHT: u16 = 3; // length
const MAIN_HEIGHT: u16 = 100; // percent

impl LayoutAreas {
    pub fn new(size: Rect) -> Self {
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
            .split(size);

        LayoutAreas {
            top_bar: LayoutAreas::create_top_bar(chunks[0]),
            main: chunks[1],
        }
    }

    pub(crate) fn update(&mut self, size: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(TOP_BAR_HEIGHT),
                    Constraint::Percentage(MAIN_HEIGHT),
                ]
                .as_ref(),
            )
            .split(size);

        self.top_bar = LayoutAreas::create_top_bar(chunks[0]);
        self.main = chunks[1];
    }

    fn create_top_bar(area: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
            .split(area)
    }

    /// Create a split in the `main` section so that the top Rect is sized correctly to display a
    /// box score.
    pub fn for_boxscore(&self) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(7), Constraint::Percentage(100)].as_ref())
            .split(self.main)
    }

    /// Create two splits for displaying game info and the plays that have happened in the current
    /// inning. This is used in the `gameday` tab.
    pub fn for_info(rect: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(13),      // game info
                    Constraint::Percentage(100), // inning plays
                ]
                .as_ref(),
            )
            .split(rect)
    }
}
