use crate::components::decision_pitchers::{DecisionPitcher, GameDecisionPitchers};
use crate::components::util::{OptionDisplayExt, last_name};
use crate::ui::styling::dim_style;
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Style;
use tui::text::{Line, Span};
use tui::widgets::{Paragraph, Widget};

pub struct DecisionPitchersWidget<'a> {
    pub decisions: &'a GameDecisionPitchers,
}

impl Widget for DecisionPitchersWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let dim = dim_style();
        let mut lines = vec![
            wl_line("W", &self.decisions.winning_pitcher, dim),
            wl_line("L", &self.decisions.losing_pitcher, dim),
        ];
        if let Some(save) = &self.decisions.save_pitcher {
            lines.push(save_line(save, dim));
        }
        Paragraph::new(lines).render(area, buf);
    }
}

fn wl_line<'a>(label: &'a str, p: &'a DecisionPitcher, dim: Style) -> Line<'a> {
    let wins = p.wins.display_or("-");
    let losses = p.losses.display_or("-");
    let era = p.era.display_or("-");
    Line::from(vec![
        Span::styled(format!("{label}: "), dim),
        Span::raw(last_name(&p.name).to_string()),
        Span::styled(format!(" {wins}-{losses}, {era} ERA"), dim),
    ])
}

fn save_line(p: &DecisionPitcher, dim: Style) -> Line<'_> {
    let saves = p.saves.display_or("-");
    let era = p.era.display_or("-");
    Line::from(vec![
        Span::styled("S: ", dim),
        Span::raw(last_name(&p.name).to_string()),
        Span::styled(format!(" {saves}, {era} ERA"), dim),
    ])
}
