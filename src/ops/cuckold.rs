use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration as StdDuration;

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use rand::seq::IndexedRandom;
use teloxide::Bot;
use teloxide::payloads::{EditMessageTextSetters, SendMessageSetters};
use teloxide::prelude::Requester;
use teloxide::types::{ChatId, Message, ParseMode};
use teloxide::utils::html::{escape, user_mention};

use crate::ops::error::Error;
use crate::ops::store::Store;

const MOSCOW_OFFSET_SECONDS: i64 = 3 * 3600;

const SECONDS_IN_DAY: i64 = 86_400;

const ACTIVE_DAYS: i64 = 30;

const DRUMROLL_MILLIS: u64 = 1500;

static DRUMROLL: AtomicU64 = AtomicU64::new(DRUMROLL_MILLIS);

const VERDICTS: &[&str] = &[
    "Хорошо куколд сука.",
    "Не понял, куколд моя бабушка?",
    "Манифест куколдистической партии принят.",
];

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
    store.set_state(chat, "cuckold_user", winner.user)?;
    store.set_state(chat, "cuckold_day", today)?;
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

/// Overrides the pause between the drum roll beats, so a test does not wait.
#[doc(hidden)]
pub fn set_drumroll_for_tests(millis: u64) {
    DRUMROLL.store(millis, Ordering::Relaxed);
}

/// Announces the cuckold of the day, drawing one when today has none yet.
///
/// # Errors
///
/// Returns an error when the answer cannot be sent.
pub async fn announce(bot: &Bot, message: &Message, store: &Store) -> Result<(), Error> {
    let chat = message.chat.id;
    let outcome = roll(store, chat.0, message.date, &mut rand::rng());
    let drawn = match outcome {
        Ok(Some(drawn)) => drawn,
        Ok(None) => return reply_and_stop(bot, chat, "Играть не с кем. Терпим.").await,
        Err(error) => {
            log::error!("cuckold of chat '{}' was not drawn: '{error}'", chat.0);
            return reply_and_stop(bot, chat, "Бля, я обосрался. Попробуйте позже.").await;
        }
    };
    let verdict = verdict_for(&drawn);
    if !drawn.fresh {
        bot.send_message(chat, verdict)
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    }
    let sent = bot.send_message(chat, "Ищу куколда дня…").await?;
    sleep().await;
    bot.edit_message_text(chat, sent.id, "Проверяю списки…")
        .await?;
    sleep().await;
    bot.edit_message_text(chat, sent.id, verdict)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

async fn reply_and_stop(bot: &Bot, chat: ChatId, text: &str) -> Result<(), Error> {
    bot.send_message(chat, text).await?;
    Ok(())
}

fn verdict_for(drawn: &Roll) -> String {
    let mention = user_mention(
        teloxide::types::UserId(u64::try_from(drawn.user).unwrap_or_default()),
        &drawn.name,
    );
    let line = VERDICTS
        .choose(&mut rand::rng())
        .copied()
        .unwrap_or("Хорошо куколд сука.");
    format!(
        "🎉 Куколд дня — {mention}. {line}\nВсего раз: {}. Серия: {}.",
        drawn.total, drawn.streak
    )
}

async fn sleep() {
    let millis = DRUMROLL.load(Ordering::Relaxed);
    if millis > 0 {
        tokio::time::sleep(StdDuration::from_millis(millis)).await;
    }
}

const MEDALS: &[&str] = &["🥇", "🥈", "🥉"];

/// Renders how often each member of the chat has been the cuckold.
#[must_use]
pub fn stats(message: &Message, store: &Store) -> String {
    let chat = message.chat.id.0;
    let ranked = store.ranking(chat, "cuckold_days").unwrap_or_default();
    if ranked.is_empty() {
        return "🏆 <b>Куколды чата</b>\n\nНикто ещё не был куколдом. Терпим.".to_string();
    }
    let lines = ranked
        .iter()
        .enumerate()
        .map(|(place, standing)| {
            let rank = MEDALS
                .get(place)
                .map_or_else(|| format!("{}.", place + 1), |medal| (*medal).to_string());
            let name = standing
                .name
                .clone()
                .unwrap_or_else(|| standing.user.to_string());
            let best = store
                .stat(chat, standing.user, "cuckold_best")
                .unwrap_or_default();
            let run = if best > 1 {
                format!(" · лучшая серия {best}")
            } else {
                String::new()
            };
            format!(
                "{rank} <b>{}</b> · {} раз{run}",
                escape(&name),
                standing.count
            )
        })
        .collect::<Vec<String>>()
        .join("\n");
    format!("🏆 <b>Куколды чата</b>\n\n{lines}{}", today(message, store))
}

fn today(message: &Message, store: &Store) -> String {
    let chat = message.chat.id.0;
    let Ok(state) = store.state(chat) else {
        return String::new();
    };
    let drawn_on = state.get("cuckold_day").copied().unwrap_or_default();
    let drawn = state.get("cuckold_user").copied().unwrap_or_default();
    if drawn == 0 || drawn_on != day_of(message.date) {
        return String::new();
    }
    match store.member_name(chat, drawn) {
        Ok(Some(name)) => format!("\n\nСегодня: {}", escape(&name)),
        Ok(None) | Err(_) => String::new(),
    }
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
    fn records_the_winners_identity_before_closing_the_days_gate() {
        let store = store_with_one_member();
        let drawn = roll(&store, CHAT, at(7, 10), &mut seeded())
            .unwrap()
            .unwrap();
        let state = store.state(CHAT).unwrap();
        let recorded = state.get("cuckold_user").copied().unwrap_or_default();
        assert_that!(recorded)
            .named("cuckold_user recorded by the draw")
            .is_equal_to(drawn.user);
        let again = roll(&store, CHAT, at(7, 18), &mut seeded())
            .unwrap()
            .unwrap();
        assert_that!(again.user)
            .named("winner repeated later the same day")
            .is_equal_to(drawn.user);
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

    #[test]
    fn fails_to_roll_when_the_chat_state_table_is_gone() {
        let store = store_with_one_member();
        store
            .with(|connection| connection.execute_batch("DROP TABLE chat_state"))
            .unwrap();
        let drawn = roll(&store, CHAT, at(7, 10), &mut seeded());
        assert_that!(drawn)
            .named("draw against a store missing its state table")
            .is_err();
    }
}
