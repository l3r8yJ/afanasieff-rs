use crate::ops::achievements::words::{MAT_EXCEPTIONS, MAT_ROOTS, PREFIXES};

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
    use super::{has_mat, tokens};

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
}
