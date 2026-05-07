// Asserts that fr-CH and it-CH locale files contain the same keys as de-CH.
// Prevents silent fallback-to-German when a key is added to de-CH but not the
// other locales. Keys under `countries.*` are exempt: country names use de-CH
// as the canonical source. Other currently-missing keys are listed in
// `KNOWN_MISSING_*` arrays below; remove an entry once it's translated, and
// the test will start enforcing parity for it.

use std::collections::HashSet;
use std::fs;

const COUNTRIES_PREFIX: &str = "countries.";

// Keys present in de-CH but deliberately not yet translated. Translator
// follow-up: shrink this list as keys are added to fr-CH.yml.
const KNOWN_MISSING_FR: &[&str] = &[
    "help.erlaubte_ausnahme_bio.Referenz",
    "help.haltbarkeit.Grundsatz",
    "help.herkunft_liv_art_16",
    "help.menge.Beispiel",
    "help.nettogewicht.Achtung",
    "help.plz",
];

const KNOWN_MISSING_IT: &[&str] = &[
    "help.bio_ch",
    "help.bio_knospe",
    "help.erlaubte_ausnahme_bio.Referenz",
    "help.haltbarkeit.Grundsatz",
    "help.herkunft_liv_art_16",
    "help.menge.Beispiel",
    "help.nettogewicht.Achtung",
    "help.plz",
    "help.verarbeitungsschritte",
    "label.saved_ingredients",
];

/// Minimal YAML key extractor for the project's locale files.
/// Tracks indentation depth to build dotted key paths. Only emits a path
/// when the line has a non-empty value after the colon (i.e., a leaf).
fn extract_keys(path: &str) -> HashSet<String> {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("could not read {path}: {e}"));
    let mut keys = HashSet::new();
    let mut stack: Vec<(usize, String)> = Vec::new();

    for raw in content.lines() {
        let line = raw.trim_end();
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('-') {
            continue;
        }
        let indent = line.len() - trimmed.len();
        let Some(colon) = trimmed.find(':') else { continue };
        let key = trimmed[..colon].trim();
        if key.is_empty() || !key.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            continue;
        }
        while stack.last().is_some_and(|(i, _)| *i >= indent) {
            stack.pop();
        }
        let full: String = stack
            .iter()
            .map(|(_, k)| k.as_str())
            .chain(std::iter::once(key))
            .collect::<Vec<_>>()
            .join(".");
        let value = trimmed[colon + 1..].trim();
        if !value.is_empty() {
            keys.insert(full);
        }
        stack.push((indent, key.to_string()));
    }
    keys
}

fn missing_keys(de: &HashSet<String>, other: &HashSet<String>, known: &[&str]) -> Vec<String> {
    let known: HashSet<&str> = known.iter().copied().collect();
    let mut out: Vec<String> = de
        .difference(other)
        .filter(|k| !k.starts_with(COUNTRIES_PREFIX))
        .filter(|k| !known.contains(k.as_str()))
        .cloned()
        .collect();
    out.sort();
    out
}

#[test]
fn fr_ch_matches_de_ch() {
    let de = extract_keys("locales/de-CH.yml");
    let fr = extract_keys("locales/fr-CH.yml");
    let missing = missing_keys(&de, &fr, KNOWN_MISSING_FR);
    assert!(
        missing.is_empty(),
        "fr-CH.yml is missing keys present in de-CH.yml:\n  {}\n\nEither add translations to fr-CH.yml or, if intentionally untranslated, list them in KNOWN_MISSING_FR.",
        missing.join("\n  ")
    );
}

#[test]
fn it_ch_matches_de_ch() {
    let de = extract_keys("locales/de-CH.yml");
    let it = extract_keys("locales/it-CH.yml");
    let missing = missing_keys(&de, &it, KNOWN_MISSING_IT);
    assert!(
        missing.is_empty(),
        "it-CH.yml is missing keys present in de-CH.yml:\n  {}\n\nEither add translations to it-CH.yml or, if intentionally untranslated, list them in KNOWN_MISSING_IT.",
        missing.join("\n  ")
    );
}

#[test]
fn known_missing_lists_have_no_stale_entries() {
    // If a key was translated since the list was written, drop it from the
    // exemption list so future regressions are caught.
    let de = extract_keys("locales/de-CH.yml");
    let fr = extract_keys("locales/fr-CH.yml");
    let it = extract_keys("locales/it-CH.yml");

    let stale_fr: Vec<&str> = KNOWN_MISSING_FR
        .iter()
        .copied()
        .filter(|k| !de.contains(*k) || fr.contains(*k))
        .collect();
    let stale_it: Vec<&str> = KNOWN_MISSING_IT
        .iter()
        .copied()
        .filter(|k| !de.contains(*k) || it.contains(*k))
        .collect();

    assert!(
        stale_fr.is_empty(),
        "KNOWN_MISSING_FR contains stale entries (already translated or no longer in de-CH): {stale_fr:?}"
    );
    assert!(
        stale_it.is_empty(),
        "KNOWN_MISSING_IT contains stale entries (already translated or no longer in de-CH): {stale_it:?}"
    );
}
