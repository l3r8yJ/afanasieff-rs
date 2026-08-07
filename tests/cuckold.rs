use std::sync::Arc;

use afanasieff_rs::handler_tree;
use afanasieff_rs::ops::cuckold::set_drumroll_for_tests;
use afanasieff_rs::ops::store::Store;
use asserting::prelude::*;
use chrono::{DateTime, Duration, Utc};
use teloxide_tests::{MockBot, MockMessageText, MockUser};

const CHAT: i64 = 12_345_678;

fn at(minutes_from_now: i64) -> DateTime<Utc> {
    Utc::now() + Duration::minutes(minutes_from_now)
}

fn store_with_one_member() -> Arc<Store> {
    let store = Arc::new(Store::in_memory().unwrap());
    store
        .upsert_member(CHAT, 7, Some("m"), "Матвей", &Utc::now().to_rfc3339())
        .unwrap();
    store
}

async fn call(command: &str, when: DateTime<Utc>, store: &Arc<Store>) -> teloxide_tests::Responses {
    set_drumroll_for_tests(0);
    let mut bot = MockBot::new(
        MockMessageText::new()
            .text(command)
            .date(when)
            .from(MockUser::new().is_bot(true).build()),
        handler_tree(),
    );
    bot.dependencies(teloxide::dptree::deps![Arc::clone(store)]);
    bot.dispatch().await;
    bot.get_responses()
}

#[tokio::test]
async fn beats_a_drum_roll_before_naming_the_cuckold() {
    let store = store_with_one_member();
    let responses = call("/cuckold", at(0), &store).await;
    let last = responses
        .edited_messages_text
        .last()
        .expect("the announcement is edited")
        .bot_request
        .text
        .clone();
    assert_that!(responses.sent_messages.len())
        .named("sent messages")
        .is_equal_to(1);
    assert_that!(responses.edited_messages_text.len())
        .named("edits")
        .is_equal_to(2);
    assert_that!(last.as_str())
        .named("final announcement")
        .contains("Матвей");
}

#[tokio::test]
async fn names_the_same_cuckold_without_a_drum_roll_for_the_rest_of_the_day() {
    let store = store_with_one_member();
    call("/cuckold", at(0), &store).await;
    let responses = call("/cuckold", at(1), &store).await;
    let answer = responses
        .sent_messages
        .last()
        .expect("the repeat is answered")
        .text()
        .expect("the answer carries text")
        .to_string();
    assert_that!(responses.edited_messages_text.len())
        .named("edits on a repeat")
        .is_equal_to(0);
    assert_that!(answer.as_str())
        .named("repeated answer")
        .contains("Матвей");
}

#[tokio::test]
async fn says_there_is_nobody_to_draw_from_in_a_silent_chat() {
    let store = Arc::new(Store::in_memory().unwrap());
    let responses = call("/cuckold", at(0), &store).await;
    let answer = responses
        .sent_messages
        .last()
        .expect("the empty chat is answered")
        .text()
        .expect("the answer carries text")
        .to_string();
    assert_that!(answer.as_str())
        .named("answer for an empty chat")
        .contains("Играть не с кем");
}
