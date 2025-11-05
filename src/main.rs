pub mod ops;

use teloxide::{Bot, dispatching::UpdateFilterExt, prelude::Dispatcher, types::Update};

use crate::ops::stream::process_stream_msg;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting the bot...");
    let bot = Bot::from_env();
    let schema = Update::filter_message().endpoint(process_stream_msg);
    Dispatcher::builder(bot, schema).build().dispatch().await;
}
