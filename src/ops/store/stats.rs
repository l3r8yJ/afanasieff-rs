use std::collections::{HashMap, HashSet};

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

    /// Records the member, refreshing their name and username when already known.
    ///
    /// # Errors
    ///
    /// Returns an error when the statement cannot be executed.
    pub fn upsert_member(
        &self,
        chat: i64,
        user: i64,
        username: Option<&str>,
        first_name: &str,
        seen_at: &str,
    ) -> rusqlite::Result<()> {
        self.with(|connection| {
            connection.execute(
                "INSERT INTO members (chat_id, user_id, username, first_name, last_seen) \
                 VALUES (?1, ?2, ?3, ?4, ?5) \
                 ON CONFLICT(chat_id, user_id) \
                 DO UPDATE SET username = ?3, first_name = ?4, last_seen = ?5",
                params![chat, user, username, first_name, seen_at],
            )?;
            Ok(())
        })
    }

    /// Returns the member behind the username, ignoring case.
    ///
    /// # Errors
    ///
    /// Returns an error when the query cannot be executed.
    pub fn member_by_username(&self, chat: i64, username: &str) -> rusqlite::Result<Option<i64>> {
        self.with(|connection| {
            connection
                .query_row(
                    "SELECT user_id FROM members \
                     WHERE chat_id = ?1 AND username = ?2 COLLATE NOCASE",
                    params![chat, username],
                    |row| row.get(0),
                )
                .optional()
        })
    }

    /// Returns the username of the member, when they have one.
    ///
    /// # Errors
    ///
    /// Returns an error when the query cannot be executed.
    pub fn member_username(&self, chat: i64, user: i64) -> rusqlite::Result<Option<String>> {
        self.with(|connection| {
            connection
                .query_row(
                    "SELECT username FROM members WHERE chat_id = ?1 AND user_id = ?2",
                    params![chat, user],
                    |row| row.get(0),
                )
                .optional()
                .map(Option::flatten)
        })
    }

    /// Tells whether the chat has a member row for the given user.
    ///
    /// # Errors
    ///
    /// Returns an error when the query cannot be executed.
    pub fn is_member(&self, chat: i64, user: i64) -> rusqlite::Result<bool> {
        self.with(|connection| {
            connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM members WHERE chat_id = ?1 AND user_id = ?2)",
                params![chat, user],
                |row| row.get(0),
            )
        })
    }

    /// Returns every state value of the chat.
    ///
    /// # Errors
    ///
    /// Returns an error when the query cannot be executed.
    pub fn state(&self, chat: i64) -> rusqlite::Result<HashMap<String, i64>> {
        self.with(|connection| {
            let mut statement =
                connection.prepare("SELECT key, value FROM chat_state WHERE chat_id = ?1")?;
            let rows = statement
                .query_map(params![chat], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect::<rusqlite::Result<HashMap<String, i64>>>()?;
            Ok(rows)
        })
    }

    /// Sets a state value of the chat.
    ///
    /// # Errors
    ///
    /// Returns an error when the statement cannot be executed.
    pub fn set_state(&self, chat: i64, key: &str, value: i64) -> rusqlite::Result<()> {
        self.with(|connection| {
            connection.execute(
                "INSERT INTO chat_state (chat_id, key, value) VALUES (?1, ?2, ?3) \
                 ON CONFLICT(chat_id, key) DO UPDATE SET value = ?3",
                params![chat, key, value],
            )?;
            Ok(())
        })
    }

    /// Returns the codes of the achievements the member already has.
    ///
    /// # Errors
    ///
    /// Returns an error when the query cannot be executed.
    pub fn owned(&self, chat: i64, user: i64) -> rusqlite::Result<HashSet<String>> {
        self.with(|connection| {
            let mut statement = connection
                .prepare("SELECT code FROM achievements WHERE chat_id = ?1 AND user_id = ?2")?;
            let rows = statement
                .query_map(params![chat, user], |row| row.get(0))?
                .collect::<rusqlite::Result<HashSet<String>>>()?;
            Ok(rows)
        })
    }

    /// Gives the achievement to the member, telling whether it was new.
    ///
    /// # Errors
    ///
    /// Returns an error when the statement cannot be executed.
    pub fn unlock(&self, chat: i64, user: i64, code: &str, at: &str) -> rusqlite::Result<bool> {
        self.with(|connection| {
            let inserted = connection.execute(
                "INSERT OR IGNORE INTO achievements (chat_id, user_id, code, unlocked_at) \
                 VALUES (?1, ?2, ?3, ?4)",
                params![chat, user, code, at],
            )?;
            Ok(inserted > 0)
        })
    }

    /// Returns the timestamp at which the member unlocked the achievement, if they have it.
    ///
    /// # Errors
    ///
    /// Returns an error when the query cannot be executed.
    pub fn unlocked_at(
        &self,
        chat: i64,
        user: i64,
        code: &str,
    ) -> rusqlite::Result<Option<String>> {
        self.with(|connection| {
            connection
                .query_row(
                    "SELECT unlocked_at FROM achievements \
                     WHERE chat_id = ?1 AND user_id = ?2 AND code = ?3",
                    params![chat, user, code],
                    |row| row.get(0),
                )
                .optional()
        })
    }
}

