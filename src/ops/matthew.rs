use rand::{Rng, rng};
use teloxide::{
    Bot,
    types::{Me, Message},
};

use crate::ops::{
    consts::{MATTHEW_KEYWORD, MATTHEW_SOURCE},
    error::Error,
    predicates::contains_ignore_case,
    send::send_reply_message_set_reaction,
    store::{random_quote, with_db},
};

#[must_use]
pub fn filter(msg: &Message) -> bool {
    contains_ignore_case(msg, MATTHEW_KEYWORD)
}

/// Send random quote with 30% chance.
///
/// # Errors
///
/// This function will return an error if message text was empty.
pub async fn send_random_matthew_quote(bot: Bot, message: Message, me: Me) -> Result<(), Error> {
    if should_reply()
        && let Some(quote) =
            with_db(|connection| random_quote(connection, MATTHEW_SOURCE)).flatten()
    {
        send_reply_message_set_reaction(&quote, "💔", &bot, &message, &me).await;
    }
    Ok(())
}

/// Return true with 30% chance.
fn should_reply() -> bool {
    let mut rng = rng();
    rng.random_bool(0.3) // 30% chance for reply (as irl)
}
