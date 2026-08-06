pub mod apply;
pub mod event;
pub mod rules;
pub mod text;
pub mod words;

use teloxide::Bot;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::sugar::request::RequestReplyExt;
use teloxide::types::{ChatId, MessageId, ParseMode, Update, UserId};
use teloxide::utils::html::{escape, user_mention};

use crate::ops::achievements::event::Event;
use crate::ops::achievements::rules::{Achievement, Stats, unlocked};
use crate::ops::store::Store;

pub async fn track_and_award(bot: &Bot, store: &Store, update: &Update) {
    let Some(event) = Event::parse(update) else {
        return;
    };
    match apply::apply(store, &event) {
        Ok(false) => {
            log::debug!(
                "message '{}' in chat '{}' was already counted, skipping",
                event.message_id,
                event.chat
            );
            return;
        }
        Ok(true) => {}
        Err(error) => {
            log::error!(
                "message '{}' in chat '{}' was not counted: '{error}'",
                event.message_id,
                event.chat
            );
            return;
        }
    }
    for achievement in earned(store, &event) {
        award(bot, store, &event, achievement).await;
    }
}

fn earned(store: &Store, event: &Event) -> Vec<Achievement> {
    let stats = match store.stats(event.chat, event.user) {
        Ok(stats) => Stats::new(stats),
        Err(error) => {
            log::error!("stats of member '{}' were not read: '{error}'", event.user);
            return Vec::new();
        }
    };
    let owned = match store.owned(event.chat, event.user) {
        Ok(owned) => owned,
        Err(error) => {
            log::error!(
                "achievements of member '{}' were not read: '{error}'",
                event.user
            );
            return Vec::new();
        }
    };
    unlocked(&stats, &owned)
}

async fn award(bot: &Bot, store: &Store, event: &Event, achievement: Achievement) {
    let given = store.unlock(
        event.chat,
        event.user,
        achievement.code(),
        &event.created_at.to_rfc3339(),
    );
    match given {
        Ok(true) => {}
        Ok(false) => return,
        Err(error) => {
            log::error!(
                "achievement '{}' of member '{}' was not stored: '{error}'",
                achievement.code(),
                event.user
            );
            return;
        }
    }
    let mention = user_mention(
        UserId(u64::try_from(event.user).unwrap_or_default()),
        &escape(&event.first_name),
    );
    let text = format!(
        "🏆 {mention} — «{}»\n{}",
        escape(achievement.title()),
        escape(achievement.text())
    );
    let sent = bot
        .send_message(ChatId(event.chat), text)
        .parse_mode(ParseMode::Html)
        .reply_to(MessageId(event.message_id))
        .await;
    match sent {
        Ok(_) => log::info!(
            "achievement '{}' announced to member '{}'",
            achievement.code(),
            event.user
        ),
        Err(error) => log::error!(
            "achievement '{}' of member '{}' was not announced: '{error}'",
            achievement.code(),
            event.user
        ),
    }
}
