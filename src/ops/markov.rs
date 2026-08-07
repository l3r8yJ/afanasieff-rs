use std::collections::HashMap;

use rand::Rng;
use rand::seq::IndexedRandom;

const MAX_WORDS: usize = 25;

const MIN_WORDS: usize = 3;

const ATTEMPTS: usize = 20;

/// Builds a phrase nobody said out of the words of the given quotes.
///
/// Returns nothing when the corpus is empty or every attempt reproduced an
/// existing quote word for word.
#[must_use]
pub fn generate(corpus: &[String], rng: &mut impl Rng) -> Option<String> {
    let starts = corpus
        .iter()
        .filter_map(|quote| quote.split_whitespace().next())
        .map(str::to_string)
        .collect::<Vec<String>>();
    if starts.is_empty() {
        return None;
    }
    let chain = chain_of(corpus);
    (0..ATTEMPTS)
        .filter_map(|_| walk(&starts, &chain, rng))
        .find(|phrase| phrase.split_whitespace().count() >= MIN_WORDS && !corpus.contains(phrase))
}

fn chain_of(corpus: &[String]) -> HashMap<String, Vec<String>> {
    let mut chain: HashMap<String, Vec<String>> = HashMap::new();
    for quote in corpus {
        let words = quote.split_whitespace().collect::<Vec<&str>>();
        for pair in words.windows(2) {
            chain
                .entry(pair[0].to_string())
                .or_default()
                .push(pair[1].to_string());
        }
    }
    chain
}

fn walk(
    starts: &[String],
    chain: &HashMap<String, Vec<String>>,
    rng: &mut impl Rng,
) -> Option<String> {
    let mut word = starts.choose(rng)?.clone();
    let mut phrase = vec![word.clone()];
    while phrase.len() < MAX_WORDS {
        let Some(next) = chain.get(&word).and_then(|options| options.choose(rng)) else {
            break;
        };
        word = next.clone();
        phrase.push(word.clone());
    }
    Some(phrase.join(" "))
}

#[cfg(test)]
mod tests {
    use asserting::prelude::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::generate;

    fn corpus() -> Vec<String> {
        [
            "я петух в законе",
            "я белогвардеец и в законе",
            "терпим и молчим",
            "мы на сво пойдем родину защищать",
        ]
        .iter()
        .map(|line| (*line).to_string())
        .collect()
    }

    fn seeded(seed: u64) -> StdRng {
        StdRng::seed_from_u64(seed)
    }

    #[test]
    fn builds_a_phrase_out_of_words_the_corpus_uses() {
        let corpus = corpus();
        let known = corpus
            .iter()
            .flat_map(|line| line.split_whitespace())
            .map(str::to_string)
            .collect::<Vec<String>>();
        let phrase = generate(&corpus, &mut seeded(7)).expect("a phrase is generated");
        for word in phrase.split_whitespace() {
            assert_that!(known.clone())
                .named("known words")
                .contains(word.to_string());
        }
    }

    #[test]
    fn never_repeats_a_quote_word_for_word() {
        let corpus = corpus();
        for seed in 0..50 {
            let phrase = generate(&corpus, &mut seeded(seed)).expect("a phrase is generated");
            assert_that!(corpus.clone())
                .named("corpus")
                .does_not_contain(phrase);
        }
    }

    #[test]
    fn generates_nothing_from_an_empty_corpus() {
        let phrase = generate(&[], &mut seeded(1));
        assert_that!(phrase)
            .named("phrase from an empty corpus")
            .is_none();
    }

    #[test]
    fn gives_up_when_the_corpus_holds_a_single_quote() {
        let only = vec!["одна единственная цитата".to_string()];
        let phrase = generate(&only, &mut seeded(1));
        assert_that!(phrase)
            .named("phrase from a one quote corpus")
            .is_none();
    }

    #[test]
    fn keeps_a_phrase_within_the_length_cap() {
        let corpus = corpus();
        for seed in 0..50 {
            let phrase = generate(&corpus, &mut seeded(seed)).expect("a phrase is generated");
            assert_that!(phrase.split_whitespace().count())
                .named("words in the phrase")
                .is_at_most(25);
        }
    }
}
