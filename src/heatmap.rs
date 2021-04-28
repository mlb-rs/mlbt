use crate::utils::centered_rect;
use mlb_api::live::LiveResponse;

use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Color, Style},
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
    let colors = temp_zones(live_game);
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
        Cell::from("").style(Style::default().bg(colors[0])),
        Cell::from("").style(Style::default().bg(colors[1])),
        Cell::from("").style(Style::default().bg(colors[2])),
    ])
    .height(height);
    let middle = Row::new(vec![
        Cell::from("").style(Style::default().bg(colors[3])),
        Cell::from("").style(Style::default().bg(colors[4])),
        Cell::from("").style(Style::default().bg(colors[5])),
    ])
    .height(height);
    let bottom = Row::new(vec![
        Cell::from("").style(Style::default().bg(colors[6])),
        Cell::from("").style(Style::default().bg(colors[7])),
        Cell::from("").style(Style::default().bg(colors[8])),
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

// starting to figure out how the heatmaps are represented in the API
// it is super nested!
pub fn temp_zones(live_game: &LiveResponse) -> Vec<Color> {
    let current_play = live_game
        .live_data
        .plays
        .current_play
        .as_ref()
        .expect("no current play!!");
    let zones = &current_play
        .matchup
        .batter_hot_cold_zone_stats
        .as_ref()
        .expect("no zones!!!!")
        .stats;
    let mut colors: Vec<Color> = Vec::with_capacity(9);
    // should only be one zone, but it's a vector
    for z in zones {
        // splits has 3 elements:
        // 0 - exit velocity
        // 1 - batting average
        // 2 - on base plus slugging
        // it's unclear if these are always ordered this way
        for split in &z.splits {
            if split.stat.name == "battingAverage" {
                assert_eq!(split.stat.zones.len(), 13); // not sure why there are 13, I only want 9
                for zone in &split.stat.zones {
                    let c = convert_color(zone.color.clone());
                    colors.push(c);
                    // print!("{:?} ", c);
                }
                // println!();
            }
        }
        assert_eq!(z.splits.len(), 3);
    }
    assert_eq!(zones.len(), 1);
    colors
}

// "rgba(255, 255, 255, 0.55)"
fn convert_color(s: String) -> Color {
    if let Some(s) = s.strip_prefix("rgba(") {
        let c: Vec<&str> = s.split(", ").collect();
        Color::Rgb(
            c[0].parse().unwrap_or(0),
            c[1].parse().unwrap_or(0),
            c[2].parse().unwrap_or(0),
        )
    } else {
        println!("color doesn't start with 'rgba(' {:?}", s);
        Color::Rgb(0, 0, 0)
    }
}

#[test]
fn test_color_conversion() {
    let tests = vec![
        ("rgba(0, 0, 0, .55)", Color::Rgb(0, 0, 0)),
        ("rgba(6, 90, 238, .55)", Color::Rgb(6, 90, 238)),
        ("rgba(150, 188, 255, .55)", Color::Rgb(150, 188, 255)),
        ("rgba(214, 41, 52, .55)", Color::Rgb(214, 41, 52)),
        ("rgba(255, 255, 255, 0.55)", Color::Rgb(255, 255, 255)),
    ];
    for t in tests {
        assert_eq!(convert_color(t.0.to_string()), t.1);
    }

    let bad = ("rgba(55, 255, 255, 0.55)", Color::Rgb(255, 255, 255));
    assert_ne!(convert_color(bad.0.to_string()), bad.1);

    let nonsense = ("rgba(-5, 255, 255, 0.55)", Color::Rgb(0, 255, 255));
    assert_eq!(convert_color(nonsense.0.to_string()), nonsense.1);
}
