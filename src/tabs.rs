use crate::app::App;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::Spans;
use tui::widgets::{Block, Borders, Paragraph, Tabs};
use tui::Frame;

pub fn render_top_bar<'a, B>(f: &mut Frame<B>, app: &'a App, layout_chunk: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
        .split(layout_chunk);

    let border_style = Style::default();

    let left_block = Block::default()
        .borders(Borders::LEFT | Borders::BOTTOM | Borders::TOP)
        .border_style(border_style);
    let right_block = Block::default()
        .borders(Borders::RIGHT | Borders::BOTTOM | Borders::TOP)
        .border_style(border_style);

    let style = Style::default().fg(Color::White);

    let titles = app.tabs.iter().map(|t| Spans::from(*t)).collect();
    let tabs = Tabs::new(titles).block(left_block).style(style);
    f.render_widget(tabs, chunks[0]);

    let help = Paragraph::new("Help: ?")
        .alignment(Alignment::Right)
        .block(right_block)
        .style(style);
    f.render_widget(help, chunks[1]);
}
