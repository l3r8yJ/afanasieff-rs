use std::sync::{LazyLock, Mutex};

use rusqlite::{Connection, OptionalExtension, params};

pub const MATTHEW_USERNAME: &str = "MatthewAFN";

const DB_FILE: &str = "afanasieff.db";

const MIGRATIONS: &[&str] = &[include_str!("../../migrations/0001_init.sql")];

static DB: LazyLock<Mutex<Connection>> = LazyLock::new(|| {
    let connection = Connection::open(DB_FILE)
        .unwrap_or_else(|error| panic!("cannot open database '{DB_FILE}': '{error}'"));
    migrate(&connection)
        .unwrap_or_else(|error| panic!("cannot migrate database '{DB_FILE}': '{error}'"));
    Mutex::new(connection)
});

/// Runs the given call against the shared database, logging failures.
pub fn with_db<T>(call: impl FnOnce(&Connection) -> rusqlite::Result<T>) -> Option<T> {
    let connection = DB.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    match call(&connection) {
        Ok(value) => Some(value),
        Err(error) => {
            log::error!("database call failed: '{error}'");
            None
        }
    }
}

/// Applies every migration the database has not seen yet.
///
/// # Errors
///
/// Returns an error when a migration cannot be executed.
pub fn migrate(connection: &Connection) -> rusqlite::Result<()> {
    let applied: usize = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    for (number, migration) in MIGRATIONS.iter().enumerate().skip(applied) {
        connection.execute_batch(migration)?;
        connection.pragma_update(None, "user_version", number + 1)?;
        log::info!("migration '{}' applied", number + 1);
    }
    Ok(())
}

/// Returns a random quote of the given source.
///
/// # Errors
///
/// Returns an error when the query cannot be executed.
pub fn random_quote(connection: &Connection, source: &str) -> rusqlite::Result<Option<String>> {
    connection
        .query_row(
            "SELECT text FROM quotes WHERE source = ?1 ORDER BY RANDOM() LIMIT 1",
            params![source],
            |row| row.get(0),
        )
        .optional()
}

/// Remembers a chat the bot has seen.
///
/// # Errors
///
/// Returns an error when the insert cannot be executed.
pub fn remember_chat(connection: &Connection, chat: i64) -> rusqlite::Result<()> {
    connection.execute(
        "INSERT OR IGNORE INTO chats (id) VALUES (?1)",
        params![chat],
    )?;
    Ok(())
}

/// Returns every chat the bot has seen.
///
/// # Errors
///
/// Returns an error when the query cannot be executed.
pub fn chats(connection: &Connection) -> rusqlite::Result<Vec<i64>> {
    let mut statement = connection.prepare("SELECT id FROM chats")?;
    let chats = statement
        .query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<Vec<i64>>>()?;
    Ok(chats)
}

/// Stores a message written by Matthew, telling whether it was a new one.
///
/// # Errors
///
/// Returns an error when the insert cannot be executed.
pub fn store_matthew_message(
    connection: &Connection,
    chat: i64,
    message: i32,
    sent_at: &str,
    text: &str,
) -> rusqlite::Result<bool> {
    let stored = connection.execute(
        "INSERT OR IGNORE INTO matthew_messages (chat_id, message_id, sent_at, text)
         VALUES (?1, ?2, ?3, ?4)",
        params![chat, message, sent_at, text],
    )?;
    Ok(stored > 0)
}

/// Moves the message Matthew wrote first into the quotes of the given source.
///
/// Returns the moved text, or nothing when no message is waiting.
///
/// # Errors
///
/// Returns an error when the move cannot be executed.
pub fn promote_oldest_matthew_message(
    connection: &Connection,
    source: &str,
) -> rusqlite::Result<Option<String>> {
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
}

#[cfg(test)]
mod tests {
    use super::{
        Connection, MIGRATIONS, chats, migrate, promote_oldest_matthew_message, random_quote,
        remember_chat, store_matthew_message,
    };

    fn connection() -> Connection {
        let connection = Connection::open_in_memory().unwrap();
        migrate(&connection).unwrap();
        connection
    }

