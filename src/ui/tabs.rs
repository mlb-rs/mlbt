use crate::app::App;
use tui::{
    backend::Backend,
    layout::Alignment,
    style::{Color, Style},
    text::Spans,
    widgets::{Block, BorderType, Borders, Paragraph, Tabs},
    Frame,
};

pub fn render_top_bar<B>(f: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let border_style = Style::default();

    let left_block = Block::default()
        .borders(Borders::LEFT | Borders::BOTTOM | Borders::TOP)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let right_block = Block::default()
        .borders(Borders::RIGHT | Borders::BOTTOM | Borders::TOP)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let style = Style::default().fg(Color::White);

    let titles = app.tabs.iter().map(|t| Spans::from(*t)).collect();
    let tabs = Tabs::new(titles).block(left_block).style(style);
    f.render_widget(tabs, app.layout.top_bar[0]);

    let help = Paragraph::new("Help: ? ")
        .alignment(Alignment::Right)
        .block(right_block)
        .style(style);
    f.render_widget(help, app.layout.top_bar[1]);
}
