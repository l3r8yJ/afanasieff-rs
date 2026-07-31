#[test]
fn exposes_the_quote_sources() {
    assert_eq!(
        afanasieff_rs::ops::consts::SOURCES.len(),
        3,
        "three quote sources are published by the library"
    );
}
