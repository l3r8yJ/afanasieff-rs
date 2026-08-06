use std::collections::HashMap;

use rusqlite::{OptionalExtension, params};

use crate::ops::store::Store;

impl Store {
    /// Adds `by` to the counter and returns its new value.
    ///
    /// # Errors
    ///
    /// Returns an error when the statement cannot be executed.
    pub fn bump(&self, chat: i64, user: i64, key: &str, by: i64) -> rusqlite::Result<i64> {
        self.with(|connection| {
            connection.query_row(
                "INSERT INTO member_stats (chat_id, user_id, key, value) VALUES (?1, ?2, ?3, ?4) \
                 ON CONFLICT(chat_id, user_id, key) DO UPDATE SET value = value + ?4 \
                 RETURNING value",
                params![chat, user, key, by],
                |row| row.get(0),
            )
        })
    }

    /// Sets the counter to the given value.
    ///
    /// # Errors
    ///
    /// Returns an error when the statement cannot be executed.
    pub fn set_stat(&self, chat: i64, user: i64, key: &str, value: i64) -> rusqlite::Result<()> {
        self.with(|connection| {
            connection.execute(
                "INSERT INTO member_stats (chat_id, user_id, key, value) VALUES (?1, ?2, ?3, ?4) \
                 ON CONFLICT(chat_id, user_id, key) DO UPDATE SET value = ?4",
                params![chat, user, key, value],
            )?;
            Ok(())
        })
    }

    /// Returns the counter, or zero when it was never set.
    ///
    /// # Errors
    ///
    /// Returns an error when the query cannot be executed.
    pub fn stat(&self, chat: i64, user: i64, key: &str) -> rusqlite::Result<i64> {
        self.with(|connection| {
            connection
                .query_row(
                    "SELECT value FROM member_stats WHERE chat_id = ?1 AND user_id = ?2 AND key = ?3",
                    params![chat, user, key],
                    |row| row.get(0),
                )
                .optional()
                .map(Option::unwrap_or_default)
        })
    }

    /// Returns every counter of the given member.
    ///
    /// # Errors
    ///
    /// Returns an error when the query cannot be executed.
    pub fn stats(&self, chat: i64, user: i64) -> rusqlite::Result<HashMap<String, i64>> {
        self.with(|connection| {
            let mut statement = connection.prepare(
                "SELECT key, value FROM member_stats WHERE chat_id = ?1 AND user_id = ?2",
            )?;
            let rows = statement
                .query_map(params![chat, user], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect::<rusqlite::Result<HashMap<String, i64>>>()?;
            Ok(rows)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::ops::store::Store;

    fn store() -> Store {
        Store::in_memory().unwrap()
    }

    #[test]
    fn accumulates_a_counter_across_bumps() {
        let store = store();
        store.bump(42, 7, "messages", 1).unwrap();
        let value = store.bump(42, 7, "messages", 1).unwrap();
        assert_eq!(
            value, 2,
            "counter after two bumps was '{value}', expected '2'"
        );
    }

    #[test]
    fn keeps_counters_of_different_members_apart() {
        let store = store();
        store.bump(42, 7, "messages", 5).unwrap();
        store.bump(42, 8, "messages", 1).unwrap();
        let seven = store.stat(42, 7, "messages").unwrap();
        let eight = store.stat(42, 8, "messages").unwrap();
        assert_eq!(
            (seven, eight),
            (5, 1),
            "counters were '{:?}', expected '(5, 1)'",
            (seven, eight)
        );
    }

    #[test]
    fn reports_zero_for_a_counter_that_was_never_bumped() {
        let store = store();
        let value = store.stat(42, 7, "night_messages").unwrap();
        assert_eq!(
            value, 0,
            "counter that was never bumped was '{value}', expected '0'"
        );
    }

    #[test]
    fn overwrites_a_counter_on_set() {
        let store = store();
        store.bump(42, 7, "longest_message", 100).unwrap();
        store.set_stat(42, 7, "longest_message", 40).unwrap();
        let value = store.stat(42, 7, "longest_message").unwrap();
        assert_eq!(value, 40, "counter after set was '{value}', expected '40'");
    }

    #[test]
    fn reads_every_counter_of_a_member_at_once() {
        let store = store();
        store.bump(42, 7, "messages", 3).unwrap();
        store.bump(42, 7, "mat", 1).unwrap();
        store.bump(42, 8, "messages", 9).unwrap();
        let stats = store.stats(42, 7).unwrap();
        let mut keys = stats.keys().cloned().collect::<Vec<String>>();
        keys.sort();
        assert_eq!(
            (keys.as_slice(), stats.get("messages").copied()),
            (
                ["mat".to_string(), "messages".to_string()].as_slice(),
                Some(3)
            ),
            "stats of member seven were '{stats:?}', expected only their own two counters"
        );
    }
}
