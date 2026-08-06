use std::collections::{HashMap, HashSet};

const TERPIM_STREAK: i64 = 10;
const OPRAVDAN_APOLOGIES: i64 = 3;
const CHAIN_LENGTH: i64 = 20;
const SHORTY_CHARS: i64 = 1000;
const POTOLOK_NIGHTS: i64 = 500;
const BELOGVARDEEC_POLITICS: i64 = 100;
const VPN_IGNORED: i64 = 20;
const GAVNIL_REPLIES: i64 = 100;
const LYSINA_MENTIONS: i64 = 50;
const STREAM_MENTIONS: i64 = 50;
const HAHA_LAUGHS: i64 = 100;
const ROBOT_REPLIES: i64 = 100;
const CALLS_IGNORED: i64 = 10;
const KLON_MONOLOGUES: i64 = 20;
const IVANCHUK_MENTIONS: i64 = 50;
const SOFIZM_MESSAGES: i64 = 50;
const PETUKH_ACHIEVEMENTS: usize = 5;

pub struct Stats(HashMap<String, i64>);

impl Stats {
    #[must_use]
    pub fn new(values: HashMap<String, i64>) -> Self {
        Self(values)
    }

    #[must_use]
    pub fn get(&self, key: &str) -> i64 {
        self.0.get(key).copied().unwrap_or_default()
    }

