#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::panic))]
#![deny(clippy::await_holding_lock)]
#![allow(clippy::multiple_crate_versions)]

use std::sync::Arc;

use chrono::Utc;
use teloxide::Bot;
use teloxide::dispatching::{UpdateFilterExt, UpdateHandler};
use teloxide::dptree;
use teloxide::types::{Message, Update, UpdateKind};

use crate::ops::error::Error;
use crate::ops::matthew::send_random_matthew_quote;
use crate::ops::store::Store;
use crate::ops::stream::send_random_stream_quote;
use crate::ops::vinograd::send_random_vinograd_quote;

pub mod cron;
pub mod ops;

const FIVE_MINS: f32 = 5.0 * 60.0;

/// Returns the dispatcher tree: it records every incoming chat, drops
/// messages older than five minutes, and replies with a random quote on
/// the stream, matthew or vinograd keyword branches.
pub fn handler_tree() -> UpdateHandler<Error> {
    dptree::entry()
        .inspect(|update: Update, store: Arc<Store>| ops::intake::observe(&store, update))
        .inspect_async(|update: Update, bot: Bot, store: Arc<Store>| async move {
            ops::achievements::track_and_award(&bot, &store, &update).await;
        })
        .filter(|u: Update| match u.kind {
            UpdateKind::Message(m) => {
                let now = Utc::now();
                let is_too_old = now.signed_duration_since(m.date).as_seconds_f32() > FIVE_MINS;
                log::info!("message is_too_old: '{is_too_old}'");
                if is_too_old {
                    log::info!("message to old skipping...")
                }
                !is_too_old
            }
            _ => false,
        })
        .branch(
            Update::filter_message()
                .filter(|m: Message| ops::stream::filter(&m))
                .endpoint(send_random_stream_quote),
        )
        .branch(
            Update::filter_message()
                .filter(|m: Message| ops::matthew::filter(&m))
                .endpoint(send_random_matthew_quote),
        )
        .branch(
            Update::filter_message()
                .filter(|m: Message| ops::vinograd::filter(&m))
                .endpoint(send_random_vinograd_quote),
        )
}
