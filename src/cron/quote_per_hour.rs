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
        let Ok(Some(quote)) = store.random_quote(MATTHEW_SOURCE) else {
            continue;
        };
        match bot.send_message(ChatId(id), quote).await {
            Ok(_) => log::info!("message sent for id: '{id}'"),
            Err(error) => log::error!("message for id '{id}' failed: '{error}'"),
        }
    }
}

fn random_minutes_count() -> u64 {
    let mut rng = rng();
    rng.random_range(60..180)
}
