use std::sync::atomic::AtomicI32;

use afanasieff_rs::ops::intake::observe;
use afanasieff_rs::ops::store::{MATTHEW_USERNAME, Store};
use teloxide_tests::IntoUpdate;
use teloxide_tests::{MockMessageText, MockUser};

fn matthew() -> MockUser {
    MockUser::new().username(MATTHEW_USERNAME)
}

fn update_from(message: MockMessageText) -> teloxide::types::Update {
    message
        .into_update(&AtomicI32::new(1))
        .pop()
        .expect("one update is produced")
}

#[test]
fn collects_a_long_message_written_by_matthew() {
    let store = Store::in_memory().unwrap();
    observe(
        &store,
        update_from(
            MockMessageText::new()
                .text("это сообщение точно длиннее десяти символов")
                .from(matthew().build()),
        ),
    );
    let promoted = store.promote_oldest_matthew_message("matthew").unwrap();
    assert!(
        promoted.is_some(),
        "a long matthew message was promoted as '{promoted:?}', expected it to be waiting"
    );
}

#[test]
fn ignores_a_message_shorter_than_the_threshold() {
    let store = Store::in_memory().unwrap();
    observe(
        &store,
        update_from(
            MockMessageText::new()
                .text("коротко")
                .from(matthew().build()),
        ),
    );
    let promoted = store.promote_oldest_matthew_message("matthew").unwrap();
    assert_eq!(
        promoted, None,
        "a message of ten characters or fewer was promoted as '{promoted:?}', expected none"
    );
}

#[test]
fn ignores_a_message_written_by_anyone_else() {
    let store = Store::in_memory().unwrap();
    observe(
        &store,
        update_from(
            MockMessageText::new()
                .text("это сообщение точно длиннее десяти символов")
                .from(MockUser::new().username("SomeoneElse").build()),
        ),
    );
    let promoted = store.promote_oldest_matthew_message("matthew").unwrap();
    assert_eq!(
        promoted, None,
        "a message from someone other than MatthewAFN was promoted as '{promoted:?}', expected none"
    );
}

#[test]
fn remembers_every_observed_chat_once() {
    let store = Store::in_memory().unwrap();
    let message = MockMessageText::new()
        .text("привет всем в этом чате")
        .from(matthew().build());
    observe(&store, update_from(message.clone()));
    observe(&store, update_from(message));
    let chats = store.chats().unwrap();
    assert_eq!(
        chats.len(),
        1,
        "remembered chats were '{chats:?}', expected exactly one chat"
    );
}

#[test]
fn collects_the_same_message_id_only_once() {
    let store = Store::in_memory().unwrap();
    let message = MockMessageText::new()
        .text("это сообщение точно длиннее десяти символов")
        .from(matthew().build());
    observe(&store, update_from(message.clone()));
    observe(&store, update_from(message));
    let first = store.promote_oldest_matthew_message("matthew").unwrap();
    assert!(
        first.is_some(),
        "the first promotion returned '{first:?}', expected the message to be stored"
    );
    let second = store.promote_oldest_matthew_message("matthew").unwrap();
    assert_eq!(
        second, None,
        "the second promotion returned '{second:?}', expected the duplicate to not create a second row"
    );
}
