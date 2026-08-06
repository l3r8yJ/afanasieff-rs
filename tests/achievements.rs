use afanasieff_rs::ops::achievements::event::{Event, Mention};
use chrono::{TimeZone, Utc};
use teloxide::types::{MessageEntity, MessageEntityKind, Update, UpdateId, UpdateKind, User};
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
