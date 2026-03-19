use crate::components::player_profile::PlayerProfile;
use crate::state::player_profile::PlayerProfileState;
use mlbt_api::season::GameType;
use mlbt_api::stats::Split;
use tui::prelude::*;
use tui::widgets::{Block, BorderType, Borders, Padding, Paragraph, Row, Table};

pub struct PlayerProfileWidget<'a> {
    pub state: &'a mut PlayerProfileState,
}

impl<'a> Widget for PlayerProfileWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
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

        let [bio_area, season_area, career_area, gamelog_area] = Layout::vertical([
            Constraint::Length(bio_height + 1), // +1 for blank line between each section
            Constraint::Length(season_height + 1),
            Constraint::Length(career_height + 1),
            Constraint::Fill(1), // game log gets remaining space
        ])
        .areas(inner);

        self.render_bio(bio_area, buf);

        let season_title = format!("{} Season", self.state.season_year);
        let season_splits = profile
            .find_stat_group("season")
            .map(|s| s.splits.as_slice())
            .unwrap_or(&[]);
        render_section(&season_title, season_splits, false, season_area, buf);

        if let Some(yby) = profile.find_stat_group("yearByYear") {
            let career_totals = profile.find_stat_group("career").map(|c| &c.splits);
            render_section_with_totals(
                "Career Stats",
                &yby.splits,
                career_totals,
                career_area,
                buf,
            );
        }

        if let Some(game_log) = profile.find_stat_group("gameLog") {
            render_stat_section(
                "Recent Games",
                &game_log.splits,
                gamelog_area,
                buf,
                PlayerProfile::build_game_log_rows,
            );
        }
    }
}

impl PlayerProfileWidget<'_> {
    fn render_bio(&self, area: Rect, buf: &mut Buffer) {
        let [bio_text_area, game_type_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(20)]).areas(area);

        self.render_game_type_selector(game_type_area, buf);
        Paragraph::new(self.state.profile.bio_lines()).render(bio_text_area, buf);
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
}

fn render_section(title: &str, splits: &[Split], show_year: bool, area: Rect, buf: &mut Buffer) {
    if area.height < 2 {
        return;
    }
    if splits.is_empty() {
        let [title_area, msg_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);
        render_section_title(title, title_area, buf);
        Paragraph::new(Span::styled(
            "  No data",
            Style::default().fg(Color::DarkGray),
        ))
        .render(msg_area, buf);
        return;
    }
    render_stat_section(title, splits, area, buf, |splits| {
        PlayerProfile::build_stat_rows(splits, show_year)
    });
}

fn render_section_with_totals(
    title: &str,
    splits: &[Split],
    career: Option<&Vec<Split>>,
    area: Rect,
    buf: &mut Buffer,
) {
    if splits.is_empty() || area.height < 2 {
        return;
    }
    render_stat_section(title, splits, area, buf, |splits| {
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

fn render_stat_section<F>(title: &str, splits: &[Split], area: Rect, buf: &mut Buffer, build: F)
where
    F: FnOnce(&[Split]) -> (Row<'static>, Vec<Constraint>, Vec<Row<'static>>),
{
    let [title_area, table_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

    render_section_title(title, title_area, buf);

    let (header, widths, rows) = build(splits);
    let table = Table::new(rows, widths).header(header).column_spacing(0);
    Widget::render(table, table_area, buf);
}

fn render_section_title(title: &str, area: Rect, buf: &mut Buffer) {
    Paragraph::new(Line::from(Span::styled(
        format!(" {title} "),
        Style::default().bg(Color::Blue).fg(Color::Black),
    )))
    .render(area, buf);
}