const REMEMBERED_PER_CHAT: i64 = 500;

#[derive(Debug)]
pub struct MessageOwner {
    pub user: i64,
    pub quote: Option<i64>,
}

impl Store {
    /// Records who wrote the message and, when given, which quote it carries.
    ///
    /// Writing the same message again refreshes the author and attaches a
    /// quote, but never clears a quote that is already attached. Only the
    /// newest messages of each chat are kept.
    ///
    /// # Errors
    ///
    /// Returns an error when a statement cannot be executed.
    pub fn remember_message(
        &self,
        chat: i64,
        message: i32,
        user: i64,
        quote: Option<i64>,
    ) -> rusqlite::Result<()> {
        self.with(|connection| {
            connection.execute(
                "INSERT INTO message_owners (chat_id, message_id, user_id, quote_id) \
                 VALUES (?1, ?2, ?3, ?4) \
                 ON CONFLICT(chat_id, message_id) \
                 DO UPDATE SET user_id = ?3, quote_id = COALESCE(?4, quote_id)",
                params![chat, message, user, quote],
            )?;
            connection.execute(
                "DELETE FROM message_owners WHERE chat_id = ?1 AND message_id NOT IN \
                 (SELECT message_id FROM message_owners WHERE chat_id = ?1 \
                  ORDER BY message_id DESC LIMIT ?2)",
                params![chat, REMEMBERED_PER_CHAT],
            )?;
            Ok(())
        })
    }

    /// Returns who wrote the message and which quote it carries.
    ///
    /// # Errors
    ///
    /// Returns an error when the query cannot be executed.
    pub fn message_owner(&self, chat: i64, message: i32) -> rusqlite::Result<Option<MessageOwner>> {
        self.with(|connection| {
            connection
                .query_row(
                    "SELECT user_id, quote_id FROM message_owners \
                     WHERE chat_id = ?1 AND message_id = ?2",
                    params![chat, message],
                    |row| {
                        Ok(MessageOwner {
                            user: row.get(0)?,
                            quote: row.get(1)?,
                        })
                    },
                )
                .optional()
        })
    }
}

pub struct Standing {
    pub user: i64,
    pub name: Option<String>,
    pub count: i64,
}

