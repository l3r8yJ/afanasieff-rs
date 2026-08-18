use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use rand::seq::IndexedRandom;
use rand::{Rng, rng};
use teloxide::payloads::SendMessageSetters;
use teloxide::types::{ChatId, ParseMode, UserId};
use teloxide::utils::html::{escape, user_mention};
use teloxide::{Bot, prelude::Requester};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::ops::{chance, consts::MATTHEW_SOURCE, send::remember, store::Store};

const ACTIVE_DAYS: i64 = 30;

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
        for step in 0..chance::burst_length() {
            if step > 0 {
                sleep(chance::burst_pause()).await;
            }
            if !send_one(bot, store, id).await {
                break;
            }
        }
    }
}

async fn send_one(bot: &Bot, store: &Store, id: i64) -> bool {
    let Some((text, quote)) = pick(store, id) else {
        return false;
    };
    let text = decorate(store, id, &text);
    match bot
        .send_message(ChatId(id), text)
        .parse_mode(ParseMode::Html)
        .await
    {
        Ok(sent) => {
            log::info!("message sent for id: '{id}'");
            remember(store, &sent, quote);
            true
        }
        Err(error) => {
            log::error!("message for id '{id}' failed: '{error}'");
            false
        }
    }
}

fn pick(store: &Store, id: i64) -> Option<(String, Option<i64>)> {
    if rng().random_bool(chance::generated_in_cron()) {
        match store.all_quotes() {
            Ok(corpus) => {
                if let Some(generated) = crate::ops::markov::generate(&corpus, &mut rng()) {
                    return Some((generated, None));
                }
            }
            Err(error) => log::error!("corpus for chat '{id}' was not read: '{error:#}'"),
        }
    }
    match store.random_quote_with_id(MATTHEW_SOURCE) {
        Ok(Some((quote, text))) => Some((text, Some(quote))),
        Ok(None) => {
            log::debug!("no quote of source '{MATTHEW_SOURCE}' to send to chat '{id}'");
            None
        }
        Err(error) => {
            log::error!("quote for chat '{id}' was not read: '{error:#}'");
            None
        }
    }
}

fn decorate(store: &Store, id: i64, text: &str) -> String {
    let escaped = escape(text);
    if !rng().random_bool(chance::tagged_in_burst()) {
        return escaped;
    }
    let since = (Utc::now() - chrono::Duration::days(ACTIVE_DAYS)).to_rfc3339();
    let members = match store.active_members(id, &since) {
        Ok(members) => members,
        Err(error) => {
            log::error!("members of chat '{id}' were not read: '{error:#}'");
            return escaped;
        }
    };
    let Some(member) = members.choose(&mut rng()) else {
        return escaped;
    };
    let mention = user_mention(
        UserId(u64::try_from(member.user).unwrap_or_default()),
        &member.name,
    );
    let words = escaped.split_whitespace().count();
    splice(&escaped, &mention, rng().random_range(0..=words))
}

fn splice(text: &str, mention: &str, at: usize) -> String {
    let mut words = text
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<String>>();
    words.insert(at.min(words.len()), mention.to_string());
    words.join(" ")
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

    use chrono::Utc;

    use crate::ops::chance;
    use crate::ops::consts::MATTHEW_SOURCE;
    use crate::ops::store::Store;

    use super::{send_to_every_chat, splice};

    static CHANCE_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    const CHAT: i64 = 12_345_678;

    async fn drive(bot: Bot, store: Arc<Store>) -> anyhow::Result<()> {
        send_to_every_chat(&bot, &store).await;
        Ok(())
    }

    fn tree() -> UpdateHandler<anyhow::Error> {
        dptree::entry().branch(Update::filter_message().endpoint(drive))
    }

    fn quiet_single_message() {
        chance::set_burst_length_for_tests(1);
        chance::set_burst_pause_for_tests(0);
        chance::set_tagged_in_burst_for_tests(0);
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
        quiet_single_message();
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
    }

    #[tokio::test]
    async fn sends_generated_text_with_no_quote_id_when_always_generated() {
        let _guard = CHANCE_LOCK.lock().await;
        quiet_single_message();
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

    #[tokio::test]
    async fn sends_several_messages_in_one_burst() {
        let _guard = CHANCE_LOCK.lock().await;
        quiet_single_message();
        chance::set_burst_length_for_tests(4);
        chance::set_generated_in_cron_for_tests(0);
        let store = Arc::new(Store::in_memory().unwrap());
        store.remember_chat(CHAT).unwrap();
        let responses = run(&store).await;
        assert_that!(responses.sent_messages.len())
            .named("messages sent in one burst")
            .is_equal_to(4);
    }

    #[tokio::test]
    async fn tags_a_member_of_the_chat_inside_the_message() {
        let _guard = CHANCE_LOCK.lock().await;
        quiet_single_message();
        chance::set_tagged_in_burst_for_tests(1000);
        chance::set_generated_in_cron_for_tests(0);
        let store = Arc::new(Store::in_memory().unwrap());
        store.remember_chat(CHAT).unwrap();
        store
            .upsert_member(CHAT, 7, Some("m"), "Матвей", &Utc::now().to_rfc3339())
            .unwrap();
        let responses = run(&store).await;
        let text = responses
            .sent_messages
            .last()
            .expect("a message is sent")
            .text()
            .expect("the sent message carries text")
            .to_string();
        assert_that!(text.as_str())
            .named("tagged burst message")
            .contains("tg://user?id=7")
            .contains("Матвей");
    }

    #[tokio::test]
    async fn leaves_the_message_alone_when_the_chat_has_no_members_to_tag() {
        let _guard = CHANCE_LOCK.lock().await;
        quiet_single_message();
        chance::set_tagged_in_burst_for_tests(1000);
        chance::set_generated_in_cron_for_tests(0);
        let store = Arc::new(Store::in_memory().unwrap());
        store.remember_chat(CHAT).unwrap();
        let responses = run(&store).await;
        let text = responses
            .sent_messages
            .last()
            .expect("a message is sent")
            .text()
            .expect("the sent message carries text")
            .to_string();
        let quotes = store.quotes(MATTHEW_SOURCE).unwrap();
        assert_that!(quotes).named("matthew quotes").contains(text);
    }

    #[test]
    fn puts_the_mention_where_it_is_asked_for() {
        let first = splice("а б в", "@x", 0);
        let middle = splice("а б в", "@x", 2);
        let past_the_end = splice("а б в", "@x", 99);
        assert_that!(first.as_str())
            .named("mention at the front")
            .is_equal_to("@x а б в");
        assert_that!(middle.as_str())
            .named("mention in the middle")
            .is_equal_to("а б @x в");
        assert_that!(past_the_end.as_str())
            .named("mention past the end")
            .is_equal_to("а б в @x");
    }

    #[tokio::test]
    async fn leaves_the_message_untagged_when_the_roll_says_so() {
        let _guard = CHANCE_LOCK.lock().await;
        quiet_single_message();
        chance::set_tagged_in_burst_for_tests(0);
        chance::set_generated_in_cron_for_tests(0);
        let store = Arc::new(Store::in_memory().unwrap());
        store.remember_chat(CHAT).unwrap();
        store
            .upsert_member(CHAT, 7, Some("m"), "Матвей", &Utc::now().to_rfc3339())
            .unwrap();
        let responses = run(&store).await;
        let text = responses
            .sent_messages
            .last()
            .expect("a message is sent")
            .text()
            .expect("the sent message carries text")
            .to_string();
        assert_that!(text.as_str())
            .named("untagged burst message")
            .does_not_contain("tg://user?id=");
    }
}
