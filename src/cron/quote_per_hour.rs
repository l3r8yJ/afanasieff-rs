use std::{sync::Arc, time::Duration};

use dashmap::DashSet;
use once_cell::sync::Lazy;
use rand::{Rng, rng};
use teloxide::{
    Bot,
    prelude::Requester,
    types::{ChatId, Update},
};
use tokio::time::interval;

use crate::ops::quotes::random_string_from;

type ChatPool = Arc<DashSet<ChatId>>;

static CHAT_POOL: Lazy<ChatPool> = Lazy::new(|| Arc::new(DashSet::new()));

const QUOTES_POOL: &[&str] = &[
    "Ты сдохнешь в аду урод",
    "Я бы тебе просто по твоей лысине вонючей c пыру въебал",
    "и че ? тебя нахуярить чтоли ты имеешь ввиду ?",
    "Я ссал стрим на голову петину",
    "терпим",
    "Извините",
    "Хорошо браток идем 1x1 с каждым 5 раундов по пол часа",
    "Нихуя вы базарите, уроды",
    "В этот день я и порвал эти шорты",
    "я петух в законе",
    "Вот именно, либералы пидорасы",
    "Губами",
    "Я Путин",
    "Я белогвардеец",
    "все как папа учил, только надо еще голым",
    "ахтубинск город заднеприводных",
    "не понял, куколд моя бабушка?",
    "слышь ты нахуй, баба ты ебаная",
    "аниме вообще для даунов",
    "/pidor@UserOfTheDayBot",
    "Тебе хуем жопу закрыли гандон блять",
    "я принесу тебе говна нахуй",
    "хорошо куколд сука",
];

pub fn put_id_into_pool(update: Update) {
    if let teloxide::types::UpdateKind::Message(message) = update.kind {
        let inserted = CHAT_POOL.insert(message.chat.id);
        log::info!("chat id: '{}', inserted: '{}'", message.chat.id, inserted);
    }
}

pub fn start_cron(bot: Bot) {
    tokio::spawn(async move {
        log::info!("iterating over hour");
        let mut random_interval = interval(Duration::from_mins(random_minutes_count()));
        loop {
            random_interval.tick().await;
            for id in CHAT_POOL.iter() {
                let b = bot.clone();
                tokio::spawn(async move {
                    let _ = b
                        .send_message(id.clone(), random_string_from(QUOTES_POOL))
                        .await;
                    log::info!("message sent for id: '{}'", id.0);
                });
            }
        }
    });
}

fn random_minutes_count() -> u64 {
    let mut rng = rng();
    rng.random_range(30..120)
}
