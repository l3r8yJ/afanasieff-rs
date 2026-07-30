use rand::{Rng, rng};
use teloxide::{
    Bot,
    types::{Me, Message},
};

use crate::ops::{
    consts::MATTHEW_KEYWORD, error::Error, predicates::contains_ignore_case,
    quotes::random_string_from, send::send_reply_message_set_reaction,
};

const POOL: &[&str] = &[
    "Ты сдохнешь в аду урод",
    "Я бы тебе просто по твоей лысине вонючей c пыру въебал",
    "и че ? тебя нахуярить чтоли ты имеешь ввиду ?",
    "Я ссал стрим на голову петину",
    "терпим",
    "Извините",
    "Хорошо браток идем 1x1 с каждым 5 раундов по пол часа",
    "Нихуя вы базарите, уроды",
    "В этот день я и порвал эти шорты",
    "я петух в законе",
    "Вот именно, либералы пидорасы",
    "Губами",
    "Я Путин",
    "Я белогвардеец",
    "все как папа учил, только надо еще голым",
    "ахтубинск город заднеприводных",
    "не понял, куколд моя бабушка?",
    "слышь ты нахуй, баба ты ебаная",
    "аниме вообще для даунов",
    "/pidor@UserOfTheDayBot",
    "Тебе хуем жопу закрыли гандон блять",
    "я принесу тебе говна нахуй",
    "хорошо куколд сука",
];

#[must_use]
pub fn filter(msg: &Message) -> bool {
    contains_ignore_case(msg, MATTHEW_KEYWORD)
}

/// Send random quote with 30% chance.
///
/// # Errors
///
/// This function will return an error if message text was empty.
pub async fn send_random_matthew_quote(bot: Bot, message: Message, me: Me) -> Result<(), Error> {
    if should_reply()
        && let Some(s) = random_string_from(POOL)
    {
        send_reply_message_set_reaction(s, "💔", &bot, &message, &me).await;
    }
    Ok(())
}

/// Return true with 30% chance.
fn should_reply() -> bool {
    let mut rng = rng();
    rng.random_bool(0.3) // 30% chance for reply (as irl)
}
