use std::sync::Arc;

use teloxide::types::MessageReactionUpdated;

use crate::ops::store::{MATTHEW_USERNAME, Store};

/// Counts a reaction: it raises the score of a quote, resets the streak of the
/// member who was reacted to, and pushes a message Matthew wrote into the
/// quotes ahead of the queue.
///
/// # Errors
///
/// Never returns an error; failures are logged so one bad reaction cannot stop
/// the dispatcher.
pub async fn observe(store: Arc<Store>, reaction: MessageReactionUpdated) -> anyhow::Result<()> {
    if reaction.new_reaction.is_empty() {
        log::debug!(
            "reaction on message '{}' was taken back",
            reaction.message_id.0
        );
        return Ok(());
    }
    let chat = reaction.chat.id.0;
    let message = reaction.message_id.0;
    let owner = match store.message_owner(chat, message) {
        Ok(Some(owner)) => owner,
        Ok(None) => {
            log::debug!("message '{message}' in chat '{chat}' is not remembered, skipping");
            return Ok(());
        }
        Err(error) => {
            log::error!("owner of message '{message}' in chat '{chat}' was not read: '{error:#}'");
            return Ok(());
        }
    };
    if let Some(quote) = owner.quote
        && reaction.old_reaction.is_empty()
        && let Err(error) = store.bump_quote_score(quote)
    {
        log::error!("score of quote '{quote}' was not raised: '{error:#}'");
    }
    let actor = reaction
        .user()
        .map(|user| i64::try_from(user.id.0).unwrap_or(i64::MAX));
    if actor != Some(owner.user)
        && !is_bot(&store, chat, owner.user)
        && let Err(error) = store.set_stat(chat, owner.user, "unanswered_streak", 0)
    {
        log::error!(
            "streak of member '{}' was not reset: '{error:#}'",
            owner.user
        );
    }
    if owner.quote.is_none() && wrote_by_matthew(&store, chat, owner.user) {
        promote(&store, chat, message, owner.user);
    }
    Ok(())
}

fn promote(store: &Store, chat: i64, message: i32, user: i64) {
    match store.promote_matthew_message(chat, message) {
        Ok(Some(quote)) => {
            log::info!("message '{message}' in chat '{chat}' promoted by a reaction");
            if let Err(error) = store.remember_message(chat, message, user, Some(quote)) {
                log::error!("promoted message '{message}' was not remembered: '{error:#}'");
            }
        }
        Ok(None) => log::debug!("message '{message}' in chat '{chat}' was already promoted"),
        Err(error) => log::error!("message '{message}' was not promoted: '{error:#}'"),
    }
}

fn wrote_by_matthew(store: &Store, chat: i64, user: i64) -> bool {
    store
        .member_username(chat, user)
        .ok()
        .flatten()
        .is_some_and(|username| username.eq_ignore_ascii_case(MATTHEW_USERNAME))
}

fn is_bot(store: &Store, chat: i64, user: i64) -> bool {
    match store.is_member(chat, user) {
        Ok(is_member) => !is_member,
        Err(error) => {
            log::error!("membership of user '{user}' in chat '{chat}' was not read: '{error:#}'");
            false
        }
    }
}