    fn quotes_of(connection: &Connection, source: &str) -> i64 {
        connection
            .query_row(
                "SELECT COUNT(*) FROM quotes WHERE source = ?1",
                [source],
                |row| row.get(0),
            )
            .unwrap()
    }

    #[test]
    fn migrates_every_source_of_quotes() {
        let connection = connection();
        for source in ["stream", "matthew", "vinograd"] {
            let stored = quotes_of(&connection, source);
            assert!(
                stored > 0,
                "quotes of source '{source}' were '{stored}', expected more than '0'"
            );
        }
    }

    #[test]
    fn returns_migrated_quote_of_asked_source() {
        let connection = connection();
        let quote = random_quote(&connection, "vinograd").unwrap().unwrap();
        let source: String = connection
            .query_row(
                "SELECT source FROM quotes WHERE text = ?1",
                [&quote],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            source, "vinograd",
            "quote '{quote}' came from source '{source}', expected 'vinograd'"
        );
    }

    #[test]
    fn returns_no_quote_of_unknown_source() {
        let connection = connection();
        let quote = random_quote(&connection, "stepan").unwrap();
        assert!(
            quote.is_none(),
            "quote of unknown source was '{quote:?}', expected none"
        );
    }

    #[test]
    fn applies_migrations_once() {
        let connection = connection();
        let before = quotes_of(&connection, "matthew");
        migrate(&connection).unwrap();
        let after = quotes_of(&connection, "matthew");
        assert_eq!(
            after, before,
            "quotes of source 'matthew' after a second migrate were '{after}', expected '{before}'"
        );
    }

    #[test]
    fn records_applied_migrations_count() {
        let connection = connection();
        let applied: usize = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
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
        let connection = connection();
        remember_chat(&connection, 42).unwrap();
        remember_chat(&connection, 42).unwrap();
        remember_chat(&connection, -100).unwrap();
        let mut remembered = chats(&connection).unwrap();
        remembered.sort_unstable();
        assert_eq!(
            remembered,
            vec![-100, 42],
            "remembered chats were '{remembered:?}', expected '[-100, 42]'"
        );
    }

    #[test]
    fn stores_matthew_message_once_per_message_id() {
        let connection = connection();
        store_matthew_message(&connection, 42, 7, "2026-07-30T17:00:00Z", "терпим").unwrap();
        store_matthew_message(&connection, 42, 7, "2026-07-30T17:00:00Z", "терпим").unwrap();
        store_matthew_message(&connection, 42, 8, "2026-07-30T17:01:00Z", "Извините").unwrap();
        let stored: i64 = connection
            .query_row("SELECT COUNT(*) FROM matthew_messages", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(
            stored, 2,
            "stored messages count was '{stored}', expected '2'"
        );
    }

    #[test]
    fn promotes_oldest_matthew_message_into_quotes() {
        let connection = connection();
        store_matthew_message(&connection, 42, 7, "2026-07-30T17:00:00Z", "первое").unwrap();
        store_matthew_message(&connection, 42, 8, "2026-07-30T17:01:00Z", "второе").unwrap();
        let promoted = promote_oldest_matthew_message(&connection, "stream")
            .unwrap()
            .unwrap();
        let waiting: i64 = connection
            .query_row("SELECT COUNT(*) FROM matthew_messages", [], |row| {
                row.get(0)
            })
            .unwrap();
        let quoted: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM quotes WHERE source = 'stream' AND text = 'первое'",
                [],
                |row| row.get(0),
            )
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
        let connection = connection();
        let promoted = promote_oldest_matthew_message(&connection, "stream").unwrap();
        assert!(
            promoted.is_none(),
            "promotion of an empty table reported '{promoted:?}', expected none"
        );
    }

    #[test]
    fn tells_new_matthew_message_from_already_stored_one() {
        let connection = connection();
        let first =
            store_matthew_message(&connection, 42, 7, "2026-07-30T17:00:00Z", "терпим").unwrap();
        let again =
            store_matthew_message(&connection, 42, 7, "2026-07-30T17:00:00Z", "терпим").unwrap();
        assert_eq!(
            (first, again),
            (true, false),
            "storing the same message twice reported '{:?}', expected '(true, false)'",
            (first, again)
        );
    }
}
