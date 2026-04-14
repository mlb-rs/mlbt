use crate::components::standings::Team;
use mlbt_api::teams::ApiTeam;
use std::collections::HashMap;
use std::sync::{LazyLock, OnceLock};

/// This maps the `divisionId` to the `shortName` for each division and league.
/// The team names are taken from the `divisions` endpoint.
pub static DIVISIONS: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        (103, "American League"),
        (104, "National League"),
        (200, "AL West"),
        (201, "AL East"),
        (202, "AL Central"),
        (203, "NL West"),
        (204, "NL East"),
        (205, "NL Central"),
    ])
});

/// This is a map of the order divisions should be shown in the standings view, keyed by the
/// users' favorite team's division.
pub static DIVISION_ORDERS: LazyLock<HashMap<u16, Vec<u16>>> = LazyLock::new(|| {
    HashMap::from([
        (200, vec![200, 201, 202, 203, 204, 205]),
        (201, vec![201, 202, 200, 203, 204, 205]),
        (202, vec![202, 200, 201, 203, 204, 205]),
        (203, vec![203, 204, 205, 200, 201, 202]),
        (204, vec![204, 205, 203, 200, 201, 202]),
        (205, vec![205, 203, 204, 200, 201, 202]),
    ])
});

/// Teams fetched from the API at startup, used as a fallback when a team is not in `TEAM_IDS`.
static DYNAMIC_TEAMS: OnceLock<HashMap<String, Team>> = OnceLock::new();

/// Register teams fetched from the API into the dynamic cache.
/// Uses `Box::leak` to promote `String` → `&'static str` so `Team` can remain `Copy`.
pub fn register_teams(api_teams: Vec<ApiTeam>) {
    let mut map = HashMap::with_capacity(api_teams.len());
    for t in api_teams {
        let team = Team {
            id: t.id,
            division_id: t.division.as_ref().map(|d| d.id).unwrap_or(0),
            name: Box::leak(t.name.clone().into_boxed_str()),
            team_name: Box::leak(t.team_name.clone().into_boxed_str()),
            abbreviation: Box::leak(t.abbreviation.clone().into_boxed_str()),
        };
        map.insert(t.name.clone(), team);
    }
    let _ = DYNAMIC_TEAMS.set(map);
}

/// Look up a team by name. Checks the static `TEAM_IDS` first, then falls back to the
/// dynamic cache populated from the API. Returns a default "unknown" team if not found.
pub fn lookup_team(name: &str) -> Team {
    lookup_team_or(name, Team::default)
}

/// Look up a team by name with a custom fallback when the team is not found in either
/// the static or dynamic caches.
pub fn lookup_team_or(name: &str, fallback: impl FnOnce() -> Team) -> Team {
    if let Some(team) = TEAM_IDS.get(name) {
        return *team;
    }
    if let Some(dynamic) = DYNAMIC_TEAMS.get()
        && let Some(team) = dynamic.get(name)
    {
        return *team;
    }
    fallback()
}

/// Look up a current MLB team by its numeric id.
/// Note: Use `lookup_team` if you have the team name.
pub fn lookup_team_by_id(id: u16) -> Option<Team> {
    CURRENT_TEAMS.get(&id).copied()
}

/// All current MLB teams, sorted by full name. Cached on first call.
pub fn current_teams_sorted() -> &'static [Team] {
    static SORTED: LazyLock<Vec<Team>> = LazyLock::new(|| {
        let mut v: Vec<Team> = CURRENT_TEAMS.values().copied().collect();
        v.sort_by_key(|t| t.name);
        v
    });
    &SORTED
}

