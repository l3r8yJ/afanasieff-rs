use afanasieff_rs::ops::consts::MATTHEW_SOURCE;
use afanasieff_rs::ops::store::Store;

#[test]
fn seeds_quotes_when_opening_a_fresh_database() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("afanasieff.db")).unwrap();
    let quote = store.random_quote(MATTHEW_SOURCE).unwrap();
    assert!(
        quote.is_some(),
        "a fresh database is seeded with matthew quotes"
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
    assert_eq!(second.chats().unwrap(), vec![777], "chats survive a reopen");
}
