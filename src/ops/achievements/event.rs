use chrono::{DateTime, FixedOffset, Timelike, Utc};
use teloxide::types::{Message, MessageEntityKind, Update, UpdateKind};

use crate::ops::achievements::text;

const MOSCOW_OFFSET_SECONDS: i32 = 3 * 3600;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mention {
    Id(i64),
    Username(String),
}

#[derive(Debug, Clone)]
pub struct Event {
    pub chat: i64,
    pub user: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub created_at: DateTime<Utc>,
    pub message_id: i32,
    pub len: usize,
    pub hour_msk: u32,
    pub reply_to_user: Option<i64>,
    pub reply_to_bot: bool,
    pub mentions: Vec<Mention>,
    pub mat: bool,
    pub apology: bool,
    pub politics: bool,
    pub laugh_only: bool,
    pub call_to_play: bool,
    pub stream: bool,
    pub vinograd: bool,
}

impl Event {
    /// Reads a message update into the facts the achievements need.
    ///
    /// Returns nothing for updates that are not messages, for messages
    /// without an author and for messages written by a bot.
    #[must_use]
    pub fn parse(update: &Update) -> Option<Self> {
        let UpdateKind::Message(message) = &update.kind else {
            return None;
        };
        let author = message.from.as_ref()?;
        if author.is_bot {
            return None;
        }
        let text = message.text().unwrap_or_default();
        let tokens = text::tokens(text);
        let offset = FixedOffset::east_opt(MOSCOW_OFFSET_SECONDS)?;
        let reply = message
            .reply_to_message()
            .and_then(|replied| replied.from.as_ref());
        Some(Self {
            chat: message.chat.id.0,
            user: cast(author.id.0),
            username: author.username.clone(),
            first_name: author.first_name.clone(),
            created_at: message.date,
            message_id: message.id.0,
            len: text.chars().count(),
            hour_msk: message.date.with_timezone(&offset).hour(),
            reply_to_user: reply.map(|user| cast(user.id.0)),
            reply_to_bot: reply.is_some_and(|user| user.is_bot),
            mentions: mentions(message),
            mat: text::has_mat(&tokens),
            apology: text::is_apology(text, &tokens),
            politics: text::is_politics(&tokens),
            laugh_only: text::is_laugh_only(&tokens),
            call_to_play: text::is_call_to_play(&tokens),
            stream: text::mentions_stream(&tokens),
            vinograd: text::mentions_vinograd(&tokens),
        })
    }
}

fn mentions(message: &Message) -> Vec<Mention> {
    message
        .parse_entities()
        .unwrap_or_default()
        .iter()
        .filter_map(|entity| match entity.kind() {
            MessageEntityKind::TextMention { user } => Some(Mention::Id(cast(user.id.0))),
            MessageEntityKind::Mention => Some(Mention::Username(
                entity.text().trim_start_matches('@').to_lowercase(),
            )),
            _ => None,
        })
        .collect()
}

fn cast(id: u64) -> i64 {
    i64::try_from(id).unwrap_or(i64::MAX)
}
