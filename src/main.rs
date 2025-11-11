#![allow(clippy::multiple_crate_versions)]

pub mod cron;
pub mod ops;

use teloxide::{
    Bot,
    dispatching::UpdateFilterExt,
    dptree,
    prelude::Dispatcher,
    types::{Message, Update},
};

use crate::ops::{
    matthew::send_random_matthew_quote, stream::send_random_stream_quote,
    vinograd::send_random_vinograd_quote,
};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting the bot...");
    let bot = Bot::from_env();
    cron::quote_per_hour::start_cron(bot.clone());
    let main_branch = dptree::entry()
        .inspect(cron::quote_per_hour::put_id_into_pool)
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
