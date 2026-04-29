use log::error;
use tui::prelude::Color;
use tui::style::Style;

/// Default text color which allows the terminal theme to supply the color.
pub const TEXT_COLOR: Color = Color::Reset;

/// Default border color which allows the terminal theme to supply the color.
const BORDER_COLOR: Color = Color::Reset;

const UNDERLINER_COLOR: Color = Color::Reset;

pub fn border_style() -> Style {
    Style::new().fg(BORDER_COLOR)
}

pub fn header_style() -> Style {
    Style::new()
        .bold()
        .underlined()
        .underline_color(UNDERLINER_COLOR)
}

pub fn dim_style() -> Style {
    Style::new().fg(TEXT_COLOR).dim()
}

pub fn selected_style() -> Style {
    Style::new().fg(Color::Black).bg(Color::Blue)
}

/// Returns a `Style` with `TEXT_COLOR` modified with `Modifier::DIM` if the value is zero,
/// otherwise a default style with `TEXT_COLOR`.
pub trait DimStyle {
    fn is_zero(&self) -> bool;

    fn dim_or_default(&self) -> Style {
        if self.is_zero() {
            dim_style()
        } else {
            Style::new().fg(TEXT_COLOR)
        }
    }
}

macro_rules! impl_is_zero {
      ($($t:ty),*) => {
          $(impl DimStyle for $t {
              fn is_zero(&self) -> bool { *self == 0 }
          })*
      };
  }
impl_is_zero!(u8, u16);

impl DimStyle for str {
    fn is_zero(&self) -> bool {
        // TODO should this check floats too, e.g. "0.0"?
        self == "0"
    }
}

/// Color for an ERA stat string
pub fn era_color(era: &str) -> Color {
    era.parse::<f64>()
        .map(|v| {
            if v <= 3.00 {
                Color::Green
            } else if v >= 5.00 {
                Color::Red
            } else {
                TEXT_COLOR
            }
        })
        .unwrap_or(TEXT_COLOR)
}

/// Style for an ERA stat string.
pub fn era_style(era: &str) -> Style {
    Style::new().fg(era_color(era))
}

/// Color for a batting average stat string.
pub fn avg_color(avg: &str) -> Color {
    avg.parse::<f64>()
        .map(|v| {
            if v == 0.0 {
                TEXT_COLOR
            } else if v >= 0.300 {
                Color::Green
            } else if v < 0.100 {
                Color::Red
            } else {
                TEXT_COLOR
            }
        })
        .unwrap_or(TEXT_COLOR)
}

/// Style for a batting average stat string.
pub fn avg_style(avg: &str) -> Style {
    Style::new().fg(avg_color(avg))
}

/// Color for a winning-percentage stat string.
pub fn win_pct_color(pct: &str) -> Color {
    pct.parse::<f64>()
        .map(|v| {
            if v == 0.0 {
                TEXT_COLOR
            } else if v >= 0.500 {
                Color::Green
            } else {
                Color::Red
            }
        })
        .unwrap_or(TEXT_COLOR)
}

/// Convert a string from the API to a Color::Rgb. The string starts out as:
/// "rgba(255, 255, 255, 0.55)".
pub fn convert_color(s: String) -> Color {
    if let Some(s) = s.strip_prefix("rgba(") {
        let c: Vec<&str> = s.split(", ").collect();
        Color::Rgb(
            c[0].parse().unwrap_or(0),
            c[1].parse().unwrap_or(0),
            c[2].parse().unwrap_or(0),
        )
    } else {
        error!("color doesn't start with 'rgba(' {s:?}");
        Color::Rgb(0, 0, 0)
    }
}

#[test]
fn test_color_conversion() {
    let tests = vec![
        ("rgba(0, 0, 0, .55)", Color::Rgb(0, 0, 0)),
        ("rgba(6, 90, 238, .55)", Color::Rgb(6, 90, 238)),
        ("rgba(150, 188, 255, .55)", Color::Rgb(150, 188, 255)),
        ("rgba(214, 41, 52, .55)", Color::Rgb(214, 41, 52)),
        ("rgba(255, 255, 255, 0.55)", Color::Rgb(255, 255, 255)),
    ];
    for t in tests {
        assert_eq!(convert_color(t.0.to_string()), t.1);
    }

    let bad = ("rgba(55, 255, 255, 0.55)", Color::Rgb(255, 255, 255));
    assert_ne!(convert_color(bad.0.to_string()), bad.1);

    let nonsense = ("rgba(-5, 255, 255, 0.55)", Color::Rgb(0, 255, 255));
    assert_eq!(convert_color(nonsense.0.to_string()), nonsense.1);
}
