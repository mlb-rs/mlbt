use crate::app::MenuItem;
use crate::draw;
use crate::state::app_state::HomeOrAway;
use crate::state::gameday::GamedayState;
use crate::ui::boxscore::TeamBatterBoxscoreWidget;
use crate::ui::gameday::at_bat::AtBatWidget;
use crate::ui::gameday::matchup::MatchupWidget;
use crate::ui::gameday::plays::InningPlaysWidget;
use crate::ui::gameday::win_probability::WinProbabilityWidget;
use crate::ui::layout::LayoutAreas;
use crate::ui::linescore::LineScoreWidget;
use tui::prelude::{Buffer, Color, Rect, Widget};

pub struct GamedayWidget<'a> {
    pub state: &'a GamedayState,
    pub active: HomeOrAway,
}

impl Widget for GamedayWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut panels = LayoutAreas::generate_gameday_panels(&self.state.panels, area);

        // I want the panels to be displayed [Info, Heat, Box] from left to right. So pop off
        // available panels starting with Box. Since `generate_layouts` takes into account how many
        // panels are active, all the pops are guaranteed to unwrap.
        if self.state.panels.boxscore {
            let p = panels.pop().unwrap();
            Self::draw_border(p, buf);
            let chunks = LayoutAreas::for_boxscore(p);

            let linescore_widget = LineScoreWidget {
                active: self.active,
                linescore: &self.state.game.linescore,
            };
            Widget::render(linescore_widget, chunks[0], buf);

            let boxscore_widget = TeamBatterBoxscoreWidget {
                active: self.active,
                boxscore: &self.state.game.boxscore,
            };
            Widget::render(boxscore_widget, chunks[1], buf);
        }
        if self.state.panels.at_bat {
            let p = panels.pop().unwrap();
            Self::draw_border(p, buf);
            let chunks = LayoutAreas::for_at_bat(p);

            let matchup_widget = MatchupWidget {
                game: &self.state.game,
                selected_at_bat: self.state.selected_at_bat(),
            };
            Widget::render(matchup_widget, chunks[0], buf);

            let at_bat_widget = AtBatWidget {
                game: &self.state.game,
                selected_at_bat: self.state.selected_at_bat(),
            };
            Widget::render(at_bat_widget, chunks[1], buf);
        }
        if self.state.panels.info {
            let p = panels.pop().unwrap();
            Self::draw_border(p, buf);
            let chunks = LayoutAreas::for_info(p, self.state.panels.win_probability);

            let innings_widget = InningPlaysWidget {
                game: &self.state.game,
                selected_at_bat: self.state.selected_at_bat(),
            };
            Widget::render(innings_widget, chunks[0], buf);

            if self.state.panels.win_probability {
                let wps_widget = WinProbabilityWidget {
                    game: &self.state.game,
                    selected_at_bat: self.state.selected_at_bat(),
                    active_tab: MenuItem::Gameday,
                };
                Widget::render(wps_widget, chunks[1], buf);
            }
        }
    }
}

impl GamedayWidget<'_> {
    fn draw_border(area: Rect, buf: &mut Buffer) {
        let block = draw::default_border(Color::White);
        Widget::render(block, area, buf);
    }
}
