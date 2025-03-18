use once_cell::sync::Lazy;
use std::collections::HashMap;

/// This maps the `teamId` to the `shortName` for each division and league.
/// The team names are taken from the `divisions` endpoint.
pub static DIVISIONS: Lazy<HashMap<u8, &'static str>> = Lazy::new(|| {
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

/// This maps the full name of a team to its short name. The short name is used in the boxscore.
/// The team names are taken from the `teams` endpoint.
pub static TEAM_NAMES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("Oakland Athletics", "Athletics"),
        ("Athletics", "Athletics"),
        ("Pittsburgh Pirates", "Pirates"),
        ("San Diego Padres", "Padres"),
        ("Seattle Mariners", "Mariners"),
        ("San Francisco Giants", "Giants"),
        ("St. Louis Cardinals", "Cardinals"),
        ("Tampa Bay Rays", "Rays"),
        ("Texas Rangers", "Rangers"),
        ("Toronto Blue Jays", "Blue Jays"),
        ("Minnesota Twins", "Twins"),
        ("Philadelphia Phillies", "Phillies"),
        ("Atlanta Braves", "Braves"),
        ("Chicago White Sox", "White Sox"),
        ("Miami Marlins", "Marlins"),
        ("Florida Marlins", "Marlins"),
        ("New York Yankees", "Yankees"),
        ("Milwaukee Brewers", "Brewers"),
        ("Los Angeles Angels", "Angels"),
        ("Arizona Diamondbacks", "D-backs"),
        ("Baltimore Orioles", "Orioles"),
        ("Boston Red Sox", "Red Sox"),
        ("Chicago Cubs", "Cubs"),
        ("Cincinnati Reds", "Reds"),
        ("Cleveland Indians", "Indians"),
        ("Cleveland Guardians", "Guardians"),
        ("Colorado Rockies", "Rockies"),
        ("Detroit Tigers", "Tigers"),
        ("Houston Astros", "Astros"),
        ("Kansas City Royals", "Royals"),
        ("Los Angeles Dodgers", "Dodgers"),
        ("Washington Nationals", "Nationals"),
        ("New York Mets", "Mets"),
        ("American League All-Stars", "AL All-Stars"),
        ("National League All-Stars", "NL All-Stars"),
    ])
});
