//! FAQ content, maintained in the locale files as `faq.questions` / `faq.answers`.
//!
//! rust-i18n flattens YAML arrays to an empty string, so `t!("faq.questions")`
//! cannot read them. The locale files are therefore embedded and parsed
//! directly here. Parsing happens once per locale switch, on a handful of
//! entries, so the cost is irrelevant next to keeping the content in the same
//! files translators already edit.

use rust_i18n::t;

const DE_CH: &str = include_str!("../locales/de-CH.yml");
const FR_CH: &str = include_str!("../locales/fr-CH.yml");
const IT_CH: &str = include_str!("../locales/it-CH.yml");

/// One question with its answer. Both are Markdown-free plain text.
#[derive(Debug, Clone, PartialEq)]
pub struct FaqEntry {
    pub question: String,
    pub answer: String,
}

fn locale_source(locale: &str) -> &'static str {
    match locale {
        "fr-CH" => FR_CH,
        "it-CH" => IT_CH,
        _ => DE_CH,
    }
}

/// Read `faq.questions` and `faq.answers` from one locale file.
fn string_list(source: &str, field: &str) -> Vec<String> {
    let Ok(doc) = serde_yaml::from_str::<serde_yaml::Value>(source) else {
        return Vec::new();
    };
    doc.get("faq")
        .and_then(|faq| faq.get(field))
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .map(|v| v.as_str().unwrap_or_default().trim().to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// The FAQ entries for one locale, pairing each question with its answer.
///
/// Questions and answers are two parallel arrays, so a length mismatch (a
/// half-finished translation) would silently shift every answer by one. Pairing
/// via `zip` truncates to the shorter list instead, which drops entries but
/// never shows a wrong answer.
pub fn entries_for_locale(locale: &str) -> Vec<FaqEntry> {
    let source = locale_source(locale);
    let questions = string_list(source, "questions");
    let answers = string_list(source, "answers");

    questions
        .into_iter()
        .zip(answers)
        .filter(|(q, a)| !q.is_empty() && !a.is_empty())
        .map(|(question, answer)| FaqEntry { question, answer })
        .collect()
}

/// FAQ entries in the active locale. Falls back to de-CH when a locale has no
/// FAQ yet, so the page is never empty just because a translation is pending.
pub fn entries() -> Vec<FaqEntry> {
    let locale = rust_i18n::locale().to_string();
    let entries = entries_for_locale(&locale);
    if entries.is_empty() && locale != "de-CH" {
        return entries_for_locale("de-CH");
    }
    entries
}

/// Page title, from the regular translation keys.
pub fn title() -> String {
    t!("faq.title").to_string()
}

/// Intro paragraph shown above the questions.
pub fn intro() -> String {
    t!("faq.intro").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Every shipped locale must render a usable FAQ: the arrays have to exist,
    // line up, and carry real text.
    #[test]
    fn every_locale_has_paired_faq_entries() {
        for locale in ["de-CH", "fr-CH", "it-CH"] {
            let source = locale_source(locale);
            let questions = string_list(source, "questions");
            let answers = string_list(source, "answers");
            assert!(
                !questions.is_empty(),
                "{locale} has no faq.questions entries"
            );
            assert_eq!(
                questions.len(),
                answers.len(),
                "{locale} has {} questions but {} answers — they must stay paired",
                questions.len(),
                answers.len()
            );
            for e in entries_for_locale(locale) {
                assert!(!e.question.is_empty() && !e.answer.is_empty());
            }
        }
    }

    // All locales must offer the same number of entries, otherwise switching
    // language would silently hide questions.
    #[test]
    fn locales_offer_the_same_number_of_entries() {
        let de = entries_for_locale("de-CH").len();
        for locale in ["fr-CH", "it-CH"] {
            assert_eq!(
                entries_for_locale(locale).len(),
                de,
                "{locale} has a different FAQ length than de-CH"
            );
        }
    }

    // A mismatched pair must drop the extra entry rather than pair a question
    // with the wrong answer.
    #[test]
    fn mismatched_lengths_truncate_instead_of_misaligning() {
        let yaml = "faq:\n  questions:\n    - Q1\n    - Q2\n  answers:\n    - A1\n";
        let questions = string_list(yaml, "questions");
        let answers = string_list(yaml, "answers");
        let paired: Vec<_> = questions.into_iter().zip(answers).collect();
        assert_eq!(paired, vec![("Q1".to_string(), "A1".to_string())]);
    }

    // An unknown locale falls back to the German file instead of panicking.
    #[test]
    fn unknown_locale_falls_back_to_de_ch() {
        assert_eq!(
            entries_for_locale("en-US"),
            entries_for_locale("de-CH"),
            "unknown locales must reuse the de-CH content"
        );
    }
}
