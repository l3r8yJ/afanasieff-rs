use std::sync::Arc;

use teloxide::Bot;
use teloxide::macros::BotCommands;
use teloxide::prelude::Requester;
use teloxide::types::Message;

use crate::ops::achievements::rules::{Achievement, Stats};
use crate::ops::error::Error;
use crate::ops::store::Store;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "snake_case")]
pub enum Command {
    Achievements,
    MyAchievements,
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
    };
    bot.send_message(message.chat.id, text).await?;
    Ok(())
}

fn catalogue() -> String {
    let lines = Achievement::ALL
        .iter()
        .map(|achievement| format!("{} — {}", achievement.title(), achievement.hint()))
        .collect::<Vec<String>>()
        .join("\n");
    format!("Ачивки, {} штук\n\n{lines}", Achievement::ALL.len())
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
            unlocked.push(format!("🏆 {} · {}", achievement.title(), day_of(&at)));
        } else {
            locked.push((achievement, achievement.progress(&stats)));
        }
    }
    locked.sort_by(|left, right| share(right.1).total_cmp(&share(left.1)));
    let locked = locked
        .iter()
        .map(|(achievement, progress)| match progress {
            Some((current, threshold)) => {
                format!("🔒 {} · {current}/{threshold}", achievement.title())
            }
            None => format!("🔒 {}", achievement.title()),
        })
        .collect::<Vec<String>>();
    let header = format!(
        "🏆 {} — {}/{}",
        author.first_name,
        owned.len(),
        Achievement::ALL.len()
    );
    [header, unlocked.join("\n"), locked.join("\n")]
        .into_iter()
        .filter(|section| !section.is_empty())
        .collect::<Vec<String>>()
        .join("\n\n")
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
