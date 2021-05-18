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
}
