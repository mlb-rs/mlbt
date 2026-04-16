use crate::components::team_page::TeamGame;
use crate::state::team_page::{TeamPageState, TeamSection};
use crate::theme::Theme;
use chrono::{Datelike, NaiveDate};
use mlbt_api::team::RosterType;
use time::{Date, Month};
use tui::prelude::*;
use tui::widgets::calendar::{CalendarEventStore, Monthly};
use tui::widgets::{Block, BorderType, Borders, Cell, Padding, Paragraph, Row, Table};

const ROSTER_HEADER: &[&str] = &["Pos", "B/T", "Ht", "Wt", "DOB"];

const TITLE_STYLE: Style = Style::new().bold().underlined();
const HOME_STYLE: Style = Style::new().fg(Color::Blue);
const AWAY_STYLE: Style = Style::new().fg(Color::White);
const TODAY_STYLE: Style = Style::new().fg(Color::Green).bold();
const PAST_STYLE: Style = Style::new().fg(Theme::DIMMED);

pub struct TeamPageWidget<'a> {
    pub state: &'a mut TeamPageState,
}

impl Widget for TeamPageWidget<'_> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let active = self.state.active_section;
        let roster_type = self.state.roster_type;

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .padding(Padding::new(1, 1, 0, 0))
            .title(Span::styled(
                format!(" {} ", self.state.team.name),
                Style::default().fg(Theme::TITLE_FG).bg(Theme::TITLE_BG),
            ));
        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height < 3 || inner.width < 10 {
            return;
        }

        // vertical split: top (roster + schedule), spacer, bottom (transactions)
        let [top, _spacer, bottom] = Layout::vertical([
            Constraint::Percentage(65),
            Constraint::Length(1),
            Constraint::Percentage(35),
        ])
        .areas(inner);

        // top: roster | separator | schedule
        let [left_with_pad, right_with_border] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(30)]).areas(top);

        let left = Rect {
            width: left_with_pad.width.saturating_sub(1),
            ..left_with_pad
        };

        // extend the separator block 1 row into the spacer
        let sep_render_area = Rect {
            height: right_with_border.height + 1,
            ..right_with_border
        };
        let sep_block = Block::default()
            .borders(Borders::LEFT)
            .border_type(BorderType::Rounded)
            .padding(Padding::new(1, 0, 0, 0));
        let right = sep_block.inner(right_with_border);
        sep_block.render(sep_render_area, buf);

        self.render_roster(left, active, roster_type, buf);
        self.render_schedule(right, active, buf);
        self.render_transactions(bottom, active, buf);
    }
}

