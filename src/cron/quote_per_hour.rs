use std::sync::Arc;
use std::time::Duration;

use rand::{Rng, rng};
use teloxide::{Bot, prelude::Requester, types::ChatId};
use tokio::time::sleep;

use crate::ops::{consts::MATTHEW_SOURCE, store::Store};

pub async fn start_cron(bot: Bot, store: Arc<Store>) {
    loop {
        log::info!("iterating over hour");
        match store.chats() {
            Ok(chats) => {
                for id in chats {
                    let bot = bot.clone();
                    let store = Arc::clone(&store);
                    tokio::spawn(async move {
                        match store.random_quote(MATTHEW_SOURCE) {
                            Ok(Some(quote)) => {
                                let _ = bot.send_message(ChatId(id), quote).await;
                                log::info!("message sent for id: '{id}'");
                            }
                            Ok(None) => log::debug!("no matthew quote available for id: '{id}'"),
                            Err(error) => {
                                log::error!("quote for id '{id}' was not read: '{error}'");
                            }
                        }
                    });
                }
            }
            Err(error) => log::error!("chats were not read: '{error}'"),
        }
        sleep(Duration::from_mins(random_minutes_count())).await;
    }
}

fn random_minutes_count() -> u64 {
    let mut rng = rng();
    rng.random_range(60..180)
}
