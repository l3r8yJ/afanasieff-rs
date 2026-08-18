use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Duration;

use rand::Rng;

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

const BURST_MIN: u32 = 1;

const BURST_MAX: u32 = 4;

static PINNED_BURST: AtomicU32 = AtomicU32::new(0);

const BURST_PAUSE_MILLIS_DEFAULT: u64 = 1200;

static BURST_PAUSE_MILLIS: AtomicU64 = AtomicU64::new(BURST_PAUSE_MILLIS_DEFAULT);

/// Returns how many messages the hourly cron sends to one chat in a row.
#[must_use]
pub fn burst_length() -> u32 {
    let pinned = PINNED_BURST.load(Ordering::Relaxed);
    if pinned > 0 {
        return pinned;
    }
    rand::rng().random_range(BURST_MIN..=BURST_MAX)
}

/// Pins the length of the hourly burst so a test can count what was sent.
/// Zero restores the random length.
#[doc(hidden)]
pub fn set_burst_length_for_tests(length: u32) {
    PINNED_BURST.store(length, Ordering::Relaxed);
}

/// Returns the pause between two messages of the same burst.
#[must_use]
pub fn burst_pause() -> Duration {
    Duration::from_millis(BURST_PAUSE_MILLIS.load(Ordering::Relaxed))
}

/// Pins the pause between two messages of the same burst, so a test does not
/// wait.
#[doc(hidden)]
pub fn set_burst_pause_for_tests(millis: u64) {
    BURST_PAUSE_MILLIS.store(millis, Ordering::Relaxed);
}

const TAGGED_IN_BURST_DEFAULT: u32 = 350;

static TAGGED_IN_BURST: AtomicU32 = AtomicU32::new(TAGGED_IN_BURST_DEFAULT);

/// Returns the share of burst messages that tag a member of the chat.
#[must_use]
pub fn tagged_in_burst() -> f64 {
    f64::from(TAGGED_IN_BURST.load(Ordering::Relaxed)) / 1000.0
}

/// Pins the share of burst messages that tag a member, in per-mille, so a test
/// can make the branch deterministic.
#[doc(hidden)]
pub fn set_tagged_in_burst_for_tests(per_mille: u32) {
    TAGGED_IN_BURST.store(per_mille, Ordering::Relaxed);
}
