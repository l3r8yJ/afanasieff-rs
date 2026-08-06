use crate::ops::achievements::words::{
    APOLOGY_EXACT, APOLOGY_PHRASES, LAUGH_EXACT, LAUGH_LETTERS, MAT_EXCEPTIONS, MAT_ROOTS,
    PLAY_EXACT, PLAY_PREFIXES, POLITICS_EXACT, POLITICS_PREFIXES, PREFIXES, STREAM_WORDS,
    VINOGRAD_WORDS,
};

#[must_use]
pub fn tokens(text: &str) -> Vec<String> {
    normalize(text)
        .split(|c: char| !c.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .collect()
}

#[must_use]
pub fn has_mat(tokens: &[String]) -> bool {
    tokens.iter().any(|token| is_mat(token))
}

fn normalize(text: &str) -> String {
    let mut normalized = String::with_capacity(text.len());
    let mut previous = None;
    let mut run = 0;
    for character in text.to_lowercase().chars().map(fold) {
        if Some(character) == previous {
            run += 1;
        } else {
            run = 1;
            previous = Some(character);
        }
        if run <= 1 {
            normalized.push(character);
        }
    }
    normalized
}

fn fold(character: char) -> char {
    match character {
        'a' => 'а',
        'e' => 'е',
        'o' => 'о',
        'p' => 'р',
        'c' => 'с',
        'x' => 'х',
        'y' => 'у',
        'k' => 'к',
        'b' => 'в',
        'm' => 'м',
        'h' => 'н',
        't' => 'т',
        other => other,
    }
}

fn is_mat(token: &str) -> bool {
    !MAT_EXCEPTIONS
        .iter()
        .any(|exception| token.starts_with(exception))
        && MAT_ROOTS
            .iter()
            .any(|root| starts_after_prefix(token, root))
}

#[must_use]
pub fn is_call_to_play(tokens: &[String]) -> bool {
    tokens.iter().any(|token| {
        PLAY_EXACT.contains(&token.as_str())
            || PLAY_PREFIXES.iter().any(|prefix| token.starts_with(prefix))
    })
}

#[must_use]
pub fn is_politics(tokens: &[String]) -> bool {
    tokens.iter().any(|token| {
        POLITICS_EXACT.contains(&token.as_str())
            || POLITICS_PREFIXES
                .iter()
                .any(|prefix| token.starts_with(prefix))
    })
}

#[must_use]
pub fn is_apology(text: &str, tokens: &[String]) -> bool {
    let normalized = normalize(text);
    tokens
        .iter()
        .any(|token| APOLOGY_EXACT.contains(&token.as_str()))
        || APOLOGY_PHRASES
            .iter()
            .any(|phrase| normalized.contains(phrase))
}

#[must_use]
pub fn is_laugh_only(tokens: &[String]) -> bool {
    !tokens.is_empty()
        && tokens.iter().all(|token| {
            LAUGH_EXACT.contains(&token.as_str())
                || (token.chars().count() >= 3
                    && token.contains('х')
                    && token.chars().all(|c| LAUGH_LETTERS.contains(&c)))
        })
}

#[must_use]
pub fn mentions_stream(tokens: &[String]) -> bool {
    contains_any(tokens, STREAM_WORDS)
}

#[must_use]
pub fn mentions_vinograd(tokens: &[String]) -> bool {
    contains_any(tokens, VINOGRAD_WORDS)
}

fn contains_any(tokens: &[String], words: &[&str]) -> bool {
    tokens
        .iter()
        .any(|token| words.iter().any(|word| token.starts_with(word)))
}

fn starts_after_prefix(token: &str, root: &str) -> bool {
    token.starts_with(root)
        || PREFIXES.iter().any(|prefix| {
            token.strip_prefix(prefix).is_some_and(|rest| {
                rest.starts_with(root)
                    || rest
                        .strip_prefix(['ъ', 'ь'])
                        .is_some_and(|after_separator| after_separator.starts_with(root))
            })
        })
}

#[cfg(test)]
mod tests {
    use super::{
        APOLOGY_EXACT, APOLOGY_PHRASES, LAUGH_EXACT, MAT_EXCEPTIONS, MAT_ROOTS, PLAY_EXACT,
        PLAY_PREFIXES, POLITICS_EXACT, POLITICS_PREFIXES, PREFIXES, STREAM_WORDS, VINOGRAD_WORDS,
        has_mat, is_apology, is_call_to_play, is_laugh_only, is_politics, mentions_stream,
        mentions_vinograd, normalize, tokens,
    };

    fn mat(text: &str) -> bool {
        has_mat(&tokens(text))
    }

    #[test]
    fn collapses_repeated_letters_into_one() {
        let collapsed = tokens("хуууууй");
        assert_eq!(
            collapsed,
            vec!["хуй".to_string()],
            "tokens were '{collapsed:?}', expected the repeats collapsed to 'хуй'"
        );
    }

    #[test]
    fn folds_latin_lookalikes_into_cyrillic() {
        let folded = tokens("CYKA");
        assert_eq!(
            folded,
            vec!["сука".to_string()],
            "tokens were '{folded:?}', expected the latin letters folded into 'сука'"
        );
    }

    #[test]
    fn splits_on_everything_that_is_not_a_letter_or_a_digit() {
        let split = tokens("идем 1х1, ага!");
        assert_eq!(
            split,
            vec!["идем".to_string(), "1х1".to_string(), "ага".to_string()],
            "tokens were '{split:?}', expected three tokens"
        );
    }

    #[test]
    fn finds_mat_behind_a_prefix() {
        for text in ["въебал", "нахуярить", "ебало", "мудак", "блядь", "хуйло"]
        {
            assert!(
                mat(text),
                "text '{text}' was not detected as mat, expected it to be"
            );
        }
    }

    #[test]
    fn leaves_innocent_words_alone() {
        for text in [
            "требуется",
            "себе",
            "хлебом",
            "мудрый",
            "бляха",
            "херсон",
            "сукно",
        ] {
            assert!(
                !mat(text),
                "text '{text}' was detected as mat, expected it not to be"
            );
        }
    }

    #[test]
    fn finds_a_call_to_play_by_exact_short_words() {
        for text in ["го", "го в доту", "катку?", "кс го", "1х1 давай", "погнали"]
        {
            let found = is_call_to_play(&tokens(text));
            assert!(
                found,
                "text '{text}' was not read as a call to play, expected it to be"
            );
        }
    }

    #[test]
    fn does_not_read_a_call_to_play_inside_longer_words() {
        for text in ["город красивый", "много всего", "гонка была"]
        {
            let found = is_call_to_play(&tokens(text));
            assert!(
                !found,
                "text '{text}' was read as a call to play, expected it not to be"
            );
        }
    }

    #[test]
    fn tells_politics_from_ordinary_words() {
        let political = is_politics(&tokens("путин и либералы"));
        let free = is_politics(&tokens("свобода это свое дело"));
        assert_eq!(
            (political, free),
            (true, false),
            "politics detection reported '{:?}', expected '(true, false)'",
            (political, free)
        );
    }

    #[test]
    fn finds_an_apology_in_words_and_in_bigrams() {
        for text in [
            "извините",
            "сорян",
            "да лан",
            "прошу прощения",
            "я был не прав",
        ] {
            let found = is_apology(text, &tokens(text));
            assert!(
                found,
                "text '{text}' was not read as an apology, expected it to be"
            );
        }
    }

    #[test]
    fn finds_a_message_made_only_of_laughter() {
        let laughs = ["ахаха", "ХААХХААХАХАХАХАХ", "хех", "лол"];
        let words = ["ахаха ну ты даешь", "нормально"];
        for text in laughs {
            assert!(
                is_laugh_only(&tokens(text)),
                "text '{text}' was not read as laughter, expected it to be"
            );
        }
        for text in words {
            assert!(
                !is_laugh_only(&tokens(text)),
                "text '{text}' was read as laughter, expected it not to be"
            );
        }
    }

    #[test]
    fn finds_a_stream_mention_by_prefix() {
        for text in ["стрим", "стримчик", "го стримить"] {
            assert!(
                mentions_stream(&tokens(text)),
                "text '{text}' was not read as a stream mention, expected it to be"
            );
        }
    }

    #[test]
    fn does_not_read_a_stream_mention_in_unrelated_text() {
        for text in ["привет", "го в доту"] {
            assert!(
                !mentions_stream(&tokens(text)),
                "text '{text}' was read as a stream mention, expected it not to be"
            );
        }
    }

    #[test]
    fn finds_a_vinograd_mention_by_prefix() {
        for text in ["виноград", "лысина", "данила", "данек", "данёк"]
        {
            assert!(
                mentions_vinograd(&tokens(text)),
                "text '{text}' was not read as a vinograd mention, expected it to be"
            );
        }
    }

    #[test]
    fn does_not_read_a_vinograd_mention_in_unrelated_text() {
        for text in ["привет", "го в доту"] {
            assert!(
                !mentions_vinograd(&tokens(text)),
                "text '{text}' was read as a vinograd mention, expected it not to be"
            );
        }
    }

    #[test]
    fn every_word_list_entry_survives_normalisation_unchanged() {
        let lists: &[(&str, &[&str])] = &[
            ("MAT_ROOTS", MAT_ROOTS),
            ("MAT_EXCEPTIONS", MAT_EXCEPTIONS),
            ("PREFIXES", PREFIXES),
            ("PLAY_EXACT", PLAY_EXACT),
            ("PLAY_PREFIXES", PLAY_PREFIXES),
            ("POLITICS_EXACT", POLITICS_EXACT),
            ("POLITICS_PREFIXES", POLITICS_PREFIXES),
            ("APOLOGY_EXACT", APOLOGY_EXACT),
            ("APOLOGY_PHRASES", APOLOGY_PHRASES),
            ("LAUGH_EXACT", LAUGH_EXACT),
            ("STREAM_WORDS", STREAM_WORDS),
            ("VINOGRAD_WORDS", VINOGRAD_WORDS),
        ];
        for (list_name, entries) in lists {
            for entry in *entries {
                let normalized = normalize(entry);
                assert_eq!(
                    &normalized, entry,
                    "entry '{entry}' in {list_name} normalised to '{normalized}', expected it unchanged since a doubled letter makes the entry unreachable"
                );
            }
        }
    }
}
