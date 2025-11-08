use teloxide::payloads::SetMessageReactionSetters;
use teloxide::prelude::Requester;
use teloxide::sugar::request::RequestReplyExt;
use teloxide::types::ReactionType;
use teloxide::{
    Bot,
    types::{Me, Message},
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
    let _ = tokio::join!(
        bot.send_message(message.chat.id, text).reply_to(message.id),
        bot.set_message_reaction(message.chat.id, message.id)
            .reaction(vec![ReactionType::Emoji {
                emoji: emoji.to_string(),
            }]),
    );
}
