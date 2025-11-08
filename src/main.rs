pub mod ops;

use teloxide::{
    Bot,
    dispatching::UpdateFilterExt,
    dptree,
    prelude::Dispatcher,
    types::{Message, Update},
};

use crate::ops::{
    consts::{MATTHEW_KEYWORD, STREAM_KEYWORD, VINOGRAD_KEYWORD},
    matthew::send_random_matthew_quote,
    stream::send_random_stream_quote,
    vinograd::send_random_vinograd_quote,
};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting the bot...");
    let bot = Bot::from_env();
    let schema = Update::filter_message()
        .branch(
            dptree::entry()
                .filter(|msg: Message| {
                    msg.text()
                        .is_some_and(|t| t.to_lowercase().contains(STREAM_KEYWORD))
                })
                .endpoint(send_random_stream_quote),
        )
        .branch(
            dptree::entry()
                .filter(|msg: Message| {
                    msg.text()
                        .is_some_and(|t| t.to_lowercase().contains(MATTHEW_KEYWORD))
                })
                .endpoint(send_random_matthew_quote),
        )
        .branch(
            dptree::entry()
                .filter(|msg: Message| {
                    msg.text()
                        .is_some_and(|t| t.to_lowercase().contains(VINOGRAD_KEYWORD))
                })
                .endpoint(send_random_vinograd_quote),
        );
    Dispatcher::builder(bot, schema).build().dispatch().await;
}
