use std::sync::atomic::AtomicI32;

use afanasieff_rs::ops::consts::MATTHEW_SOURCE;
use afanasieff_rs::ops::intake::observe;
use afanasieff_rs::ops::store::{MATTHEW_USERNAME, Store};
use asserting::prelude::*;
use teloxide_tests::IntoUpdate;
use teloxide_tests::{MockMessageText, MockUser};

const SPOKEN: &str = "это сообщение матвея точно длиннее десяти символов";

#[test]
fn promotes_a_matthew_message_into_quotes_verbatim() {
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
        .promote_oldest_matthew_message(MATTHEW_SOURCE)
        .unwrap()
        .expect("the observed message is promoted into quotes");
    assert_that!(promoted.as_str())
        .named("promoted quote")
        .is_equal_to(SPOKEN);
}
