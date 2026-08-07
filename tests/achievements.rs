use afanasieff_rs::handler_tree;
use afanasieff_rs::ops::achievements::apply::apply;
use afanasieff_rs::ops::achievements::event::{Event, Mention};
use afanasieff_rs::ops::achievements::record_bot_reply;
use afanasieff_rs::ops::store::Store;
use asserting::prelude::*;
use chrono::{TimeZone, Utc};
use std::sync::Arc;
use teloxide::types::{MessageEntity, MessageEntityKind, Update, UpdateId, UpdateKind, User};
use teloxide_tests::mock_bot::MockBot;
use teloxide_tests::{MockMessageText, MockUser};

fn update_of(message: teloxide::types::Message) -> Update {
    Update {
        id: UpdateId(1),
        kind: UpdateKind::Message(message),
    }
}

fn user(id: u64, username: &str) -> User {
    MockUser::new()
        .id(id)
        .username(username.to_string())
        .build()
}

fn user_named(id: u64, username: &str, first_name: &str) -> User {
    MockUser::new()
        .id(id)
        .username(username.to_string())
        .first_name(first_name.to_string())
        .build()
}

fn event_of(text: &str, author: u64, message_id: i32) -> Event {
    let message = MockMessageText::new()
        .text(text)
        .from(user(author, "matthew"))
        .id(message_id)
        .build();
    Event::parse(&update_of(message)).expect("the message parses into an event")
}

#[test]
fn reads_the_author_and_the_flags_of_a_message() {
    let message = MockMessageText::new()
        .text("слышь ты нахуй, го в доту")
        .from(user(7, "matthew"))
        .date(Utc.with_ymd_and_hms(2026, 8, 6, 23, 30, 0).unwrap())
        .build();
    let event = Event::parse(&update_of(message)).expect("a text message parses into an event");
    assert_that!(event.user)
        .named("event author")
        .is_equal_to(7);
    assert_that!(event.mat).named("event mat flag").is_true();
    assert_that!(event.call_to_play)
        .named("event call-to-play flag")
        .is_true();
    assert_that!(event.hour_msk)
        .named("event hour (MSK)")
        .is_equal_to(2);
}

#[test]
fn reads_the_reply_target_of_a_message() {
    let replied_to = MockMessageText::new()
        .text("терпим")
        .from(user(8, "stream"))
        .build();
    let message = MockMessageText::new()
        .text("сам терпи")
        .from(user(7, "matthew"))
        .reply_to_message(replied_to)
        .build();
    let event = Event::parse(&update_of(message)).expect("a reply parses into an event");
    assert_that!(event.reply_to_user)
        .named("reply target")
        .is_equal_to(Some(8));
}

#[test]
fn skips_a_message_written_by_a_bot() {
    let message = MockMessageText::new()
        .text("терпим")
        .from(MockUser::new().id(9).is_bot(true).build())
        .build();
    let event = Event::parse(&update_of(message));
    assert_that!(event)
        .named("event parsed from a bot message")
        .is_none();
}

#[test]
fn collects_mentions_by_username_and_by_id() {
    let message = MockMessageText::new()
        .text("@stream иди сюда")
        .from(user(7, "matthew"))
        .entities(vec![MessageEntity::new(MessageEntityKind::Mention, 0, 7)])
        .build();
    let event = Event::parse(&update_of(message)).expect("a mention parses into an event");
    assert_that!(event.mentions)
        .named("mentions")
        .contains_exactly([Mention::Username("stream".to_string())]);
}

#[test]
fn counts_a_message_once_per_message_id() {
    let store = Store::in_memory().unwrap();
    let event = event_of("терпим", 7, 5);
    let first = apply(&store, &event).unwrap();
    let again = apply(&store, &event).unwrap();
    let counted = store
        .stat(event.chat, event.user, "unanswered_streak")
        .unwrap();
    assert_that!(first).named("first apply").is_true();
    assert_that!(again)
        .named("second apply of the same message id")
        .is_false();
    assert_that!(counted)
        .named("unanswered_streak after both applies")
        .is_equal_to(1);
}

#[test]
fn skips_a_message_whose_id_is_lower_than_the_last_processed_one() {
    let store = Store::in_memory().unwrap();
    let first = event_of("терпим", 7, 5);
    let earlier = event_of("терпим", 7, 3);
    apply(&store, &first).unwrap();
    let counted_earlier = apply(&store, &earlier).unwrap();
    let counted = store
        .stat(first.chat, first.user, "unanswered_streak")
        .unwrap();
    assert_that!(counted_earlier)
        .named("apply result for the earlier-id message")
        .is_false();
    assert_that!(counted)
        .named("unanswered_streak after the earlier message")
        .is_equal_to(1);
}

#[test]
fn grows_the_unanswered_streak_until_someone_replies() {
    let store = Store::in_memory().unwrap();
    let mut chat = 0;
    for id in 1..=3 {
        let event = event_of("ну че вы", 7, id);
        chat = event.chat;
        apply(&store, &event).unwrap();
    }
    let grown = store.stat(chat, 7, "unanswered_streak").unwrap();
    let replied_to = MockMessageText::new()
        .text("терпим")
        .from(user(7, "matthew"))
        .build();
    let reply = MockMessageText::new()
        .text("да понял я")
        .from(user(8, "stream"))
        .id(4)
        .reply_to_message(replied_to)
        .build();
    apply(&store, &Event::parse(&update_of(reply)).unwrap()).unwrap();
    let reset = store.stat(chat, 7, "unanswered_streak").unwrap();
    assert_that!(grown)
        .named("streak before the reply")
        .is_equal_to(3);
    assert_that!(reset)
        .named("streak after the reply")
        .is_equal_to(0);
}

