use crate::state::app_state::HomeOrAway;
use crate::state::boxscore::BoxscoreState;
use tui::prelude::*;
use tui::symbols::scrollbar::DOUBLE_VERTICAL;
use tui::widgets::{Block, Borders, Row, Scrollbar, ScrollbarOrientation, Table};

const BATTER_WIDTHS: [Constraint; 9] = [
    Constraint::Length(25), // player name
    Constraint::Length(4),  // ab
    Constraint::Length(4),  // r
    Constraint::Length(4),  // h
    Constraint::Length(4),  // rbi
    Constraint::Length(4),  // bb
    Constraint::Length(4),  // k
    Constraint::Length(4),  // lob
    Constraint::Length(5),  // avg
];

const PITCHER_WIDTHS: [Constraint; 9] = [
    Constraint::Length(25), // pitcher name
    Constraint::Length(5),  // ip
    Constraint::Length(4),  // h
    Constraint::Length(4),  // r
    Constraint::Length(4),  // er
    Constraint::Length(4),  // bb
    Constraint::Length(4),  // k
    Constraint::Length(4),  // hr
    Constraint::Length(5),  // era
];

pub struct TeamBatterBoxscoreWidget<'a> {
    pub active: HomeOrAway,
    pub state: &'a mut BoxscoreState,
}

impl Widget for TeamBatterBoxscoreWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.state
            .update_scroll_state_for_viewport(area.height, area.width);

        let (
            batting_height,
            notes_height,
            pitching_height,
            game_notes_height,
            total_content_height,
        ) = self.state.get_content_heights(self.active);

        let batting_rows = self.state.get_batting_rows(self.active);
        let batting_notes_paragraph = self.state.get_batting_notes_paragraph(self.active);
        let pitching_rows = self.state.get_pitching_rows(self.active);
        let game_notes_paragraph = self.state.get_game_notes_paragraph();

        if total_content_height > area.height {
            // Create a large virtual area for rendering
            let virtual_area = Rect {
                x: area.x,
                y: 0,
                width: area.width,
                height: total_content_height,
            };

            // Calculate layout in virtual space
            let [boxscore_area, notes_area, pitchers_area, game_notes_area] = Layout::vertical([
                Constraint::Length(batting_height),
                Constraint::Length(notes_height),
                Constraint::Length(pitching_height),
                Constraint::Length(game_notes_height),
            ])
            .spacing(1)
            .areas(virtual_area);

            // Adjust areas based on scroll position and clip to visible area
            let scroll_offset = self.state.scroll as i32;
            let visible_top = area.y as i32;
            let visible_bottom = (area.y + area.height) as i32;

            let adjust_area = |area: Rect| -> Option<Rect> {
                let area_top = area.y as i32 - scroll_offset + visible_top;
                let area_bottom = area_top + area.height as i32;

                if area_bottom <= visible_top || area_top >= visible_bottom {
                    return None; // Area is not visible
                }

                let clipped_top = area_top.max(visible_top);
                let clipped_bottom = area_bottom.min(visible_bottom);
                let clipped_height = (clipped_bottom - clipped_top) as u16;

                if clipped_height > 0 {
                    Some(Rect {
                        x: area.x,
                        y: clipped_top as u16,
                        width: area.width,
                        height: clipped_height,
                    })
                } else {
                    None
                }
            };

            // Render components that are visible
            if let Some(visible_boxscore) = adjust_area(boxscore_area) {
                Widget::render(
                    Table::new(
                        batting_rows
                            .into_iter()
                            .skip(scroll_offset as usize)
                            .map(Row::new),
                        BATTER_WIDTHS,
                    )
                    .column_spacing(0)
                    .style(Style::default().fg(Color::White))
                    .header(
                        Row::new(self.state.boxscore.get_batting_header().iter().copied())
                            .bold()
                            .underlined(),
                    )
                    .block(Block::default().borders(Borders::NONE)),
                    visible_boxscore,
                    buf,
                );
            }

            if let Some(visible_note) = adjust_area(notes_area) {
                if let Some(paragraph) = batting_notes_paragraph {
                    let offset = (scroll_offset as u16).saturating_sub(batting_height + 1); // +1 for space
                    let scrolled = if offset > 0 {
                        &paragraph.clone().scroll((offset, 0))
                    } else {
                        paragraph
                    };
                    Widget::render(scrolled, visible_note, buf);
                }
            }

            if let Some(visible_pitchers) = adjust_area(pitchers_area) {
                let offset = (scroll_offset as u16)
                    .saturating_sub(batting_height + 1)
                    .saturating_sub(notes_height + 1); // +1 for space
                Widget::render(
                    Table::new(
                        pitching_rows
                            .into_iter()
                            .skip(offset as usize)
                            .map(Row::new),
                        PITCHER_WIDTHS,
                    )
                    .column_spacing(0)
                    .style(Style::default().fg(Color::White))
                    .header(
                        Row::new(self.state.boxscore.get_pitching_header().iter().copied())
                            .bold()
                            .underlined(),
                    )
                    .block(Block::default().borders(Borders::NONE)),
                    visible_pitchers,
                    buf,
                );
            }

            if let Some(visible_game_notes) = adjust_area(game_notes_area) {
                if let Some(paragraph) = game_notes_paragraph {
                    let offset = (scroll_offset as u16)
                        .saturating_sub(batting_height + 1)
                        .saturating_sub(notes_height + 1)
                        .saturating_sub(pitching_height + 1); // +1 for space
                    let scrolled = if offset > 0 {
                        &paragraph.clone().scroll((offset, 0))
                    } else {
                        paragraph
                    };

                    Widget::render(scrolled, visible_game_notes, buf);
                }
            }

            // Render scrollbar
            let scrollbar_area = Rect {
                x: area.x + area.width + 1, // + 1 to move it over the border
                y: area.y,
                width: 1,
                height: area.height,
            };

            StatefulWidget::render(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    // use the same thumb and track symbol to hide the thumb
                    .thumb_symbol(DOUBLE_VERTICAL.track)
                    .track_symbol(Some(DOUBLE_VERTICAL.track))
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓")),
                scrollbar_area,
                buf,
                &mut self.state.scroll_state,
            );
        } else {
            // Render normally without scrolling
            let [boxscore, note, pitchers, game_notes_area] = Layout::vertical([
                Constraint::Length(batting_height),
                Constraint::Length(notes_height),
                Constraint::Length(pitching_height),
                Constraint::Fill(1),
            ])
            .spacing(1)
            .areas(area);

            Widget::render(
                Table::new(batting_rows.into_iter().map(Row::new), BATTER_WIDTHS)
                    .column_spacing(0)
                    .style(Style::default().fg(Color::White))
                    .header(
                        Row::new(self.state.boxscore.get_batting_header().iter().copied())
                            .bold()
                            .underlined(),
                    )
                    .block(Block::default().borders(Borders::NONE)),
                boxscore,
                buf,
            );

            if let Some(paragraph) = batting_notes_paragraph {
                Widget::render(paragraph, note, buf);
            }

            Widget::render(
                Table::new(pitching_rows.into_iter().map(Row::new), PITCHER_WIDTHS)
                    .column_spacing(0)
                    .style(Style::default().fg(Color::White))
                    .header(
                        Row::new(self.state.boxscore.get_pitching_header().iter().copied())
                            .bold()
                            .underlined(),
                    )
                    .block(Block::default().borders(Borders::NONE)),
                pitchers,
                buf,
            );

            if let Some(paragraph) = game_notes_paragraph {
                Widget::render(paragraph, game_notes_area, buf);
            }
        }
    }
}
