use std::sync::Arc;
use std::time::Duration;

use rand::{rng, seq::IndexedRandom};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::ops::{consts::SOURCES, intake::preview, store::Store};

const TICK: Duration = Duration::from_secs(30);

pub async fn start_promoting(store: Arc<Store>, shutdown: CancellationToken) {
    loop {
        tokio::select! {
            () = shutdown.cancelled() => break,
            () = sleep(TICK) => promote_one_message(&store),
        }
    }
}

fn promote_one_message(store: &Store) {
    let Some(source) = random_source() else {
        return;
    };
    match store.promote_oldest_matthew_message(source) {
        Ok(Some(text)) => log::info!(
            "matthew message promoted into quotes of source '{source}': '{}'",
            preview(&text)
        ),
        Ok(None) => log::debug!("no matthew messages waiting to be promoted"),
        Err(error) => {
            log::error!("matthew message was not promoted into source '{source}': '{error}'");
        }
    }
}

fn random_source() -> Option<&'static str> {
    SOURCES.choose(&mut rng()).copied()
}
