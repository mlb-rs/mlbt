use crate::state::app_state::HomeOrAway;
use crate::state::boxscore::BoxscoreState;
use tui::prelude::*;
use tui::symbols::scrollbar::DOUBLE_VERTICAL;
use tui::widgets::{Block, Borders, Cell, Row, Scrollbar, ScrollbarOrientation, Table};

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
const BATTING_HEADER: &[&str] = &["player", "ab", "r", "h", "rbi", "bb", "k", "lob", "avg"];

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
const PITCHING_HEADER: &[&str] = &["pitcher", "ip", "h", "r", "er", "bb", "k", "hr", "era"];

#[derive(Clone, Copy)]
struct ScrollParams {
    scroll_offset: i32,
    visible_top: i32,
    visible_bottom: i32,
}

pub struct TeamBatterBoxscoreWidget<'a> {
    pub active: HomeOrAway,
    pub state: &'a mut BoxscoreState,
}

impl Widget for TeamBatterBoxscoreWidget<'_> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        self.state.sync_scrollbar(area.height, area.width);

        if self.state.get_total_content_height() > area.height {
            self.render_scrollable(area, buf);
        } else {
            self.render_static(area, buf);
        }
    }
}

impl TeamBatterBoxscoreWidget<'_> {
    fn render_scrollable(&mut self, area: Rect, buf: &mut Buffer) {
        let (batting_height, notes_height, pitching_height, _, total_content_height) =
            self.state.get_content_heights(self.active);

        // larger area than is visible
        let virtual_area = Rect {
            x: area.x,
            y: 0,
            width: area.width,
            height: total_content_height,
        };

        let [boxscore_area, notes_area, pitchers_area, game_notes_area] =
            self.get_layout_areas(virtual_area);

        let scroll_offset = self.state.scroll as u16;
        let params = ScrollParams {
            scroll_offset: scroll_offset as i32,
            visible_top: area.y as i32,
            visible_bottom: (area.y + area.height) as i32,
        };

        if let Some(visible_boxscore) = adjust_area_for_scroll(boxscore_area, params) {
            Widget::render(
                create_table(
                    self.state.get_batting_rows(self.active),
                    &BATTER_WIDTHS,
                    BATTING_HEADER,
                    scroll_offset as usize,
                ),
                visible_boxscore,
                buf,
            );
        }

        if let Some(visible_note) = adjust_area_for_scroll(notes_area, params) {
            if let Some(paragraph) = self.state.get_batting_notes_paragraph(self.active) {
                let offset = scroll_offset.saturating_sub(batting_height + 1); // +1 for space
                render_paragraph_with_scroll(paragraph, offset, visible_note, buf);
            }
        }

        if let Some(visible_pitchers) = adjust_area_for_scroll(pitchers_area, params) {
            let offset = scroll_offset
                .saturating_sub(batting_height + 1)
                .saturating_sub(notes_height + 1); // +1 for space
            Widget::render(
                create_table(
                    self.state.get_pitching_rows(self.active),
                    &PITCHER_WIDTHS,
                    PITCHING_HEADER,
                    offset as usize,
                ),
                visible_pitchers,
                buf,
            );
        }

        if let Some(visible_game_notes) = adjust_area_for_scroll(game_notes_area, params) {
            if let Some(paragraph) = self.state.get_game_notes_paragraph() {
                let offset = scroll_offset
                    .saturating_sub(batting_height + 1)
                    .saturating_sub(notes_height + 1)
                    .saturating_sub(pitching_height + 1); // +1 for space
                render_paragraph_with_scroll(paragraph, offset, visible_game_notes, buf);
            }
        }

        self.render_scrollbar(area, buf);
    }

    fn render_static(&mut self, area: Rect, buf: &mut Buffer) {
        let [boxscore, note, pitchers, game_notes_area] = self.get_layout_areas(area);

        Widget::render(
            create_table(
                self.state.get_batting_rows(self.active),
                &BATTER_WIDTHS,
                BATTING_HEADER,
                0,
            ),
            boxscore,
            buf,
        );

        if let Some(paragraph) = self.state.get_batting_notes_paragraph(self.active) {
            Widget::render(paragraph, note, buf);
        }

        Widget::render(
            create_table(
                self.state.get_pitching_rows(self.active),
                &PITCHER_WIDTHS,
                PITCHING_HEADER,
                0,
            ),
            pitchers,
            buf,
        );

        if let Some(paragraph) = self.state.get_game_notes_paragraph() {
            Widget::render(paragraph, game_notes_area, buf);
        }
    }

    fn render_scrollbar(&mut self, area: Rect, buf: &mut Buffer) {
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
    }

    fn get_layout_areas(&mut self, area: Rect) -> [Rect; 4] {
        let (batting_height, notes_height, pitching_height, game_notes_height, _) =
            self.state.get_content_heights(self.active);

        Layout::vertical([
            Constraint::Length(batting_height),
            Constraint::Length(notes_height),
            Constraint::Length(pitching_height),
            Constraint::Length(game_notes_height),
        ])
        .spacing(1)
        .areas(area)
    }
}

fn create_table<'a, I, R>(
    rows: I,
    widths: &[Constraint],
    header: &'static [&'static str],
    skip_rows: usize,
) -> Table<'a>
where
    I: IntoIterator<Item = R>,
    R: IntoIterator<Item = Cell<'a>>,
{
    Table::new(rows.into_iter().skip(skip_rows).map(Row::new), widths)
        .column_spacing(0)
        .style(Style::default().fg(Color::White))
        .header(Row::new(header.iter().copied()).bold().underlined())
        .block(Block::default().borders(Borders::NONE))
}

fn render_paragraph_with_scroll(
    paragraph: &tui::widgets::Paragraph<'static>,
    scroll_offset: u16,
    area: Rect,
    buf: &mut Buffer,
) {
    if scroll_offset > 0 {
        let scrolled = paragraph.clone().scroll((scroll_offset, 0));
        Widget::render(scrolled, area, buf);
    } else {
        Widget::render(paragraph, area, buf);
    };
}

/// Adjust areas based on scroll position and clip to visible area
fn adjust_area_for_scroll(area: Rect, params: ScrollParams) -> Option<Rect> {
    let area_top = area.y as i32 - params.scroll_offset + params.visible_top;
    let area_bottom = area_top + area.height as i32;

    if area_bottom <= params.visible_top || area_top >= params.visible_bottom {
        return None; // Area is not visible
    }

    let clipped_top = area_top.max(params.visible_top);
    let clipped_bottom = area_bottom.min(params.visible_bottom);
    let clipped_height = clipped_bottom - clipped_top;

    if clipped_height > 0 {
        Some(Rect {
            x: area.x,
            y: clipped_top as u16,
            width: area.width,
            height: clipped_height as u16,
        })
    } else {
        None
    }
}
