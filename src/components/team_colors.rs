use std::collections::HashMap;
use std::sync::LazyLock;
use tui::style::Color;

/// Terminal-friendly colors brightened for readability on dark backgrounds.
pub static TEAM_COLORS: LazyLock<HashMap<&'static str, Color>> = LazyLock::new(|| {
    HashMap::from([
        // AL West
        ("ATH", Color::Rgb(0,   135,  43)),   // Athletics: green
        ("SEA", Color::Rgb(0,   132, 132)),   // Mariners: teal
        ("LAA", Color::Rgb(186,   0, 33)),    // Angels: red
        ("HOU", Color::Rgb(235, 110, 31)),    // Astros: orange
        ("TEX", Color::Rgb(192,  17, 31)),    // Rangers: red
        // AL Central
        ("CWS", Color::Rgb(196, 206, 211)),   // White Sox: silver
        ("DET", Color::Rgb(250,  70, 22)),    // Tigers: orange
        ("KC",  Color::Rgb(65,  137, 210)),   // Royals: blue
        ("MIN", Color::Rgb(211,  17, 69)),    // Twins: red
        ("CLE", Color::Rgb(227,  25, 55)),    // Guardians: red
        // AL East
        ("NYY", Color::Rgb(175, 175, 175)),   // Yankees: silver
        ("BOS", Color::Rgb(189,  48, 57)),    // Red Sox: red
        ("TOR", Color::Rgb(55,  115, 185)),   // Blue Jays: blue
        ("TB",  Color::Rgb(143, 188, 230)),   // Rays: light blue
        ("BAL", Color::Rgb(223,  70,  1)),    // Orioles: orange
        // NL West
        ("LAD", Color::Rgb(57,  131, 206)),   // Dodgers: blue
        ("SF",  Color::Rgb(253,  90, 30)),    // Giants: orange
        ("AZ",  Color::Rgb(167,  25, 48)),    // Diamondbacks: red
        ("SD",  Color::Rgb(181, 155, 114)),   // Padres: sand/tan
        ("COL", Color::Rgb(131,  91, 157)),   // Rockies: purple
        // NL Central
        ("CHC", Color::Rgb(47,   93, 180)),   // Cubs: blue
        ("STL", Color::Rgb(196,  30, 58)),    // Cardinals: red
        ("MIL", Color::Rgb(255, 197, 47)),    // Brewers: gold
        ("CIN", Color::Rgb(198,   1, 31)),    // Reds: red
        ("PIT", Color::Rgb(253, 184, 39)),    // Pirates: gold
        // NL East
        ("NYM", Color::Rgb(40,   87, 165)),   // Mets: blue
        ("ATL", Color::Rgb(206,  17, 65)),    // Braves: red
        ("PHI", Color::Rgb(232,  24, 40)),    // Phillies: red
        ("MIA", Color::Rgb(0,   163, 224)),   // Marlins: teal/blue
        ("WSH", Color::Rgb(171,   0,  3)),    // Nationals: red
    ])
});

/// Official team primary colors — accurate but some dark blues are hard to
/// read on dark terminal backgrounds.
pub static TEAM_COLORS_OFFICIAL: LazyLock<HashMap<&'static str, Color>> = LazyLock::new(|| {
    HashMap::from([
        // AL West
        ("ATH", Color::Rgb(0,   100,  0)),    // Athletics: green
        ("SEA", Color::Rgb(0,    92, 92)),    // Mariners: teal
        ("LAA", Color::Rgb(186,   0, 33)),    // Angels: red
        ("HOU", Color::Rgb(235, 110, 31)),    // Astros: orange
        ("TEX", Color::Rgb(192,  17, 31)),    // Rangers: red
        // AL Central
        ("CWS", Color::Rgb(196, 206, 211)),   // White Sox: silver
        ("DET", Color::Rgb(250,  70, 22)),    // Tigers: orange
        ("KC",  Color::Rgb(0,    70, 135)),   // Royals: blue
        ("MIN", Color::Rgb(211,  17, 69)),    // Twins: red
        ("CLE", Color::Rgb(227,  25, 55)),    // Guardians: red
        // AL East
        ("NYY", Color::Rgb(175, 175, 175)),   // Yankees: silver
        ("BOS", Color::Rgb(189,  48, 57)),    // Red Sox: red
        ("TOR", Color::Rgb(19,   74, 142)),   // Blue Jays: blue
        ("TB",  Color::Rgb(143, 188, 230)),   // Rays: light blue
        ("BAL", Color::Rgb(223,  70,  1)),    // Orioles: orange
        // NL West
        ("LAD", Color::Rgb(0,    90, 156)),   // Dodgers: blue
        ("SF",  Color::Rgb(253,  90, 30)),    // Giants: orange
        ("AZ",  Color::Rgb(167,  25, 48)),    // Diamondbacks: red
        ("SD",  Color::Rgb(181, 155, 114)),   // Padres: sand/tan
        ("COL", Color::Rgb(131,  91, 157)),   // Rockies: purple
        // NL Central
        ("CHC", Color::Rgb(14,   51, 134)),   // Cubs: blue
        ("STL", Color::Rgb(196,  30, 58)),    // Cardinals: red
        ("MIL", Color::Rgb(255, 197, 47)),    // Brewers: gold
        ("CIN", Color::Rgb(198,   1, 31)),    // Reds: red
        ("PIT", Color::Rgb(253, 184, 39)),    // Pirates: gold
        // NL East
        ("NYM", Color::Rgb(0,    45, 114)),   // Mets: blue
        ("ATL", Color::Rgb(206,  17, 65)),    // Braves: red
        ("PHI", Color::Rgb(232,  24, 40)),    // Phillies: red
        ("MIA", Color::Rgb(0,   163, 224)),   // Marlins: teal/blue
        ("WSH", Color::Rgb(171,   0,  3)),    // Nationals: red
    ])
});

/// Returns the team color for the given abbreviation.
/// When `official` is true, uses the official team colors.
/// When false, uses terminal-friendly brightened colors.
pub fn get(abbreviation: &str, official: bool) -> Option<Color> {
    if official {
        TEAM_COLORS_OFFICIAL.get(abbreviation).copied()
    } else {
        TEAM_COLORS.get(abbreviation).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CURRENT_TEAMS: [&str; 30] = [
        "ATH", "SEA", "LAA", "HOU", "TEX",
        "CWS", "DET", "KC",  "MIN", "CLE",
        "NYY", "BOS", "TOR", "TB",  "BAL",
        "LAD", "SF",  "AZ",  "SD",  "COL",
        "CHC", "STL", "MIL", "CIN", "PIT",
        "NYM", "ATL", "PHI", "MIA", "WSH",
    ];

    #[test]
    fn all_current_teams_have_colors() {
        for abbr in CURRENT_TEAMS {
            assert!(get(abbr, false).is_some(), "missing readable color for {abbr}");
            assert!(get(abbr, true).is_some(), "missing official color for {abbr}");
        }
    }

    #[test]
    fn unknown_team_returns_none() {
        assert!(get("XYZ", false).is_none());
        assert!(get("XYZ", true).is_none());
        assert!(get("", false).is_none());
    }
}
