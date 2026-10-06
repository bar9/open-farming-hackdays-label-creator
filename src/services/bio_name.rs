//! DEC-23/24: «Bio» in der Sachbezeichnung.
//!
//! Früher hängte die Etikette automatisch « Bio» hinten an die Sachbezeichnung,
//! sobald die Rezeptur die Bio- bzw. Knospe-Anforderungen erfüllte. Das wirkte
//! komisch («Frühstücksflocken mit Beeren Bio») und verdoppelte sich, wenn die
//! Nutzerin «Bio» schon selbst geschrieben hatte. «Bio» in der Sachbezeichnung
//! ist zudem erlaubt, aber nicht Pflicht.
//!
//! Neu bietet die Eingabemaske einen Knopf an, der «Bio» in das Feld selbst
//! schreibt: sichtbar und änderbar. Diese reinen Funktionen entscheiden, ob
//! der Text schon «Bio» enthält, und fügen es sprachgerecht ein.

/// Enthält der Text das Wort «bio» (Gross/klein egal, als eigenes Wort oder
/// als Wortanfang mit Bindestrich wie «Bio-Müesli»)? «Biologisch» zählt auch,
/// «Biomasse»/«Antibiotika» nicht.
pub fn contains_bio_word(text: &str) -> bool {
    let lower = text.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    let n = chars.len();
    let mut i = 0;
    while i + 3 <= n {
        if chars[i] == 'b' && chars[i + 1] == 'i' && chars[i + 2] == 'o' {
            let before_ok = i == 0 || !chars[i - 1].is_alphanumeric();
            if before_ok {
                let rest: String = chars[i + 3..].iter().collect();
                let after = chars.get(i + 3).copied();
                let word_end = after.is_none_or(|c| !c.is_alphanumeric());
                // «biologisch», «biologique», «biologico», «biologica» …
                let biolog = rest.starts_with("log");
                if word_end || biolog {
                    return true;
                }
            }
        }
        i += 1;
    }
    false
}

/// Schreibt «Bio» sprachgerecht in die Sachbezeichnung: im Deutschen vorne
/// («Bio Frühstücksflocken»), im Französischen und Italienischen hinten
/// («Flocons … bio», «Fiocchi … bio»). Enthält der Text schon «Bio», bleibt
/// er unverändert.
pub fn add_bio(text: &str, locale: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() || contains_bio_word(trimmed) {
        return text.to_string();
    }
    if locale.starts_with("fr") || locale.starts_with("it") {
        format!("{trimmed} bio")
    } else {
        format!("Bio {trimmed}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_bio_as_word_prefix_or_adjective() {
        assert!(contains_bio_word("Bio Frühstücksflocken"));
        assert!(contains_bio_word("Frühstücksflocken mit Beeren Bio"));
        assert!(contains_bio_word("bio-Müesli"));
        assert!(contains_bio_word("Müesli (BIO)"));
        assert!(contains_bio_word("Confiture biologique"));
        assert!(contains_bio_word("Marmellata biologica"));
    }

    #[test]
    fn ignores_bio_inside_other_words() {
        assert!(!contains_bio_word("Biomasse-Brot"));
        assert!(!contains_bio_word("Antibiotika-frei"));
        assert!(!contains_bio_word("Frühstücksflocken"));
        assert!(!contains_bio_word("Symbiose"));
    }

    #[test]
    fn add_bio_is_language_aware_and_idempotent() {
        assert_eq!(add_bio("Frühstücksflocken", "de-CH"), "Bio Frühstücksflocken");
        assert_eq!(add_bio("Flocons d'avoine", "fr-CH"), "Flocons d'avoine bio");
        assert_eq!(add_bio("Fiocchi d'avena", "it-CH"), "Fiocchi d'avena bio");
        // Already there: unchanged, no «Bio Bio …».
        assert_eq!(add_bio("Bio Frühstücksflocken", "de-CH"), "Bio Frühstücksflocken");
        // Nothing to prefix.
        assert_eq!(add_bio("  ", "de-CH"), "  ");
    }
}
