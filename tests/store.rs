use afanasieff_rs::ops::consts::MATTHEW_SOURCE;
use afanasieff_rs::ops::store::Store;

#[test]
fn seeds_quotes_when_opening_a_fresh_database() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("afanasieff.db")).unwrap();
    let quote = store.random_quote(MATTHEW_SOURCE).unwrap();
    assert!(
        quote.is_some(),
        "matthew quote from a fresh database was '{quote:?}', expected a seeded quote"
    );
}

#[test]
fn keeps_its_rows_when_reopening_an_existing_database() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("afanasieff.db");
    let first = Store::open(&path).unwrap();
    first.remember_chat(777).unwrap();
    drop(first);
    let second = Store::open(&path).unwrap();
    let chats = second.chats().unwrap();
    assert_eq!(
        chats,
        vec![777],
        "chats after reopening were '{chats:?}', expected '[777]'"
    );
}
