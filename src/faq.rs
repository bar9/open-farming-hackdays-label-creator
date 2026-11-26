//! FAQ content, maintained in the locale files under `faq`.
//!
//! The keys follow the POEditor naming agreed with Bio Suisse (context `faq`):
//!
//! * `qNN` – question number NN
//! * `aNN` – the answer to `qNN`
//! * `hNN` – an optional section heading, shown directly before `qNN`
//!
//! Numbers may have gaps so new entries can be slotted in later. Entries are
//! sorted by number, so the order in the file does not matter. A question
//! without an answer (or the other way round) is dropped rather than shown
//! half-finished.
//!
//! rust-i18n cannot enumerate keys, so the locale files are embedded and
//! parsed directly here. Parsing happens once per render on a few dozen
//! entries, which is irrelevant next to keeping the content in the same files
//! translators already edit.

use rust_i18n::t;
use std::collections::BTreeMap;

const DE_CH: &str = include_str!("../locales/de-CH.yml");
const FR_CH: &str = include_str!("../locales/fr-CH.yml");
const IT_CH: &str = include_str!("../locales/it-CH.yml");

/// One block of the FAQ page.
#[derive(Debug, Clone, PartialEq)]
pub enum FaqItem {
    /// Section heading grouping the following questions.
    Heading(String),
    /// A question with its answer. Both are Markdown-free plain text.
    Entry(FaqEntry),
}

/// One question with its answer.
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

#[derive(Default)]
struct Slot {
    heading: Option<String>,
    question: Option<String>,
    answer: Option<String>,
}

/// Split a key like `q04` into (`'q'`, 4).
fn parse_key(key: &str) -> Option<(char, u32)> {
    let mut chars = key.chars();
    let kind = chars.next()?;
    if !matches!(kind, 'q' | 'a' | 'h') {
        return None;
    }
    let digits = chars.as_str();
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some((kind, digits.parse().ok()?))
}

/// Parse the `faq` section of one locale file into ordered page items.
fn items_from_source(source: &str) -> Vec<FaqItem> {
    let Ok(doc) = serde_yaml::from_str::<serde_yaml::Value>(source) else {
        return Vec::new();
    };
    let Some(faq) = doc.get("faq").and_then(|v| v.as_mapping()) else {
        return Vec::new();
    };

    let mut slots: BTreeMap<u32, Slot> = BTreeMap::new();
    for (key, value) in faq {
        let (Some(key), Some(text)) = (key.as_str(), value.as_str()) else {
            continue;
        };
        let Some((kind, number)) = parse_key(key) else {
            continue;
        };
        let text = text.trim();
        if text.is_empty() {
            continue;
        }
        let slot = slots.entry(number).or_default();
        let text = Some(text.to_string());
        match kind {
            'q' => slot.question = text,
            'a' => slot.answer = text,
            _ => slot.heading = text,
        }
    }

    let mut items = Vec::new();
    for slot in slots.into_values() {
        if let Some(heading) = slot.heading {
            items.push(FaqItem::Heading(heading));
        }
        if let (Some(question), Some(answer)) = (slot.question, slot.answer) {
            items.push(FaqItem::Entry(FaqEntry { question, answer }));
        }
    }
    // A heading with no question after it would dangle at the end or sit
    // directly on top of the next heading.
    let mut cleaned: Vec<FaqItem> = Vec::with_capacity(items.len());
    for item in items {
        if matches!(item, FaqItem::Heading(_))
            && matches!(cleaned.last(), Some(FaqItem::Heading(_)))
        {
            cleaned.pop();
        }
        cleaned.push(item);
    }
    if matches!(cleaned.last(), Some(FaqItem::Heading(_))) {
        cleaned.pop();
    }
    cleaned
}

/// Page items (headings and entries) for one locale.
pub fn items_for_locale(locale: &str) -> Vec<FaqItem> {
    items_from_source(locale_source(locale))
}

/// Only the question/answer pairs for one locale, in page order.
pub fn entries_for_locale(locale: &str) -> Vec<FaqEntry> {
    items_for_locale(locale)
        .into_iter()
        .filter_map(|item| match item {
            FaqItem::Entry(e) => Some(e),
            FaqItem::Heading(_) => None,
        })
        .collect()
}

