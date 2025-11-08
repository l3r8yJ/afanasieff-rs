use teloxide::{
    Bot,
    types::{Me, Message},
};

use crate::ops::{error::Error, quotes::random_string_from, send::send_reply_message_set_reaction};

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
/// # Panics
///
/// Panics if message text is empty.
///
/// # Errors
///
/// This function will return an error if message text is empty.
pub async fn send_radom_vinograd_quote(bot: Bot, message: Message, me: Me) -> Result<(), Error> {
    Ok(send_reply_message_set_reaction(random_string_from(POOL), "🍇", &bot, &message, &me).await)
}
