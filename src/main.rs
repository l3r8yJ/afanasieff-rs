#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::panic))]
#![deny(clippy::await_holding_lock)]
#![allow(clippy::multiple_crate_versions)]

use std::path::PathBuf;
use std::sync::Arc;

use afanasieff_rs::ops::commands::Command;
use afanasieff_rs::ops::store::Store;
use afanasieff_rs::{cron, handler_tree};
use anyhow::Context;
use teloxide::prelude::Requester;
use teloxide::utils::command::BotCommands;
use teloxide::{Bot, dptree, prelude::Dispatcher};
use tokio::signal::unix::{SignalKind, signal};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    log::info!("Starting the bot...");
    let bot = Bot::from_env();
    let path = database_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating the database directory '{}'", parent.display()))?;
    }
    let store = Arc::new(Store::open(&path)?);
    match bot.set_my_commands(Command::bot_commands()).await {
        Ok(_) => log::info!("commands registered: '{:?}'", Command::bot_commands()),
        Err(error) => log::error!("commands were not registered: '{error:#}'"),
    }
    let shutdown = CancellationToken::new();
    let hourly = tokio::spawn(cron::quote_per_hour::start_cron(
        bot.clone(),
        Arc::clone(&store),
        shutdown.clone(),
    ));
    let promoting = tokio::spawn(cron::messages_to_quotes::start_promoting(
        Arc::clone(&store),
        shutdown.clone(),
    ));
    let mut dispatcher = Dispatcher::builder(bot, handler_tree())
        .dependencies(dptree::deps![store])
        .default_handler(|update| async move {
            log::debug!("unhandled update: '{update:?}'");
        })
        .enable_ctrlc_handler()
        .build();
    let mut terminate =
        signal(SignalKind::terminate()).context("installing the SIGTERM handler")?;
    tokio::select! {
        () = dispatcher.dispatch() => {},
        _ = terminate.recv() => log::info!("received SIGTERM, shutting down"),
    }
    shutdown.cancel();
    if let Err(error) = hourly.await {
        log::error!("the hourly quote task ended badly: '{error:#}'");
    }
    if let Err(error) = promoting.await {
        log::error!("the promotion task ended badly: '{error:#}'");
    }
    Ok(())
}

fn database_path() -> anyhow::Result<PathBuf> {
    if let Ok(path) = std::env::var("AFANASIEFF_DB") {
        return Ok(PathBuf::from(path));
    }
    let home = std::env::var("HOME").context("reading HOME to locate the database")?;
    Ok(PathBuf::from(home).join(".local/state/afanasieff/afanasieff.db"))
}
