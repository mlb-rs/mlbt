use tui::layout::{Constraint, Direction, Layout, Rect};

pub struct LayoutAreas {
    pub top_bar: Vec<Rect>,
    pub main: Rect,
}

impl LayoutAreas {
    pub fn new(size: Rect) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(100)].as_ref())
            .split(size);

        LayoutAreas {
            top_bar: LayoutAreas::create_top_bar(chunks[0]),
            main: chunks[1],
        }
    }

    fn create_top_bar(area: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(50),
                    Constraint::Percentage(40),
                    Constraint::Percentage(10),
                ]
                .as_ref(),
            )
            .split(area)
    }
}
