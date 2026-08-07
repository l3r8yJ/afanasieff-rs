use std::sync::Arc;
use std::time::Duration;

use rand::{Rng, rng};
use teloxide::{Bot, prelude::Requester, types::ChatId};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::ops::{consts::MATTHEW_SOURCE, store::Store};

pub async fn start_cron(bot: Bot, store: Arc<Store>, shutdown: CancellationToken) {
    loop {
        tokio::select! {
            () = shutdown.cancelled() => break,
            () = sleep(Duration::from_mins(random_minutes_count())) => {
                log::info!("iterating over hour");
                send_to_every_chat(&bot, &store).await;
            }
        }
    }
}

async fn send_to_every_chat(bot: &Bot, store: &Store) {
    let chats = match store.chats() {
        Ok(chats) => chats,
        Err(error) => {
            log::error!("chats were not read: '{error}'");
            return;
        }
    };
    for id in chats {
        let generated = if rng().random_bool(crate::ops::chance::generated_in_cron()) {
            match store.all_quotes() {
                Ok(corpus) => crate::ops::markov::generate(&corpus, &mut rng()),
                Err(error) => {
                    log::error!("corpus for chat '{id}' was not read: '{error}'");
                    None
                }
            }
        } else {
            None
        };
        let text = match generated {
            Some(generated) => generated,
            None => match store.random_quote(MATTHEW_SOURCE) {
                Ok(Some(quote)) => quote,
                Ok(None) => {
                    log::debug!("no quote of source '{MATTHEW_SOURCE}' to send to chat '{id}'");
                    continue;
                }
                Err(error) => {
                    log::error!("quote for chat '{id}' was not read: '{error}'");
                    continue;
                }
            },
        };
        match bot.send_message(ChatId(id), text).await {
            Ok(_) => log::info!("message sent for id: '{id}'"),
            Err(error) => log::error!("message for id '{id}' failed: '{error}'"),
        }
    }
}

fn random_minutes_count() -> u64 {
    let mut rng = rng();
    rng.random_range(60..180)
}
