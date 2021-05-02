use super::super::heatmap::Heatmap;
use super::super::utils::centered_rect;
use mlb_api::live::LiveResponse;
use tui::layout::Alignment;
use tui::style::Color;
use tui::widgets::Paragraph;
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

pub fn render_debug<B>(f: &mut Frame<B>, rect: Rect)
where
    B: Backend,
{
    let border_style = Style::default();

    let center_block = Block::default()
        .borders(Borders::LEFT | Borders::BOTTOM | Borders::TOP)
        .border_style(border_style);

    let style = Style::default().fg(Color::White);

    let help = Paragraph::new("debug")
        .alignment(Alignment::Center)
        .block(center_block)
        .style(style);
    f.render_widget(help, rect);
}