#[test]
fn counts_a_monologue_of_five_messages() {
    let store = Store::in_memory().unwrap();
    let mut chat = 0;
    for id in 1..=5 {
        let event = event_of("говорю сам с собой", 7, id);
        chat = event.chat;
        apply(&store, &event).unwrap();
    }
    let monologues = store.stat(chat, 7, "monologues").unwrap();
    assert_that!(monologues)
        .named("monologues after five messages in a row")
        .is_equal_to(1);
}

#[test]
fn counts_an_unanswered_call_when_someone_else_answers_after_the_timeout() {
    let store = Store::in_memory().unwrap();
    let call = MockMessageText::new()
        .text("го в доту")
        .from(user(7, "matthew"))
        .id(1)
        .date(Utc.with_ymd_and_hms(2026, 8, 6, 12, 0, 0).unwrap())
        .build();
    let call_event = Event::parse(&update_of(call)).unwrap();
    apply(&store, &call_event).unwrap();
    let late_reply = MockMessageText::new()
        .text("да я занят")
        .from(user(8, "stream"))
        .id(2)
        .date(Utc.with_ymd_and_hms(2026, 8, 6, 12, 11, 0).unwrap())
        .build();
    apply(&store, &Event::parse(&update_of(late_reply)).unwrap()).unwrap();
    let unanswered = store
        .stat(call_event.chat, call_event.user, "unanswered_calls")
        .unwrap();
    assert_that!(unanswered)
        .named("unanswered calls after a late reply from someone else")
        .is_equal_to(1);
}

#[test]
fn does_not_count_a_call_answered_by_someone_else_within_the_timeout() {
    let store = Store::in_memory().unwrap();
    let call = MockMessageText::new()
        .text("го в доту")
        .from(user(7, "matthew"))
        .id(1)
        .date(Utc.with_ymd_and_hms(2026, 8, 6, 12, 0, 0).unwrap())
        .build();
    let call_event = Event::parse(&update_of(call)).unwrap();
    apply(&store, &call_event).unwrap();
    let quick_reply = MockMessageText::new()
        .text("да я занят")
        .from(user(8, "stream"))
        .id(2)
        .date(Utc.with_ymd_and_hms(2026, 8, 6, 12, 5, 0).unwrap())
        .build();
    apply(&store, &Event::parse(&update_of(quick_reply)).unwrap()).unwrap();
    let unanswered = store
        .stat(call_event.chat, call_event.user, "unanswered_calls")
        .unwrap();
    assert_that!(unanswered)
        .named("unanswered calls after a timely reply from someone else")
        .is_equal_to(0);
}

#[test]
fn counts_a_quote_the_bot_sent_to_a_member() {
    let store = Store::in_memory().unwrap();
    let message = MockMessageText::new()
        .text("а виноград то вкусный")
        .from(user(7, "matthew"))
        .build();
    record_bot_reply(&store, &message);
    record_bot_reply(&store, &message);
    let counted = store.stat(message.chat.id.0, 7, "bot_replies").unwrap();
    assert_that!(counted)
        .named("bot replies counted")
        .is_equal_to(2);
}

#[tokio::test]
async fn announces_the_streak_achievement_once() {
    let store = Arc::new(Store::in_memory().unwrap());
    let mut announced = 0;
    for id in 1..=12 {
        let mut bot = MockBot::new(
            MockMessageText::new()
                .text("ну и ладно")
                .from(user(7, "matthew"))
                .id(id),
            handler_tree(),
        );
        bot.dependencies(teloxide::dptree::deps![Arc::clone(&store)]);
        bot.dispatch().await;
        announced += bot
            .get_responses()
            .sent_messages
            .iter()
            .filter(|message| {
                message
                    .text()
                    .is_some_and(|text| text.contains("Тебе никто не ответил"))
            })
            .count();
    }
    assert_that!(announced)
        .named("times the streak achievement was announced")
        .is_equal_to(1);
}

#[tokio::test]
async fn escapes_html_special_characters_in_the_announced_display_name() {
    let store = Arc::new(Store::in_memory().unwrap());
    let mut announcement = None;
    for id in 1..=10 {
        let mut bot = MockBot::new(
            MockMessageText::new()
                .text("ну и ладно")
                .from(user_named(7, "matthew", "Ма<b>твей & Co"))
                .id(id),
            handler_tree(),
        );
        bot.dependencies(teloxide::dptree::deps![Arc::clone(&store)]);
        bot.dispatch().await;
        announcement = announcement.or_else(|| {
            bot.get_responses()
                .sent_messages
                .iter()
                .find_map(|message| message.text().map(str::to_string))
                .filter(|text| text.contains("Тебе никто не ответил"))
        });
    }
    let announcement =
        announcement.expect("the streak achievement was announced once across the ten messages");
    assert_that!(announcement.as_str())
        .named("announcement")
        .contains("Ма&lt;b&gt;твей &amp; Co")
        .does_not_contain("<b>твей");
}
