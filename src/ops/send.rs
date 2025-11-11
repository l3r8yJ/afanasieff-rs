use teloxide::payloads::SetMessageReactionSetters;
use teloxide::prelude::Requester;
use teloxide::sugar::request::RequestReplyExt;
use teloxide::types::ReactionType;
use teloxide::{
    types::{Me, Message},
    Bot,
};

/// Replies to given message and set reaction on it.
///
/// # Panics
///
/// Panics if text in message was empty.
///
/// # Errors
///
/// This function will return an error if message was empty.
pub async fn send_reply_message_set_reaction(
    text: &str,
    emoji: &str,
    bot: &Bot,
    message: &Message,
    _: &Me,
) {
    let (message, reaction) = tokio::join!(
        bot.send_message(message.chat.id, text).reply_to(message.id),
        bot.set_message_reaction(message.chat.id, message.id)
            .reaction(vec![ReactionType::Emoji {
                emoji: emoji.to_string(),
            }]),
    );
    match message {
        Ok(msg) => log::info!("message '{msg:?}' was successfully sent"),
        Err(err) => log::error!("message failed: '{err:?}'"),
    }
    match reaction {
        Ok(react) => log::info!("reaction was set: '{react:?}'"),
        Err(err) => log::error!("reaction failed: '{err:?}'"),
    }
}
