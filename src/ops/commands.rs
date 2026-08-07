use std::sync::Arc;

use teloxide::Bot;
use teloxide::macros::BotCommands;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{Message, ParseMode};
use teloxide::utils::html::escape;

use crate::ops::achievements::rules::{Achievement, Stats};
use crate::ops::error::Error;
use crate::ops::store::Store;

const BAR_CELLS: i64 = 10;

const BAR_CELLS_USIZE: usize = 10;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "snake_case")]
pub enum Command {
    #[command(description = "все ачивки и за что их дают")]
    Achievements,
    #[command(description = "что открыто у тебя, а что нет")]
    MyAchievements,
    #[command(description = "кто сколько ачивок собрал")]
    Top,
    #[command(description = "фраза, которой никто не говорил")]
    Bred,
}

/// Answers the achievement commands.
///
/// # Errors
///
/// Returns an error when the answer cannot be sent.
pub async fn answer(
    bot: Bot,
    message: Message,
    command: Command,
    store: Arc<Store>,
) -> Result<(), Error> {
    let text = match command {
        Command::Achievements => catalogue(),
        Command::MyAchievements => personal(&message, &store),
        Command::Top => top(&message, &store),
        Command::Bred => bred(&store),
    };
    bot.send_message(message.chat.id, text)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

fn catalogue() -> String {
    let lines = Achievement::ALL
        .iter()
        .map(|achievement| {
            format!(
                "🏅 <b>{}</b>\n     <i>{}</i>",
                escape(achievement.title()),
                escape(achievement.hint())
            )
        })
        .collect::<Vec<String>>()
        .join("\n\n");
    format!(
        "🏆 <b>Ачивки</b> · всего {}\n\n{lines}",
        Achievement::ALL.len()
    )
}

fn personal(message: &Message, store: &Store) -> String {
    let Some(author) = message.from.as_ref() else {
        return "Не вижу кто спрашивает.".to_string();
    };
    let chat = message.chat.id.0;
    let user = i64::try_from(author.id.0).unwrap_or(i64::MAX);
    let stats = Stats::new(store.stats(chat, user).unwrap_or_default());
    let owned = store.owned(chat, user).unwrap_or_default();
    let mut unlocked = Vec::new();
    let mut locked = Vec::new();
    for achievement in Achievement::ALL.iter().copied() {
        if owned.contains(achievement.code()) {
            let at = store
                .unlocked_at(chat, user, achievement.code())
                .ok()
                .flatten()
                .unwrap_or_default();
            unlocked.push(format!(
                "🏅 <b>{}</b> · <i>{}</i>",
                escape(achievement.title()),
                day_of(&at)
            ));
        } else {
            locked.push((achievement, achievement.progress(&stats)));
        }
    }
    locked.sort_by(|left, right| share(right.1).total_cmp(&share(left.1)));
    let locked = locked
        .iter()
        .map(|(achievement, progress)| match progress {
            Some((current, threshold)) => format!(
                "🔒 <b>{}</b>\n     {} <code>{current}/{threshold}</code>",
                escape(achievement.title()),
                bar(*current, *threshold)
            ),
            None => format!("🔒 <b>{}</b>", escape(achievement.title())),
        })
        .collect::<Vec<String>>();
    let header = format!(
        "🏆 <b>{}</b> · {} из {}\n{}",
        escape(&author.first_name),
        owned.len(),
        Achievement::ALL.len(),
        bar(
            i64::try_from(owned.len()).unwrap_or(i64::MAX),
            i64::try_from(Achievement::ALL.len()).unwrap_or(i64::MAX)
        )
    );
    [header, unlocked.join("\n"), locked.join("\n")]
        .into_iter()
        .filter(|section| !section.is_empty())
        .collect::<Vec<String>>()
        .join("\n\n")
}

fn bar(current: i64, threshold: i64) -> String {
    let scale = threshold.max(1);
    let reached = (current.max(0).saturating_mul(BAR_CELLS) + scale / 2) / scale;
    let filled = usize::try_from(reached).unwrap_or(0).min(BAR_CELLS_USIZE);
    format!(
        "{}{}",
        "▰".repeat(filled),
        "▱".repeat(BAR_CELLS_USIZE - filled)
    )
}

fn share(progress: Option<(i64, i64)>) -> f64 {
    match progress {
        Some((_, threshold)) if threshold <= 0 => 0.0,
        #[allow(clippy::cast_precision_loss)]
        Some((current, threshold)) => current as f64 / threshold as f64,
        None => -1.0,
    }
}

fn day_of(timestamp: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(timestamp)
        .map(|parsed| parsed.format("%d.%m.%Y").to_string())
        .unwrap_or_else(|_| timestamp.to_string())
}

const MEDALS: &[&str] = &["🥇", "🥈", "🥉"];

fn top(message: &Message, store: &Store) -> String {
    let standings = store.leaderboard(message.chat.id.0).unwrap_or_default();
    if standings.is_empty() {
        return "🏆 <b>Ачивки чата</b>\n\nПока никто ничего не собрал. Терпим.".to_string();
    }
    let lines = standings
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
            format!(
                "{rank} <b>{}</b> · {} из {}",
                escape(&name),
                standing.owned,
                Achievement::ALL.len()
            )
        })
        .collect::<Vec<String>>()
        .join("\n");
    format!("🏆 <b>Ачивки чата</b>\n\n{lines}")
}

fn bred(store: &Store) -> String {
    let corpus = store.all_quotes().unwrap_or_default();
    crate::ops::markov::generate(&corpus, &mut rand::rng()).map_or_else(
        || "Нечего сказать. Терпим.".to_string(),
        |phrase| escape(&phrase),
    )
}
