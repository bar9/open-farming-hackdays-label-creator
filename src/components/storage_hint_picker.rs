//! Aufbewahrungshinweise zum Anwählen (R-14, Livetesting 09.11.2026).
//!
//! Die Testerinnen hatten «viele gute Ideen» für Aufbewahrungshinweise, mussten
//! sie aber aus der Tabelle im Hilfetext abschreiben. Die Liste selbst kommt aus
//! POEditor (Import 28.10.2026, `help.aufbewahrungshinweis`); hier wird sie zu
//! Knöpfen unter dem Feld. Ein Klick hängt den Text als neue Zeile an, das Feld
//! bleibt frei editierbar («eigener Text» heisst einfach: tippen).

use dioxus::prelude::*;
use rust_i18n::t;

/// Schlüssel der Optionen unter `storage_options.*`, in der Reihenfolge der
/// POEditor-Tabelle. `*_hint` ist die Erklärung am Knopf (z. B. «Eiprodukte»),
/// sie gehört nicht auf die Etikette.
pub const STORAGE_OPTION_KEYS: [&str; 8] = [
    "freezer",
    "max_4",
    "max_5",
    "cool_dry",
    "fridge_after_opening",
    "room_temperature",
    "dark",
    "chilled_eggs",
];

/// Hängt `option` an den bestehenden Hinweis an.
///
/// Neue Zeile statt Komma, weil der Hinweis mehrzeilig auf die Etikette geht.
/// Steht der Text schon drin, passiert nichts: ein Doppelklick soll keine
/// doppelte Zeile erzeugen.
pub fn append_storage_hint(current: &str, option: &str) -> String {
    let trimmed = current.trim_end();
    if trimmed.lines().any(|l| l.trim() == option) {
        return current.to_string();
    }
    if trimmed.is_empty() {
        option.to_string()
    } else {
        format!("{trimmed}\n{option}")
    }
}

#[component]
pub fn StorageHintPicker(bound_value: Signal<String>) -> Element {
    let mut bound_value = bound_value;
    rsx! {
        div {
            class: "flex flex-wrap gap-1 mt-2",
            "data-testid": "storage-hint-picker",
            for key in STORAGE_OPTION_KEYS {
                {
                    let text = t!(format!("storage_options.{key}")).to_string();
                    let hint_key = format!("storage_options.{key}_hint");
                    let hint = t!(&hint_key).to_string();
                    // rust-i18n liefert den Schlüssel zurück, wenn es keinen
                    // Text gibt: dann gibt es auch keine Erklärung.
                    let has_hint = !hint.is_empty() && !hint.ends_with(&hint_key);
                    let already = bound_value.read().lines().any(|l| l.trim() == text);
                    rsx! {
                        button {
                            r#type: "button",
                            class: if already { "btn btn-xs btn-primary" } else { "btn btn-xs btn-outline btn-primary" },
                            title: if has_hint { hint.clone() } else { String::new() },
                            onclick: {
                                let text = text.clone();
                                move |_| {
                                    let next = append_storage_hint(&bound_value.peek(), &text);
                                    bound_value.set(next);
                                }
                            },
                            "+ {text}"
                            if has_hint {
                                span { class: "opacity-70 font-normal", " ({hint})" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_on_a_new_line() {
        assert_eq!(
            append_storage_hint("", "kühl und trocken lagern"),
            "kühl und trocken lagern"
        );
        assert_eq!(
            append_storage_hint("kühl und trocken lagern", "lichtgeschützt lagern"),
            "kühl und trocken lagern\nlichtgeschützt lagern"
        );
    }

    #[test]
    fn does_not_duplicate_an_existing_line() {
        let current = "kühl und trocken lagern\nlichtgeschützt lagern";
        assert_eq!(
            append_storage_hint(current, "lichtgeschützt lagern"),
            current
        );
    }

    #[test]
    fn every_option_has_text_in_all_locales() {
        for locale in ["de-CH", "fr-CH", "it-CH"] {
            for key in STORAGE_OPTION_KEYS {
                let full = format!("storage_options.{key}");
                let text = t!(&full, locale = locale).to_string();
                assert!(
                    !text.is_empty() && text != full && !text.ends_with(&full),
                    "{locale}: missing {full}"
                );
            }
        }
    }
}
