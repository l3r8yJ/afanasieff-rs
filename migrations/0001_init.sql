CREATE TABLE IF NOT EXISTS chats (
    id INTEGER PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS quotes (
    id INTEGER PRIMARY KEY,
    source TEXT NOT NULL,
    text TEXT NOT NULL,
    UNIQUE (source, text)
);

CREATE TABLE IF NOT EXISTS matthew_messages (
    id INTEGER PRIMARY KEY,
    chat_id INTEGER NOT NULL,
    message_id INTEGER NOT NULL,
    sent_at TEXT NOT NULL,
    text TEXT NOT NULL,
    UNIQUE (chat_id, message_id)
);

INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', 'Ты сдохнешь в аду урод');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', 'Я бы тебе просто по твоей лысине вонючей c пыру въебал');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', 'и че ? тебя нахуярить чтоли ты имеешь ввиду ?');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', 'Я ссал стрим на голову петину');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', 'терпим');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', 'Извините');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', 'Хорошо браток идем 1x1 с каждым 5 раундов по пол часа');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', 'Нихуя вы базарите, уроды');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', 'В этот день я и порвал эти шорты');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', 'я петух в законе');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', 'Вот именно, либералы пидорасы');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', 'Губами');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', 'Я Путин');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', 'Я белогвардеец');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', 'все как папа учил, только надо еще голым');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', 'ахтубинск город заднеприводных');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', 'не понял, куколд моя бабушка?');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', 'слышь ты нахуй, баба ты ебаная');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', 'аниме вообще для даунов');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', '/pidor@UserOfTheDayBot');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', 'Тебе хуем жопу закрыли гандон блять');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', 'я принесу тебе говна нахуй');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('matthew', 'хорошо куколд сука');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Ты сдохнешь в аду урод');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Я бы тебе просто по твоей лысине вонючей c пыру въебал');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Да да нет нет, подумать до завтра есть время у тебя');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Мы на сво пойдем родину защищать ?');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'слава богу мы живем в россии, а не в америке');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Да, ракурс делает вещи, в жизни ты бы увидел, писе не поднялась бы');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Девушка бориса');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'ты 1х1 не хочешь пойти попиздиться? чисто за честь владимира путина');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Я за люля кебаб на пятьсот Рублевой купюре');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Я бы вмешался, но мне впн дороже');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Адекватный человек такое читать не будет');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Ну я понял гуляш говно потому что напоминает ссср');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Я брал последнее время пикник и там было 2 разновидности, один обычный, другой какой-то нестандартный, я всегда брал дефолт, решил попробовать экзотику, я взял, попробовал и пхуел, я такого говна не пробовал никогда, это видимо от какого-то местного производителя');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Это 90 айкью юмор я Даун');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Да, давай с тобой подеремся, если я побеждаю, я тебя из квартиры выписываю, а если ты, я просто заплачу и домой пойду');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Манифест куколдистической партии. Принципы куколдизма');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'РАБотают рабы, я созерцал');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'и че ? тебя нахуярить чтоли ты имеешь ввиду ?');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Да он уже в опущенке, там его разбирать даже смысла нет, его смотрят уже только конченные фрико зомби');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Степан как обычно очень сильно всех зауважал и убежал домой');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', '+7 (999) 629-78-39');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Мне нужно 3 месяца на подготовку');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Ты говорил что ты под дулом автомата пенисы сосать будешь');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Те кто хотят, делают, не делают ленивые неудачники, я смог, почему вы не можете ? Все эти коммунисты это просто ленивые люди, которые хотят жить за счет других, это как так получается ? Я сидел, работал, а теперь я должен свой доход разбивать на всех ? Даже тех, кто нихуя не делал ? Да пошел нахуй этот коммунизм блять сралин этот диктатор я его мать ебал, вот щас времена, хочешь покупай джинсы, хочешь езжай куда хочешь, стань кем хочешь, все дорогу открыты, сейчас свобода, а раньше была диктатура, все эти тупорылые бабки которые твердят «раньше было лучше», это просто зомби которые подверженны ностальгии и говорят, что «а вот раньше была трава зеленей», там нахуй и при нацисткой Германии найдутся те кто говорят что было заебись, ебанаты нахуй, сука Путин страну поднял уроды блять идите на работу гандоны нахуй');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Ну в моей личной системе ценностей ты тогда крыса вонючая не рукопожатия');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'ты хуже гитлера');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Хорошо говно собачье');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Да забей блять он уже ебанулся в край этот пердулин блять');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Какая мать');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Ты коммунист');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Закрыл рот падаль');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', '350 рублей');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Трамп объявил неделю антикоммунизма');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Я буду погибать');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Ну мы это проверим в честном бою 1х1 до смерти');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Еще раз кто-нибудь сюда это хуйло скинет, ебальник слетит нахуй');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'мужчин тут нет');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'мы нули');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Данил термит ломает Ваня дебов чинит');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Я русский🐵');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Да не ебашу ч диалоге с собой биоскот ты ебаный');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Ты сначала начни с себя, сделай чтобы тебе было хорошо и потом уже другим помогай');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Анархия дочь шлюхи блять');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('stream', 'Да чето хотел сказать, а потом подумал, хули говорить и так все понятно');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('vinograd', '
    С вами я, Данил Виноградов,
    Ночью сплю бодро, не дрочу,
    Что папу уволят с работы,
    И не будет денег.
    Продолжай работать папа,
    Для меня ты воздух, вода и машина,
    Легенда вы, герой для всей семьи,
    Мы поедем за тобой до конца!
    ');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('vinograd', 'нет гавнил это ты пошел нахуй');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('vinograd', 'Я гавнил гавнодавов я щас пишу это с верту за лям долларов');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('vinograd', 'Я Данил Виноградов, у меня сверкающая лысина, Матвей Афанасьев лучший тестостероновый гигант');
INSERT OR IGNORE INTO quotes (source, text) VALUES ('vinograd', 'Попочка ложечка данек гавно давить');
