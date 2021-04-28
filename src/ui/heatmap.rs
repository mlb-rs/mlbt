use super::super::heatmap::Heatmap;
use super::super::utils::centered_rect;
use mlb_api::live::LiveResponse;
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

pub fn render_heatmap<B>(f: &mut Frame<B>, rect: Rect, live_game: &LiveResponse)
where
    B: Backend,
{
    // these should be determined by the terminal size
    let width = 5;
    let height = 3;
    let heatmap = Heatmap::from_live_data(live_game);
    // TODO figure out why these don't work
    // let cells: Vec<Cell> = colors
    //     .iter()
    //     .map(|c| Cell::from("").style(Style::default().bg(*c)))
    //     .collect();
    // let rows: Vec<Row> = cells
    //     .chunks(3)
    //     .iter()
    //     .map(|row| Row::new(row).height(height))
    //     .collect();

    // let top = Row::new(cells[..3]).height(height);
    // let middle = Row::new(&cells[3..6]).height(height);
    // let bottom = Row::new(&cells[6..9]).height(height);
    let top = Row::new(vec![
        Cell::from("").style(Style::default().bg(heatmap.cells[0])),
        Cell::from("").style(Style::default().bg(heatmap.cells[1])),
        Cell::from("").style(Style::default().bg(heatmap.cells[2])),
    ])
    .height(height);
    let middle = Row::new(vec![
        Cell::from("").style(Style::default().bg(heatmap.cells[3])),
        Cell::from("").style(Style::default().bg(heatmap.cells[4])),
        Cell::from("").style(Style::default().bg(heatmap.cells[5])),
    ])
    .height(height);
    let bottom = Row::new(vec![
        Cell::from("").style(Style::default().bg(heatmap.cells[6])),
        Cell::from("").style(Style::default().bg(heatmap.cells[7])),
        Cell::from("").style(Style::default().bg(heatmap.cells[8])),
    ])
    .height(height);

    let widths = [
        Constraint::Length(width),
        Constraint::Length(width),
        Constraint::Length(width),
    ];
    // let t = Table::new(rows)
    let t = Table::new(vec![top, middle, bottom])
        .block(Block::default().borders(Borders::NONE).title("heatmap"))
        .widths(&widths)
        .column_spacing(0);

    // TODO the size of the centered rect needs to be dynamic
    let area = centered_rect(15, 15, rect);
    f.render_widget(t, area);
}