/// FAQ items in the active locale. Falls back to de-CH when a locale has no
/// FAQ yet, so the page is never empty just because a translation is pending.
pub fn items() -> Vec<FaqItem> {
    let locale = rust_i18n::locale().to_string();
    let items = items_for_locale(&locale);
    if items.is_empty() && locale != "de-CH" {
        return items_for_locale("de-CH");
    }
    items
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

    const LOCALES: [&str; 3] = ["de-CH", "fr-CH", "it-CH"];

    fn faq_keys(locale: &str) -> Vec<String> {
        let doc: serde_yaml::Value = serde_yaml::from_str(locale_source(locale)).unwrap();
        doc.get("faq")
            .and_then(|v| v.as_mapping())
            .unwrap()
            .keys()
            .filter_map(|k| k.as_str())
            .filter(|k| parse_key(k).is_some())
            .map(str::to_string)
            .collect()
    }

    // Every shipped locale must render a usable FAQ, and every question must
    // have its answer (no orphaned qNN/aNN that would silently disappear).
    #[test]
    fn every_locale_has_paired_faq_entries() {
        for locale in LOCALES {
            let keys = faq_keys(locale);
            assert!(!keys.is_empty(), "{locale} has no FAQ entries");
            for key in &keys {
                let (kind, n) = parse_key(key).unwrap();
                let partner = match kind {
                    'q' => format!("a{n:02}"),
                    'a' => format!("q{n:02}"),
                    _ => continue,
                };
                assert!(
                    keys.contains(&partner),
                    "{locale}: {key} has no matching {partner}"
                );
            }
        }
    }

    // All locales must offer the same keys, otherwise switching language
    // would silently hide questions or headings.
    #[test]
    fn locales_offer_the_same_faq_keys() {
        let mut de = faq_keys("de-CH");
        de.sort();
        for locale in ["fr-CH", "it-CH"] {
            let mut other = faq_keys(locale);
            other.sort();
            assert_eq!(other, de, "{locale} FAQ keys differ from de-CH");
        }
    }

    // Questions must end in a question mark, so a heading or answer that was
    // pasted into the wrong POEditor term is caught.
    #[test]
    fn every_question_is_a_question() {
        for locale in LOCALES {
            for e in entries_for_locale(locale) {
                assert!(
                    e.question.ends_with('?'),
                    "{locale}: question does not end with '?': {:?}",
                    e.question
                );
                assert!(
                    !e.answer.contains("[ID") && !e.answer.contains("[FB"),
                    "{locale}: translator comment marker left in answer: {:?}",
                    e.answer
                );
            }
        }
    }

    #[test]
    fn entries_are_sorted_by_number_with_headings_in_front() {
        let yaml = "faq:\n  title: T\n  a10: A10\n  q10: Q10?\n  h10: H\n  q02: Q2?\n  a02: A2\n  q05: lonely?\n";
        assert_eq!(
            items_from_source(yaml),
            vec![
                FaqItem::Entry(FaqEntry {
                    question: "Q2?".into(),
                    answer: "A2".into()
                }),
                FaqItem::Heading("H".into()),
                FaqItem::Entry(FaqEntry {
                    question: "Q10?".into(),
                    answer: "A10".into()
                }),
            ]
        );
    }

    #[test]
    fn dangling_headings_are_dropped() {
        let yaml = "faq:\n  h01: Empty\n  h02: Real\n  q02: Q?\n  a02: A\n  h09: Trailing\n";
        assert_eq!(
            items_from_source(yaml),
            vec![
                FaqItem::Heading("Real".into()),
                FaqItem::Entry(FaqEntry {
                    question: "Q?".into(),
                    answer: "A".into()
                }),
            ]
        );
    }

    // An unknown locale falls back to the German file instead of panicking.
    #[test]
    fn unknown_locale_falls_back_to_de_ch() {
        assert_eq!(items_for_locale("en-US"), items_for_locale("de-CH"));
    }

    // R-17 (Testing Guetzlimischung 29.10.2026): the mixture question must be
    // answered by the mixture answer in every language (not shifted by one),
    // recommend per-variety composites and mention the allergen reason. The
    // short-link FAQ answer (R-18) must keep its exception sentence.
    #[test]
    fn mixture_entry_is_paired_and_recommends_per_variety_composites() {
        let cases = [
            (
                "de-CH",
                "Mischung",
                "zusammengesetzte Zutat",
                "Allergen",
                "Kurz-Link",
            ),
            (
                "fr-CH",
                "mélange",
                "ingrédient composé",
                "allerg",
                "lien court",
            ),
            (
                "it-CH",
                "miscela",
                "ingrediente composto",
                "allerg",
                "link breve",
            ),
        ];
        for (locale, q_word, a_word, allergen_word, short_word) in cases {
            let entries = entries_for_locale(locale);
            let mix = entries
                .iter()
                .find(|e| e.question.contains(q_word))
                .unwrap_or_else(|| panic!("{locale}: no mixture question"));
            assert!(mix.answer.contains(a_word), "{locale}: {:?}", mix.answer);
            assert!(
                mix.answer.contains(allergen_word),
                "{locale}: {:?}",
                mix.answer
            );
            let server = &entries[1];
            assert!(
                server.answer.contains(short_word),
                "{locale}: server answer must name the short-link exception: {:?}",
                server.answer
            );
        }
    }
}
