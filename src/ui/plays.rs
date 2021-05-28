use crate::plays::InningPlays;
use tui::{
    backend::Backend,
    layout::{Constraint, Corner, Direction, Layout, Rect},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

impl InningPlays {
    pub fn as_list(&self) -> Vec<ListItem> {
        self.play_results
            .iter()
            .map(|play| match play.description.is_empty() {
                false => ListItem::new(vec![
                    Spans::from(Span::raw(&play.description)),
                    Spans::from(Span::raw(format!(
                        "  outs: {} balls: {} strikes: {}",
                        &play.count.outs, &play.count.balls, &play.count.strikes
                    ))),
                ]),
                true => ListItem::new(vec![]),
            })
            .rev()
            .collect()
    }
}

impl InningPlays {
    pub fn render<B>(&self, f: &mut Frame<B>, rect: Rect)
    where
        B: Backend,
    {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(3)
            .constraints(
                [
                    Constraint::Percentage(30), // game info
                    Constraint::Percentage(70), // inning plays
                ]
                .as_ref(),
            )
            .split(rect);

        let events_list = List::new(self.as_list())
            .block(Block::default().borders(Borders::NONE))
            .start_corner(Corner::TopLeft);
        f.render_widget(events_list, chunks[1]);
    }
}
