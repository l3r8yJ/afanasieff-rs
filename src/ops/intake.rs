use teloxide::types::{Message, Update, UpdateKind};

use crate::ops::store::{MATTHEW_USERNAME, remember_chat, store_matthew_message, with_db};

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
    if !is_written_by_matthew(message) {
        return;
    }
    let Some(text) = message.text() else {
        return;
    };
    if is_too_short(text) {
        return;
    }
    let sent_at = message.date.to_rfc3339();
    with_db(|connection| store_matthew_message(connection, chat, message.id.0, &sent_at, text));
    log::info!(
        "matthew message '{}' collected in chat '{chat}'",
        message.id.0
    );
}

fn is_written_by_matthew(message: &Message) -> bool {
    message.from.as_ref().is_some_and(|user| {
        user.username
            .as_deref()
            .is_some_and(|username| username.eq_ignore_ascii_case(MATTHEW_USERNAME))
    })
}

fn is_too_short(str: &str) -> bool {
    str.len() > 10
}
