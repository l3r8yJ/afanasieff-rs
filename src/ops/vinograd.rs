use teloxide::{
    Bot,
    types::{Me, Message},
};

use crate::ops::{
    consts::{VINOGRAD_KEYWORD, VINOGRAD_SOURCE},
    error::Error,
    predicates::contains_ignore_case,
    send::send_reply_message_set_reaction,
    store::{random_quote, with_db},
};

/// Sends random vinograd quote.
///
/// # Errors
///
/// This function will return an error if message text is empty.
pub async fn send_random_vinograd_quote(bot: Bot, message: Message, me: Me) -> Result<(), Error> {
    if let Some(quote) = with_db(|connection| random_quote(connection, VINOGRAD_SOURCE)).flatten() {
        send_reply_message_set_reaction(&quote, "💩", &bot, &message, &me).await;
    }
    Ok(())
}

#[must_use]
pub fn filter(msg: &Message) -> bool {
    contains_ignore_case(msg, VINOGRAD_KEYWORD)
}
