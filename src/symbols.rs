use crate::theme::{Theme, ThemeLevel};

pub struct Symbols {
    nerd_fonts: bool,
    team_colors: bool,
    theme: Theme,
}

impl Symbols {
    pub fn new(nerd_fonts: bool, team_colors: bool, theme_level: ThemeLevel) -> Self {
        Self {
            nerd_fonts,
            team_colors,
            theme: Theme::new(theme_level),
        }
    }

    pub fn nerd_fonts(&self) -> bool {
        self.nerd_fonts
    }

    pub fn team_colors(&self) -> bool {
        self.team_colors
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn tab_scoreboard(&self) -> &'static str {
        if self.nerd_fonts { "\u{F073} " } else { "" }
    }

    pub fn tab_gameday(&self) -> &'static str {
        if self.nerd_fonts { "\u{F008} " } else { "" }
    }

    pub fn tab_stats(&self) -> &'static str {
        if self.nerd_fonts { "\u{F080} " } else { "" }
    }

    pub fn tab_standings(&self) -> &'static str {
        if self.nerd_fonts { "\u{F091} " } else { "" }
    }

    /// Cursor shown next to the selected play in the at-bat plays list.
    pub fn selection_cursor(&self) -> char {
        if self.nerd_fonts { '\u{F0DA}' } else { '>' }
    }

    /// Indicator shown for scoring plays.
    pub fn scoring_play(&self) -> char {
        if self.nerd_fonts { '\u{F43F}' } else { '!' }
    }

    /// Filled base (runner on base).
    // Standard Unicode diamonds — these shapes have no PUA equivalent in Nerd Fonts.
    pub fn base_occupied(&self) -> char {
        if self.nerd_fonts { '◆' } else { '■' }
    }

    /// Empty base (no runner).
    // Standard Unicode diamonds — these shapes have no PUA equivalent in Nerd Fonts.
    pub fn base_empty(&self) -> char {
        if self.nerd_fonts { '◇' } else { '□' }
    }

    /// Scrollbar begin symbol (top of content).
    pub fn scroll_up(&self) -> &'static str {
        if self.nerd_fonts { "\u{F062}" } else { "↑" }
    }

    /// Scrollbar end symbol (bottom of content).
    pub fn scroll_down(&self) -> &'static str {
        if self.nerd_fonts { "\u{F063}" } else { "↓" }
    }

    /// Sort ascending column header indicator.
    // Plain Unicode arrows intentionally — these are text indicators in column headers,
    // not decorative icons. Scroll arrows use PUA glyphs because they are icons.
    pub fn sort_asc(&self) -> &'static str {
        if self.nerd_fonts { "↑" } else { "^" }
    }

    /// Sort descending column header indicator.
    // Plain Unicode arrows intentionally — see sort_asc comment.
    pub fn sort_desc(&self) -> &'static str {
        if self.nerd_fonts { "↓" } else { "v" }
    }

    /// Prefix shown before the favorite team's game. Always 2 chars wide.
    pub fn favorite_marker(&self) -> &'static str {
        if self.nerd_fonts { "★ " } else { "  " }
    }

    /// Weather icon for the given condition string from the MLB API.
    /// Returns a Nerd Font weather icon or a plain-text fallback.
    pub fn weather_icon(&self, condition: &str) -> &'static str {
        if self.nerd_fonts {
            let lower = condition.to_lowercase();
            if lower.contains("sun") || lower.contains("clear") {
                "\u{E302}" // nf-weather-day_sunny
            } else if lower.contains("partly") || lower.contains("few clouds") {
                "\u{E37B}" // nf-weather-day_cloudy
            } else if lower.contains("cloud") || lower.contains("overcast") {
                "\u{E312}" // nf-weather-cloudy
            } else if lower.contains("rain")
                || lower.contains("drizzle")
                || lower.contains("shower")
            {
                "\u{E318}" // nf-weather-rain
            } else if lower.contains("snow") {
                "\u{E31A}" // nf-weather-snow
            } else if lower.contains("thunder") || lower.contains("storm") {
                "\u{E31D}" // nf-weather-thunderstorm
            } else if lower.contains("fog") || lower.contains("mist") || lower.contains("haze") {
                "\u{E313}" // nf-weather-fog
            } else if lower.contains("wind") {
                "\u{E34B}" // nf-weather-strong_wind
            } else if lower.contains("dome") || lower.contains("roof") {
                "\u{F015}" // nf-fa-home (dome/roof closed)
            } else {
                "\u{E302}" // default to sunny
            }
        } else {
            "" // no icon in plain mode
        }
    }

    /// Format a weather string for display. Returns something like "☀ 72°F" or "72°F".
    pub fn format_weather(&self, condition: &str, temp: &str) -> String {
        let icon = self.weather_icon(condition);
        if icon.is_empty() {
            format!("{temp}°F")
        } else {
            format!("{icon} {temp}°F")
        }
    }

    /// Format wind string from the API into compact arrow notation.
    /// "11 mph, Out To RF" -> "11 mph ↗" (nerd fonts) or "11 mph Out-RF" (plain)
    /// "7 mph, R To L" -> "7 mph ←" or "7 mph R-L"
    pub fn format_wind(&self, wind: &str) -> String {
        // Split "11 mph, Out To RF" into speed ("11 mph") and direction ("Out To RF")
        let (speed, direction) = match wind.split_once(", ") {
            Some((s, d)) => (s, Some(d)),
            None => (wind, None),
        };

        let Some(dir) = direction else {
            return speed.to_string();
        };

        if self.nerd_fonts {
            let arrow = match dir {
                "Out To RF" => "↗",
                "Out To CF" => "↑",
                "Out To LF" => "↖",
                "In From RF" => "↙",
                "In From CF" => "↓",
                "In From LF" => "↘",
                "R To L" => "←",
                "L To R" => "→",
                "Calm" => "·",
                _ => dir,
            };
            format!("{speed} {arrow}")
        } else {
            // Compact text: "Out To RF" -> "Out-RF"
            let short = dir
                .replace("Out To ", "Out-")
                .replace("In From ", "In-")
                .replace("R To L", "R-L")
                .replace("L To R", "L-R");
            format!("{speed} {short}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_mode_returns_ascii() {
        let s = Symbols::new(false, false, ThemeLevel::default());
        assert_eq!(s.tab_scoreboard(), "");
        assert_eq!(s.tab_gameday(), "");
        assert_eq!(s.tab_stats(), "");
        assert_eq!(s.tab_standings(), "");
        assert_eq!(s.selection_cursor(), '>');
        assert_eq!(s.scoring_play(), '!');
        assert_eq!(s.base_occupied(), '■');
        assert_eq!(s.base_empty(), '□');
        assert_eq!(s.scroll_up(), "↑");
        assert_eq!(s.scroll_down(), "↓");
        assert_eq!(s.sort_asc(), "^");
        assert_eq!(s.sort_desc(), "v");
        assert_eq!(s.favorite_marker(), "  ");
    }

    #[test]
    fn nerd_fonts_mode_returns_glyphs() {
        let s = Symbols::new(true, false, ThemeLevel::default());
        assert_eq!(s.tab_scoreboard(), "\u{F073} ");
        assert_eq!(s.tab_gameday(), "\u{F008} ");
        assert_eq!(s.tab_stats(), "\u{F080} ");
        assert_eq!(s.tab_standings(), "\u{F091} ");
        assert_eq!(s.selection_cursor(), '\u{F0DA}');
        assert_eq!(s.scoring_play(), '\u{F43F}');
        assert_eq!(s.base_occupied(), '◆');
        assert_eq!(s.base_empty(), '◇');
        assert_eq!(s.scroll_up(), "\u{F062}");
        assert_eq!(s.scroll_down(), "\u{F063}");
        assert_eq!(s.sort_asc(), "↑");
        assert_eq!(s.sort_desc(), "↓");
        assert_eq!(s.favorite_marker(), "★ ");
    }
}