impl Store {
    /// Returns the members of the chat ranked by how many achievements they
    /// own, the one who got there first ahead on a tie.
    ///
    /// # Errors
    ///
    /// Returns an error when the query cannot be executed.
    pub fn leaderboard(&self, chat: i64) -> rusqlite::Result<Vec<Standing>> {
        self.with(|connection| {
            let mut statement = connection.prepare(
                "SELECT a.user_id, m.first_name, COUNT(*) AS owned, MAX(a.unlocked_at) AS latest \
                 FROM achievements a \
                 LEFT JOIN members m ON m.chat_id = a.chat_id AND m.user_id = a.user_id \
                 WHERE a.chat_id = ?1 \
                 GROUP BY a.user_id \
                 ORDER BY owned DESC, latest ASC",
            )?;
            let standings = statement
                .query_map(params![chat], |row| {
                    Ok(Standing {
                        user: row.get(0)?,
                        name: row.get(1)?,
                        count: row.get(2)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<Standing>>>()?;
            Ok(standings)
        })
    }
}

pub struct Member {
    pub user: i64,
    pub name: String,
}

impl Store {
    /// Returns the members of the chat last seen at or after the given moment.
    ///
    /// # Errors
    ///
    /// Returns an error when the query cannot be executed.
    pub fn active_members(&self, chat: i64, since: &str) -> rusqlite::Result<Vec<Member>> {
        self.with(|connection| {
            let mut statement = connection.prepare(
                "SELECT user_id, first_name FROM members \
                 WHERE chat_id = ?1 AND last_seen >= ?2",
            )?;
            let members = statement
                .query_map(params![chat, since], |row| {
                    Ok(Member {
                        user: row.get(0)?,
                        name: row.get(1)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<Member>>>()?;
            Ok(members)
        })
    }

    /// Returns the members of the chat ranked by the given counter, highest
    /// first, leaving out everyone whose counter never moved.
    ///
    /// # Errors
    ///
    /// Returns an error when the query cannot be executed.
    pub fn ranking(&self, chat: i64, key: &str) -> rusqlite::Result<Vec<Standing>> {
        self.with(|connection| {
            let mut statement = connection.prepare(
                "SELECT s.user_id, m.first_name, s.value FROM member_stats s \
                 LEFT JOIN members m ON m.chat_id = s.chat_id AND m.user_id = s.user_id \
                 WHERE s.chat_id = ?1 AND s.key = ?2 AND s.value > 0 \
                 ORDER BY s.value DESC, s.user_id ASC",
            )?;
            let ranked = statement
                .query_map(params![chat, key], |row| {
                    Ok(Standing {
                        user: row.get(0)?,
                        name: row.get(1)?,
                        count: row.get(2)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<Standing>>>()?;
            Ok(ranked)
        })
    }

    /// Returns the member's first name.
    ///
    /// # Errors
    ///
    /// Returns an error when the query cannot be executed.
    pub fn member_name(&self, chat: i64, user: i64) -> rusqlite::Result<Option<String>> {
        self.with(|connection| {
            connection
                .query_row(
                    "SELECT first_name FROM members WHERE chat_id = ?1 AND user_id = ?2",
                    params![chat, user],
                    |row| row.get(0),
                )
                .optional()
        })
    }
}

#[cfg(test)]
mod tests {
    use asserting::prelude::*;

    use crate::ops::store::Store;

    fn store() -> Store {
        Store::in_memory().unwrap()
    }

    #[test]
    fn accumulates_a_counter_across_bumps() {
        let store = store();
        store.bump(42, 7, "messages", 1).unwrap();
        let value = store.bump(42, 7, "messages", 1).unwrap();
        assert_that!(value)
            .named("counter after two bumps")
            .is_equal_to(2);
    }

    #[test]
    fn keeps_counters_of_different_members_apart() {
        let store = store();
        store.bump(42, 7, "messages", 5).unwrap();
        store.bump(42, 8, "messages", 1).unwrap();
        let seven = store.stat(42, 7, "messages").unwrap();
        let eight = store.stat(42, 8, "messages").unwrap();
        assert_that!(seven)
            .named("member seven's counter")
            .is_equal_to(5);
        assert_that!(eight)
            .named("member eight's counter")
            .is_equal_to(1);
    }

    #[test]
    fn reports_zero_for_a_counter_that_was_never_bumped() {
        let store = store();
        let value = store.stat(42, 7, "night_messages").unwrap();
        assert_that!(value)
            .named("counter that was never bumped")
            .is_equal_to(0);
    }

    #[test]
    fn overwrites_a_counter_on_set() {
        let store = store();
        store.bump(42, 7, "longest_message", 100).unwrap();
        store.set_stat(42, 7, "longest_message", 40).unwrap();
        let value = store.stat(42, 7, "longest_message").unwrap();
        assert_that!(value)
            .named("counter after set")
            .is_equal_to(40);
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
        assert_that!(keys)
            .named("member seven's stat keys")
            .contains_exactly(["mat".to_string(), "messages".to_string()]);
        assert_that!(stats.get("messages").copied())
            .named("member seven's message counter")
            .is_equal_to(Some(3));
    }

    #[test]
    fn resolves_a_member_by_username_ignoring_case() {
        let store = store();
        store
            .upsert_member(42, 7, Some("MatthewAFN"), "Матвей", "2026-08-06T10:00:00Z")
            .unwrap();
        let found = store.member_by_username(42, "matthewafn").unwrap();
        assert_that!(found)
            .named("member resolved by username")
            .is_equal_to(Some(7));
    }

    #[test]
    fn updates_a_member_instead_of_duplicating_them() {
        let store = store();
        store
            .upsert_member(42, 7, Some("old"), "Матвей", "2026-08-06T10:00:00Z")
            .unwrap();
        store
            .upsert_member(42, 7, Some("new"), "Матвей А", "2026-08-06T11:00:00Z")
            .unwrap();
        let by_old = store.member_by_username(42, "old").unwrap();
        let by_new = store.member_by_username(42, "new").unwrap();
        assert_that!(by_old)
            .named("member resolved by the old username")
            .is_none();
        assert_that!(by_new)
            .named("member resolved by the new username")
            .is_equal_to(Some(7));
    }

    #[test]
    fn keeps_chat_state_per_chat() {
        let store = store();
        store.set_state(42, "last_message_id", 100).unwrap();
        store.set_state(-100, "last_message_id", 7).unwrap();
        store.set_state(42, "last_message_id", 101).unwrap();
        let first = store.state(42).unwrap();
        let second = store.state(-100).unwrap();
        assert_that!(first.get("last_message_id").copied())
            .named("chat 42's last message id")
            .is_equal_to(Some(101));
        assert_that!(second.get("last_message_id").copied())
            .named("chat -100's last message id")
            .is_equal_to(Some(7));
    }

    #[test]
    fn unlocks_an_achievement_only_once() {
        let store = store();
        let first = store
            .unlock(42, 7, "terpim", "2026-08-06T10:00:00Z")
            .unwrap();
        let again = store
            .unlock(42, 7, "terpim", "2026-08-06T11:00:00Z")
            .unwrap();
        assert_that!(first).named("first unlock").is_true();
        assert_that!(again)
            .named("second unlock of the same achievement")
            .is_false();
    }

    #[test]
    fn lists_the_codes_a_member_owns() {
        let store = store();
        store
            .unlock(42, 7, "terpim", "2026-08-06T10:00:00Z")
            .unwrap();
        store.unlock(42, 7, "haha", "2026-08-06T10:00:00Z").unwrap();
        store
            .unlock(42, 8, "robot", "2026-08-06T10:00:00Z")
            .unwrap();
        let owned = store.owned(42, 7).unwrap();
        let mut codes = owned.iter().cloned().collect::<Vec<String>>();
        codes.sort();
        assert_that!(codes)
            .named("owned codes")
            .contains_exactly(["haha".to_string(), "terpim".to_string()]);
    }

    #[test]
    fn remembers_who_wrote_a_message() {
        let store = store();
        store.remember_message(42, 7, 100, None).unwrap();
        let owner = store.message_owner(42, 7).unwrap().unwrap();
        assert_that!(owner.user)
            .named("owner of the message")
            .is_equal_to(100);
        assert_that!(owner.quote)
            .named("quote of a plain message")
            .is_none();
    }

    #[test]
    fn keeps_the_quote_when_a_later_write_does_not_carry_one() {
        let store = store();
        store.remember_message(42, 7, 100, Some(5)).unwrap();
        store.remember_message(42, 7, 100, None).unwrap();
        let owner = store.message_owner(42, 7).unwrap().unwrap();
        assert_that!(owner.quote)
            .named("quote after a write without one")
            .is_equal_to(Some(5));
    }

    #[test]
    fn attaches_a_quote_to_an_already_remembered_message() {
        let store = store();
        store.remember_message(42, 7, 100, None).unwrap();
        store.remember_message(42, 7, 100, Some(9)).unwrap();
        let owner = store.message_owner(42, 7).unwrap().unwrap();
        assert_that!(owner.quote)
            .named("quote attached later")
            .is_equal_to(Some(9));
    }

    #[test]
    fn forgets_a_message_that_fell_out_of_the_window() {
        let store = store();
        for message in 1..=520 {
            store.remember_message(42, message, 100, None).unwrap();
        }
        let oldest = store.message_owner(42, 1).unwrap();
        let newest = store.message_owner(42, 520).unwrap();
        assert_that!(oldest)
            .named("owner of the oldest message")
            .is_none();
        assert_that!(newest)
            .named("owner of the newest message")
            .is_some();
    }

    #[test]
    fn keeps_the_windows_of_two_chats_apart() {
        let store = store();
        store.remember_message(42, 7, 100, None).unwrap();
        for message in 1..=520 {
            store.remember_message(-100, message, 200, None).unwrap();
        }
        let survived = store.message_owner(42, 7).unwrap();
        assert_that!(survived)
            .named("owner in the untouched chat")
            .is_some();
    }

    #[test]
    fn reports_no_owner_for_an_unknown_message() {
        let store = store();
        let owner = store.message_owner(42, 7).unwrap();
        assert_that!(owner)
            .named("owner of an unknown message")
            .is_none();
    }

    #[test]
    fn lists_only_the_members_seen_since_the_given_moment() {
        let store = store();
        store
            .upsert_member(42, 1, Some("fresh"), "Свежий", "2026-08-07T10:00:00+00:00")
            .unwrap();
        store
            .upsert_member(42, 2, Some("stale"), "Древний", "2026-01-01T10:00:00+00:00")
            .unwrap();
        let active = store
            .active_members(42, "2026-07-08T00:00:00+00:00")
            .unwrap();
        let names = active
            .iter()
            .map(|member| member.name.clone())
            .collect::<Vec<String>>();
        assert_that!(names)
            .named("active members")
            .contains_exactly(["Свежий".to_string()]);
    }

    #[test]
    fn keeps_the_active_members_of_two_chats_apart() {
        let store = store();
        store
            .upsert_member(42, 1, Some("here"), "Наш", "2026-08-07T10:00:00+00:00")
            .unwrap();
        store
            .upsert_member(-100, 2, Some("there"), "Чужой", "2026-08-07T10:00:00+00:00")
            .unwrap();
        let active = store
            .active_members(42, "2026-07-08T00:00:00+00:00")
            .unwrap();
        assert_that!(active.len())
            .named("active members of one chat")
            .is_equal_to(1);
    }

    #[test]
    fn ranks_members_by_a_counter_highest_first() {
        let store = store();
        store
            .upsert_member(42, 1, Some("one"), "Первый", "2026-08-07T10:00:00+00:00")
            .unwrap();
        store
            .upsert_member(42, 2, Some("two"), "Второй", "2026-08-07T10:00:00+00:00")
            .unwrap();
        store.set_stat(42, 1, "cuckold_days", 3).unwrap();
        store.set_stat(42, 2, "cuckold_days", 9).unwrap();
        let ranked = store.ranking(42, "cuckold_days").unwrap();
        let names = ranked
            .iter()
            .map(|standing| standing.name.clone().unwrap_or_default())
            .collect::<Vec<String>>();
        assert_that!(names)
            .named("ranking")
            .contains_exactly(["Второй".to_string(), "Первый".to_string()]);
    }

    #[test]
    fn breaks_a_tie_in_the_ranking_by_ascending_user_id_on_every_call() {
        let store = store();
        store
            .upsert_member(42, 2, Some("two"), "Второй", "2026-08-07T10:00:00+00:00")
            .unwrap();
        store
            .upsert_member(42, 1, Some("one"), "Первый", "2026-08-07T10:00:00+00:00")
            .unwrap();
        store.set_stat(42, 2, "cuckold_days", 3).unwrap();
        store.set_stat(42, 1, "cuckold_days", 3).unwrap();
        let first_call = store
            .ranking(42, "cuckold_days")
            .unwrap()
            .iter()
            .map(|standing| standing.user)
            .collect::<Vec<i64>>();
        let second_call = store
            .ranking(42, "cuckold_days")
            .unwrap()
            .iter()
            .map(|standing| standing.user)
            .collect::<Vec<i64>>();
        assert_that!(first_call)
            .named("ranking order on a tie")
            .contains_exactly([1, 2]);
        assert_that!(second_call)
            .named("ranking order on a repeated call")
            .contains_exactly([1, 2]);
    }

    #[test]
    fn leaves_a_zero_counter_out_of_the_ranking() {
        let store = store();
        store
            .upsert_member(42, 1, Some("one"), "Первый", "2026-08-07T10:00:00+00:00")
            .unwrap();
        store.set_stat(42, 1, "cuckold_days", 0).unwrap();
        let ranked = store.ranking(42, "cuckold_days").unwrap();
        assert_that!(ranked.len())
            .named("ranking of a zero counter")
            .is_equal_to(0);
    }

    #[test]
    fn returns_the_name_of_a_known_member() {
        let store = store();
        store
            .upsert_member(42, 7, Some("m"), "Матвей", "2026-08-07T10:00:00+00:00")
            .unwrap();
        let name = store.member_name(42, 7).unwrap();
        assert_that!(name)
            .named("member name")
            .is_equal_to(Some("Матвей".to_string()));
    }
}
