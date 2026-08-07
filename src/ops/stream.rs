use std::sync::Arc;

use rand::{Rng, rng};
use teloxide::{Bot, types::Message};

use crate::ops::{
    achievements::record_bot_reply,
    consts::{STREAM_KEYWORD, STREAM_SOURCE},
    predicates::contains_ignore_case,
    send::send_reply_message_set_reaction,
    store::Store,
};

#[must_use]
pub fn filter(msg: &Message) -> bool {
    contains_ignore_case(msg, STREAM_KEYWORD)
}

/// Sends a random stream quote.
///
/// # Errors
///
/// Returns an error when the quote cannot be read from the store.
pub async fn send_random_stream_quote(
    bot: Bot,
    message: Message,
    store: Arc<Store>,
) -> anyhow::Result<()> {
    if rng().random_bool(crate::ops::chance::generated_on_keyword()) {
        let corpus = store.all_quotes()?;
        let phrase = crate::ops::markov::generate(&corpus, &mut rng());
        if let Some(phrase) = phrase {
            send_reply_message_set_reaction(&phrase, "🤡", &bot, &message, &store, None).await;
            return Ok(());
        }
    }
    if let Some((id, quote)) = store.random_quote_with_id(STREAM_SOURCE)? {
        send_reply_message_set_reaction(&quote, "🤡", &bot, &message, &store, Some(id)).await;
        record_bot_reply(&store, &message);
    }
    Ok(())
}
