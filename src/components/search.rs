use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Matcher, Utf32Str};

/// Minimum number of characters before fuzzy matching kicks in.
const MIN_QUERY_LEN: usize = 2;

/// Maximum number of results to display.
const MAX_RESULTS: usize = 50;

#[derive(Default)]
pub struct SearchState {
    /// Whether the search input bar is open.
    pub is_open: bool,
    /// The current search input text.
    pub input: String,
    /// Fuzzy matcher for the search input.
    matcher: Matcher,
    /// Row indices (into the stats table) that match the query, sorted by score.
    pub matched_indices: Vec<usize>,
}

impl SearchState {
    pub fn open(&mut self) {
        self.is_open = true;
    }

    /// Cancel search entirely — clears input, matches, and closes.
    pub fn close(&mut self) {
        self.is_open = false;
        self.input.clear();
        self.matched_indices.clear();
    }

    /// Close the input bar but keep the filtered results.
    pub fn submit(&mut self) {
        self.is_open = false;
    }

    pub fn handle_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn handle_backspace(&mut self) {
        self.input.pop();
    }

    /// Returns true if the stats table should be filtered.
    pub fn is_filtering(&self) -> bool {
        self.input.len() >= MIN_QUERY_LEN
    }

    /// Run fuzzy matching against the given names and populate `matched_indices`.
    pub fn update_matches(&mut self, names: &[String]) {
        self.matched_indices.clear();

        if self.input.len() < MIN_QUERY_LEN {
            return;
        }

        let pattern = Pattern::parse(&self.input, CaseMatching::Ignore, Normalization::Smart);
        let mut buf = Vec::new();

        let mut scored: Vec<(usize, u32)> = names
            .iter()
            .enumerate()
            .filter_map(|(idx, name)| {
                let haystack = Utf32Str::new(name, &mut buf);
                let score = pattern.score(haystack, &mut self.matcher)?;
                Some((idx, score))
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.truncate(MAX_RESULTS);

        self.matched_indices = scored.into_iter().map(|(idx, _)| idx).collect();
    }
}
