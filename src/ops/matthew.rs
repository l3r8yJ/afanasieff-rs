use std::sync::Arc;

use rand::{Rng, rng};
use teloxide::{Bot, types::Message};

use crate::ops::{
    consts::{MATTHEW_KEYWORD, MATTHEW_SOURCE},
    error::Error,
    predicates::contains_ignore_case,
    send::send_reply_message_set_reaction,
    store::Store,
};

#[must_use]
pub fn filter(msg: &Message) -> bool {
    contains_ignore_case(msg, MATTHEW_KEYWORD)
}

/// Sends a random matthew quote with a 30% chance.
///
/// # Errors
///
/// Returns an error when the quote cannot be read from the store.
pub async fn send_random_matthew_quote(
    bot: Bot,
    message: Message,
    store: Arc<Store>,
) -> Result<(), Error> {
    if should_reply() {
        reply_with_quote(&bot, &message, &store).await?;
    }
    Ok(())
}

/// Sends a random matthew quote unconditionally.
///
/// # Errors
///
/// Returns an error when the quote cannot be read from the store.
pub async fn reply_with_quote(bot: &Bot, message: &Message, store: &Store) -> Result<(), Error> {
    if let Some(quote) = store.random_quote(MATTHEW_SOURCE)? {
        send_reply_message_set_reaction(&quote, "💔", bot, message).await;
    }
    Ok(())
}

fn should_reply() -> bool {
    let mut rng = rng();
    rng.random_bool(0.3)
}
