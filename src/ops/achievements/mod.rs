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
    log::debug!(
        "message '{}' in chat '{}' by '{}': mat '{}', apology '{}', politics '{}', \
         laugh '{}', call '{}', len '{}'",
        event.message_id,
        event.chat,
        event.user,
        event.mat,
        event.apology,
        event.politics,
        event.laugh_only,
        event.call_to_play,
        event.len
    );
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

pub fn record_bot_reply(store: &Store, message: &teloxide::types::Message) {
    let Some(author) = message.from.as_ref() else {
        return;
    };
    let chat = message.chat.id.0;
    let user = i64::try_from(author.id.0).unwrap_or(i64::MAX);
    if let Err(error) = store.bump(chat, user, "bot_replies", 1) {
        log::error!("bot reply to member '{user}' in chat '{chat}' was not counted: '{error}'");
    }
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
        &event.first_name,
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
