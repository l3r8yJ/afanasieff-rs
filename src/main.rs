#![allow(clippy::multiple_crate_versions)]

pub mod cron;
pub mod ops;

use chrono::Utc;
use teloxide::{
    Bot,
    dispatching::UpdateFilterExt,
    dptree,
    prelude::Dispatcher,
    types::{Message, Update, UpdateKind},
};

use crate::ops::{
    matthew::send_random_matthew_quote, stream::send_random_stream_quote,
    vinograd::send_random_vinograd_quote,
};

const FIVE_MINS: f32 = 5.0 * 60.0;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting the bot...");
    let bot = Bot::from_env();
    tokio::spawn(cron::quote_per_hour::start_cron(bot.clone()));
    let main_branch = dptree::entry()
        .inspect(cron::quote_per_hour::put_id_into_pool)
        .filter(|u: Update| match u.kind {
            UpdateKind::Message(m) => {
                let now = Utc::now();
                let is_too_old = now.signed_duration_since(m.date).as_seconds_f32() > FIVE_MINS;
                log::info!("message is_too_old: '{}'", is_too_old);
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
        );
    Dispatcher::builder(bot, main_branch)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
