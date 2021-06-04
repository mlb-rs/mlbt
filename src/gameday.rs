use crate::at_bat::AtBat;
use crate::boxscore_stats::TeamBatterBoxscore;
use crate::linescore::LineScore;
use crate::matchup::Matchup;
use crate::plays::InningPlays;

use mlb_api::live::LiveResponse;

use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::widgets::{Block, BorderType, Borders};
use tui::Frame;

pub trait GamedayPanel {
    fn from_live_data(&self, live_game: &LiveResponse) -> Self;
    fn toggle_active(&mut self);
    /// Render the panel as a blank block to create the borders
    fn draw_border<B>(f: &mut Frame<B>, rect: Rect)
    where
        B: Backend,
    {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        f.render_widget(block, rect);
    }
}

/// Store the active/inactive state of the views. Used for debugging.
#[derive(Default, Debug, Copy, Clone)]
pub struct GamedayViews {
    info: bool,
    at_bat: bool,
    boxscore: bool,
}

/// Store the panels used to render the Gameday information.
pub struct Gameday {
    // layouts: Vec<Rect>, // TODO store these?
    pub info: InfoPanel,
    pub at_bat: AtBatPanel,
    pub boxscore: BoxPanel,
}

#[derive(Default)]
pub struct InfoPanel {
    pub active: bool,
    pub matchup: Matchup,
    pub plays: InningPlays,
}

#[derive(Default)]
pub struct AtBatPanel {
    pub active: bool,
    pub at_bat: AtBat,
}

#[derive(Default)]
pub struct BoxPanel {
    pub active: bool,
    pub scoreboard: LineScore,
    pub stats: TeamBatterBoxscore,
}

impl GamedayPanel for InfoPanel {
    fn from_live_data(&self, live_game: &LiveResponse) -> Self {
        InfoPanel {
            active: self.active,
            matchup: Matchup::from_live_data(&live_game),
            plays: InningPlays::from_live_data(&live_game),
        }
    }

    fn toggle_active(&mut self) {
        self.active = !self.active;
    }
}

impl GamedayPanel for AtBatPanel {
    fn from_live_data(&self, live_game: &LiveResponse) -> Self {
        AtBatPanel {
            active: self.active,
            at_bat: AtBat::from_live_data(live_game),
        }
    }

    fn toggle_active(&mut self) {
        self.active = !self.active;
    }
}

impl GamedayPanel for BoxPanel {
    fn from_live_data(&self, live_game: &LiveResponse) -> Self {
        TeamBatterBoxscore::from_live_data(live_game);
        let mut bp = BoxPanel {
            active: self.active,
            scoreboard: LineScore::from_live_data(live_game),
            stats: TeamBatterBoxscore::from_live_data(live_game),
        };
        bp.scoreboard.mini = true;
        bp
    }

    fn toggle_active(&mut self) {
        self.active = !self.active;
    }
}

impl Gameday {
    pub fn new() -> Self {
        let mut g = Gameday {
            info: InfoPanel::default(),
            at_bat: AtBatPanel::default(),
            boxscore: BoxPanel::default(),
        };
        g.info.active = true;
        g.at_bat.active = true;
        g.boxscore.active = true;
        g
    }
    pub fn load_live_data(&mut self, live_game: &LiveResponse) {
        self.info = self.info.from_live_data(live_game);
        self.at_bat = self.at_bat.from_live_data(live_game);
        self.boxscore = self.boxscore.from_live_data(live_game);
    }
    pub fn toggle_info(&mut self) {
        self.info.toggle_active();
    }
    pub fn toggle_heat(&mut self) {
        self.at_bat.toggle_active();
    }
    pub fn toggle_box(&mut self) {
        self.boxscore.toggle_active();
    }
    pub fn get_active(&self) -> GamedayViews {
        GamedayViews {
            info: self.info.active,
            at_bat: self.at_bat.active,
            boxscore: self.boxscore.active,
        }
    }
    pub fn generate_layouts(&self, area: Rect) -> Vec<Rect> {
        let mut active = 0;
        if self.info.active {
            active += 1;
        }
        if self.at_bat.active {
            active += 1;
        }
        if self.boxscore.active {
            active += 1;
        }
        let constraints = match active {
            0 | 1 => vec![Constraint::Percentage(100)],
            2 => vec![Constraint::Percentage(50), Constraint::Percentage(50)],
            3 => vec![
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                // Constraint::Percentage(33),
                // Constraint::Percentage(34),
                // Constraint::Percentage(33),
            ],
            _ => vec![],
        };
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints.as_slice())
            .split(area)
    }
}
