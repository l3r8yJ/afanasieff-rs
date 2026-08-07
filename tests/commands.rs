use std::sync::Arc;

use afanasieff_rs::handler_tree;
use afanasieff_rs::ops::consts::STREAM_SOURCE;
use afanasieff_rs::ops::store::Store;
use asserting::prelude::*;
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
    assert_that!(answer.as_str())
        .named("catalogue answer")
        .contains("<b>Терпим</b>\n     <i>10 своих сообщений подряд")
        .contains("<b>Петух в законе</b>\n     <i>собрать пять любых других</i>");
}

#[tokio::test]
async fn shows_locked_and_unlocked_achievements_of_the_caller() {
    let store = Arc::new(Store::in_memory().unwrap());
    let answer = answer_of("/my_achievements", &store).await;
    assert_that!(answer.as_str())
        .named("personal answer")
        .contains("0 из 17")
        .contains("🔒")
        .contains("▱▱▱▱▱▱▱▱▱▱");
}

#[tokio::test]
async fn falls_through_to_the_quote_branch_when_text_merely_mentions_achievements() {
    let store = Arc::new(Store::in_memory().unwrap());
    let answer = answer_of("achievements упоминают стрим", &store).await;
    let stream_quotes = store
        .quotes(STREAM_SOURCE)
        .expect("the stream quotes are readable");
    assert_that!(answer.as_str())
        .named("answer to a keyword-bearing message with no leading slash")
        .does_not_contain("Ачивки");
    assert_that!(stream_quotes)
        .named("stream quotes")
        .contains(answer);
}

#[tokio::test]
async fn joins_the_header_and_the_locked_list_with_a_single_blank_line_when_nothing_is_owned() {
    let store = Arc::new(Store::in_memory().unwrap());
    let answer = answer_of("/my_achievements", &store).await;
    assert_that!(answer.as_str())
        .named("personal answer with nothing owned")
        .contains("0 из 17")
        .does_not_contain("\n\n\n");
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
    assert_that!(answer.as_str())
        .named("personal answer")
        .contains("▰▰▰▰▰▰▰▰▱▱ <code>40/50</code>");
    assert_that!(closest)
        .named("position of the closest achievement")
        .is_less_than(farthest);
    assert_that!(farthest)
        .named("position of the farthest achievement")
        .is_less_than(meta);
}

#[tokio::test]
async fn ranks_members_by_how_many_achievements_they_own() {
    let store = Arc::new(Store::in_memory().unwrap());
    store
        .upsert_member(
            DEFAULT_CHAT,
            1,
            Some("first"),
            "Первый",
            "2026-08-07T10:00:00Z",
        )
        .unwrap();
    store
        .upsert_member(
            DEFAULT_CHAT,
            2,
            Some("second"),
            "Второй",
            "2026-08-07T10:00:00Z",
        )
        .unwrap();
    store
        .unlock(DEFAULT_CHAT, 1, "terpim", "2026-08-01T10:00:00Z")
        .unwrap();
    store
        .unlock(DEFAULT_CHAT, 1, "haha", "2026-08-02T10:00:00Z")
        .unwrap();
    store
        .unlock(DEFAULT_CHAT, 2, "robot", "2026-08-01T10:00:00Z")
        .unwrap();
    let answer = answer_of("/top", &store).await;
    let first = answer.find("Первый").expect("the leader is listed");
    let second = answer.find("Второй").expect("the runner up is listed");
    assert_that!(answer.as_str())
        .named("leaderboard")
        .contains("🥇")
        .contains("2 из 17");
    assert_that!(first)
        .named("position of the leader")
        .is_less_than(second);
}

#[tokio::test]
async fn puts_the_earlier_finisher_first_on_a_tie() {
    let store = Arc::new(Store::in_memory().unwrap());
    store
        .upsert_member(
            DEFAULT_CHAT,
            1,
            Some("late"),
            "Поздний",
            "2026-08-07T10:00:00Z",
        )
        .unwrap();
    store
        .upsert_member(
            DEFAULT_CHAT,
            2,
            Some("early"),
            "Ранний",
            "2026-08-07T10:00:00Z",
        )
        .unwrap();
    store
        .unlock(DEFAULT_CHAT, 1, "terpim", "2026-08-05T10:00:00Z")
        .unwrap();
    store
        .unlock(DEFAULT_CHAT, 2, "haha", "2026-08-01T10:00:00Z")
        .unwrap();
    let answer = answer_of("/top", &store).await;
    let early = answer.find("Ранний").expect("the early finisher is listed");
    let late = answer.find("Поздний").expect("the late finisher is listed");
    assert_that!(early)
        .named("position of the earlier finisher")
        .is_less_than(late);
}

#[tokio::test]
async fn says_so_when_nobody_collected_anything() {
    let store = Arc::new(Store::in_memory().unwrap());
    let answer = answer_of("/top", &store).await;
    assert_that!(answer.as_str())
        .named("empty leaderboard")
        .contains("Пока никто ничего не собрал");
}
