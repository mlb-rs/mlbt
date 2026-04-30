/// Display an `Option<T>` as a string, using a default if `None`.
/// e.g. `bio.height.display_or("-")`
pub trait OptionDisplayExt {
    fn display_or(&self, default: &str) -> String;
}

impl<T: std::fmt::Display> OptionDisplayExt for Option<T> {
    fn display_or(&self, default: &str) -> String {
        self.as_ref()
            .map(|v| v.to_string())
            .unwrap_or_else(|| default.to_string())
    }
}

/// Map an `Option<T>` through a function, then display as a string with a default if `None`.
/// e.g. `bio.weight.map_display_or(|w| format!("{w}lb"), "")`
pub trait OptionMapDisplayExt<T> {
    fn map_display_or<U: std::fmt::Display, F: FnOnce(&T) -> U>(
        &self,
        f: F,
        default: &str,
    ) -> String;
}

impl<T> OptionMapDisplayExt<T> for Option<T> {
    fn map_display_or<U: std::fmt::Display, F: FnOnce(&T) -> U>(
        &self,
        f: F,
        default: &str,
    ) -> String {
        self.as_ref()
            .map(f)
            .map(|v| v.to_string())
            .unwrap_or_else(|| default.to_string())
    }
}

/// Surname for compact display. Skips trailing generational suffixes so "Vladimir Guerrero Jr."
/// returns "Guerrero" instead of "Jr."
pub fn last_name(full: &str) -> &str {
    let mut parts = full.rsplitn(3, ' ');
    let tail = parts.next().unwrap_or(full);
    if matches!(tail, "Jr." | "Sr." | "II" | "III" | "IV") {
        parts.next().unwrap_or(tail)
    } else {
        tail
    }
}

#[test]
fn test_last_name() {
    assert_eq!(last_name("Jack Flaherty"), "Flaherty");
    assert_eq!(last_name("J.P. France"), "France");
    assert_eq!(last_name("Vladimir Guerrero Jr."), "Guerrero");
    assert_eq!(last_name("Cal Ripken Jr."), "Ripken");
    assert_eq!(last_name("Ken Griffey Sr."), "Griffey");
    assert_eq!(last_name("Cal Ripken III"), "Ripken");
    assert_eq!(last_name("Robert Person II"), "Person");
    assert_eq!(last_name("Madison"), "Madison");
    assert_eq!(last_name(""), "");
    // suffix-only input falls back to the suffix
    assert_eq!(last_name("Jr."), "Jr.");
}
