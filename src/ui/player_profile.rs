use crate::components::player_profile::PlayerProfile;
use crate::state::player_profile::PlayerProfileState;
use mlbt_api::season::GameType;
use mlbt_api::stats::Split;
use tui::prelude::*;
use tui::symbols::scrollbar::DOUBLE_VERTICAL;
use tui::widgets::{
    Block, BorderType, Borders, Padding, Paragraph, Row, Scrollbar, ScrollbarOrientation, Table,
};

#[derive(Clone, Copy)]
struct ScrollParams {
    scroll_offset: i32,
    visible_top: i32,
    visible_bottom: i32,
}

pub struct PlayerProfileWidget<'a> {
    pub state: &'a mut PlayerProfileState,
}

impl<'a> Widget for PlayerProfileWidget<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let profile = &self.state.profile;

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .padding(Padding::new(1, 1, 0, 0))
            .title(Line::from(vec![
                Span::styled(
                    format!(" #{} {} ", profile.number, profile.name),
                    Style::default().fg(Color::Black).bg(Color::Blue),
                ),
                Span::styled(
                    format!(" {} ", profile.team.abbreviation),
                    Style::default().fg(Color::Black).bg(Color::Cyan),
                ),
            ]));
        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        // Calculate section heights
        let bio_height = 4;

        let season_splits = profile
            .find_stat_group("season")
            .map(|s| s.splits.len())
            .unwrap_or(0);
        let season_height = if season_splits > 0 {
            // With data: title + header + rows.
            season_splits as u16 + 2
        } else {
            // Empty: title + "No data"
            2
        };

        let yby_splits = profile
            .find_stat_group("yearByYear")
            .map(|s| s.splits.len())
            .unwrap_or(0);
        let career_splits = profile
            .find_stat_group("career")
            .map(|s| s.splits.len())
            .unwrap_or(0);
        let career_height = if yby_splits > 0 {
            yby_splits as u16 + career_splits as u16 + 2 // career_splits adds the totals row
        } else {
            0
        };

        let game_log_rows = profile
            .find_stat_group("gameLog")
            .map(|s| s.splits.len().min(15))
            .unwrap_or(0);
        let game_log_height = if game_log_rows > 0 {
            game_log_rows as u16 + 2
        } else {
            0
        };

        let section_heights = [
            bio_height + 1, // +1 for blank line below section
            season_height + 1,
            career_height + 1,
            game_log_height,
        ];
        let total_content_height = section_heights.iter().sum::<u16>();
        self.state.content_height = total_content_height;
        self.state.viewport_height = inner.height;

        // If content fits, render without scrolling
        if total_content_height <= inner.height {
            let [bio_area, season_area, career_area, gamelog_area] =
                Layout::vertical(section_heights).areas(inner);

            self.render_bio(bio_area, buf);
            self.render_season(season_area, buf);
            self.render_career(career_area, buf);
            self.render_game_log(gamelog_area, buf);
            return;
        }

        // Scrollable rendering
        let virtual_area = Rect {
            x: inner.x,
            y: 0,
            width: inner.width,
            height: total_content_height,
        };

        let [bio_area, season_area, career_area, gamelog_area] =
            Layout::vertical(section_heights).areas(virtual_area);

        let scroll_offset = self.state.scroll_offset;
        let params = ScrollParams {
            scroll_offset: scroll_offset as i32,
            visible_top: inner.y as i32,
            visible_bottom: (inner.y + inner.height) as i32,
        };

        if let Some((area, skip)) = adjust_area_for_scroll(bio_area, params) {
            self.render_bio_scrolled(area, skip, buf);
        }
        if let Some((area, skip)) = adjust_area_for_scroll(season_area, params) {
            self.render_season_scrolled(area, skip, buf);
        }
        if let Some((area, skip)) = adjust_area_for_scroll(career_area, params) {
            self.render_career_scrolled(area, skip, buf);
        }
        if let Some((area, skip)) = adjust_area_for_scroll(gamelog_area, params) {
            self.render_game_log_scrolled(area, skip, buf);
        }

        self.state.sync_scrollbar();
        self.render_scrollbar(inner, buf);
    }
}