    #[must_use]
    pub fn max_with_prefix(&self, prefix: &str) -> i64 {
        self.0
            .iter()
            .filter(|(key, _)| key.starts_with(prefix))
            .map(|(_, value)| *value)
            .max()
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Achievement {
    Terpim,
    Opravdan,
    OdinNaOdin,
    Shorty,
    Potolok,
    Belogvardeec,
    Vpn,
    Gavnil,
    Lysina,
    SsalStream,
    Haha,
    Robot,
    VseZanyaty,
    Klon,
    Ivanchuk,
    Sofizm,
    Petukh,
}

impl Achievement {
    pub const ALL: &'static [Self] = &[
        Self::Terpim,
        Self::Opravdan,
        Self::OdinNaOdin,
        Self::Shorty,
        Self::Potolok,
        Self::Belogvardeec,
        Self::Vpn,
        Self::Gavnil,
        Self::Lysina,
        Self::SsalStream,
        Self::Haha,
        Self::Robot,
        Self::VseZanyaty,
        Self::Klon,
        Self::Ivanchuk,
        Self::Sofizm,
        Self::Petukh,
    ];

    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Terpim => "terpim",
            Self::Opravdan => "opravdan",
            Self::OdinNaOdin => "odin_na_odin",
            Self::Shorty => "shorty",
            Self::Potolok => "potolok",
            Self::Belogvardeec => "belogvardeec",
            Self::Vpn => "vpn",
            Self::Gavnil => "gavnil",
            Self::Lysina => "lysina",
            Self::SsalStream => "ssal_stream",
            Self::Haha => "haha",
            Self::Robot => "robot",
            Self::VseZanyaty => "vse_zanyaty",
            Self::Klon => "klon",
            Self::Ivanchuk => "ivanchuk",
            Self::Sofizm => "sofizm",
            Self::Petukh => "petukh",
        }
    }

    #[must_use]
    pub fn title(self) -> &'static str {
        match self {
            Self::Terpim => "Терпим",
            Self::Opravdan => "Оправдан в зале суда",
            Self::OdinNaOdin => "1х1 5 раундов по полчаса",
            Self::Shorty => "В этот день я и порвал эти шорты",
            Self::Potolok => "9500 потолок",
            Self::Belogvardeec => "Я белогвардеец",
            Self::Vpn => "Мне впн дороже",
            Self::Gavnil => "Я гавнил гавнодавов",
            Self::Lysina => "Сверкающая лысина",
            Self::SsalStream => "Я ссал стрим на голову",
            Self::Haha => "ХААХХААХАХАХАХАХ",
            Self::Robot => "Робот ебаный",
            Self::VseZanyaty => "К сожалению все заняты",
            Self::Klon => "Клонировать себя, чтоб в 5 в доту",
            Self::Ivanchuk => "Иванчук, обработать",
            Self::Sofizm => "Ну понятно, софизм",
            Self::Petukh => "Петух в законе",
        }
    }

    #[must_use]
    pub fn hint(self) -> &'static str {
        match self {
            Self::Terpim => "10 своих сообщений подряд, на которые никто не ответил",
            Self::Opravdan => "3 раза выматериться и извиниться в течение минуты",
            Self::OdinNaOdin => "20 реплаев подряд с одним человеком за полчаса",
            Self::Shorty => "сообщение длиннее 1000 символов",
            Self::Potolok => "500 сообщений с 02:00 до 06:00",
            Self::Belogvardeec => "100 сообщений про политику",
            Self::Vpn => "20 раз промолчать час после того, как тебя позвали",
            Self::Gavnil => "100 раз получить цитату от бота",
            Self::Lysina => "50 упоминаний Виноградова",
            Self::SsalStream => "50 упоминаний стрима",
            Self::Haha => "100 сообщений из одного ржача",
            Self::Robot => "100 ответов боту",
            Self::VseZanyaty => "10 раз позвать в игру и получить тишину",
            Self::Klon => "20 раз накидать 5 сообщений подряд",
            Self::Ivanchuk => "50 тегов одного и того же человека",
            Self::Sofizm => "50 сообщений длиннее 300 символов",
            Self::Petukh => "собрать пять любых других",
        }
    }

    #[must_use]
    pub fn text(self) -> &'static str {
        match self {
            Self::Terpim => "Тебе никто не ответил. И мы потерпим.",
            Self::Opravdan => "Обписян в зале суда. Оправдан в зале суда.",
            Self::OdinNaOdin => "Хорошо браток идем 1x1 с каждым 5 раундов по пол часа.",
            Self::Shorty => "Пёр как дед. Шорты не выдержали.",
            Self::Potolok => "у сатаника 9к, у меня 5600, чувствуете на каком я уровне?",
            Self::Belogvardeec => "Я Путин. Слава роду славянскому.",
            Self::Vpn => "Я бы вмешался, но мне впн дороже.",
            Self::Gavnil => "Я щас пишу это с верту за лям долларов.",
            Self::Lysina => "Я бы тебе просто по твоей лысине вонючей c пыру въебал.",
            Self::SsalStream => "Стрим, тебе с потолка капает.",
            Self::Haha => "Это 90 айкью юмор я Даун.",
            Self::Robot => {
                "но вопрос, а вот картину он нарисует робот ебаный? сетку москитную он повесит?"
            }
            Self::VseZanyaty => "Лол, а кто тебя позовет то.",
            Self::Klon => "я с детства мечтал клонировать себя, чтобы в 5 в доту играть.",
            Self::Ivanchuk => "он за меня общается.",
            Self::Sofizm => "Казалось бы.",
            Self::Petukh => "я петух в законе.",
        }
    }

    #[must_use]
    pub fn progress(self, stats: &Stats) -> Option<(i64, i64)> {
        match self {
            Self::Terpim => Some((stats.get("unanswered_streak"), TERPIM_STREAK)),
            Self::Opravdan => Some((stats.get("apologies"), OPRAVDAN_APOLOGIES)),
            Self::OdinNaOdin => Some((stats.get("chain_len"), CHAIN_LENGTH)),
            Self::Shorty => Some((stats.get("longest_message"), SHORTY_CHARS)),
            Self::Potolok => Some((stats.get("night_messages"), POTOLOK_NIGHTS)),
            Self::Belogvardeec => Some((stats.get("politics"), BELOGVARDEEC_POLITICS)),
            Self::Vpn => Some((stats.get("ignored_pings"), VPN_IGNORED)),
            Self::Gavnil => Some((stats.get("bot_replies"), GAVNIL_REPLIES)),
            Self::Lysina => Some((stats.get("vinograd_mentions"), LYSINA_MENTIONS)),
            Self::SsalStream => Some((stats.get("stream_mentions"), STREAM_MENTIONS)),
            Self::Haha => Some((stats.get("laugh_only"), HAHA_LAUGHS)),
            Self::Robot => Some((stats.get("replies_to_bot"), ROBOT_REPLIES)),
            Self::VseZanyaty => Some((stats.get("unanswered_calls"), CALLS_IGNORED)),
            Self::Klon => Some((stats.get("monologues"), KLON_MONOLOGUES)),
            Self::Ivanchuk => Some((stats.max_with_prefix("mention:"), IVANCHUK_MENTIONS)),
            Self::Sofizm => Some((stats.get("long_messages"), SOFIZM_MESSAGES)),
            Self::Petukh => None,
        }
    }

    #[must_use]
    pub fn is_unlocked(self, stats: &Stats, owned: &HashSet<String>) -> bool {
        match self.progress(stats) {
            Some((current, threshold)) => current >= threshold,
            None => owned.len() >= PETUKH_ACHIEVEMENTS,
        }
    }
}

