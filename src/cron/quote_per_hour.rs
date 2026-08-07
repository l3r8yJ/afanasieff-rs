use std::sync::Arc;
use std::time::Duration;

use rand::{Rng, rng};
use teloxide::{Bot, prelude::Requester, types::ChatId};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::ops::{consts::MATTHEW_SOURCE, send::remember, store::Store};

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
            log::error!("chats were not read: '{error:#}'");
            return;
        }
    };
    for id in chats {
        let generated = if rng().random_bool(crate::ops::chance::generated_in_cron()) {
            match store.all_quotes() {
                Ok(corpus) => crate::ops::markov::generate(&corpus, &mut rng()),
                Err(error) => {
                    log::error!("corpus for chat '{id}' was not read: '{error:#}'");
                    None
                }
            }
        } else {
            None
        };
        let (text, quote) = match generated {
            Some(generated) => (generated, None),
            None => match store.random_quote_with_id(MATTHEW_SOURCE) {
                Ok(Some((quote, text))) => (text, Some(quote)),
                Ok(None) => {
                    log::debug!("no quote of source '{MATTHEW_SOURCE}' to send to chat '{id}'");
                    continue;
                }
                Err(error) => {
                    log::error!("quote for chat '{id}' was not read: '{error:#}'");
                    continue;
                }
            },
        };
        match bot.send_message(ChatId(id), text).await {
            Ok(sent) => {
                log::info!("message sent for id: '{id}'");
                remember(store, &sent, quote);
            }
            Err(error) => log::error!("message for id '{id}' failed: '{error:#}'"),
        }
    }
}

fn random_minutes_count() -> u64 {
    let mut rng = rng();
    rng.random_range(60..180)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, LazyLock};

    use asserting::prelude::*;
    use teloxide::dispatching::{UpdateFilterExt, UpdateHandler};
    use teloxide::types::Update;
    use teloxide::{Bot, dptree};
    use teloxide_tests::{MockBot, MockMessageText};
    use tokio::sync::Mutex;

    use crate::ops::chance;
    use crate::ops::consts::MATTHEW_SOURCE;
    use crate::ops::store::Store;

    use super::send_to_every_chat;

    static CHANCE_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    const CHAT: i64 = 12_345_678;

    async fn drive(bot: Bot, store: Arc<Store>) -> anyhow::Result<()> {
        send_to_every_chat(&bot, &store).await;
        Ok(())
    }

    fn tree() -> UpdateHandler<anyhow::Error> {
        dptree::entry().branch(Update::filter_message().endpoint(drive))
    }

    async fn run(store: &Arc<Store>) -> teloxide_tests::Responses {
        let mut bot = MockBot::new(MockMessageText::new().text("tick"), tree());
        bot.dependencies(dptree::deps![Arc::clone(store)]);
        bot.dispatch().await;
        bot.get_responses()
    }

    #[tokio::test]
    async fn sends_a_real_quote_and_remembers_its_id_when_never_generated() {
        let _guard = CHANCE_LOCK.lock().await;
        chance::set_generated_in_cron_for_tests(0);
        let store = Arc::new(Store::in_memory().unwrap());
        store.remember_chat(CHAT).unwrap();
        let responses = run(&store).await;
        let sent = responses.sent_messages.last().expect("a message is sent");
        let text = sent
            .text()
            .expect("the sent message carries text")
            .to_string();
        let matthew_quotes = store.quotes(MATTHEW_SOURCE).unwrap();
        assert_that!(matthew_quotes)
            .named("matthew quotes")
            .contains(text);
        let owner = store
            .message_owner(CHAT, sent.id.0)
            .unwrap()
            .expect("the hourly message is remembered");
        assert_that!(owner.quote)
            .named("quote id carried by the hourly message")
            .is_some();
        assert_that!(store.quote_score(owner.quote.unwrap()))
            .named("score of the quote the hourly message carried")
            .is_ok();
    }

    #[tokio::test]
    async fn sends_generated_text_with_no_quote_id_when_always_generated() {
        let _guard = CHANCE_LOCK.lock().await;
        chance::set_generated_in_cron_for_tests(1000);
        let store = Arc::new(Store::in_memory().unwrap());
        store.remember_chat(CHAT).unwrap();
        let responses = run(&store).await;
        let sent = responses.sent_messages.last().expect("a message is sent");
        let owner = store
            .message_owner(CHAT, sent.id.0)
            .unwrap()
            .expect("the hourly message is remembered");
        assert_that!(owner.quote)
            .named("quote id of a generated hourly message")
            .is_none();
    }
}