impl PlayerProfileWidget<'_> {
    fn render_bio(&self, area: Rect, buf: &mut Buffer) {
        self.render_bio_scrolled(area, 0, buf);
    }

    fn render_bio_scrolled(&self, area: Rect, scroll: u16, buf: &mut Buffer) {
        let [bio_text_area, game_type_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(20)]).areas(area);

        self.render_game_type_selector(game_type_area, buf);
        Paragraph::new(self.state.profile.bio_lines())
            .scroll((scroll, 0))
            .render(bio_text_area, buf);
    }

    fn render_game_type_selector(&self, area: Rect, buf: &mut Buffer) {
        let selected = Style::default().fg(Color::Black).bg(Color::Blue);
        let normal = Style::default().fg(Color::DarkGray);

        let (reg_style, st_style) = match self.state.game_type {
            GameType::RegularSeason => (selected, normal),
            GameType::SpringTraining => (normal, selected),
        };

        Paragraph::new(vec![
            Line::from(Span::styled(" Regular Season  ", reg_style)),
            Line::from(Span::styled(" Spring Training ", st_style)),
        ])
        .alignment(Alignment::Right)
        .render(area, buf);
    }

    fn render_season(&self, area: Rect, buf: &mut Buffer) {
        self.render_season_scrolled(area, 0, buf);
    }

    fn render_season_scrolled(&self, area: Rect, skip: u16, buf: &mut Buffer) {
        let title = format!("{} Season", self.state.season_year);
        let splits = self
            .state
            .profile
            .find_stat_group("season")
            .map(|s| s.splits.as_slice())
            .unwrap_or(&[]);
        render_section(&title, splits, false, area, skip, buf);
    }

    fn render_career(&self, area: Rect, buf: &mut Buffer) {
        self.render_career_scrolled(area, 0, buf);
    }

    fn render_career_scrolled(&self, area: Rect, skip: u16, buf: &mut Buffer) {
        let profile = &self.state.profile;
        if let Some(yby) = profile.find_stat_group("yearByYear") {
            let career_totals = profile.find_stat_group("career").map(|c| &c.splits);
            render_section_with_totals("Career Stats", &yby.splits, career_totals, area, skip, buf);
        }
    }

    fn render_game_log(&self, area: Rect, buf: &mut Buffer) {
        self.render_game_log_scrolled(area, 0, buf);
    }

    fn render_game_log_scrolled(&self, area: Rect, skip: u16, buf: &mut Buffer) {
        if let Some(game_log) = self.state.profile.find_stat_group("gameLog") {
            render_stat_section(
                "Recent Games",
                &game_log.splits,
                area,
                skip,
                buf,
                PlayerProfile::build_game_log_rows,
            );
        }
    }

    fn render_scrollbar(&mut self, area: Rect, buf: &mut Buffer) {
        let scrollbar_area = Rect {
            x: area.x + area.width + 1, // +1 to render over the border
            y: area.y,
            width: 1,
            height: area.height,
        };
        StatefulWidget::render(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .thumb_symbol(DOUBLE_VERTICAL.track)
                .track_symbol(Some(DOUBLE_VERTICAL.track))
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            scrollbar_area,
            buf,
            &mut self.state.scroll_state,
        );
    }
}

fn render_section(
    title: &str,
    splits: &[Split],
    show_year: bool,
    area: Rect,
    skip: u16,
    buf: &mut Buffer,
) {
    if area.height < 2 {
        return;
    }
    if splits.is_empty() {
        if skip == 0 {
            let [title_area, msg_area] =
                Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);
            render_section_title(title, title_area, buf);
            Paragraph::new(Span::styled(
                "  No data",
                Style::default().fg(Color::DarkGray),
            ))
            .render(msg_area, buf);
        }
        return;
    }
    render_stat_section(title, splits, area, skip, buf, |splits| {
        PlayerProfile::build_stat_rows(splits, show_year)
    });
}

fn render_section_with_totals(
    title: &str,
    splits: &[Split],
    career: Option<&Vec<Split>>,
    area: Rect,
    skip: u16,
    buf: &mut Buffer,
) {
    if splits.is_empty() || area.height < 2 {
        return;
    }
    render_stat_section(title, splits, area, skip, buf, |splits| {
        let (header, widths, mut rows) = PlayerProfile::build_stat_rows(splits, true);
        if let Some(career_splits) = career {
            for split in career_splits {
                rows.push(
                    Row::new(PlayerProfile::career_total_cells(split))
                        .style(Style::default().bold()),
                );
            }
        }
        (header, widths, rows)
    });
}

fn render_stat_section<F>(
    title: &str,
    splits: &[Split],
    area: Rect,
    skip: u16,
    buf: &mut Buffer,
    build: F,
) where
    F: FnOnce(&[Split]) -> (Row<'static>, Vec<Constraint>, Vec<Row<'static>>),
{
    let (header, widths, rows) = build(splits);

    if skip == 0 {
        // Title visible, render normally
        let [title_area, table_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
        render_section_title(title, title_area, buf);
        let table = Table::new(rows, widths).header(header).column_spacing(0);
        Widget::render(table, table_area, buf);
    } else {
        // Title is clipped, skip data rows (-1 for the title row)
        let data_skip = (skip - 1) as usize;
        let rows: Vec<Row> = rows.into_iter().skip(data_skip).collect();
        let table = Table::new(rows, widths).header(header).column_spacing(0);
        Widget::render(table, area, buf);
    }
}

fn render_section_title(title: &str, area: Rect, buf: &mut Buffer) {
    Paragraph::new(Line::from(Span::styled(
        format!(" {title} "),
        Style::default().bg(Color::Blue).fg(Color::Black),
    )))
    .render(area, buf);
}

/// Adjust area based on scroll position and clip to visible viewport.
/// Returns (clipped_area, rows_clipped_from_top).
fn adjust_area_for_scroll(area: Rect, params: ScrollParams) -> Option<(Rect, u16)> {
    let area_top = area.y as i32 - params.scroll_offset + params.visible_top;
    let area_bottom = area_top + area.height as i32;

    if area_bottom <= params.visible_top || area_top >= params.visible_bottom {
        return None;
    }

    let clipped_top = area_top.max(params.visible_top);
    let clipped_bottom = area_bottom.min(params.visible_bottom);
    let clipped_height = clipped_bottom - clipped_top;
    let rows_clipped = (clipped_top - area_top) as u16;

    if clipped_height > 0 {
        Some((
            Rect {
                x: area.x,
                y: clipped_top as u16,
                width: area.width,
                height: clipped_height as u16,
            },
            rows_clipped,
        ))
    } else {
        None
    }
}
