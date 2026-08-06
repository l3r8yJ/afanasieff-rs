use std::sync::Arc;

use teloxide::{Bot, types::Message};

use crate::ops::{
    achievements::record_bot_reply,
    consts::{VINOGRAD_KEYWORD, VINOGRAD_SOURCE},
    error::Error,
    predicates::contains_ignore_case,
    send::send_reply_message_set_reaction,
    store::Store,
};

/// Sends a random vinograd quote.
///
/// # Errors
///
/// Returns an error when the quote cannot be read from the store.
pub async fn send_random_vinograd_quote(
    bot: Bot,
    message: Message,
    store: Arc<Store>,
) -> Result<(), Error> {
    if let Some(quote) = store.random_quote(VINOGRAD_SOURCE)? {
        send_reply_message_set_reaction(&quote, "💩", &bot, &message).await;
        record_bot_reply(&store, &message);
    }
    Ok(())
}

#[must_use]
pub fn filter(msg: &Message) -> bool {
    contains_ignore_case(msg, VINOGRAD_KEYWORD)
}
