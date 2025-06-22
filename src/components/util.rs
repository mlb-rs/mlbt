use log::error;
use tui::style::Color;

/// Convert a string from the API to a Color::Rgb. The string starts out as:
/// "rgba(255, 255, 255, 0.55)".
pub(crate) fn convert_color(s: String) -> Color {
    if let Some(s) = s.strip_prefix("rgba(") {
        let c: Vec<&str> = s.split(", ").collect();
        Color::Rgb(
            c[0].parse().unwrap_or(0),
            c[1].parse().unwrap_or(0),
            c[2].parse().unwrap_or(0),
        )
    } else {
        error!("color doesn't start with 'rgba(' {:?}", s);
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
