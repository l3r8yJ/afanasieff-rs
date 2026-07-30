use teloxide::{
    Bot,
    types::{Me, Message},
};

use crate::ops::{
    consts::VINOGRAD_KEYWORD, error::Error, predicates::contains_ignore_case,
    quotes::random_string_from, send::send_reply_message_set_reaction,
};

const POOL: &[&str] = &[
    r"
    С вами я, Данил Виноградов,
    Ночью сплю бодро, не дрочу,
    Что папу уволят с работы,
    И не будет денег.
    Продолжай работать папа,
    Для меня ты воздух, вода и машина,
    Легенда вы, герой для всей семьи,
    Мы поедем за тобой до конца!
    ",
    "нет гавнил это ты пошел нахуй",
    "Я гавнил гавнодавов я щас пишу это с верту за лям долларов",
    "Я Данил Виноградов, у меня сверкающая лысина, Матвей Афанасьев лучший тестостероновый гигант",
    "Попочка ложечка данек гавно давить",
];

/// Sends random vinograd quote.
///
/// # Errors
///
/// This function will return an error if message text is empty.
pub async fn send_random_vinograd_quote(bot: Bot, message: Message, me: Me) -> Result<(), Error> {
    if let Some(s) = random_string_from(POOL) {
        send_reply_message_set_reaction(s, "💩", &bot, &message, &me).await;
    }
    Ok(())
}

#[must_use]
pub fn filter(msg: &Message) -> bool {
    contains_ignore_case(msg, VINOGRAD_KEYWORD)
}
