use std::path::Path;
use std::sync::{Mutex, PoisonError};

use rusqlite::{Connection, OptionalExtension, params};

mod stats;

pub const MATTHEW_USERNAME: &str = "MatthewAFN";

const MIGRATIONS: &[&str] = &[
    include_str!("../../../migrations/0001_init.sql"),
    include_str!("../../../migrations/0002_achievements.sql"),
    include_str!("../../../migrations/0003_reactions.sql"),
];

pub struct Store {
    connection: Mutex<Connection>,
}

impl Store {
    /// Opens the database at the given path and applies every pending migration.
    ///
    /// # Errors
    ///
    /// Returns an error when the file cannot be opened or a migration fails.
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        Self::from_connection(Connection::open(path)?)
    }

    /// Opens an in-memory database with the same schema production uses.
    ///
    /// # Errors
    ///
    /// Returns an error when a migration fails.
    pub fn in_memory() -> rusqlite::Result<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(connection: Connection) -> rusqlite::Result<Self> {
        migrate(&connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub(super) fn with<T>(
        &self,
        call: impl FnOnce(&Connection) -> rusqlite::Result<T>,
    ) -> rusqlite::Result<T> {
        let connection = self
            .connection
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        call(&connection)
    }

    /// Returns a random quote of the given source.
    ///
    /// # Errors
    ///
    /// Returns an error when the query cannot be executed.
    pub fn random_quote(&self, source: &str) -> rusqlite::Result<Option<String>> {
        Ok(self.random_quote_with_id(source)?.map(|(_, text)| text))
    }

    /// Returns a random quote of the given source together with its identifier.
    ///
    /// # Errors
    ///
    /// Returns an error when the query cannot be executed.
    pub fn random_quote_with_id(&self, source: &str) -> rusqlite::Result<Option<(i64, String)>> {
        self.with(|connection| {
            connection
                .query_row(
                    "SELECT id, text FROM quotes WHERE source = ?1 \
                     ORDER BY (MIN(score, 20) + 1) * (ABS(RANDOM()) % 1000) DESC LIMIT 1",
                    params![source],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()
        })
    }

    /// Returns every stored quote of the given source.
    ///
    /// # Errors
    ///
    /// Returns an error when the query cannot be executed.
    pub fn quotes(&self, source: &str) -> rusqlite::Result<Vec<String>> {
        self.with(|connection| {
            let mut statement = connection.prepare("SELECT text FROM quotes WHERE source = ?1")?;
            let quotes = statement
                .query_map(params![source], |row| row.get(0))?
                .collect::<rusqlite::Result<Vec<String>>>()?;
            Ok(quotes)
        })
    }

    /// Remembers a chat the bot has seen.
    ///
    /// # Errors
    ///
    /// Returns an error when the insert cannot be executed.
    pub fn remember_chat(&self, chat: i64) -> rusqlite::Result<()> {
        self.with(|connection| {
            connection.execute(
                "INSERT OR IGNORE INTO chats (id) VALUES (?1)",
                params![chat],
            )?;
            Ok(())
        })
    }

    /// Returns every chat the bot has seen.
    ///
    /// # Errors
    ///
    /// Returns an error when the query cannot be executed.
    pub fn chats(&self) -> rusqlite::Result<Vec<i64>> {
        self.with(|connection| {
            let mut statement = connection.prepare("SELECT id FROM chats")?;
            let chats = statement
                .query_map([], |row| row.get(0))?
                .collect::<rusqlite::Result<Vec<i64>>>()?;
            Ok(chats)
        })
    }

    /// Stores a message written by Matthew, telling whether it was a new one.
    ///
    /// # Errors
    ///
    /// Returns an error when the insert cannot be executed.
    pub fn store_matthew_message(
        &self,
        chat: i64,
        message: i32,
        sent_at: &str,
        text: &str,
    ) -> rusqlite::Result<bool> {
        self.with(|connection| {
            let stored = connection.execute(
                "INSERT OR IGNORE INTO matthew_messages (chat_id, message_id, sent_at, text) VALUES (?1, ?2, ?3, ?4)",
                params![chat, message, sent_at, text],
            )?;
            Ok(stored > 0)
        })
    }

    /// Moves the message Matthew wrote first into the quotes of the given source.
    ///
    /// Returns the moved text, or nothing when no message is waiting.
    ///
    /// # Errors
    ///
    /// Returns an error when the move cannot be executed.
    pub fn promote_oldest_matthew_message(&self, source: &str) -> rusqlite::Result<Option<String>> {
        self.with(|connection| {
            let transaction = connection.unchecked_transaction()?;
            let oldest = transaction
                .query_row(
                    "SELECT id, text FROM matthew_messages ORDER BY sent_at, id LIMIT 1",
                    [],
                    |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
                )
                .optional()?;
            let Some((id, text)) = oldest else {
                return Ok(None);
            };
            transaction.execute(
                "INSERT OR IGNORE INTO quotes (source, text) VALUES (?1, ?2)",
                params![source, text],
            )?;
            transaction.execute("DELETE FROM matthew_messages WHERE id = ?1", params![id])?;
            transaction.commit()?;
            Ok(Some(text))
        })
    }

    /// Adds to the score of the quote, which makes it come up more often.
    ///
    /// # Errors
    ///
    /// Returns an error when the statement cannot be executed.
    pub fn bump_quote_score(&self, quote: i64) -> rusqlite::Result<()> {
        self.with(|connection| {
            connection.execute(
                "UPDATE quotes SET score = score + 1 WHERE id = ?1",
                params![quote],
            )?;
            Ok(())
        })
    }

    /// Returns the score of the quote.
    ///
    /// # Errors
    ///
    /// Returns an error when the query cannot be executed, including when the
    /// quote does not exist.
    pub fn quote_score(&self, quote: i64) -> rusqlite::Result<i64> {
        self.with(|connection| {
            connection.query_row(
                "SELECT score FROM quotes WHERE id = ?1",
                params![quote],
                |row| row.get(0),
            )
        })
    }

    /// Returns every quote of every source, which is the corpus the generator
    /// builds its chain from.
    ///
    /// # Errors
    ///
    /// Returns an error when the query cannot be executed.
    pub fn all_quotes(&self) -> rusqlite::Result<Vec<String>> {
        self.with(|connection| {
            let mut statement = connection.prepare("SELECT text FROM quotes")?;
            let quotes = statement
                .query_map([], |row| row.get(0))?
                .collect::<rusqlite::Result<Vec<String>>>()?;
            Ok(quotes)
        })
    }

    /// Moves one named message Matthew wrote into the quotes of his own
    /// source, ahead of the queue, and returns the identifier of the quote.
    ///
    /// Returns nothing when the message is no longer waiting, which happens
    /// when the promotion cron took it first.
    ///
    /// # Errors
    ///
    /// Returns an error when a statement cannot be executed.
    pub fn promote_matthew_message(
        &self,
        chat: i64,
        message: i32,
    ) -> rusqlite::Result<Option<i64>> {
        self.with(|connection| {
            let transaction = connection.unchecked_transaction()?;
            let waiting = transaction
                .query_row(
                    "SELECT id, text FROM matthew_messages WHERE chat_id = ?1 AND message_id = ?2",
                    params![chat, message],
                    |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
                )
                .optional()?;
            let Some((id, text)) = waiting else {
                return Ok(None);
            };
            transaction.execute(
                "INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', ?1)",
                params![text],
            )?;
            let quote = transaction.query_row(
                "SELECT id FROM quotes WHERE source = 'matthew' AND text = ?1",
                params![text],
                |row| row.get(0),
            )?;
            transaction.execute("DELETE FROM matthew_messages WHERE id = ?1", params![id])?;
            transaction.commit()?;
            Ok(Some(quote))
        })
    }
}

/// Applies every migration the database has not seen yet.
///
/// # Errors
///
/// Returns an error when a migration cannot be executed.
fn migrate(connection: &Connection) -> rusqlite::Result<()> {
    let applied: usize = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    for (number, migration) in MIGRATIONS.iter().enumerate().skip(applied) {
        connection.execute_batch(migration)?;
        connection.pragma_update(None, "user_version", number + 1)?;
        log::info!("migration '{}' applied", number + 1);
    }
    Ok(())
}

impl Store {
    /// Drops the `quotes` table, so a test can force every subsequent quote
    /// read against this `Store` to fail. Exists for tests only and is not
    /// part of the intended API.
    ///
    /// # Errors
    ///
    /// Returns an error when the drop cannot be executed.
    #[doc(hidden)]
    pub fn drop_quotes_table_for_tests(&self) -> rusqlite::Result<()> {
        self.with(|connection| connection.execute_batch("DROP TABLE quotes"))
    }
}

#[cfg(test)]
impl Store {
    fn quotes_of(&self, source: &str) -> i64 {
        self.with(|connection| {
            connection.query_row(
                "SELECT COUNT(*) FROM quotes WHERE source = ?1",
                params![source],
                |row| row.get(0),
            )
        })
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use asserting::prelude::*;

    use super::{MIGRATIONS, Store};

    fn store() -> Store {
        Store::in_memory().unwrap()
    }

    #[test]
    fn migrates_every_source_of_quotes() {
        let store = store();
        for source in ["stream", "matthew", "vinograd"] {
            let stored = store.quotes_of(source);
            assert_that!(stored)
                .named("stored quotes")
                .is_greater_than(0);
        }
    }

    #[test]
    fn returns_migrated_quote_of_asked_source() {
        let store = store();
        let quote = store.random_quote("vinograd").unwrap().unwrap();
        let source: String = store
            .with(|connection| {
                connection.query_row(
                    "SELECT source FROM quotes WHERE text = ?1",
                    [&quote],
                    |row| row.get(0),
                )
            })
            .unwrap();
        assert_that!(source)
            .named("source of the returned quote")
            .is_equal_to("vinograd");
    }

    #[test]
    fn returns_no_quote_of_unknown_source() {
        let store = store();
        let quote = store.random_quote("stepan").unwrap();
        assert_that!(quote)
            .named("quote of an unknown source")
            .is_none();
    }

    #[test]
    fn applies_migrations_once() {
        let store = store();
        let before = store.quotes_of("matthew");
        store.with(super::migrate).unwrap();
        let after = store.quotes_of("matthew");
        assert_that!(after)
            .named("quotes of source 'matthew' after a second migrate")
            .is_equal_to(before);
    }

    #[test]
    fn records_applied_migrations_count() {
        let store = store();
        let applied: usize = store
            .with(|connection| connection.query_row("PRAGMA user_version", [], |row| row.get(0)))
            .unwrap();
        assert_that!(applied)
            .named("applied migrations")
            .is_equal_to(MIGRATIONS.len());
    }

    #[test]
    fn remembers_every_chat_once() {
        let store = store();
        store.remember_chat(42).unwrap();
        store.remember_chat(42).unwrap();
        store.remember_chat(-100).unwrap();
        let mut remembered = store.chats().unwrap();
        remembered.sort_unstable();
        assert_that!(remembered)
            .named("remembered chats")
            .contains_exactly([-100, 42]);
    }

    #[test]
    fn stores_matthew_message_once_per_message_id() {
        let store = store();
        store
            .store_matthew_message(42, 7, "2026-07-30T17:00:00Z", "терпим")
            .unwrap();
        store
            .store_matthew_message(42, 7, "2026-07-30T17:00:00Z", "терпим")
            .unwrap();
        store
            .store_matthew_message(42, 8, "2026-07-30T17:01:00Z", "Извините")
            .unwrap();
        let stored: i64 = store
            .with(|connection| {
                connection.query_row("SELECT COUNT(*) FROM matthew_messages", [], |row| {
                    row.get(0)
                })
            })
            .unwrap();
        assert_that!(stored)
            .named("stored messages count")
            .is_equal_to(2);
    }

    #[test]
    fn promotes_oldest_matthew_message_into_quotes() {
        let store = store();
        store
            .store_matthew_message(42, 7, "2026-07-30T17:00:00Z", "первое")
            .unwrap();
        store
            .store_matthew_message(42, 8, "2026-07-30T17:01:00Z", "второе")
            .unwrap();
        let promoted = store
            .promote_oldest_matthew_message("stream")
            .unwrap()
            .unwrap();
        let waiting: i64 = store
            .with(|connection| {
                connection.query_row("SELECT COUNT(*) FROM matthew_messages", [], |row| {
                    row.get(0)
                })
            })
            .unwrap();
        let quoted: i64 = store
            .with(|connection| {
                connection.query_row(
                    "SELECT COUNT(*) FROM quotes WHERE source = 'stream' AND text = 'первое'",
                    [],
                    |row| row.get(0),
                )
            })
            .unwrap();
        assert_that!(promoted.as_str())
            .named("promoted text")
            .is_equal_to("первое");
        assert_that!(waiting)
            .named("messages still waiting")
            .is_equal_to(1);
        assert_that!(quoted)
            .named("quotes stored under 'stream'")
            .is_equal_to(1);
    }

    #[test]
    fn promotes_nothing_when_no_matthew_message_waits() {
        let store = store();
        let promoted = store.promote_oldest_matthew_message("stream").unwrap();
        assert_that!(promoted)
            .named("promotion of an empty table")
            .is_none();
    }

    #[test]
    fn tells_new_matthew_message_from_already_stored_one() {
        let store = store();
        let first = store
            .store_matthew_message(42, 7, "2026-07-30T17:00:00Z", "терпим")
            .unwrap();
        let again = store
            .store_matthew_message(42, 7, "2026-07-30T17:00:00Z", "терпим")
            .unwrap();
        assert_that!(first).named("first store").is_true();
        assert_that!(again)
            .named("second store of the same message")
            .is_false();
    }

    #[test]
    fn favours_a_quote_with_a_higher_score() {
        let store = store();
        store
            .with(|connection| connection.execute_batch("DELETE FROM quotes"))
            .unwrap();
        store
            .with(|connection| {
                connection.execute_batch(
                    "INSERT INTO quotes (source, text, score) VALUES ('matthew', 'редкая', 0); \
                     INSERT INTO quotes (source, text, score) VALUES ('matthew', 'частая', 9);",
                )
            })
            .unwrap();
        let mut often = 0;
        for _ in 0..200 {
            if store.random_quote("matthew").unwrap().as_deref() == Some("частая") {
                often += 1;
            }
        }
        assert_that!(often)
            .named("draws of the higher scored quote")
            .is_greater_than(100);
    }

    #[test]
    fn keeps_a_zero_score_quote_reachable_against_an_absurd_rival_score() {
        let store = store();
        store
            .with(|connection| connection.execute_batch("DELETE FROM quotes"))
            .unwrap();
        store
            .with(|connection| {
                connection.execute_batch(
                    "INSERT INTO quotes (source, text, score) VALUES ('matthew', 'обычная', 0); \
                     INSERT INTO quotes (source, text, score) VALUES ('matthew', 'звезда', 1000000);",
                )
            })
            .unwrap();
        let mut drawn = 0;
        for _ in 0..500 {
            if store.random_quote("matthew").unwrap().as_deref() == Some("обычная") {
                drawn += 1;
            }
        }
        assert_that!(drawn)
            .named("draws of the zero-score quote against an absurd rival score")
            .is_greater_than(0);
    }

    #[test]
    fn raises_the_score_of_a_quote() {
        let store = store();
        let id: i64 = store
            .with(|connection| {
                connection.query_row("SELECT id FROM quotes LIMIT 1", [], |row| row.get(0))
            })
            .unwrap();
        store.bump_quote_score(id).unwrap();
        store.bump_quote_score(id).unwrap();
        let score: i64 = store
            .with(|connection| {
                connection.query_row("SELECT score FROM quotes WHERE id = ?1", [id], |row| {
                    row.get(0)
                })
            })
            .unwrap();
        assert_that!(score)
            .named("score after two reactions")
            .is_equal_to(2);
    }

    #[test]
    fn reads_every_quote_of_every_source() {
        let store = store();
        let all = store.all_quotes().unwrap();
        let matthew = store.quotes("matthew").unwrap();
        assert_that!(all.len())
            .named("all quotes")
            .is_greater_than(matthew.len());
    }

    #[test]
    fn promotes_one_named_message_into_the_matthew_source() {
        let store = store();
        store
            .store_matthew_message(42, 7, "2026-08-07T10:00:00Z", "вне очереди")
            .unwrap();
        store
            .store_matthew_message(42, 8, "2026-08-07T10:01:00Z", "в очереди")
            .unwrap();
        let promoted = store.promote_matthew_message(42, 7).unwrap();
        let matthew = store.quotes("matthew").unwrap();
        let waiting: i64 = store
            .with(|connection| {
                connection.query_row("SELECT COUNT(*) FROM matthew_messages", [], |row| {
                    row.get(0)
                })
            })
            .unwrap();
        assert_that!(promoted).named("promoted quote id").is_some();
        assert_that!(matthew)
            .named("matthew quotes")
            .contains("вне очереди".to_string());
        assert_that!(waiting)
            .named("messages still waiting")
            .is_equal_to(1);
    }

    #[test]
    fn promotes_nothing_for_a_message_the_cron_already_took() {
        let store = store();
        let promoted = store.promote_matthew_message(42, 7).unwrap();
        assert_that!(promoted)
            .named("promotion of a message that is gone")
            .is_none();
    }
}
