use serde::{Deserialize, Serialize};
use tui::style::{Color, Style};

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeLevel {
    Lean,
    #[default]
    Classic,
    Rainbow,
}

pub struct Theme {
    level: ThemeLevel,
}

#[allow(dead_code)]
impl Theme {
    pub fn new(level: ThemeLevel) -> Self {
        Self { level }
    }

    pub fn level(&self) -> ThemeLevel {
        self.level
    }

    // --- Tier queries ---

    /// Stat colors (Fangraphs blue-to-red scale) — classic and rainbow.
    pub fn use_stat_colors(&self) -> bool {
        !matches!(self.level, ThemeLevel::Lean)
    }

    /// Background highlights on rows — rainbow only.
    pub fn use_backgrounds(&self) -> bool {
        matches!(self.level, ThemeLevel::Rainbow)
    }

    // --- Fangraphs-inspired stat colors (foreground) ---

    pub const EXCELLENT: Color = Color::Rgb(69, 133, 207);
    pub const GOOD: Color = Color::Rgb(131, 178, 224);
    pub const BELOW_AVG: Color = Color::Rgb(214, 153, 33);
    pub const POOR: Color = Color::Rgb(204, 36, 29);

    // --- Fangraphs-inspired stat backgrounds (rainbow tier) ---
    // Muted versions of the stat colors, suitable as cell backgrounds on dark terminals.

    pub const EXCELLENT_BG: Color = Color::Rgb(20, 50, 90);
    pub const GOOD_BG: Color = Color::Rgb(30, 50, 75);
    pub const BELOW_AVG_BG: Color = Color::Rgb(70, 45, 10);
    pub const POOR_BG: Color = Color::Rgb(80, 20, 15);

    // --- UI chrome ---

    pub const ACCENT_BG: Color = Color::Rgb(69, 133, 136);
    pub const ACCENT_FG: Color = Color::Rgb(235, 219, 178);
    pub const DIMMED: Color = Color::Rgb(146, 131, 116);
    pub const BORDER: Color = Color::Rgb(80, 73, 69);
    pub const POSITIVE: Color = Color::Rgb(152, 151, 26);
    pub const TITLE_BG: Color = Color::Rgb(40, 60, 80);
    pub const TITLE_FG: Color = Color::Rgb(235, 219, 178);

    // --- Rainbow-only backgrounds ---

    pub const ROW_HIGHLIGHT: Color = Color::Rgb(55, 55, 60);
    pub const FAVORITE_BG: Color = Color::Rgb(60, 48, 20);
    pub const LIVE_GAME_BG: Color = Color::Rgb(20, 45, 30);

    // --- Convenience accessors ---
    // Lean returns stock app defaults; Classic and Rainbow return themed values.

    pub fn border(&self) -> Color {
        match self.level {
            ThemeLevel::Lean => Color::White,
            _ => Self::BORDER,
        }
    }

    pub fn dimmed(&self) -> Color {
        match self.level {
            ThemeLevel::Lean => Color::DarkGray,
            _ => Self::DIMMED,
        }
    }

    pub fn selection_style(&self) -> Style {
        match self.level {
            ThemeLevel::Lean => Style::default().bg(Color::Blue).fg(Color::Black),
            _ => Style::default().bg(Self::ACCENT_BG).fg(Self::ACCENT_FG),
        }
    }

    pub fn title_style(&self) -> Style {
        match self.level {
            ThemeLevel::Lean => Style::default().fg(Color::Black).bg(Color::Blue),
            _ => Style::default().fg(Self::TITLE_FG).bg(Self::TITLE_BG),
        }
    }

    /// Returns a style for a stat cell based on the stat color.
    /// - Lean: no stat coloring (dimmed zeros still apply).
    /// - Classic: colored foreground only.
    /// - Rainbow: colored background with contrasting foreground (Fangraphs style).
    pub fn stat_style(&self, fg_color: Color) -> Style {
        if !self.use_stat_colors() {
            // Lean: preserve dimmed zeros for readability, skip tier colors.
            if fg_color == Self::DIMMED {
                return Style::default().fg(Self::DIMMED);
            }
            return Style::default();
        }
        if !self.use_backgrounds() {
            return Style::default().fg(fg_color);
        }
        let bg = match fg_color {
            c if c == Self::EXCELLENT => Self::EXCELLENT_BG,
            c if c == Self::GOOD => Self::GOOD_BG,
            c if c == Self::BELOW_AVG => Self::BELOW_AVG_BG,
            c if c == Self::POOR => Self::POOR_BG,
            _ => return Style::default().fg(fg_color),
        };
        Style::default().fg(fg_color).bg(bg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lean_tier_queries() {
        let t = Theme::new(ThemeLevel::Lean);
        assert!(!t.use_stat_colors());
        assert!(!t.use_backgrounds());
    }

    #[test]
    fn classic_tier_queries() {
        let t = Theme::new(ThemeLevel::Classic);
        assert!(t.use_stat_colors());
        assert!(!t.use_backgrounds());
    }

    #[test]
    fn rainbow_tier_queries() {
        let t = Theme::new(ThemeLevel::Rainbow);
        assert!(t.use_stat_colors());
        assert!(t.use_backgrounds());
    }

    #[test]
    fn stat_style_no_color_in_lean() {
        let t = Theme::new(ThemeLevel::Lean);
        let style = t.stat_style(Theme::EXCELLENT);
        assert_eq!(style.fg, None);
        assert_eq!(style.bg, None);
    }

    #[test]
    fn stat_style_preserves_dimmed_in_lean() {
        let t = Theme::new(ThemeLevel::Lean);
        let style = t.stat_style(Theme::DIMMED);
        assert_eq!(style.fg, Some(Theme::DIMMED));
    }

    #[test]
    fn default_is_classic() {
        assert_eq!(ThemeLevel::default(), ThemeLevel::Classic);
    }

    #[test]
    fn stat_style_fg_only_in_classic() {
        let t = Theme::new(ThemeLevel::Classic);
        let style = t.stat_style(Theme::EXCELLENT);
        assert_eq!(style.fg, Some(Theme::EXCELLENT));
        assert_eq!(style.bg, None);
    }

    #[test]
    fn stat_style_bg_in_rainbow() {
        let t = Theme::new(ThemeLevel::Rainbow);
        let style = t.stat_style(Theme::EXCELLENT);
        assert_eq!(style.fg, Some(Theme::EXCELLENT));
        assert_eq!(style.bg, Some(Theme::EXCELLENT_BG));
    }

    #[test]
    fn stat_style_all_tiers_have_bg_in_rainbow() {
        let t = Theme::new(ThemeLevel::Rainbow);
        for color in [Theme::EXCELLENT, Theme::GOOD, Theme::BELOW_AVG, Theme::POOR] {
            let style = t.stat_style(color);
            assert!(style.bg.is_some(), "missing bg for stat color {color:?}");
        }
    }
}
