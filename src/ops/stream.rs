use teloxide::{
    Bot,
    types::{Me, Message},
};

use crate::ops::{
    consts::{STREAM_KEYWORD, STREAM_SOURCE},
    error::Error,
    predicates::contains_ignore_case,
    send::send_reply_message_set_reaction,
    store::{random_quote, with_db},
};

#[must_use]
pub fn filter(msg: &Message) -> bool {
    contains_ignore_case(msg, STREAM_KEYWORD)
}

/// Send random stream message.
///
/// # Errors
///
/// This function will return an error if message was empty.
pub async fn send_random_stream_quote(bot: Bot, message: Message, me: Me) -> Result<(), Error> {
    if let Some(quote) = with_db(|connection| random_quote(connection, STREAM_SOURCE)).flatten() {
        send_reply_message_set_reaction(&quote, "🤡", &bot, &message, &me).await;
    }
    Ok(())
}
