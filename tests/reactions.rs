use std::sync::Arc;

use afanasieff_rs::handler_tree;
use afanasieff_rs::ops::store::Store;
use asserting::prelude::*;
use chrono::Utc;
use teloxide::types::{
    Chat, ChatId, ChatKind, ChatPublic, MaybeAnonymousUser, MessageId, MessageReactionUpdated,
    PublicChatKind, PublicChatSupergroup, ReactionType, Update, UpdateId, UpdateKind,
};
use teloxide_tests::{MockBot, MockUser};

const CHAT: i64 = -100;

fn chat() -> Chat {
    Chat {
        id: ChatId(CHAT),
        kind: ChatKind::Public(ChatPublic {
            title: Some("тест".to_string()),
            kind: PublicChatKind::Supergroup(PublicChatSupergroup {
                username: None,
                is_forum: false,
            }),
        }),
    }
}

fn reaction(message: i32, actor: u64, set: bool) -> Update {
    Update {
        id: UpdateId(1),
        kind: UpdateKind::MessageReaction(MessageReactionUpdated {
            chat: chat(),
            message_id: MessageId(message),
            actor: MaybeAnonymousUser::User(MockUser::new().id(actor).build()),
            date: Utc::now(),
            old_reaction: Vec::new(),
            new_reaction: if set {
                vec![ReactionType::Emoji {
                    emoji: "🔥".to_string(),
                }]
            } else {
                Vec::new()
            },
        }),
    }
}

fn swap(message: i32, actor: u64) -> Update {
    Update {
        id: UpdateId(1),
        kind: UpdateKind::MessageReaction(MessageReactionUpdated {
            chat: chat(),
            message_id: MessageId(message),
            actor: MaybeAnonymousUser::User(MockUser::new().id(actor).build()),
            date: Utc::now(),
            old_reaction: vec![ReactionType::Emoji {
                emoji: "🔥".to_string(),
            }],
            new_reaction: vec![ReactionType::Emoji {
                emoji: "👍".to_string(),
            }],
        }),
    }
}

async fn dispatch(update: Update, store: &Arc<Store>) {
    let mut bot = MockBot::new(update, handler_tree());
    bot.dependencies(teloxide::dptree::deps![Arc::clone(store)]);
    bot.dispatch().await;
}

#[tokio::test]
async fn still_resets_the_streak_when_the_reaction_is_swapped() {
    let store = Arc::new(Store::in_memory().unwrap());
    store
        .upsert_member(CHAT, 100, Some("someone"), "Кто-то", "2026-08-07T10:00:00Z")
        .unwrap();
    store.set_stat(CHAT, 100, "unanswered_streak", 6).unwrap();
    store.remember_message(CHAT, 5, 100, None).unwrap();
    dispatch(swap(5, 7), &store).await;
    let streak = store.stat(CHAT, 100, "unanswered_streak").unwrap();
    assert_that!(streak)
        .named("streak after a reaction swap")
        .is_equal_to(0);
}

#[tokio::test]
async fn resets_the_streak_of_the_member_whose_message_was_reacted_to() {
    let store = Arc::new(Store::in_memory().unwrap());
    store
        .upsert_member(CHAT, 100, Some("someone"), "Кто-то", "2026-08-07T10:00:00Z")
        .unwrap();
    store.set_stat(CHAT, 100, "unanswered_streak", 6).unwrap();
    store.remember_message(CHAT, 5, 100, None).unwrap();
    dispatch(reaction(5, 7, true), &store).await;
    let streak = store.stat(CHAT, 100, "unanswered_streak").unwrap();
    assert_that!(streak)
        .named("streak after a reaction")
        .is_equal_to(0);
}

#[tokio::test]
async fn resets_the_streak_of_a_member_who_has_no_username_on_file() {
    let store = Arc::new(Store::in_memory().unwrap());
    store
        .upsert_member(CHAT, 100, None, "Кто-то", "2026-08-07T10:00:00Z")
        .unwrap();
    store.set_stat(CHAT, 100, "unanswered_streak", 6).unwrap();
    store.remember_message(CHAT, 5, 100, None).unwrap();
    dispatch(reaction(5, 7, true), &store).await;
    let streak = store.stat(CHAT, 100, "unanswered_streak").unwrap();
    assert_that!(streak)
        .named("streak of a member without a username")
        .is_equal_to(0);
}

#[tokio::test]
async fn leaves_every_streak_alone_when_the_reacted_message_has_no_member_row() {
    let store = Arc::new(Store::in_memory().unwrap());
    store.set_stat(CHAT, 999, "unanswered_streak", 6).unwrap();
    store.remember_message(CHAT, 5, 999, None).unwrap();
    dispatch(reaction(5, 7, true), &store).await;
    let streak = store.stat(CHAT, 999, "unanswered_streak").unwrap();
    assert_that!(streak)
        .named("streak of the message owner with no member row")
        .is_equal_to(6);
}

#[tokio::test]
async fn leaves_the_streak_alone_when_the_author_reacted_to_themselves() {
    let store = Arc::new(Store::in_memory().unwrap());
    store.set_stat(CHAT, 100, "unanswered_streak", 6).unwrap();
    store.remember_message(CHAT, 5, 100, None).unwrap();
    dispatch(reaction(5, 100, true), &store).await;
    let streak = store.stat(CHAT, 100, "unanswered_streak").unwrap();
    assert_that!(streak)
        .named("streak after a self reaction")
        .is_equal_to(6);
}

#[tokio::test]
async fn promotes_a_matthew_message_out_of_turn() {
    let store = Arc::new(Store::in_memory().unwrap());
    let matthew = 555;
    store
        .upsert_member(
            CHAT,
            matthew,
            Some("MatthewAFN"),
            "Матвей",
            "2026-08-07T10:00:00Z",
        )
        .unwrap();
    store
        .store_matthew_message(CHAT, 5, "2026-08-07T10:00:00Z", "вне очереди")
        .unwrap();
    store.remember_message(CHAT, 5, matthew, None).unwrap();
    dispatch(reaction(5, 7, true), &store).await;
    let quotes = store.quotes("matthew").unwrap();
    assert_that!(quotes)
        .named("matthew quotes")
        .contains("вне очереди".to_string());
}

#[tokio::test]
async fn does_nothing_for_an_unknown_message() {
    let store = Arc::new(Store::in_memory().unwrap());
    dispatch(reaction(404, 7, true), &store).await;
    let owner = store.message_owner(CHAT, 404).unwrap();
    assert_that!(owner)
        .named("owner of an unknown message")
        .is_none();
}

#[tokio::test]
async fn does_nothing_when_a_reaction_is_taken_back() {
    let store = Arc::new(Store::in_memory().unwrap());
    store.set_stat(CHAT, 100, "unanswered_streak", 6).unwrap();
    store.remember_message(CHAT, 5, 100, None).unwrap();
    dispatch(reaction(5, 7, false), &store).await;
    let streak = store.stat(CHAT, 100, "unanswered_streak").unwrap();
    assert_that!(streak)
        .named("streak after a reaction was removed")
        .is_equal_to(6);
}
