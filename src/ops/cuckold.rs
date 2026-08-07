use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use rand::seq::IndexedRandom;

use crate::ops::store::Store;

const MOSCOW_OFFSET_SECONDS: i64 = 3 * 3600;

const SECONDS_IN_DAY: i64 = 86_400;

const ACTIVE_DAYS: i64 = 30;

#[derive(Debug)]
pub struct Roll {
    pub user: i64,
    pub name: String,
    pub total: i64,
    pub streak: i64,
    pub fresh: bool,
}

/// Returns the Moscow day the moment falls into, counted from the epoch.
#[must_use]
pub fn day_of(at: DateTime<Utc>) -> i64 {
    (at.timestamp() + MOSCOW_OFFSET_SECONDS).div_euclid(SECONDS_IN_DAY)
}

/// Draws the cuckold of the day, or repeats the one already drawn today.
///
/// Returns nothing when no member of the chat has been seen in the last thirty
/// days, which is also the case for a chat the bot has never heard from.
///
/// # Errors
///
/// Returns an error when a statement cannot be executed.
pub fn roll(
    store: &Store,
    chat: i64,
    at: DateTime<Utc>,
    rng: &mut impl Rng,
) -> rusqlite::Result<Option<Roll>> {
    let today = day_of(at);
    let state = store.state(chat)?;
    let drawn_on = state.get("cuckold_day").copied().unwrap_or_default();
    let drawn = state.get("cuckold_user").copied().unwrap_or_default();
    if drawn_on == today && drawn != 0 {
        return repeat(store, chat, drawn);
    }
    let since = (at - Duration::days(ACTIVE_DAYS)).to_rfc3339();
    let members = store.active_members(chat, &since)?;
    let Some(winner) = members.choose(rng) else {
        return Ok(None);
    };
    let streak = if drawn == winner.user && drawn_on == today - 1 {
        store.stat(chat, winner.user, "cuckold_streak")? + 1
    } else {
        1
    };
    let total = store.bump(chat, winner.user, "cuckold_days", 1)?;
    store.set_stat(chat, winner.user, "cuckold_streak", streak)?;
    let best = store.stat(chat, winner.user, "cuckold_best")?;
    if streak > best {
        store.set_stat(chat, winner.user, "cuckold_best", streak)?;
    }
    store.set_state(chat, "cuckold_day", today)?;
    store.set_state(chat, "cuckold_user", winner.user)?;
    Ok(Some(Roll {
        user: winner.user,
        name: winner.name.clone(),
        total,
        streak,
        fresh: true,
    }))
}

fn repeat(store: &Store, chat: i64, user: i64) -> rusqlite::Result<Option<Roll>> {
    let Some(name) = store.member_name(chat, user)? else {
        return Ok(None);
    };
    Ok(Some(Roll {
        user,
        name,
        total: store.stat(chat, user, "cuckold_days")?,
        streak: store.stat(chat, user, "cuckold_streak")?,
        fresh: false,
    }))
}

#[cfg(test)]
mod tests {
    use asserting::prelude::*;
    use chrono::{DateTime, TimeZone, Utc};
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::{day_of, roll};
    use crate::ops::store::Store;

    const CHAT: i64 = 42;

    fn store_with_one_member() -> Store {
        let store = Store::in_memory().unwrap();
        store
            .upsert_member(CHAT, 7, Some("m"), "Матвей", "2026-08-07T10:00:00+00:00")
            .unwrap();
        store
    }

    fn at(day: u32, hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, day, hour, 0, 0).unwrap()
    }

    fn seeded() -> StdRng {
        StdRng::seed_from_u64(1)
    }

    #[test]
    fn counts_an_evening_and_the_night_after_it_as_one_moscow_day() {
        let evening = day_of(at(7, 21));
        let night = day_of(at(7, 22));
        assert_that!(night)
            .named("moscow day of the night after")
            .is_equal_to(evening);
    }

    #[test]
    fn starts_a_new_moscow_day_at_nine_in_the_evening_utc() {
        let before = day_of(at(7, 20));
        let after = day_of(at(7, 21));
        assert_that!(after)
            .named("moscow day after the boundary")
            .is_equal_to(before + 1);
    }

    #[test]
    fn draws_once_a_day_and_repeats_itself_afterwards() {
        let store = store_with_one_member();
        let first = roll(&store, CHAT, at(7, 10), &mut seeded())
            .unwrap()
            .unwrap();
        let again = roll(&store, CHAT, at(7, 18), &mut seeded())
            .unwrap()
            .unwrap();
        assert_that!(first.fresh)
            .named("first draw of the day")
            .is_true();
        assert_that!(again.fresh)
            .named("second call of the same day")
            .is_false();
        assert_that!(again.user)
            .named("repeated winner")
            .is_equal_to(first.user);
        assert_that!(again.total)
            .named("tally after a repeat")
            .is_equal_to(1);
    }

    #[test]
    fn draws_again_on_the_next_day() {
        let store = store_with_one_member();
        roll(&store, CHAT, at(7, 10), &mut seeded()).unwrap();
        let next = roll(&store, CHAT, at(8, 10), &mut seeded())
            .unwrap()
            .unwrap();
        assert_that!(next.fresh)
            .named("draw on a new day")
            .is_true();
        assert_that!(next.total)
            .named("tally after two days")
            .is_equal_to(2);
    }

    #[test]
    fn grows_the_run_over_consecutive_days() {
        let store = store_with_one_member();
        roll(&store, CHAT, at(7, 10), &mut seeded()).unwrap();
        roll(&store, CHAT, at(8, 10), &mut seeded()).unwrap();
        let third = roll(&store, CHAT, at(9, 10), &mut seeded())
            .unwrap()
            .unwrap();
        assert_that!(third.streak)
            .named("run over three days")
            .is_equal_to(3);
    }

    #[test]
    fn breaks_the_run_when_a_day_is_skipped() {
        let store = store_with_one_member();
        roll(&store, CHAT, at(7, 10), &mut seeded()).unwrap();
        roll(&store, CHAT, at(8, 10), &mut seeded()).unwrap();
        let after_gap = roll(&store, CHAT, at(10, 10), &mut seeded())
            .unwrap()
            .unwrap();
        let best = store.stat(CHAT, after_gap.user, "cuckold_best").unwrap();
        assert_that!(after_gap.streak)
            .named("run after a skipped day")
            .is_equal_to(1);
        assert_that!(best).named("best run").is_equal_to(2);
    }

    #[test]
    fn draws_nobody_when_every_member_went_quiet() {
        let store = Store::in_memory().unwrap();
        store
            .upsert_member(CHAT, 7, Some("m"), "Матвей", "2026-01-01T10:00:00+00:00")
            .unwrap();
        let drawn = roll(&store, CHAT, at(7, 10), &mut seeded()).unwrap();
        assert_that!(drawn)
            .named("draw from a stale chat")
            .is_none();
    }

    #[test]
    fn draws_nobody_from_an_empty_chat() {
        let store = Store::in_memory().unwrap();
        let drawn = roll(&store, CHAT, at(7, 10), &mut seeded()).unwrap();
        assert_that!(drawn)
            .named("draw from an empty chat")
            .is_none();
    }
}
