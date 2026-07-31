#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::panic))]
#![deny(clippy::await_holding_lock)]
#![allow(clippy::multiple_crate_versions)]

use std::path::PathBuf;
use std::sync::Arc;

use afanasieff_rs::ops::store::Store;
use afanasieff_rs::{cron, handler_tree};
use teloxide::{Bot, dptree, prelude::Dispatcher};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting the bot...");
    let bot = Bot::from_env();
    let path = database_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("the database directory is creatable");
    }
    let store = Arc::new(Store::open(&path).expect("the database opens and migrates at startup"));
    tokio::spawn(cron::quote_per_hour::start_cron(
        bot.clone(),
        Arc::clone(&store),
    ));
    tokio::spawn(cron::messages_to_quotes::start_promoting(Arc::clone(
        &store,
    )));
    Dispatcher::builder(bot, handler_tree())
        .dependencies(dptree::deps![store])
        .default_handler(|update| async move {
            log::debug!("unhandled update: '{update:?}'");
        })
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn database_path() -> PathBuf {
    if let Ok(path) = std::env::var("AFANASIEFF_DB") {
        return PathBuf::from(path);
    }
    let home = std::env::var("HOME").expect("HOME is set for the service user");
    PathBuf::from(home).join(".local/state/afanasieff/afanasieff.db")
}
