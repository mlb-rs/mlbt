use crate::app::MenuItem;
use crate::components::game::live_game::GameStateV2;
use crate::components::game::matchup::Summary;
use crate::components::game::win_probability::WinProbabilityAtBat;
use crate::ui::gameday::plays::{BLUE, GREEN, RED};
use indexmap::IndexMap;
use tui::prelude::*;
use tui::widgets::{
    Axis, Bar, BarChart, BarGroup, Block, Borders, Cell, Chart, Dataset, GraphType, Row, Table,
};

type ChartPoint = (f64, f64);

pub struct WinProbabilityWidget<'a> {
    pub game: &'a GameStateV2,
    pub selected_at_bat: Option<u8>,
    pub active_tab: MenuItem,
}

struct WinProbabilityData<'a> {
    summary: &'a Summary,
    at_bats: &'a IndexMap<u8, WinProbabilityAtBat>,
    selected_at_bat_index: Option<u8>,
    table_height: u16,
}

impl WinProbabilityWidget<'_> {
    pub fn get_min_table_height() -> usize {
        WinProbabilityData::MINIMUM_TABLE_HEIGHT
    }
}

impl Widget for WinProbabilityWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.active_tab {
            MenuItem::Scoreboard => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Fill(1),
                        Constraint::Length(WinProbabilityData::MINIMUM_TABLE_HEIGHT as u16),
                    ])
                    .horizontal_margin(2)
                    .vertical_margin(1)
                    .split(area);
                let data =
                    WinProbabilityData::new(self.game, self.selected_at_bat, chunks[1].height);
                data.render_line_chart(chunks[1], buf);
            }
            MenuItem::Gameday => {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(30), Constraint::Fill(1)].as_ref())
                    .horizontal_margin(2)
                    .vertical_margin(1)
                    .split(area);
                let data =
                    WinProbabilityData::new(self.game, self.selected_at_bat, chunks[0].height);
                data.render_table(chunks[0], buf);
                data.render_chart(chunks[1], buf);
            }
            _ => (),
        }
    }
}

impl<'a> WinProbabilityData<'a> {
    const INTERPOLATION_TARGET_COUNT: usize = 50;
    const MINIMUM_TABLE_HEIGHT: usize = 10;

    fn new(game: &'a GameStateV2, selected_at_bat_index: Option<u8>, table_height: u16) -> Self {
        Self {
            at_bats: &game.win_probability.at_bats,
            summary: &game.summary,
            selected_at_bat_index,
            table_height,
        }
    }

    fn create_table_row(&self, at_bat: &WinProbabilityAtBat) -> Row {
        let label = match at_bat.is_top_inning {
            true => format!("top {}", at_bat.inning),
            false => format!("bot {}", at_bat.inning),
        };

        let home_wp = at_bat.home_team_wp.clamp(0.0, 100.0);
        let wp = if home_wp == 100.0 || home_wp == 0.0 {
            format!("{:.0}%", home_wp) // just show 100% or 0%
        } else {
            format!("{:.1}%", home_wp)
        };

        let wp_color = match home_wp {
            99.0..=100.0 => BLUE,
            45.0..=55.0 => GREEN,
            0.0..=0.99 => Color::Red,
            _ => Color::White,
        };

        let leverage = at_bat.leverage_index;
        let li = if leverage == 0.0 {
            "0".to_string() // don't show "0.00", just "0"
        } else {
            format!("{:.2}", leverage)
        };
        let leverage_color = if leverage > 2.0 { RED } else { Color::White };

        // -10.0 is the longest wpa possible because the smallest wpa possible is -99.9.
        // so align everything with 4 characters and ignore the minus sign
        let wpa = if at_bat.home_team_wp_added <= -10.0 {
            format!("{:4.1}", at_bat.home_team_wp_added)
        } else {
            // add space to align with `-` sign
            format!(" {:4.1}", at_bat.home_team_wp_added)
        };

        Row::new([
            Cell::from(format!("{:<8}", label)),
            Cell::from(format!(" {:<4}", li)).style(Style::default().fg(leverage_color)),
            Cell::from(wpa),
            Cell::from(format!("{:<6}", wp)).style(Style::default().fg(wp_color)),
        ])
    }