/// Returns the achievements the member earned but does not own yet.
#[must_use]
pub fn unlocked(stats: &Stats, owned: &HashSet<String>) -> Vec<Achievement> {
    Achievement::ALL
        .iter()
        .copied()
        .filter(|achievement| !owned.contains(achievement.code()))
        .filter(|achievement| achievement.is_unlocked(stats, owned))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::{Achievement, Stats, unlocked};

    fn stats(pairs: &[(&str, i64)]) -> Stats {
        Stats::new(
            pairs
                .iter()
                .map(|(key, value)| ((*key).to_string(), *value))
                .collect::<HashMap<String, i64>>(),
        )
    }

    #[test]
    fn gives_every_achievement_a_unique_code() {
        let codes = Achievement::ALL
            .iter()
            .map(|achievement| achievement.code())
            .collect::<HashSet<&str>>();
        assert_eq!(
            codes.len(),
            Achievement::ALL.len(),
            "unique codes were '{}', expected '{}'",
            codes.len(),
            Achievement::ALL.len()
        );
    }

    #[test]
    fn unlocks_a_counter_achievement_at_its_threshold() {
        let below = unlocked(&stats(&[("unanswered_streak", 9)]), &HashSet::new());
        let at = unlocked(&stats(&[("unanswered_streak", 10)]), &HashSet::new());
        assert_eq!(
            (below.len(), at.first().map(Achievement::code)),
            (0, Some("terpim")),
            "unlocking below and at the threshold reported '{:?}', expected none then 'terpim'",
            (below.len(), at.first().map(Achievement::code))
        );
    }

    #[test]
    fn skips_an_achievement_the_member_already_owns() {
        let owned = HashSet::from(["terpim".to_string()]);
        let given = unlocked(&stats(&[("unanswered_streak", 40)]), &owned);
        assert!(
            given.is_empty(),
            "unlocking an owned achievement reported '{:?}', expected none",
            given.iter().map(|a| a.code()).collect::<Vec<&str>>()
        );
    }

    #[test]
    fn unlocks_the_meta_achievement_on_five_others() {
        let owned = ["terpim", "haha", "robot", "sofizm", "vpn"]
            .iter()
            .map(|code| (*code).to_string())
            .collect::<HashSet<String>>();
        let given = unlocked(&stats(&[]), &owned);
        assert_eq!(
            given.first().map(Achievement::code),
            Some("petukh"),
            "meta unlock reported '{:?}', expected 'petukh'",
            given.first().map(Achievement::code)
        );
    }

    #[test]
    fn unlocks_the_mention_achievement_from_the_largest_pair_counter() {
        let given = unlocked(
            &stats(&[("mention:8", 12), ("mention:9", 50)]),
            &HashSet::new(),
        );
        assert_eq!(
            given.first().map(Achievement::code),
            Some("ivanchuk"),
            "mention unlock reported '{:?}', expected 'ivanchuk'",
            given.first().map(Achievement::code)
        );
    }

    #[test]
    fn reports_progress_towards_a_counter_achievement() {
        let progress = Achievement::Terpim.progress(&stats(&[("unanswered_streak", 4)]));
        assert_eq!(
            progress,
            Some((4, 10)),
            "progress was '{progress:?}', expected 'Some((4, 10))'"
        );
    }
}
