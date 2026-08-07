const GENERATED_ON_KEYWORD: f64 = 0.2;

const KEYWORD_VARIABLE: &str = "AFANASIEFF_GENERATED_ON_KEYWORD";

/// Returns the share of keyword replies that are generated rather than quoted.
///
/// Reads `AFANASIEFF_GENERATED_ON_KEYWORD` so a test can pin it, and falls back
/// to the default when the variable is absent or unparseable.
#[must_use]
pub fn generated_on_keyword() -> f64 {
    std::env::var(KEYWORD_VARIABLE)
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|share| (0.0..=1.0).contains(share))
        .unwrap_or(GENERATED_ON_KEYWORD)
}
