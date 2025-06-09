use crate::components::game::live_game::GameStateV2;
use crate::components::game::win_probability::WinProbabilityAtBat;
use crate::ui::gameday::plays::{BLUE, GREEN, RED};
use indexmap::IndexMap;
use tui::prelude::*;
use tui::widgets::{Bar, BarChart, BarGroup, Cell, Row, Table};

pub struct WinProbabilityWidget<'a> {
    pub game: &'a GameStateV2,
    pub selected_at_bat: Option<u8>,
}

struct WinProbabilityData<'a> {
    at_bats: &'a IndexMap<u8, WinProbabilityAtBat>,
    selected_at_bat_index: Option<u8>,
    table_height: u16,
}

impl Widget for WinProbabilityWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(26), Constraint::Fill(1)].as_ref())
            .horizontal_margin(2)
            .vertical_margin(1)
            .split(area);

        let data = WinProbabilityData::new(self.game, self.selected_at_bat, chunks[0].height);

        data.render_table(chunks[0], buf);
        data.render_chart(chunks[1], buf);
    }
}

impl<'a> WinProbabilityData<'a> {
    fn new(game: &'a GameStateV2, selected_at_bat_index: Option<u8>, table_height: u16) -> Self {
        Self {
            at_bats: &game.win_probability.at_bats,
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
}
