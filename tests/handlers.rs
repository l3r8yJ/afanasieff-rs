use std::sync::Arc;

use afanasieff_rs::handler_tree;
use afanasieff_rs::ops::consts::{STREAM_SOURCE, VINOGRAD_SOURCE};
use afanasieff_rs::ops::error::Error;
use afanasieff_rs::ops::matthew::reply_with_quote;
use afanasieff_rs::ops::store::Store;
use teloxide::Bot;
use teloxide::dispatching::{UpdateFilterExt, UpdateHandler};
use teloxide::types::{Message, ReactionType, Update};
use teloxide_tests::mock_bot::DistributionKey;
use teloxide_tests::{MockBot, MockMessageText};

fn bot_with_store(text: &str) -> (MockBot<Error, DistributionKey>, Arc<Store>) {
    let store = Arc::new(Store::in_memory().unwrap());
    let mut bot = MockBot::new(MockMessageText::new().text(text), handler_tree());
    bot.dependencies(teloxide::dptree::deps![Arc::clone(&store)]);
    (bot, store)
}

async fn reply_with_quote_endpoint(
    bot: Bot,
    message: Message,
    store: Arc<Store>,
) -> Result<(), Error> {
    reply_with_quote(&bot, &message, &store).await
}

fn matthew_reply_tree() -> UpdateHandler<Error> {
    teloxide::dptree::entry().branch(Update::filter_message().endpoint(reply_with_quote_endpoint))
}

#[tokio::test]
async fn replies_with_a_vinograd_quote_and_a_dung_reaction() {
    let (mut bot, store) = bot_with_store("а виноград то вкусный");
    bot.dispatch().await;
    let responses = bot.get_responses();
    assert_eq!(
        responses.sent_messages.len(),
        1,
        "sent messages count was '{}', expected one reply",
        responses.sent_messages.len()
    );
    let sent = responses
        .sent_messages
        .last()
        .expect("a message is sent")
        .text()
        .expect("the sent message carries text")
        .to_string();
    let vinograd_quotes = store
        .quotes(VINOGRAD_SOURCE)
        .expect("the vinograd quotes are readable");
    assert!(
        vinograd_quotes.contains(&sent),
        "sent message was '{sent}', expected one of the vinograd quotes '{vinograd_quotes:?}'"
    );
    let reaction = responses
        .set_message_reaction
        .last()
        .expect("a reaction is set")
        .reaction
        .clone()
        .expect("the reaction list is present");
    assert_eq!(
        reaction[0],
        ReactionType::Emoji {
            emoji: "💩".to_string()
        },
        "vinograd reaction was '{:?}', expected the dung emoji",
        reaction[0]
    );
}

#[tokio::test]
async fn replies_with_a_stream_quote_and_a_clown_reaction() {
    let (mut bot, store) = bot_with_store("когда стрим будет");
    bot.dispatch().await;
    let responses = bot.get_responses();
    assert_eq!(
        responses.sent_messages.len(),
        1,
        "sent messages count was '{}', expected one reply",
        responses.sent_messages.len()
    );
    let sent = responses
        .sent_messages
        .last()
        .expect("a message is sent")
        .text()
        .expect("the sent message carries text")
        .to_string();
    let stream_quotes = store
        .quotes(STREAM_SOURCE)
        .expect("the stream quotes are readable");
    assert!(
        stream_quotes.contains(&sent),
        "sent message was '{sent}', expected one of the stream quotes '{stream_quotes:?}'"
    );
    let reaction = responses
        .set_message_reaction
        .last()
        .expect("a reaction is set")
        .reaction
        .clone()
        .expect("the reaction list is present");
    assert_eq!(
        reaction[0],
        ReactionType::Emoji {
            emoji: "🤡".to_string()
        },
        "stream reaction was '{:?}', expected the clown emoji",
        reaction[0]
    );
}

#[tokio::test]
async fn stays_quiet_when_no_keyword_matches() {
    let (mut bot, _store) = bot_with_store("обычное сообщение без ключевых слов");
    bot.dispatch().await;
    let sent = bot.get_responses().sent_messages.len();
    assert_eq!(
        sent, 0,
        "sent messages count was '{sent}', expected no reply for a message with no keyword"
    );
}

#[tokio::test]
async fn stays_quiet_when_the_quotes_table_is_gone() {
    let store = Arc::new(Store::in_memory().unwrap());
    store
        .drop_quotes_table_for_tests()
        .expect("the quotes table exists before it is dropped");
    let mut bot = MockBot::new(
        MockMessageText::new().text("а виноград то вкусный"),
        handler_tree(),
    );
    bot.dependencies(teloxide::dptree::deps![Arc::clone(&store)]);
    bot.dispatch().await;
    let sent = bot.get_responses().sent_messages.len();
    assert_eq!(
        sent, 0,
        "sent messages count was '{sent}', expected no reply from an unreadable database"
    );
    let queried = store.random_quote("vinograd");
    assert!(
        queried.is_err(),
        "querying a dropped quotes table returned '{queried:?}', expected an error rather than an empty result"
    );
}

#[tokio::test]
async fn reply_with_quote_sends_a_matthew_quote_with_a_broken_heart_reaction() {
    let store = Arc::new(Store::in_memory().unwrap());
    let mut bot = MockBot::new(MockMessageText::new().text("matthew"), matthew_reply_tree());
    bot.dependencies(teloxide::dptree::deps![store]);
    bot.dispatch().await;
    let responses = bot.get_responses();
    assert_eq!(
        responses.sent_messages.len(),
        1,
        "sent messages count was '{}', expected one reply",
        responses.sent_messages.len()
    );
    let reaction = responses
        .set_message_reaction
        .last()
        .expect("a reaction is set")
        .reaction
        .clone()
        .expect("the reaction list is present");
    assert_eq!(
        reaction[0],
        ReactionType::Emoji {
            emoji: "💔".to_string()
        },
        "matthew reaction was '{:?}', expected the broken heart emoji",
        reaction[0]
    );
}
