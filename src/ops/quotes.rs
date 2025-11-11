use core::panic;

use rand::{rng, seq::IndexedRandom};

/// Returns random String from provided pool of Strings.
///
/// # Panics
///
/// Panics if quote was null.
#[must_use]
pub fn random_string_from<'a>(pool: &'a [&'a str]) -> &'a str {
    let mut rng = rng();
    pool.choose(&mut rng)
        .map_or_else(|| panic!("Can't take the quote"), |s| *s)
}
