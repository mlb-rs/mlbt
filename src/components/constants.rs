use crate::components::standings::Team;
use std::collections::HashMap;
use std::sync::LazyLock;

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

/// This maps the full name of a team to its full `Team` struct.
/// The data is from the `teams` endpoint.
// TODO generate from API?
#[rustfmt::skip]
pub static TEAM_IDS: LazyLock<HashMap<&'static str, Team>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("Oakland Athletics", Team { id: 133, division_id: 200, name: "Athletics", team_name: "Athletics", abbreviation: "ATH" });
    m.insert("Athletics", Team { id: 133, division_id: 200, name: "Athletics", team_name: "Athletics", abbreviation: "ATH" });
    m.insert("Pittsburgh Pirates", Team { id: 134, division_id: 205, name: "Pittsburgh Pirates", team_name: "Pirates", abbreviation: "PIT" });
    m.insert("San Diego Padres", Team { id: 135, division_id: 203, name: "San Diego Padres", team_name: "Padres", abbreviation: "SD" });
    m.insert("Seattle Mariners", Team { id: 136, division_id: 200, name: "Seattle Mariners", team_name: "Mariners", abbreviation: "SEA" });
    m.insert("San Francisco Giants", Team { id: 137, division_id: 203, name: "San Francisco Giants", team_name: "Giants", abbreviation: "SF" });
    m.insert("St. Louis Cardinals", Team { id: 138, division_id: 205, name: "St. Louis Cardinals", team_name: "Cardinals", abbreviation: "STL" });
    m.insert("Tampa Bay Rays", Team { id: 139, division_id: 201, name: "Tampa Bay Rays", team_name: "Rays", abbreviation: "TB" });
    m.insert("Tampa Bay Devil Rays", Team { id: 139, division_id: 201, name: "Tampa Bay Devil Rays", team_name: "Devil Rays", abbreviation: "TB" });
    m.insert("Texas Rangers", Team { id: 140, division_id: 200, name: "Texas Rangers", team_name: "Rangers", abbreviation: "TEX" });
    m.insert("Toronto Blue Jays", Team { id: 141, division_id: 201, name: "Toronto Blue Jays", team_name: "Blue Jays", abbreviation: "TOR" });
    m.insert("Minnesota Twins", Team { id: 142, division_id: 202, name: "Minnesota Twins", team_name: "Twins", abbreviation: "MIN" });
    m.insert("Philadelphia Phillies", Team { id: 143, division_id: 204, name: "Philadelphia Phillies", team_name: "Phillies", abbreviation: "PHI" });
    m.insert("Atlanta Braves", Team { id: 144, division_id: 204, name: "Atlanta Braves", team_name: "Braves", abbreviation: "ATL" });
    m.insert("Chicago White Sox", Team { id: 145, division_id: 202, name: "Chicago White Sox", team_name: "White Sox", abbreviation: "CWS" });
    m.insert("Florida Marlins", Team { id: 146, division_id: 204, name: "Miami Marlins", team_name: "Marlins", abbreviation: "MIA" });
    m.insert("Miami Marlins", Team { id: 146, division_id: 204, name: "Miami Marlins", team_name: "Marlins", abbreviation: "MIA" });
    m.insert("New York Yankees", Team { id: 147, division_id: 201, name: "New York Yankees", team_name: "Yankees", abbreviation: "NYY" });
    m.insert("Milwaukee Brewers", Team { id: 158, division_id: 205, name: "Milwaukee Brewers", team_name: "Brewers", abbreviation: "MIL" });
    m.insert("Seattle Pilots", Team { id: 158, division_id: 200, name: "Seattle Pilots", team_name: "Pilots", abbreviation: "SEA" });
    m.insert("Los Angeles Angels", Team { id: 108, division_id: 200, name: "Los Angeles Angels", team_name: "Angels", abbreviation: "LAA" });
    m.insert("Anaheim Angels", Team { id: 108, division_id: 200, name: "Anaheim Angels", team_name: "Angels", abbreviation: "ANA" });
    m.insert("California Angels", Team { id: 108, division_id: 200, name: "California Angels", team_name: "Angels", abbreviation: "CAL" });
    m.insert("Arizona Diamondbacks", Team { id: 109, division_id: 203, name: "Arizona Diamondbacks", team_name: "D-backs", abbreviation: "AZ" });
    m.insert("Baltimore Orioles", Team { id: 110, division_id: 201, name: "Baltimore Orioles", team_name: "Orioles", abbreviation: "BAL" });
    m.insert("Boston Red Sox", Team { id: 111, division_id: 201, name: "Boston Red Sox", team_name: "Red Sox", abbreviation: "BOS" });
    m.insert("Chicago Cubs", Team { id: 112, division_id: 205, name: "Chicago Cubs", team_name: "Cubs", abbreviation: "CHC" });
    m.insert("Cincinnati Reds", Team { id: 113, division_id: 205, name: "Cincinnati Reds", team_name: "Reds", abbreviation: "CIN" });
    m.insert("Cleveland Indians", Team { id: 114, division_id: 202, name: "Cleveland Guardians", team_name: "Guardians", abbreviation: "CLE" });
    m.insert("Cleveland Guardians", Team { id: 114, division_id: 202, name: "Cleveland Guardians", team_name: "Guardians", abbreviation: "CLE" });
    m.insert("Colorado Rockies", Team { id: 115, division_id: 203, name: "Colorado Rockies", team_name: "Rockies", abbreviation: "COL" });
    m.insert("Detroit Tigers", Team { id: 116, division_id: 202, name: "Detroit Tigers", team_name: "Tigers", abbreviation: "DET" });
    m.insert("Houston Astros", Team { id: 117, division_id: 200, name: "Houston Astros", team_name: "Astros", abbreviation: "HOU" });
    m.insert("Kansas City Royals", Team { id: 118, division_id: 202, name: "Kansas City Royals", team_name: "Royals", abbreviation: "KC" });
    m.insert("Los Angeles Dodgers", Team { id: 119, division_id: 203, name: "Los Angeles Dodgers", team_name: "Dodgers", abbreviation: "LAD" });
    m.insert("Washington Nationals", Team { id: 120, division_id: 204, name: "Washington Nationals", team_name: "Nationals", abbreviation: "WSH" });
    m.insert("Montreal Expos", Team { id: 120, division_id: 204, name: "Montreal Expos", team_name: "Expos", abbreviation: "MON" });
    m.insert("New York Mets", Team { id: 121, division_id: 204, name: "New York Mets", team_name: "Mets", abbreviation: "NYM" });
    m.insert("American League All-Stars", Team { id: 159, division_id: 103, name: "American League All-Stars", team_name: "AL All-Stars", abbreviation: "AL" });
    m.insert("National League All-Stars", Team { id: 160, division_id: 104, name: "National League All-Stars", team_name: "NL All-Stars", abbreviation: "NL" });
    // pre 1969 teams didn't have divisions so just setting it to `0`
    m.insert("Houston Colt 45's", Team { id: 117, division_id: 0, name: "Houston Colt 45's", team_name: "Colt 45's", abbreviation: "HOU" });
    m.insert("Kansas City Athletics", Team { id: 133, division_id: 0, name: "Kansas City Athletics", team_name: "Athletics", abbreviation: "KCA" });
    m.insert("Washington Senators", Team { id: 140, division_id: 0, name: "Washington Senators", team_name: "Senators", abbreviation: "WAS" });
    m.insert("Milwaukee Braves", Team { id: 144, division_id: 0, name: "Milwaukee Braves", team_name: "Braves", abbreviation: "MIL" });
    m.insert("Cincinnati Redlegs", Team { id: 113, division_id: 0, name: "Cincinnati Redlegs", team_name: "Redlegs", abbreviation: "CIN" });
    m.insert("Philadelphia Athletics", Team { id: 133, division_id: 0, name: "Philadelphia Athletics", team_name: "Athletics", abbreviation: "PHA" });
    m.insert("Brooklyn Dodgers", Team { id: 119, division_id: 0, name: "Brooklyn Dodgers", team_name: "Dodgers", abbreviation: "BRO" });
    m.insert("New York Giants", Team { id: 137, division_id: 0, name: "New York Giants", team_name: "Giants", abbreviation: "NYG" });
    m.insert("Boston Braves", Team { id: 144, division_id: 0, name: "Boston Braves", team_name: "Braves", abbreviation: "BSN" });
    m.insert("St. Louis Browns", Team { id: 110, division_id: 0, name: "St. Louis Browns", team_name: "Browns", abbreviation: "SLB" });
    m
});
