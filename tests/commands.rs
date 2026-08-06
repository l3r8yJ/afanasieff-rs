use std::sync::Arc;

use afanasieff_rs::handler_tree;
use afanasieff_rs::ops::consts::STREAM_SOURCE;
use afanasieff_rs::ops::store::Store;
use teloxide_tests::{MockBot, MockMessageText};

const DEFAULT_CHAT: i64 = 12_345_678;
const DEFAULT_USER: i64 = 12_345_678;

async fn answer_of(command: &str, store: &Arc<Store>) -> String {
    let mut bot = MockBot::new(MockMessageText::new().text(command), handler_tree());
    bot.dependencies(teloxide::dptree::deps![Arc::clone(store)]);
    bot.dispatch().await;
    bot.get_responses()
        .sent_messages
        .last()
        .expect("the command is answered")
        .text()
        .expect("the answer carries text")
        .to_string()
}

#[tokio::test]
async fn lists_every_achievement_with_its_condition() {
    let store = Arc::new(Store::in_memory().unwrap());
    let answer = answer_of("/achievements", &store).await;
    assert!(
        answer.contains("Терпим — 10 своих сообщений подряд")
            && answer.contains("Петух в законе — собрать пять любых других"),
        "catalogue answer was '{answer}', expected it to list titles with conditions"
    );
}

#[tokio::test]
async fn shows_locked_and_unlocked_achievements_of_the_caller() {
    let store = Arc::new(Store::in_memory().unwrap());
    let answer = answer_of("/my_achievements", &store).await;
    assert!(
        answer.contains("0/17") && answer.contains("🔒"),
        "personal answer was '{answer}', expected a zero score and locked entries"
    );
}

#[tokio::test]
async fn falls_through_to_the_quote_branch_when_text_merely_mentions_achievements() {
    let store = Arc::new(Store::in_memory().unwrap());
    let answer = answer_of("achievements упоминают стрим", &store).await;
    let stream_quotes = store
        .quotes(STREAM_SOURCE)
        .expect("the stream quotes are readable");
    assert!(
        stream_quotes.contains(&answer) && !answer.contains("Ачивки"),
        "answer to a keyword-bearing message with no leading slash was '{answer}', expected one of the stream quotes '{stream_quotes:?}' rather than the catalogue text"
    );
}

#[tokio::test]
async fn orders_locked_achievements_by_closeness_to_their_threshold_with_the_meta_one_last() {
    let store = Arc::new(Store::in_memory().unwrap());
    store
        .set_stat(DEFAULT_CHAT, DEFAULT_USER, "vinograd_mentions", 40)
        .expect("the vinograd mentions stat is written");
    store
        .set_stat(DEFAULT_CHAT, DEFAULT_USER, "unanswered_streak", 1)
        .expect("the unanswered streak stat is written");
    let answer = answer_of("/my_achievements", &store).await;
    let closest = answer
        .find("Сверкающая лысина")
        .expect("the closest-to-threshold achievement is listed");
    let farthest = answer
        .find("Терпим")
        .expect("the farther-from-threshold achievement is listed");
    let meta = answer
        .find("Петух в законе")
        .expect("the meta achievement is listed");
    assert!(
        closest < farthest && farthest < meta,
        "locked order was closest={closest}, farthest={farthest}, meta={meta} in '{answer}', expected the achievement nearer its threshold before the farther one, and the no-progress meta achievement last"
    );
}
