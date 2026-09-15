//! Die Etikette als reiner Text.
//!
//! Die Vorschau ist HTML und lässt sich nicht brauchbar herauskopieren: beim
//! Markieren mit der Maus kommen Badges, Hinweise und Layout-Abstände mit, und
//! die fett gesetzten Allergene erscheinen als `<b>`-Schnipsel. Wer den Text
//! in ein Druck- oder Schreibprogramm übernehmen will, braucht ihn als Zeilen
//! mit sauberen Umbrüchen.
//!
//! Aufgenommen wird nur, was auf der physischen Etikette steht. Die Bio- und
//! Knospe-Hinweise unterhalb der weissen Karte bleiben draussen, sie sind
//! Hinweise *über* die Etikette, nicht Teil von ihr.

/// Baut den Etikettentext aus bereits aufbereiteten Abschnitten.
///
/// Leere Abschnitte fallen weg, damit keine Lücken aus zwei, drei
/// Leerzeilen entstehen, wenn etwa Preis oder Zertifizierung fehlen.
pub fn join_sections(sections: &[String]) -> String {
    sections
        .iter()
        .map(|s| s.trim_matches('\n').to_string())
        .filter(|s| !s.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Macht aus dem HTML der Zutatenliste reinen Text.
///
/// Die Liste entsteht in `core` mit `<b>` für Allergene und HTML-Entities für
/// Sonderzeichen (siehe `html_escape` dort). Beides muss hier rückgängig
/// gemacht werden, sonst klebt `&amp;` oder `<b>` im kopierten Text.
///
/// Fett gesetzte Allergene gehen dabei verloren, weil reiner Text keine
/// Auszeichnung kennt. Der Wortlaut bleibt vollständig, und genau der ist
/// beim Übertragen in ein Druckprogramm gefragt.
pub fn html_to_plain(html: &str) -> String {
    // Erst die Tags weg, dann die Entities: andernfalls würde ein im Text
    // stehendes «&lt;b&gt;» zu einem echten Tag und anschliessend entfernt.
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }

    // Reihenfolge: `&amp;` zuletzt, sonst würde aus «&amp;lt;» fälschlich «<».
    out.replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

/// Legt Text in die Zwischenablage. `true`, wenn es geklappt hat.
///
/// Bewusst über ein verstecktes `<textarea>` und `execCommand('copy')` statt
/// über die Clipboard-API: letztere verlangt einen sicheren Kontext und
/// scheitert in genau den Fällen still, in denen Nutzer sie brauchen (ältere
/// Safari-Versionen, eingebettete Ansichten). Der Umweg funktioniert überall.
///
/// Liegt in `services`, weil sowohl der Teilen-Dialog als auch der Knopf unter
/// der Etikette kopieren; vorher stand die Logik nur im Dialog.
pub fn copy_to_clipboard(text: &str) -> bool {
    use wasm_bindgen::JsCast;
    use web_sys::{js_sys, window, HtmlTextAreaElement};

    let Some(window) = window() else {
        return false;
    };
    let Some(document) = window.document() else {
        return false;
    };
    let Ok(element) = document.create_element("textarea") else {
        return false;
    };
    let Ok(textarea) = element.dyn_into::<HtmlTextAreaElement>() else {
        return false;
    };

    textarea.set_value(text);
    textarea
        .set_attribute("style", "position: fixed; left: -999999px; top: -999999px;")
        .ok();

    let Some(body) = document.body() else {
        return false;
    };
    let Ok(node) = textarea.clone().dyn_into::<web_sys::Node>() else {
        return false;
    };

    body.append_child(&node).ok();
    textarea.select();
    let copied = js_sys::eval("document.execCommand('copy')").is_ok();
    body.remove_child(&node).ok();

    copied
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bold_allergens_become_plain_words() {
        assert_eq!(
            html_to_plain("Mehl (<b>Weizen</b>), Wasser"),
            "Mehl (Weizen), Wasser"
        );
    }

    #[test]
    fn entities_turn_back_into_their_characters() {
        assert_eq!(
            html_to_plain("Salz &amp; Pfeffer, 5 &lt; 10, &quot;Bio&quot;, Hans&#x27; Hof"),
            "Salz & Pfeffer, 5 < 10, \"Bio\", Hans' Hof"
        );
    }

    #[test]
    fn an_escaped_tag_stays_visible_text() {
        // Ein Nutzer, der «<b>» in einen Zutatennamen tippt, hat es in der
        // Liste als «&lt;b&gt;» stehen. Im Klartext muss es sichtbar bleiben
        // und darf nicht als Auszeichnung verschwinden.
        assert_eq!(html_to_plain("Zutat &lt;b&gt;"), "Zutat <b>");
    }

    #[test]
    fn the_degree_marker_for_wild_collection_survives() {
        // Das ° trägt bei Wildsammlung eine rechtliche Bedeutung (DEC-11)
        // und muss im kopierten Text erhalten bleiben.
        assert_eq!(html_to_plain("Bärlauch°, Salz"), "Bärlauch°, Salz");
    }

    #[test]
    fn empty_sections_leave_no_gaps() {
        let text = join_sections(&[
            "Bergkäse".to_string(),
            String::new(),
            "   ".to_string(),
            "Zutaten: Rohmilch, Salz".to_string(),
        ]);

        assert_eq!(text, "Bergkäse\n\nZutaten: Rohmilch, Salz");
        assert!(!text.contains("\n\n\n"), "no run of blank lines: {text:?}");
    }

    #[test]
    fn sections_are_separated_by_exactly_one_blank_line() {
        let text = join_sections(&["A".to_string(), "B".to_string(), "C".to_string()]);
        assert_eq!(text, "A\n\nB\n\nC");
    }

    #[test]
    fn lines_within_a_section_keep_their_single_breaks() {
        // Die Herstelleradresse ist mehrzeilig und muss mehrzeilig bleiben,
        // ohne dass zwischen Name und Strasse eine Leerzeile entsteht.
        let text = join_sections(&[
            "Bergkäse".to_string(),
            "Hof Muster\nDorfstrasse 1\n3000 Bern".to_string(),
        ]);

        assert_eq!(text, "Bergkäse\n\nHof Muster\nDorfstrasse 1\n3000 Bern");
    }

    #[test]
    fn nothing_at_all_yields_an_empty_text() {
        assert_eq!(join_sections(&[]), "");
        assert_eq!(join_sections(&[String::new(), "  ".to_string()]), "");
    }
}
