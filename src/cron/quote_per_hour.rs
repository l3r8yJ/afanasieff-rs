use std::time::Duration;

use rand::{Rng, rng};
use teloxide::{Bot, prelude::Requester, types::ChatId};
use tokio::time::sleep;

use crate::ops::{
    consts::MATTHEW_SOURCE,
    store::{chats, random_quote, with_db},
};

pub async fn start_cron(bot: Bot) {
    loop {
        log::info!("iterating over hour");
        for id in with_db(chats).unwrap_or_default() {
            let bot = bot.clone();
            tokio::spawn(async move {
                if let Some(quote) =
                    with_db(|connection| random_quote(connection, MATTHEW_SOURCE)).flatten()
                {
                    let _ = bot.send_message(ChatId(id), quote).await;
                    log::info!("message sent for id: '{id}'");
                }
            });
        }
        sleep(Duration::from_mins(random_minutes_count())).await;
    }
}

fn random_minutes_count() -> u64 {
    let mut rng = rng();
    rng.random_range(60..180)
}
