use crate::app::App;
use crate::gameday::{AtBatPanel, BoxPanel, Gameday, GamedayPanel, InfoPanel};

use tui::backend::Backend;
use tui::layout::Rect;
use tui::terminal::Frame;

impl Gameday {
    pub fn render<B>(&self, f: &mut Frame<B>, rect: Rect, app: &App)
    where
        B: Backend,
    {
        let mut panels = self.generate_layouts(rect);
        // I want the panels to be displayed [Info, Heat, Box] from left to right. So pop off
        // available panels starting with Box. Since `generate_layouts` takes into account how many
        // panels are active, all the pops are guaranteed to unwrap.
        if self.boxscore.active {
            let p = panels.pop().unwrap();
            BoxPanel::draw_border(f, p);
            // self.boxscore.scoreboard.render(f, p);
            self.boxscore.stats.render(f, p, app);
        }
        if self.at_bat.active {
            let p = panels.pop().unwrap();
            AtBatPanel::draw_border(f, p);
            self.at_bat.at_bat.render(f, p);
        }
        if self.info.active {
            let p = panels.pop().unwrap();
            InfoPanel::draw_border(f, p);
            self.info.matchup.render(f, p);
            self.info.plays.render(f, p);
        }
    }
}
