use asserting::prelude::*;

#[test]
fn exposes_the_quote_sources() {
    let published = afanasieff_rs::ops::consts::SOURCES.len();
    assert_that!(published)
        .named("quote sources published by the library")
        .is_equal_to(3);
}