impl TeamPageWidget<'_> {
    fn render_roster(
        &mut self,
        area: Rect,
        active: TeamSection,
        roster_type: RosterType,
        buf: &mut Buffer,
    ) {
        if area.height < 1 {
            return;
        }

        let roster = &self.state.roster;
        if roster.is_empty() {
            Paragraph::new(Span::styled(
                "  No roster data",
                Style::default().fg(Theme::DIMMED),
            ))
            .render(area, buf);
            return;
        }

        let is_40man = roster_type == RosterType::FortyMan;
        let roster_label = match roster_type {
            RosterType::Active => "Roster (active)",
            RosterType::FortyMan => "Roster (40 man)",
        };

        let mut header_cells: Vec<Cell> = vec![Cell::from(roster_label)];
        header_cells.extend(ROSTER_HEADER.iter().map(|h| Cell::from(*h)));
        let mut widths = vec![
            Constraint::Length(26), // ## Name
            Constraint::Length(4),  // Pos
            Constraint::Length(5),  // B/T
            Constraint::Length(7),  // Ht
            Constraint::Length(5),  // Wt
            Constraint::Length(12), // DOB
        ];
        if is_40man {
            header_cells.push(Cell::from("Status"));
            widths.push(Constraint::Fill(1));
        }

        let header = Row::new(header_cells).style(TITLE_STYLE);

        let mut rows: Vec<Row> = Vec::new();
        let mut current_group = None;

        for row in roster {
            if current_group != Some(row.position_group) {
                current_group = Some(row.position_group);
                rows.push(Row::new(vec![Cell::from(Span::styled(
                    row.position_group.label(),
                    Style::default().bold(),
                ))]));
            }

            let mut cells = vec![
                Cell::from(Line::from(vec![
                    Span::styled(
                        format!("{:>2}  ", row.number),
                        Style::default().fg(Theme::DIMMED),
                    ),
                    Span::raw(row.name.as_str()),
                ])),
                Cell::from(row.position.as_str()),
                Cell::from(row.bats_throws.as_str()),
                Cell::from(row.height.as_str()),
                Cell::from(row.weight.as_str()),
                Cell::from(row.dob.as_str()),
            ];
            if is_40man {
                cells.push(Cell::from(Span::styled(
                    row.status.as_str(),
                    il_status_style(&row.status_code),
                )));
            }
            rows.push(Row::new(cells));
        }

        let table = Table::new(rows, widths)
            .header(header)
            .row_highlight_style(highlight_style(active, TeamSection::Roster))
            .column_spacing(1);

        StatefulWidget::render(table, area, buf, &mut self.state.roster_selection);
    }

    fn render_calendar(&self, area: Rect, buf: &mut Buffer) {
        let selected_date = self
            .state
            .schedule_selection
            .selected()
            .and_then(|i| self.state.schedule.get(i))
            .map(|g| g.date)
            .unwrap_or(self.state.date);

        let selected_month = (selected_date.year(), selected_date.month());
        let mut events = CalendarEventStore::default();
        for game in &self.state.schedule {
            if (game.date.year(), game.date.month()) != selected_month {
                continue;
            }
            let style = if game.is_home { HOME_STYLE } else { AWAY_STYLE };
            events.add(chrono_to_time(game.date), style);
        }
        events.add(chrono_to_time(self.state.date), TODAY_STYLE);

        let cal = Monthly::new(chrono_to_time(selected_date), events)
            .show_weekdays_header(Style::default())
            .default_style(PAST_STYLE);
        let cal_width = cal.width();
        let pad_left = area.width.saturating_sub(cal_width) / 2;
        let centered = Rect {
            x: area.x + pad_left.saturating_sub(1),
            width: cal_width,
            ..area
        };
        cal.render(centered, buf);
    }

    fn render_schedule(&mut self, area: Rect, active: TeamSection, buf: &mut Buffer) {
        if area.height < 1 {
            return;
        }

        let cal_height = if self.state.show_calendar { 7 } else { 0 };
        let [header_area, cal_area, list_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(cal_height),
            Constraint::Fill(1),
        ])
        .areas(area);

        let padded = format!("{:<width$}", "Schedule", width = header_area.width as usize);
        Line::from(Span::styled(padded, TITLE_STYLE)).render(header_area, buf);

        if self.state.show_calendar {
            self.render_calendar(cal_area, buf);
        }

        let games = &self.state.schedule;
        if games.is_empty() {
            Paragraph::new(Span::styled(
                "  No schedule data",
                Style::default().fg(Theme::DIMMED),
            ))
            .render(list_area, buf);
            return;
        }

        let widths = [
            Constraint::Length(8), // Date
            Constraint::Length(9), // Opponent
            Constraint::Fill(1),   // Time/Score
        ];

        let rows: Vec<Row> = games
            .iter()
            .map(|g| {
                let (date_style, text_style, score_style) = style_schedule_game(self.state.date, g);
                Row::new(vec![
                    Cell::from(Span::styled(g.date_display.as_str(), date_style)),
                    Cell::from(Span::styled(g.opponent.as_str(), text_style)),
                    Cell::from(Span::styled(g.time_or_score.as_str(), score_style)),
                ])
            })
            .collect();

        let table = Table::new(rows, widths)
            .row_highlight_style(highlight_style(active, TeamSection::Schedule))
            .column_spacing(1);

        StatefulWidget::render(table, list_area, buf, &mut self.state.schedule_selection);
    }

    fn render_transactions(&mut self, area: Rect, active: TeamSection, buf: &mut Buffer) {
        if area.height < 1 {
            return;
        }

        let [header_area, body_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

        let padded = format!(
            "{:<width$}",
            "Transactions",
            width = header_area.width as usize
        );
        Line::from(Span::styled(padded, TITLE_STYLE)).render(header_area, buf);

        if self.state.transactions.is_empty() {
            Paragraph::new(Span::styled(
                "  No recent transactions",
                Style::default().fg(Theme::DIMMED),
            ))
            .render(body_area, buf);
            return;
        }

        // update scroll before building lines to avoid borrow conflict
        let is_active = active == TeamSection::Transactions;
        if is_active {
            self.state
                .update_transaction_scroll(body_area.width, body_area.height);
        }

        let selected = self.state.selected_transaction;
        let highlight_style = highlight_style(active, TeamSection::Transactions);
        let date_width = TeamPageState::TRANSACTION_DATE_WIDTH;
        let lines: Vec<Line> = self
            .state
            .transactions
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let (date_style, text_style) = if is_active && i == selected {
                    (highlight_style, highlight_style)
                } else {
                    (Style::default().fg(Theme::DIMMED), Style::default())
                };
                Line::from(vec![
                    Span::styled(format!("{:<date_width$}", t.date), date_style),
                    Span::styled(&t.description, text_style),
                ])
            })
            .collect();

        Paragraph::new(lines)
            .wrap(tui::widgets::Wrap { trim: false })
            .scroll((self.state.transaction_scroll, 0))
            .render(body_area, buf);
    }
}

/// Style for IL status codes in the 40-man roster view.
fn il_status_style(code: &str) -> Style {
    match code {
        "D10" | "D15" => Style::default().fg(Color::Yellow),
        "D60" => Style::default().fg(Color::Red),
        "RM" => Style::default().fg(Color::DarkGray),
        _ => Style::default(),
    }
}

/// Style for the selected row in the roster and schedule tables.
fn highlight_style(active: TeamSection, desired: TeamSection) -> Style {
    if active == desired {
        Style::default().bg(Theme::ACCENT_BG).fg(Theme::ACCENT_FG)
    } else {
        Style::default()
    }
}

/// Style for the calendar day and list text of a game in the schedule.
fn style_schedule_game(today: NaiveDate, g: &TeamGame) -> (Style, Style, Style) {
    let date_style = if g.date == today {
        TODAY_STYLE
    } else if g.is_past {
        PAST_STYLE
    } else if g.is_home {
        HOME_STYLE
    } else {
        AWAY_STYLE
    };
    let text_style = if g.is_past {
        PAST_STYLE
    } else {
        Style::default()
    };
    let score_style = match g.is_win {
        Some(true) => Style::new().fg(Color::Green),
        Some(false) => Style::new().fg(Color::Red),
        None => text_style,
    };
    (date_style, text_style, score_style)
}

fn chrono_to_time(d: NaiveDate) -> Date {
    let month = Month::try_from(d.month() as u8).unwrap();
    Date::from_calendar_date(d.year(), month, d.day() as u8).unwrap()
}
