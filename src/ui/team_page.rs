use crate::state::team_page::{TeamPageState, TeamSection};
use mlbt_api::team::RosterType;
use tui::prelude::*;
use tui::widgets::{Block, BorderType, Borders, Cell, Padding, Paragraph, Row, Table};

const ROSTER_HEADER: &[&str] = &["Pos", "B/T", "Ht", "Wt", "DOB"];

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
                Style::default().fg(Color::Black).bg(Color::Blue),
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
        Self::render_transactions(bottom, &self.state.transactions, buf);
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
                Style::default().fg(Color::DarkGray),
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

        let header = Row::new(header_cells)
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

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
                        Style::default().fg(Color::DarkGray),
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

    fn render_schedule(&mut self, area: Rect, active: TeamSection, buf: &mut Buffer) {
        if area.height < 1 {
            return;
        }

        let games = &self.state.schedule;
        if games.is_empty() {
            Paragraph::new(Span::styled(
                "  No schedule data",
                Style::default().fg(Color::DarkGray),
            ))
            .render(area, buf);
            return;
        }

        let today_idx = self.state.today_schedule_idx;

        let widths = [
            Constraint::Length(8), // Date
            Constraint::Length(9), // Opponent
            Constraint::Fill(1),   // Time/Score
        ];

        let header = Row::new(vec![
            Cell::from("Schedule"),
            Cell::default(),
            Cell::default(),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

        let rows: Vec<Row> = games
            .iter()
            .enumerate()
            .map(|(i, g)| {
                let style = if i == today_idx {
                    Style::default().fg(Color::Green)
                } else if g.is_past {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };
                Row::new(vec![
                    Cell::from(g.date_display.as_str()),
                    Cell::from(g.opponent.as_str()),
                    Cell::from(g.time_or_score.as_str()),
                ])
                .style(style)
            })
            .collect();

        let table = Table::new(rows, widths)
            .header(header)
            .row_highlight_style(highlight_style(active, TeamSection::Schedule))
            .column_spacing(1);

        StatefulWidget::render(table, area, buf, &mut self.state.schedule_selection);
    }

    fn render_transactions(
        area: Rect,
        transactions: &[crate::components::team_page::TransactionRow],
        buf: &mut Buffer,
    ) {
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
        Line::from(Span::styled(
            padded,
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ))
        .render(header_area, buf);

        if transactions.is_empty() {
            Paragraph::new(Span::styled(
                "  No recent transactions",
                Style::default().fg(Color::DarkGray),
            ))
            .render(body_area, buf);
            return;
        }

        let lines: Vec<Line> = transactions
            .iter()
            .map(|t| {
                Line::from(vec![
                    Span::styled(
                        format!("{:<8}", t.date),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::raw(&t.description),
                ])
            })
            .collect();

        Paragraph::new(lines)
            .wrap(tui::widgets::Wrap { trim: false })
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
        Style::default().bg(Color::Blue).fg(Color::Black)
    } else {
        Style::default()
    }
}
