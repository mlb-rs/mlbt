use crate::state::gameday::GamedayState;

pub struct GamedayWidget {
    state: GamedayState,
}

// impl Widget for GamedayWidget {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         // let mut panels = LayoutAreas::generate_gameday_panels(&self.state.panels, area);
//
//         // I want the panels to be displayed [Info, Heat, Box] from left to right. So pop off
//         // available panels starting with Box. Since `generate_layouts` takes into account how many
//         // panels are active, all the pops are guaranteed to unwrap.
//         if self.state.panels.boxscore {
//             // let p = panels.pop().unwrap();
//             // crate::draw::draw_border(f, p, Color::White);
//             // crate::draw::draw_linescore_boxscore(f, p, app);
//         }
//         if self.state.panels.at_bat {
//             // let p = panels.pop().unwrap();
//             // crate::draw::draw_border(f, p, Color::White);
//             // AtBatWidget::render(p, buf, &mut state.game.at_bat)
//             // StatefulWidget::render(t, area, buf, &mut state.state);
//         }
//         if self.state.panels.info {
//             // let p = panels.pop().unwrap();
//             // crate::draw::draw_border(f, p, Color::White);
//             // f.render_stateful_widget(MatchupWidget {}, p, &mut state.game.matchup);
//             // f.render_stateful_widget(InningPlaysWidget {}, p, &mut state.game.plays);
//         }
//
//         Widget::render(self, area, buf);
//     }
// }
//
// // fn draw_gameday(f: &mut Frame, rect: Rect, app: &mut App) {
// //     let mut panels = LayoutAreas::generate_gameday_panels(&app.state.gameday, rect);
// //
// //     // I want the panels to be displayed [Info, Heat, Box] from left to right. So pop off
// //     // available panels starting with Box. Since `generate_layouts` takes into account how many
// //     // panels are active, all the pops are guaranteed to unwrap.
// //     if app.state.gameday.boxscore {
// //         let p = panels.pop().unwrap();
// //         crate::draw::draw_border(f, p, Color::White);
// //         crate::draw::draw_linescore_boxscore(f, p, app);
// //     }
// //     if app.state.gameday.at_bat {
// //         let p = panels.pop().unwrap();
// //         crate::draw::draw_border(f, p, Color::White);
// //         f.render_stateful_widget(AtBatWidget {}, p, &mut app.state.live_game.at_bat);
// //     }
// //     if app.state.gameday.info {
// //         let p = panels.pop().unwrap();
// //         crate::draw::draw_border(f, p, Color::White);
// //         f.render_stateful_widget(MatchupWidget {}, p, &mut app.state.live_game.matchup);
// //         f.render_stateful_widget(InningPlaysWidget {}, p, &mut app.state.live_game.plays);
// //     }
// // }