/// Current MLB teams keyed by id. Unlike `TEAM_IDS` which maps names (including historical aliases)
/// to teams, this map has exactly one entry per current franchise with a unique id.
#[rustfmt::skip]
static CURRENT_TEAMS: LazyLock<HashMap<u16, Team>> = LazyLock::new(|| {
    HashMap::from([
        (108, Team { id: 108, division_id: 200, name: "Los Angeles Angels", team_name: "Angels", abbreviation: "LAA" }),
        (109, Team { id: 109, division_id: 203, name: "Arizona Diamondbacks", team_name: "D-backs", abbreviation: "AZ" }),
        (110, Team { id: 110, division_id: 201, name: "Baltimore Orioles", team_name: "Orioles", abbreviation: "BAL" }),
        (111, Team { id: 111, division_id: 201, name: "Boston Red Sox", team_name: "Red Sox", abbreviation: "BOS" }),
        (112, Team { id: 112, division_id: 205, name: "Chicago Cubs", team_name: "Cubs", abbreviation: "CHC" }),
        (113, Team { id: 113, division_id: 205, name: "Cincinnati Reds", team_name: "Reds", abbreviation: "CIN" }),
        (114, Team { id: 114, division_id: 202, name: "Cleveland Guardians", team_name: "Guardians", abbreviation: "CLE" }),
        (115, Team { id: 115, division_id: 203, name: "Colorado Rockies", team_name: "Rockies", abbreviation: "COL" }),
        (116, Team { id: 116, division_id: 202, name: "Detroit Tigers", team_name: "Tigers", abbreviation: "DET" }),
        (117, Team { id: 117, division_id: 200, name: "Houston Astros", team_name: "Astros", abbreviation: "HOU" }),
        (118, Team { id: 118, division_id: 202, name: "Kansas City Royals", team_name: "Royals", abbreviation: "KC" }),
        (119, Team { id: 119, division_id: 203, name: "Los Angeles Dodgers", team_name: "Dodgers", abbreviation: "LAD" }),
        (120, Team { id: 120, division_id: 204, name: "Washington Nationals", team_name: "Nationals", abbreviation: "WSH" }),
        (121, Team { id: 121, division_id: 204, name: "New York Mets", team_name: "Mets", abbreviation: "NYM" }),
        (133, Team { id: 133, division_id: 200, name: "Athletics", team_name: "Athletics", abbreviation: "ATH" }),
        (134, Team { id: 134, division_id: 205, name: "Pittsburgh Pirates", team_name: "Pirates", abbreviation: "PIT" }),
        (135, Team { id: 135, division_id: 203, name: "San Diego Padres", team_name: "Padres", abbreviation: "SD" }),
        (136, Team { id: 136, division_id: 200, name: "Seattle Mariners", team_name: "Mariners", abbreviation: "SEA" }),
        (137, Team { id: 137, division_id: 203, name: "San Francisco Giants", team_name: "Giants", abbreviation: "SF" }),
        (138, Team { id: 138, division_id: 205, name: "St. Louis Cardinals", team_name: "Cardinals", abbreviation: "STL" }),
        (139, Team { id: 139, division_id: 201, name: "Tampa Bay Rays", team_name: "Rays", abbreviation: "TB" }),
        (140, Team { id: 140, division_id: 200, name: "Texas Rangers", team_name: "Rangers", abbreviation: "TEX" }),
        (141, Team { id: 141, division_id: 201, name: "Toronto Blue Jays", team_name: "Blue Jays", abbreviation: "TOR" }),
        (142, Team { id: 142, division_id: 202, name: "Minnesota Twins", team_name: "Twins", abbreviation: "MIN" }),
        (143, Team { id: 143, division_id: 204, name: "Philadelphia Phillies", team_name: "Phillies", abbreviation: "PHI" }),
        (144, Team { id: 144, division_id: 204, name: "Atlanta Braves", team_name: "Braves", abbreviation: "ATL" }),
        (145, Team { id: 145, division_id: 202, name: "Chicago White Sox", team_name: "White Sox", abbreviation: "CWS" }),
        (146, Team { id: 146, division_id: 204, name: "Miami Marlins", team_name: "Marlins", abbreviation: "MIA" }),
        (147, Team { id: 147, division_id: 201, name: "New York Yankees", team_name: "Yankees", abbreviation: "NYY" }),
        (158, Team { id: 158, division_id: 205, name: "Milwaukee Brewers", team_name: "Brewers", abbreviation: "MIL" }),
    ])
});

