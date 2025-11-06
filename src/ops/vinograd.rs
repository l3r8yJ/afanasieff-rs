use rand::seq::IndexedRandom;
use teloxide::{
    Bot,
    payloads::SetMessageReactionSetters,
    prelude::Requester,
    sugar::request::RequestReplyExt,
    types::{Me, Message, ReactionType},
};

use crate::ops::error::Error;

fn random_quote() -> String {
    let pool = vec![
        r###"
        С вами я, Данил Виноградов, 
        Ночью сплю бодро, не дрочу,
        Что папу уволят с работы,
        И не будет денег.
        Продолжай работать папа, 
        Для меня ты воздух, вода и машина,
        Легенда вы, герой для всей семьи, 
        Мы поедем за тобой до конца!
        "###,
        "нет гавнил это ты пошел нахуй",
        "Я гавнил гавнодавов я щас пишу это с верту за лям долларов",
        "Я Данил Виноградов, у меня сверкающая лысина, Матвей Афанасьев лучший тестостероновый гигант",
        "Попочка ложечка данек гавно давить",
    ];
    let mut rng = rand::rng();
    match pool.choose(&mut rng) {
        Some(q) => q.to_string(),
        None => panic!("Can't find some quotes"),
    }
}

pub async fn process_vinograd_msg(bot: Bot, message: Message, _: Me) -> Result<(), Error> {
    match message.text() {
        Some(_) => {
            let q = random_quote();
            let _ = bot
                .send_message(message.chat.id, q)
                .reply_to(message.id)
                .await;
            let _ = bot
                .set_message_reaction(message.chat.id, message.id)
                .reaction(vec![ReactionType::Emoji {
                    emoji: "🍇".to_string(),
                }])
                .await;
            Ok(())
        }
        None => Ok(()),
    }
}
