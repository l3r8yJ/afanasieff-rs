use std::collections::HashMap;

use crate::ops::achievements::event::{Event, Mention};
use crate::ops::store::Store;

const NIGHT_FROM_HOUR: u32 = 2;
const NIGHT_TO_HOUR: u32 = 6;
const LONG_MESSAGE_CHARS: usize = 300;
const APOLOGY_WINDOW_SECONDS: i64 = 60;
const PING_TIMEOUT_SECONDS: i64 = 3600;
const CALL_TIMEOUT_SECONDS: i64 = 600;
const CHAIN_WINDOW_SECONDS: i64 = 1800;
const MONOLOGUE_RUN: i64 = 5;

/// Applies the event to the counters of its author and to the chat state.
///
/// Returns `false` when the message was already counted, which happens when
/// Telegram redelivers an update after a restart.
///
/// # Errors
///
/// Returns an error when a statement cannot be executed.
pub fn apply(store: &Store, event: &Event) -> anyhow::Result<bool> {
    let state = store.state(event.chat)?;
    let seen = state.get("last_message_id").copied().unwrap_or_default();
    if i64::from(event.message_id) <= seen {
        return Ok(false);
    }
    store.upsert_member(
        event.chat,
        event.user,
        event.username.as_deref(),
        &event.first_name,
        &event.created_at.to_rfc3339(),
    )?;
    let targets = targets(store, event)?;
    count_message(store, event)?;
    count_words(store, event)?;
    settle_apology(store, event)?;
    settle_streak(store, event)?;
    settle_ping(store, event, &targets)?;
    settle_call(store, event, &state)?;
    settle_monologue(store, event, &state)?;
    settle_chain(store, event, &state)?;
    count_mentions(store, event, &targets)?;
    store.set_state(event.chat, "last_message_id", i64::from(event.message_id))?;
    Ok(true)
}

fn count_message(store: &Store, event: &Event) -> anyhow::Result<()> {
    if event.hour_msk >= NIGHT_FROM_HOUR && event.hour_msk < NIGHT_TO_HOUR {
        store.bump(event.chat, event.user, "night_messages", 1)?;
    }
    if event.len > LONG_MESSAGE_CHARS {
        store.bump(event.chat, event.user, "long_messages", 1)?;
    }
    let length = i64::try_from(event.len).unwrap_or(i64::MAX);
    if length > store.stat(event.chat, event.user, "longest_message")? {
        store.set_stat(event.chat, event.user, "longest_message", length)?;
    }
    Ok(())
}

fn count_words(store: &Store, event: &Event) -> anyhow::Result<()> {
    if event.politics {
        store.bump(event.chat, event.user, "politics", 1)?;
    }
    if event.laugh_only {
        store.bump(event.chat, event.user, "laugh_only", 1)?;
    }
    if event.stream {
        store.bump(event.chat, event.user, "stream_mentions", 1)?;
    }
    if event.vinograd {
        store.bump(event.chat, event.user, "vinograd_mentions", 1)?;
    }
    if event.reply_to_bot {
        store.bump(event.chat, event.user, "replies_to_bot", 1)?;
    }
    Ok(())
}

fn settle_apology(store: &Store, event: &Event) -> anyhow::Result<()> {
    let now = event.created_at.timestamp();
    if event.apology {
        let last = store.stat(event.chat, event.user, "last_mat_at")?;
        if last > 0 && now - last <= APOLOGY_WINDOW_SECONDS {
            store.bump(event.chat, event.user, "apologies", 1)?;
            store.set_stat(event.chat, event.user, "last_mat_at", 0)?;
            return Ok(());
        }
    }
    if event.mat {
        store.set_stat(event.chat, event.user, "last_mat_at", now)?;
    }
    Ok(())
}

fn settle_streak(store: &Store, event: &Event) -> anyhow::Result<()> {
    store.bump(event.chat, event.user, "unanswered_streak", 1)?;
    if let Some(target) = event.reply_to_user
        && target != event.user
    {
        store.set_stat(event.chat, target, "unanswered_streak", 0)?;
    }
    Ok(())
}

