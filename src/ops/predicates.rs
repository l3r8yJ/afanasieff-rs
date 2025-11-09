use teloxide::types::Message;

pub fn contains_ignore_case(msg: Message, keyword: &str) -> bool {
    msg.text()
        .is_some_and(|t| t.to_lowercase().contains(keyword))
}
