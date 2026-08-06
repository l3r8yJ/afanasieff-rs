#[test]
fn exposes_the_quote_sources() {
    let published = afanasieff_rs::ops::consts::SOURCES.len();
    assert_eq!(
        published, 3,
        "quote sources published by the library were '{published}', expected '3'"
    );
}
