use std::time::Duration;

use rand::{rng, seq::IndexedRandom};
use tokio::time::sleep;

use crate::ops::{
    consts::SOURCES,
    intake::preview,
    store::{promote_oldest_matthew_message, with_db},
};

const TICK: Duration = Duration::from_secs(30);

pub async fn start_promoting() {
    loop {
        promote_one_message();
        sleep(TICK).await;
    }
}

fn promote_one_message() {
    let Some(source) = random_source() else {
        return;
    };
    match with_db(|connection| promote_oldest_matthew_message(connection, source)) {
        Some(Some(text)) => log::info!(
            "matthew message promoted into quotes of source '{source}': '{}'",
            preview(&text)
        ),
        Some(None) => log::debug!("no matthew messages waiting to be promoted"),
        None => log::error!("matthew message was not promoted into source '{source}'"),
    }
}

fn random_source() -> Option<&'static str> {
    SOURCES.choose(&mut rng()).copied()
}
