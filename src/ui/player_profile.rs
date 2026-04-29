use crate::components::stats::player_profile::PlayerProfile;
use crate::state::player_profile::PlayerProfileState;
use crate::ui::scroll::{ScrollParams, adjust_area_for_scroll, render_scrollbar};
use crate::ui::styling::{border_style, dim_style, selected_style};
use mlbt_api::client::StatGroup;
use mlbt_api::season::GameType;
use mlbt_api::stats::Split;
use tui::prelude::*;
use tui::widgets::{Block, BorderType, Borders, Padding, Paragraph, Row, Table};

pub struct PlayerProfileWidget<'a> {
    pub state: &'a mut PlayerProfileState,
}

impl Widget for PlayerProfileWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let profile = &self.state.profile;

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style())
            .padding(Padding::new(1, 1, 0, 0))
            .title(Line::from(vec![
                Span::styled(
                    format!(" #{} {} ", profile.number, profile.name),
                    selected_style(),
                ),
                Span::styled(
                    if profile.is_minor_league {
                        format!(" {} (MiLB) ", profile.team.abbreviation)
                    } else {
                        format!(" {} ", profile.team.abbreviation)
                    },
                    Style::default().fg(Color::Black).bg(Color::Cyan),
                ),
            ]));
        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        let section_heights = self.state.section_heights();
        let total_content_height = section_heights.iter().sum::<u16>();
        self.state.content_height = total_content_height;
        self.state.viewport_height = inner.height;

        if total_content_height <= inner.height {
            let [
                bio_area,
                season_area,
                splits_area,
                career_area,
                gamelog_area,
            ] = Layout::vertical(section_heights).areas(inner);

            self.render_bio(bio_area, 0, buf);
            self.render_season(season_area, 0, buf);
            self.render_splits(splits_area, 0, buf);
            self.render_career(career_area, 0, buf);
            self.render_game_log(gamelog_area, 0, buf);
            return;
        }

        // for scrollable rendering the layout is in a virtual area, then clip to viewport
        let virtual_area = Rect {
            x: inner.x,
            y: 0,
            width: inner.width,
            height: total_content_height,
        };

        let [
            bio_area,
            season_area,
            splits_area,
            career_area,
            gamelog_area,
        ] = Layout::vertical(section_heights).areas(virtual_area);

        let params = ScrollParams {
            scroll_offset: self.state.scroll_offset as i32,
            visible_top: inner.y as i32,
            visible_bottom: (inner.y + inner.height) as i32,
        };

        if let Some((area, skip)) = adjust_area_for_scroll(bio_area, params) {
            self.render_bio(area, skip, buf);
        }
        if let Some((area, skip)) = adjust_area_for_scroll(season_area, params) {
            self.render_season(area, skip, buf);
        }
        if let Some((area, skip)) = adjust_area_for_scroll(splits_area, params) {
            self.render_splits(area, skip, buf);
        }
        if let Some((area, skip)) = adjust_area_for_scroll(career_area, params) {
            self.render_career(area, skip, buf);
        }
        if let Some((area, skip)) = adjust_area_for_scroll(gamelog_area, params) {
            self.render_game_log(area, skip, buf);
        }

        self.state.sync_scrollbar();
        render_scrollbar(inner, &mut self.state.scroll_state, buf);
    }
}

impl PlayerProfileWidget<'_> {
    fn render_bio(&self, area: Rect, scroll: u16, buf: &mut Buffer) {
        let [bio_text_area, game_type_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(20)]).areas(area);

        self.render_game_type_selector(game_type_area, buf);
        Paragraph::new(self.state.profile.bio.clone())
            .scroll((scroll, 0))
            .render(bio_text_area, buf);
    }

    fn render_game_type_selector(&self, area: Rect, buf: &mut Buffer) {
        let selected = selected_style();
        let normal = dim_style();

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

    fn render_season(&self, area: Rect, skip: u16, buf: &mut Buffer) {
        let title = format!("{} Season", self.state.season_year);
        let splits = &self.state.profile.splits.season;
        render_stat_table(&title, splits, None, false, area, skip, buf);
    }

    fn render_career(&self, area: Rect, skip: u16, buf: &mut Buffer) {
        let splits = &self.state.profile.splits;
        if !splits.year_by_year.is_empty() {
            let career_totals = if splits.career.is_empty() {
                None
            } else {
                Some(&splits.career)
            };
            render_stat_table(
                "Career Stats",
                &splits.year_by_year,
                career_totals,
                true,
                area,
                skip,
                buf,
            );
        }
    }

    fn render_splits(&self, area: Rect, skip: u16, buf: &mut Buffer) {
        let recent_splits = &self.state.profile.splits.recent_splits;
        let is_hitting = matches!(self.state.stat_group, StatGroup::Hitting);
        if let Some((header, widths, rows)) =
            PlayerProfile::build_splits_rows(recent_splits, is_hitting)
        {
            render_table_with_title("Splits", header, widths, rows, area, skip, buf);
        }
    }

    fn render_game_log(&self, area: Rect, skip: u16, buf: &mut Buffer) {
        let splits = &self.state.profile.splits.game_log;
        if let Some((header, widths, rows)) = PlayerProfile::build_game_log_rows(splits) {
            render_table_with_title("Recent Games", header, widths, rows, area, skip, buf);
        }
    }
}

/// Render a stat section with a title, header, data rows, and optional career totals.
fn render_stat_table(
    title: &str,
    splits: &[Split],
    career: Option<&Vec<Split>>,
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
            Paragraph::new(Span::styled("  No data", dim_style())).render(msg_area, buf);
        }
        return;
    }

    if let Some((header, widths, mut rows)) = PlayerProfile::build_stat_rows(splits, show_year) {
        if let Some(total) = career.and_then(|c| c.first()) {
            rows.push(
                Row::new(PlayerProfile::career_total_cells(total)).style(Style::default().bold()),
            );
        }
        render_table_with_title(title, header, widths, rows, area, skip, buf);
    }
}

/// Render a titled table, handling scroll clipping of the title row.
fn render_table_with_title<'a>(
    title: &str,
    header: Row<'a>,
    widths: Vec<Constraint>,
    rows: Vec<Row<'a>>,
    area: Rect,
    skip: u16,
    buf: &mut Buffer,
) {
    if skip == 0 {
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
        selected_style(),
    )))
    .render(area, buf);
}
