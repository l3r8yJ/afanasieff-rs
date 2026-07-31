use teloxide::types::{Message, Update, UpdateKind};

use crate::ops::store::{MATTHEW_USERNAME, remember_chat, store_matthew_message, with_db};

const SHORT_MESSAGE_CHARS: usize = 10;

const PREVIEW_CHARS: usize = 60;

pub fn observe(update: Update) {
    let UpdateKind::Message(message) = update.kind else {
        return;
    };
    let chat = message.chat.id.0;
    with_db(|connection| remember_chat(connection, chat));
    log::info!("chat id: '{chat}' remembered");
    collect_matthew_message(&message, chat);
}

fn collect_matthew_message(message: &Message, chat: i64) {
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
    match with_db(|connection| store_matthew_message(connection, chat, id, &sent_at, text)) {
        Some(true) => log::info!(
            "matthew message '{id}' in chat '{chat}' collected: '{}'",
            preview(text)
        ),
        Some(false) => {
            log::info!("matthew message '{id}' in chat '{chat}' was already collected, skipping");
        }
        None => log::error!("matthew message '{id}' in chat '{chat}' was not collected"),
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
