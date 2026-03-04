use serde::Deserialize;
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum SportId {
    /// Major League Baseball
    Mlb = 1,
    /// International Baseball (World Baseball Classic, etc.)
    International = 51,
}

impl fmt::Display for SportId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", *self as u16)
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct TeamsResponse {
    pub teams: Vec<ApiTeam>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiTeam {
    pub id: u16,
    pub name: String,
    pub abbreviation: String,
    pub team_name: String,
    pub division: Option<IdLink>,
    pub sport: Option<IdLink>,
}

#[derive(Debug, Deserialize)]
pub struct IdLink {
    pub id: u16,
}