/// Maps team full names to `Team` structs. Current teams are sourced from `CURRENT_TEAMS`,
/// then historical and alternate names are added as aliases.
#[rustfmt::skip]
pub static TEAM_IDS: LazyLock<HashMap<&'static str, Team>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    // insert current teams keyed by their full name.
    for team in CURRENT_TEAMS.values() {
        m.insert(team.name, *team);
    }
    // all-Star teams
    m.insert("American League All-Stars", Team { id: 159, division_id: 103, name: "American League All-Stars", team_name: "AL All-Stars", abbreviation: "AL" });
    m.insert("National League All-Stars", Team { id: 160, division_id: 104, name: "National League All-Stars", team_name: "NL All-Stars", abbreviation: "NL" });
    // historical and alternate team names
    m.insert("Oakland Athletics", *CURRENT_TEAMS.get(&133).unwrap());
    m.insert("Tampa Bay Devil Rays", Team { id: 139, division_id: 201, name: "Tampa Bay Devil Rays", team_name: "Devil Rays", abbreviation: "TB" });
    m.insert("Florida Marlins", Team { id: 146, division_id: 204, name: "Miami Marlins", team_name: "Marlins", abbreviation: "MIA" });
    m.insert("Anaheim Angels", Team { id: 108, division_id: 200, name: "Anaheim Angels", team_name: "Angels", abbreviation: "ANA" });
    m.insert("California Angels", Team { id: 108, division_id: 200, name: "California Angels", team_name: "Angels", abbreviation: "CAL" });
    m.insert("Cleveland Indians", Team { id: 114, division_id: 202, name: "Cleveland Guardians", team_name: "Guardians", abbreviation: "CLE" });
    m.insert("Montreal Expos", Team { id: 120, division_id: 204, name: "Montreal Expos", team_name: "Expos", abbreviation: "MON" });
    // pre 1969 teams didn't have divisions so just setting it to `0`
    m.insert("Houston Colt 45's", Team { id: 117, division_id: 0, name: "Houston Colt 45's", team_name: "Colt 45's", abbreviation: "HOU" });
    m.insert("Kansas City Athletics", Team { id: 133, division_id: 0, name: "Kansas City Athletics", team_name: "Athletics", abbreviation: "KCA" });
    m.insert("Washington Senators", Team { id: 140, division_id: 0, name: "Washington Senators", team_name: "Senators", abbreviation: "WAS" });
    m.insert("Milwaukee Braves", Team { id: 144, division_id: 0, name: "Milwaukee Braves", team_name: "Braves", abbreviation: "MIL" });
    m.insert("Cincinnati Redlegs", Team { id: 113, division_id: 0, name: "Cincinnati Redlegs", team_name: "Redlegs", abbreviation: "CIN" });
    m.insert("Philadelphia Athletics", Team { id: 133, division_id: 0, name: "Philadelphia Athletics", team_name: "Athletics", abbreviation: "PHA" });
    m.insert("Seattle Pilots", Team { id: 158, division_id: 200, name: "Seattle Pilots", team_name: "Pilots", abbreviation: "SEA" });
    m.insert("Brooklyn Dodgers", Team { id: 119, division_id: 0, name: "Brooklyn Dodgers", team_name: "Dodgers", abbreviation: "BRO" });
    m.insert("New York Giants", Team { id: 137, division_id: 0, name: "New York Giants", team_name: "Giants", abbreviation: "NYG" });
    m.insert("Boston Braves", Team { id: 144, division_id: 0, name: "Boston Braves", team_name: "Braves", abbreviation: "BSN" });
    m.insert("St. Louis Browns", Team { id: 110, division_id: 0, name: "St. Louis Browns", team_name: "Browns", abbreviation: "SLB" });
    m
});
