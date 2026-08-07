use teloxide::types::{Message, Update, UpdateKind};

use crate::ops::store::{MATTHEW_USERNAME, Store};

const SHORT_MESSAGE_CHARS: usize = 10;

const PREVIEW_CHARS: usize = 60;

pub fn observe(store: &Store, update: Update) {
    let UpdateKind::Message(message) = update.kind else {
        return;
    };
    let chat = message.chat.id.0;
    if let Err(error) = store.remember_chat(chat) {
        log::error!("chat id: '{chat}' was not remembered: '{error}'");
        return;
    }
    log::info!("chat id: '{chat}' remembered");
    if let Some(author) = message.from.as_ref() {
        let user = i64::try_from(author.id.0).unwrap_or(i64::MAX);
        if let Err(error) = store.remember_message(chat, message.id.0, user, None) {
            log::error!(
                "message '{}' in chat '{chat}' was not remembered: '{error}'",
                message.id.0
            );
        }
    }
    collect_matthew_message(store, &message, chat);
}

fn collect_matthew_message(store: &Store, message: &Message, chat: i64) {
    let id = message.id.0;
    if !is_written_by_matthew(message) {
        log::debug!("message '{id}' in chat '{chat}' is not written by matthew, skipping");
        return;
    }
    let Some(text) = message.text() else {
        log::info!("matthew message '{id}' in chat '{chat}' has no text, skipping");
        return;
    };
    let length = text.chars().count();
    if length <= SHORT_MESSAGE_CHARS {
        log::info!(
            "matthew message '{id}' in chat '{chat}' is '{length}' chars long, \
             not longer than '{SHORT_MESSAGE_CHARS}', skipping"
        );
        return;
    }
    let sent_at = message.date.to_rfc3339();
    match store.store_matthew_message(chat, id, &sent_at, text) {
        Ok(true) => log::info!(
            "matthew message '{id}' in chat '{chat}' collected: '{}'",
            preview(text)
        ),
        Ok(false) => {
            log::info!("matthew message '{id}' in chat '{chat}' was already collected, skipping");
        }
        Err(error) => {
            log::error!("matthew message '{id}' in chat '{chat}' was not collected: '{error}'");
        }
    }
}

fn is_written_by_matthew(message: &Message) -> bool {
    message.from.as_ref().is_some_and(|user| {
        user.username
            .as_deref()
            .is_some_and(|username| username.eq_ignore_ascii_case(MATTHEW_USERNAME))
    })
}

pub fn preview(text: &str) -> String {
    let shortened: String = text.chars().take(PREVIEW_CHARS).collect();
    if shortened.chars().count() < text.chars().count() {
        format!("{shortened}...")
    } else {
        shortened
    }
}
