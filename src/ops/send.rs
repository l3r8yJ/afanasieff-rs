use teloxide::payloads::SetMessageReactionSetters;
use teloxide::prelude::Requester;
use teloxide::sugar::request::RequestReplyExt;
use teloxide::types::ReactionType;
use teloxide::{Bot, types::Message};

use crate::ops::store::Store;

/// Replies to given message, sets a reaction on it, and remembers which quote
/// the reply carried, so a reaction on that reply can be counted later.
pub async fn send_reply_message_set_reaction(
    text: &str,
    emoji: &str,
    bot: &Bot,
    message: &Message,
    store: &Store,
    quote: Option<i64>,
) {
    let (sent, reaction) = tokio::join!(
        bot.send_message(message.chat.id, text).reply_to(message.id),
        bot.set_message_reaction(message.chat.id, message.id)
            .reaction(vec![ReactionType::Emoji {
                emoji: emoji.to_string(),
            }]),
    );
    match &sent {
        Ok(msg) => {
            log::info!("message '{msg:?}' was successfully sent");
            remember(store, msg, quote);
        }
        Err(err) => log::error!("message failed: '{err:?}'"),
    }
    match reaction {
        Ok(react) => log::info!("reaction was set: '{react:?}'"),
        Err(err) => log::error!("reaction failed: '{err:?}'"),
    }
}

pub(crate) fn remember(store: &Store, sent: &Message, quote: Option<i64>) {
    let Some(author) = sent.from.as_ref() else {
        return;
    };
    let user = i64::try_from(author.id.0).unwrap_or(i64::MAX);
    if let Err(error) = store.remember_message(sent.chat.id.0, sent.id.0, user, quote) {
        log::error!(
            "sent message '{}' was not remembered: '{error:#}'",
            sent.id.0
        );
    }
}
