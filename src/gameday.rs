use crate::boxscore::BoxScore;
use crate::heatmap::Heatmap;
use crate::matchup::Matchup;
use crate::pitches::Pitches;
use mlb_api::live::LiveResponse;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::widgets::{Block, BorderType, Borders};
use tui::Frame;

trait GamedayPanel {
    fn from_live_data(&self, live_game: &LiveResponse) -> Self;
    fn toggle_active(&mut self);
    /// Render the panel as a blank block to create the borders
    fn render_panel<B>(&self, f: &mut Frame<B>, rect: Rect)
    where
        B: Backend,
    {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        f.render_widget(block, rect);
    }
}

#[derive(Default)]
pub struct InfoPanel {
    active: bool,
    matchup: Matchup,
    // pub plays: TODO
}

#[derive(Default)]
pub struct HeatMapPanel {
    active: bool,
    heatmap: Heatmap,
    pitches: Pitches,
}

#[derive(Default)]
pub struct BoxPanel {
    active: bool,
    scoreboard: BoxScore,
    // pub stats: TODO
}

impl GamedayPanel for InfoPanel {
    fn from_live_data(&self, live_game: &LiveResponse) -> Self {
        InfoPanel {
            active: self.active,
            matchup: Matchup::from_live_data(live_game),
        }
    }

    fn toggle_active(&mut self) {
        self.active = !self.active;
    }
}

impl GamedayPanel for HeatMapPanel {
    fn from_live_data(&self, live_game: &LiveResponse) -> Self {
        HeatMapPanel {
            active: self.active,
            heatmap: Heatmap::from_live_data(live_game),
            pitches: Pitches::from_live_data(live_game),
        }
    }

    fn toggle_active(&mut self) {
        self.active = !self.active;
    }
}

impl GamedayPanel for BoxPanel {
    fn from_live_data(&self, live_game: &LiveResponse) -> Self {
        let mut bp = BoxPanel {
            active: self.active,
            scoreboard: BoxScore::from_live_data(live_game),
        };
        bp.scoreboard.mini = true;
        bp
    }

    fn toggle_active(&mut self) {
        self.active = !self.active;
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct GamedayViews {
    info: bool,
    heat: bool,
    boxscore: bool,
}

pub struct Gameday {
    // layouts: Vec<Rect>, // TODO store these?
    pub info: InfoPanel,
    pub heat: HeatMapPanel,
    pub boxscore: BoxPanel,
}

impl Gameday {
    pub fn new() -> Self {
        let mut g = Gameday {
            info: InfoPanel::default(),
            heat: HeatMapPanel::default(),
            boxscore: BoxPanel::default(),
        };
        g.info.active = true;
        g.heat.active = true;
        g.boxscore.active = true;
        g
    }
    pub fn load_live_data(&mut self, live_game: &LiveResponse) {
        self.info = self.info.from_live_data(live_game);
        self.heat = self.heat.from_live_data(live_game);
        self.boxscore = self.boxscore.from_live_data(live_game);
    }
    pub fn toggle_info(&mut self) {
        self.info.toggle_active();
    }
    pub fn toggle_heat(&mut self) {
        self.heat.toggle_active();
    }
    pub fn toggle_box(&mut self) {
        self.boxscore.toggle_active();
    }
    pub fn get_active(&self) -> GamedayViews {
        GamedayViews {
            info: self.info.active,
            heat: self.heat.active,
            boxscore: self.boxscore.active,
        }
    }
    fn generate_layouts(&self, area: Rect) -> Vec<Rect> {
        let mut active = 0;
        if self.info.active {
            active += 1;
        }
        if self.heat.active {
            active += 1;
        }
        if self.boxscore.active {
            active += 1;
        }
        let constraints = match active {
            0 | 1 => vec![Constraint::Percentage(100)],
            2 => vec![Constraint::Percentage(50), Constraint::Percentage(50)],
            3 => vec![
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ],
            _ => vec![],
        };
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints.as_slice())
            .split(area)
    }
    //temp rendering
    pub fn render<B>(&self, f: &mut Frame<B>, rect: Rect)
    where
        B: Backend,
    {
        let mut panels = self.generate_layouts(rect);
        // I want the panels to be displayed [Info, Heat, Box] from left to right. So pop off
        // available panels starting with Box. Since `generate_layouts` takes into account how many
        // panels are active, all the pops are guaranteed to unwrap.
        if self.boxscore.active {
            // split vertically
            let p = panels.pop().unwrap();
            self.boxscore.render_panel(f, p);
            self.boxscore.scoreboard.render(f, p);
        }
        if self.heat.active {
            // split vertically
            let p = panels.pop().unwrap();
            self.heat.render_panel(f, p);
            self.heat.heatmap.render(f, p);
            self.heat.pitches.render(f, p);
        }
        if self.info.active {
            let p = panels.pop().unwrap();
            self.info.render_panel(f, p);
            self.info.matchup.render(f, p);
        }
    }
}
