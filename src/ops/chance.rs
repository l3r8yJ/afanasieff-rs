use std::sync::atomic::{AtomicU32, Ordering};

const GENERATED_ON_KEYWORD_DEFAULT: u32 = 200;

static GENERATED_ON_KEYWORD: AtomicU32 = AtomicU32::new(GENERATED_ON_KEYWORD_DEFAULT);

const GENERATED_IN_CRON_DEFAULT: u32 = 250;

static GENERATED_IN_CRON: AtomicU32 = AtomicU32::new(GENERATED_IN_CRON_DEFAULT);

/// Returns the share of keyword replies that are generated rather than quoted.
#[must_use]
pub fn generated_on_keyword() -> f64 {
    f64::from(GENERATED_ON_KEYWORD.load(Ordering::Relaxed)) / 1000.0
}

/// Pins the share of keyword replies that are generated, in per-mille, so a
/// test can make the branch deterministic.
#[doc(hidden)]
pub fn set_generated_on_keyword_for_tests(per_mille: u32) {
    GENERATED_ON_KEYWORD.store(per_mille, Ordering::Relaxed);
}

/// Returns the share of hourly cron messages that are generated rather than quoted.
#[must_use]
pub fn generated_in_cron() -> f64 {
    f64::from(GENERATED_IN_CRON.load(Ordering::Relaxed)) / 1000.0
}

/// Pins the share of hourly cron messages that are generated, in per-mille,
/// so a test can make the branch deterministic.
#[doc(hidden)]
pub fn set_generated_in_cron_for_tests(per_mille: u32) {
    GENERATED_IN_CRON.store(per_mille, Ordering::Relaxed);
}