fn settle_ping(store: &Store, event: &Event, targets: &[i64]) -> anyhow::Result<()> {
    let now = event.created_at.timestamp();
    let pinged_at = store.stat(event.chat, event.user, "pinged_at")?;
    if pinged_at > 0 {
        if now - pinged_at > PING_TIMEOUT_SECONDS {
            store.bump(event.chat, event.user, "ignored_pings", 1)?;
        }
        store.set_stat(event.chat, event.user, "pinged_at", 0)?;
    }
    for target in targets {
        if *target != event.user && store.stat(event.chat, *target, "pinged_at")? == 0 {
            store.set_stat(event.chat, *target, "pinged_at", now)?;
        }
    }
    Ok(())
}

fn settle_call(store: &Store, event: &Event, state: &HashMap<String, i64>) -> anyhow::Result<()> {
    let now = event.created_at.timestamp();
    let caller = state.get("call_by").copied().unwrap_or_default();
    let called_at = state.get("call_at").copied().unwrap_or_default();
    if caller != 0 {
        if now - called_at > CALL_TIMEOUT_SECONDS {
            store.bump(event.chat, caller, "unanswered_calls", 1)?;
            store.set_state(event.chat, "call_by", 0)?;
        } else if caller != event.user {
            store.set_state(event.chat, "call_by", 0)?;
        }
    }
    if event.call_to_play {
        store.set_state(event.chat, "call_by", event.user)?;
        store.set_state(event.chat, "call_at", now)?;
    }
    Ok(())
}

fn settle_monologue(
    store: &Store,
    event: &Event,
    state: &HashMap<String, i64>,
) -> anyhow::Result<()> {
    let previous = state.get("last_user_id").copied().unwrap_or_default();
    let run = if previous == event.user {
        state.get("run_len").copied().unwrap_or_default() + 1
    } else {
        1
    };
    store.set_state(event.chat, "last_user_id", event.user)?;
    store.set_state(event.chat, "run_len", run)?;
    if run == MONOLOGUE_RUN {
        store.bump(event.chat, event.user, "monologues", 1)?;
    }
    Ok(())
}

fn settle_chain(store: &Store, event: &Event, state: &HashMap<String, i64>) -> anyhow::Result<()> {
    let Some(target) = event.reply_to_user else {
        return Ok(());
    };
    let now = event.created_at.timestamp();
    let first = state.get("chain_a").copied().unwrap_or_default();
    let second = state.get("chain_b").copied().unwrap_or_default();
    let started = state.get("chain_started_at").copied().unwrap_or_default();
    let same_pair =
        (first == event.user && second == target) || (first == target && second == event.user);
    let length = if same_pair && now - started <= CHAIN_WINDOW_SECONDS {
        state.get("chain_len").copied().unwrap_or_default() + 1
    } else {
        store.set_state(event.chat, "chain_a", event.user)?;
        store.set_state(event.chat, "chain_b", target)?;
        store.set_state(event.chat, "chain_started_at", now)?;
        1
    };
    store.set_state(event.chat, "chain_len", length)?;
    store.set_stat(event.chat, event.user, "chain_len", length)?;
    store.set_stat(event.chat, target, "chain_len", length)?;
    Ok(())
}

fn count_mentions(store: &Store, event: &Event, targets: &[i64]) -> anyhow::Result<()> {
    for target in targets {
        if *target != event.user {
            store.bump(event.chat, event.user, &format!("mention:{target}"), 1)?;
        }
    }
    Ok(())
}

fn targets(store: &Store, event: &Event) -> anyhow::Result<Vec<i64>> {
    let mut targets = event.reply_to_user.into_iter().collect::<Vec<i64>>();
    for mention in &event.mentions {
        match mention {
            Mention::Id(id) => targets.push(*id),
            Mention::Username(username) => {
                if let Some(id) = store.member_by_username(event.chat, username)? {
                    targets.push(id);
                }
            }
        }
    }
    targets.sort_unstable();
    targets.dedup();
    Ok(targets)
}
