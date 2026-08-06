use std::sync::Arc;

use afanasieff_rs::handler_tree;
use afanasieff_rs::ops::store::Store;
use teloxide_tests::{MockBot, MockMessageText};

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
