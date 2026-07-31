use std::sync::atomic::AtomicI32;

use afanasieff_rs::ops::intake::observe;
use afanasieff_rs::ops::store::{MATTHEW_USERNAME, Store};
use teloxide_tests::IntoUpdate;
use teloxide_tests::{MockMessageText, MockUser};

const SPOKEN: &str = "это сообщение матвея точно длиннее десяти символов";

#[test]
fn serves_back_a_message_matthew_once_sent() {
    let store = Store::in_memory().unwrap();
    let spoken = MockMessageText::new()
        .text(SPOKEN)
        .from(MockUser::new().username(MATTHEW_USERNAME).build());
    observe(
        &store,
        spoken
            .into_update(&AtomicI32::new(1))
            .pop()
            .expect("one update is produced"),
    );
    let promoted = store
        .promote_oldest_matthew_message("matthew")
        .unwrap()
        .expect("the observed message is promoted into quotes");
    assert_eq!(
        promoted, SPOKEN,
        "promoted quote was '{promoted}', expected the exact message matthew sent '{SPOKEN}'"
    );
    let servable = store.random_quote("matthew").unwrap();
    assert!(
        servable.is_some(),
        "random_quote for source 'matthew' returned '{servable:?}', expected a servable quote after promotion"
    );
}
