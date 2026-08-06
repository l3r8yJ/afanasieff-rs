use std::path::Path;
use std::sync::{Mutex, PoisonError};

use rusqlite::{Connection, OptionalExtension, params};

mod stats;

pub const MATTHEW_USERNAME: &str = "MatthewAFN";

const MIGRATIONS: &[&str] = &[
    include_str!("../../../migrations/0001_init.sql"),
    include_str!("../../../migrations/0002_achievements.sql"),
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
        self.with(|connection| {
            connection
                .query_row(
                    "SELECT text FROM quotes WHERE source = ?1 ORDER BY RANDOM() LIMIT 1",
                    params![source],
                    |row| row.get(0),
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
    use super::{MIGRATIONS, Store};

    fn store() -> Store {
        Store::in_memory().unwrap()
    }

    #[test]
    fn migrates_every_source_of_quotes() {
        let store = store();
        for source in ["stream", "matthew", "vinograd"] {
            let stored = store.quotes_of(source);
            assert!(
                stored > 0,
                "quotes of source '{source}' were '{stored}', expected more than '0'"
            );
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
        assert_eq!(
            source, "vinograd",
            "quote '{quote}' came from source '{source}', expected 'vinograd'"
        );
    }

    #[test]
    fn returns_no_quote_of_unknown_source() {
        let store = store();
        let quote = store.random_quote("stepan").unwrap();
        assert!(
            quote.is_none(),
            "quote of unknown source was '{quote:?}', expected none"
        );
    }

    #[test]
    fn applies_migrations_once() {
        let store = store();
        let before = store.quotes_of("matthew");
        store.with(super::migrate).unwrap();
        let after = store.quotes_of("matthew");
        assert_eq!(
            after, before,
            "quotes of source 'matthew' after a second migrate were '{after}', expected '{before}'"
        );
    }

    #[test]
    fn records_applied_migrations_count() {
        let store = store();
        let applied: usize = store
            .with(|connection| connection.query_row("PRAGMA user_version", [], |row| row.get(0)))
            .unwrap();
        assert_eq!(
            applied,
            MIGRATIONS.len(),
            "applied migrations were '{applied}', expected '{}'",
            MIGRATIONS.len()
        );
    }

    #[test]
    fn remembers_every_chat_once() {
        let store = store();
        store.remember_chat(42).unwrap();
        store.remember_chat(42).unwrap();
        store.remember_chat(-100).unwrap();
        let mut remembered = store.chats().unwrap();
        remembered.sort_unstable();
        assert_eq!(
            remembered,
            vec![-100, 42],
            "remembered chats were '{remembered:?}', expected '[-100, 42]'"
        );
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
        assert_eq!(
            stored, 2,
            "stored messages count was '{stored}', expected '2'"
        );
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
        assert_eq!(
            (promoted.as_str(), waiting, quoted),
            ("первое", 1, 1),
            "promotion reported '{promoted}' with '{waiting}' waiting and '{quoted}' quoted, \
             expected 'первое' with '1' waiting and '1' quoted"
        );
    }

    #[test]
    fn promotes_nothing_when_no_matthew_message_waits() {
        let store = store();
        let promoted = store.promote_oldest_matthew_message("stream").unwrap();
        assert!(
            promoted.is_none(),
            "promotion of an empty table reported '{promoted:?}', expected none"
        );
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
        assert_eq!(
            (first, again),
            (true, false),
            "storing the same message twice reported '{:?}', expected '(true, false)'",
            (first, again)
        );
    }
}