    fn render_table(&self, area: Rect, buf: &mut Buffer) {
        let header = Row::new([
            Cell::from(format!("{:<8}", "inning")),
            Cell::from(format!(" {:<4}", "li")),
            Cell::from(format!("{:^5}", "wpa")),
            Cell::from(format!("{:<6}", "win")),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

        let (start_idx, end_idx, selected_row_index) = self.calculate_visible_range();

        let visible_rows: Vec<Row> = self
            .at_bats
            .values()
            .rev() // newest first
            .skip(start_idx)
            .take(end_idx - start_idx)
            .map(|at_bat| self.create_table_row(at_bat))
            .collect();

        let mut table_state = tui::widgets::TableState::default();
        table_state.select(selected_row_index);

        let table = Table::new(
            visible_rows,
            [
                Constraint::Min(8), // inning
                Constraint::Min(5), // leverage index
                Constraint::Min(7), // wpa
                Constraint::Min(6), // win %
            ],
        )
        .style(Style::default().fg(Color::White))
        .row_highlight_style(Style::default().bg(BLUE).add_modifier(Modifier::BOLD))
        .header(header);

        StatefulWidget::render(table, area, buf, &mut table_state);
    }

    fn create_chart_bar(&self, at_bat: &WinProbabilityAtBat) -> Bar {
        let home_wp = (at_bat.home_team_wp.round() as u8).clamp(0, 100);
        Bar::default()
            .value(home_wp.into())
            .text_value("".to_string())
            .style(Style::default().fg(BLUE))
    }

    fn create_header_bar(&self, width: u16) -> Bar {
        Bar::default()
            .value(100)
            // use the width of the the area to ensure underline goes all the way across
            .text_value(format!("{: <1$}", "", width as usize))
            .value_style(
                Style::default()
                    .fg(Color::Gray)
                    .underlined()
                    .underline_color(Color::White),
            )
            .style(Style::default().fg(Color::Black))
    }

    fn render_chart(&self, area: Rect, buf: &mut Buffer) {
        let (start_idx, end_idx, _) = self.calculate_visible_range();

        let bars: Vec<Bar> = self
            .at_bats
            .values()
            .rev() // newest first
            .skip(start_idx)
            .take(end_idx - start_idx)
            .map(|at_bat| self.create_chart_bar(at_bat))
            .collect();

        let mut all_bars = Vec::with_capacity(1 + bars.len());
        // add header bar at the top
        all_bars.push(self.create_header_bar(area.width));
        all_bars.extend(bars);

        let chart = BarChart::default()
            .data(BarGroup::default().bars(&all_bars))
            .direction(Direction::Horizontal)
            .bar_width(1)
            .bar_gap(0)
            .value_style(Style::default().fg(BLUE).add_modifier(Modifier::BOLD))
            .max(100);

        Widget::render(chart, area, buf);
    }

    fn get_selected_position(&self) -> Option<usize> {
        let selected_ab = self.selected_at_bat_index?;
        // since IndexMap maintains order and at_bat_index is the key, we can use get_index_of
        self.at_bats.get_index_of(&selected_ab)
    }

    /// Calculate the number of rows the should be visible for both the bar chart and the table.
    fn calculate_visible_range(&self) -> (usize, usize, Option<usize>) {
        let total_rows = self.at_bats.len();
        let visible_count = self.table_height.saturating_sub(1) as usize; // -1 for header

        if let Some(selected_pos) = self.get_selected_position() {
            // reverse the position to display newest first
            let reversed_pos = total_rows.saturating_sub(1).saturating_sub(selected_pos);

            let scroll_offset = if reversed_pos == total_rows.saturating_sub(1) {
                // if selecting the last item (newest), position it at the bottom
                reversed_pos.saturating_sub(visible_count.saturating_sub(1))
            } else if reversed_pos >= visible_count {
                // otherwise, center it in the view
                reversed_pos.saturating_sub(visible_count / 2)
            } else {
                0
            };

            // ensure we don't scroll past the end
            let max_scroll = total_rows.saturating_sub(visible_count);
            let scroll_offset = scroll_offset.min(max_scroll);
            let end_idx = (scroll_offset + visible_count).min(total_rows);
            let relative_selection = Some(reversed_pos.saturating_sub(scroll_offset));

            (scroll_offset, end_idx, relative_selection)
        } else {
            // no selection, show from start (newest)
            let end_idx = visible_count.min(total_rows);
            (0, end_idx, None)
        }
    }

    fn render_line_chart(&self, area: Rect, buf: &mut Buffer) {
        let (points, x_axis_bounds) = self.prepare_chart_data();

        // split points into home and away teams so they can be different colors
        let (away_points, home_points): (Vec<ChartPoint>, Vec<ChartPoint>) =
            points.iter().copied().partition(|p| p.1 > 0.0);

        let inning_lines = self.generate_inning_lines();
        let datasets = self.create_datasets(&home_points, &away_points, &inning_lines);
        let chart = self.create_chart(datasets, x_axis_bounds);

        Widget::render(chart, area, buf);
    }

    fn create_chart<'c>(&self, datasets: Vec<Dataset<'c>>, x_axis_bounds: f64) -> Chart<'c> {
        Chart::new(datasets)
            .block(
                Block::default()
                    .title(Line::from(" Game Win Probability ").centered())
                    .borders(Borders::TOP),
            )
            .x_axis(
                Axis::default()
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, x_axis_bounds]),
            )
            .y_axis(
                Axis::default()
                    .style(Style::default().fg(Color::Gray))
                    .labels([
                        self.summary.home_team.abbreviation.to_string(),
                        "50%".into(),
                        self.summary.away_team.abbreviation.to_string(),
                    ])
                    .bounds([-50.0, 50.0]),
            )
    }

    fn prepare_chart_data(&self) -> (Vec<ChartPoint>, f64) {
        let mut points: Vec<ChartPoint> = self
            .at_bats
            .values()
            .map(|at_bat| {
                (
                    at_bat.at_bat_index as f64,
                    // - 50% should be at 0.0
                    // - 100% for the home team should be at -50.0
                    // - 100% for the away team should be at 50.0
                    50.0 - at_bat.home_team_wp as f64,
                )
            })
            .collect();

        // The bounds should be based on the number of at-bats, not interpolated points.
        // Add 1.0 to the max index for better padding on the chart.
        let x_axis_bounds = self
            .at_bats
            .values()
            .last()
            .map_or(0.0, |ab| ab.at_bat_index as f64 + 1.0);

        if points.len() > 1 && points.len() < Self::INTERPOLATION_TARGET_COUNT {
            points = Self::interpolate_points(points, Self::INTERPOLATION_TARGET_COUNT);
        }

        (points, x_axis_bounds)
    }

    fn create_datasets<'d>(
        &self,
        home_points: &'d [ChartPoint],
        away_points: &'d [ChartPoint],
        inning_lines: &'d [[ChartPoint; 2]],
    ) -> Vec<Dataset<'d>> {
        let mut datasets = Vec::with_capacity(inning_lines.len() + 2);

        // add lines indicating inning changes
        for line in inning_lines {
            datasets.push(
                Dataset::default()
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(Color::Gray))
                    .graph_type(GraphType::Line)
                    .data(line),
            );
        }

        // add the team data
        datasets.push(Self::create_team_dataset(home_points, Color::Blue));
        datasets.push(Self::create_team_dataset(away_points, Color::Green));

        datasets
    }

    fn create_team_dataset(points: &[ChartPoint], color: Color) -> Dataset {
        Dataset::default()
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(color))
            .graph_type(GraphType::Bar)
            .data(points)
    }

    fn generate_inning_lines(&self) -> Vec<[ChartPoint; 2]> {
        let mut inning_lines = Vec::new();
        let mut prev_state = None;

        for at_bat in self.at_bats.values() {
            let current_state = (at_bat.inning, at_bat.is_top_inning);

            if let Some(prev) = prev_state {
                if prev != current_state {
                    let x = at_bat.at_bat_index as f64;
                    // full inning lines are longer than half inning lines
                    let line = if at_bat.is_top_inning {
                        [(x, -25.0), (x, 25.0)]
                    } else {
                        [(x, -15.0), (x, 15.0)]
                    };
                    inning_lines.push(line);
                }
            }

            prev_state = Some(current_state);
        }

        inning_lines
    }

    /// Interpolate points when there aren't that many at bats in a game yet. This fills in the
    /// chart and makes it a lot easier to read in the first few innings of a game.
    fn interpolate_points(points: Vec<ChartPoint>, target_count: usize) -> Vec<ChartPoint> {
        let current_count = points.len();
        if current_count <= 1 {
            return points;
        }

        let points_to_add = target_count.saturating_sub(current_count);
        let segments = current_count - 1;
        let points_per_segment = points_to_add / segments;
        let extra_points = points_to_add % segments;

        let mut new_points = Vec::with_capacity(target_count);

        for i in 0..segments {
            new_points.push(points[i]);

            let mut insert_count = points_per_segment;
            if i < extra_points {
                insert_count += 1;
            }

            if insert_count > 0 {
                new_points.extend(Self::create_interpolated_points(
                    points[i],
                    points[i + 1],
                    insert_count,
                ));
            }
        }

        new_points.push(points[current_count - 1]);
        new_points
    }

    fn create_interpolated_points(
        start: ChartPoint,
        end: ChartPoint,
        count: usize,
    ) -> impl Iterator<Item = ChartPoint> {
        (1..=count).map(move |p| {
            let ratio = p as f64 / (count + 1) as f64;
            let x = start.0 + ratio * (end.0 - start.0);
            let y = start.1 + ratio * (end.1 - start.1);
            (x, y)
        })
    }
}
