use afanasieff_rs::handler_tree;
use afanasieff_rs::ops::achievements::apply::apply;
use afanasieff_rs::ops::achievements::event::{Event, Mention};
use afanasieff_rs::ops::store::Store;
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
    assert_eq!(
        (event.user, event.mat, event.call_to_play, event.hour_msk),
        (7, true, true, 2),
        "parsed event was '{:?}', expected author seven, mat, a call to play and hour two",
        (event.user, event.mat, event.call_to_play, event.hour_msk)
    );
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
    assert_eq!(
        event.reply_to_user,
        Some(8),
        "reply target was '{:?}', expected 'Some(8)'",
        event.reply_to_user
    );
}

#[test]
fn skips_a_message_written_by_a_bot() {
    let message = MockMessageText::new()
        .text("терпим")
        .from(MockUser::new().id(9).is_bot(true).build())
        .build();
    let event = Event::parse(&update_of(message));
    assert!(
        event.is_none(),
        "event of a bot message was '{event:?}', expected none"
    );
}

#[test]
fn collects_mentions_by_username_and_by_id() {
    let message = MockMessageText::new()
        .text("@stream иди сюда")
        .from(user(7, "matthew"))
        .entities(vec![MessageEntity::new(MessageEntityKind::Mention, 0, 7)])
        .build();
    let event = Event::parse(&update_of(message)).expect("a mention parses into an event");
    assert_eq!(
        event.mentions,
        vec![Mention::Username("stream".to_string())],
        "mentions were '{:?}', expected one username mention",
        event.mentions
    );
}

#[test]
fn counts_a_message_once_per_message_id() {
    let store = Store::in_memory().unwrap();
    let event = event_of("терпим", 7, 5);
    let first = apply(&store, &event).unwrap();
    let again = apply(&store, &event).unwrap();
    let counted = store.stat(event.chat, event.user, "messages").unwrap();
    assert_eq!(
        (first, again, counted),
        (true, false, 1),
        "applying the same message twice reported '{:?}', expected it counted once",
        (first, again, counted)
    );
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
    assert_eq!(
        (grown, reset),
        (3, 0),
        "streak was '{grown}' before the reply and '{reset}' after, expected '3' then '0'"
    );
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
    assert_eq!(
        monologues, 1,
        "monologues after five messages in a row were '{monologues}', expected '1'"
    );
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
    assert_eq!(
        announced, 1,
        "the streak achievement was announced '{announced}' times, expected exactly one"
    );
}
