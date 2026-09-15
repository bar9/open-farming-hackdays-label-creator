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
/// Die Liste entsteht in `core` mit `<b>` für Allergene, `<br>` vor der
/// Fussnotenlegende und HTML-Entities für Sonderzeichen (siehe `html_escape`
/// dort). Alles drei muss hier übersetzt werden, sonst klebt `&amp;` im Text
/// oder die Legende «* aus biologischer Landwirtschaft» hängt ohne Umbruch an
/// der letzten Zutat, wodurch deren Fussnotenzeichen zu «**» verschmilzt.
///
/// Fett gesetzte Allergene gehen verloren, weil reiner Text keine Auszeichnung
/// kennt. Der Wortlaut bleibt vollständig, und genau der ist beim Übertragen
/// in ein Druckprogramm gefragt.
pub fn html_to_plain(html: &str) -> String {
    // Erst die Tags weg, dann die Entities: andernfalls würde ein im Text
    // stehendes «&lt;b&gt;» zu einem echten Tag und anschliessend entfernt.
    let mut out = String::with_capacity(html.len());
    let mut tag = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => {
                in_tag = true;
                tag.clear();
            }
            '>' if in_tag => {
                in_tag = false;
                // `<br>` trennt in der Vorschau die Fussnotenlegende von der
                // Zutatenliste. Im Klartext muss daraus ein echter Umbruch
                // werden, sonst geht die Trennung verloren.
                if tag.trim_end_matches('/').trim().eq_ignore_ascii_case("br") {
                    out.push('\n');
                }
            }
            c if in_tag => tag.push(c),
            c => out.push(c),
        }
    }

    // Reihenfolge: `&amp;` zuletzt, sonst würde aus «&amp;lt;» fälschlich «<».
    let text = out
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&");

    // Aus `<br><br>` würden zwei Umbrüche und damit eine Leerzeile mitten im
    // Abschnitt. Die Vorschau setzt die Legende direkt unter die Liste, also
    // bleibt es bei einem Umbruch.
    let mut collapsed = String::with_capacity(text.len());
    let mut last_was_newline = false;
    for ch in text.chars() {
        if ch == '\n' {
            if last_was_newline {
                continue;
            }
            last_was_newline = true;
        } else {
            last_was_newline = false;
        }
        collapsed.push(ch);
    }

    collapsed.trim().to_string()
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
    fn the_footnote_legend_gets_its_own_line() {
        // So sieht die Liste bei Bio-Zutaten aus: die Legende steht in der
        // Vorschau unter der Liste, getrennt durch <br><br>. Ohne Umbruch
        // stünde dort «Bärlauch* * aus biologischer Landwirtschaft», und das
        // Fussnotenzeichen der letzten Zutat verschmölze optisch zu «**».
        let html = "<b>Weizenmehl</b>*, Bärlauch*<br><br>* aus biologischer Landwirtschaft";
        assert_eq!(
            html_to_plain(html),
            "Weizenmehl*, Bärlauch*\n* aus biologischer Landwirtschaft"
        );
    }

    #[test]
    fn a_single_br_also_becomes_a_line_break() {
        assert_eq!(
            html_to_plain("Zeile eins<br>Zeile zwei"),
            "Zeile eins\nZeile zwei"
        );
    }

    #[test]
    fn the_wild_collection_legend_gets_its_own_line_too() {
        // `core` hängt die °-Legende mit einem einfachen `<br>` an (DEC-11).
        // Ohne Umbruch stünde «Bärlauch°° aus zertifizierter Wildsammlung»,
        // also ein Zeichen, das es auf der Etikette nicht gibt.
        let html = "Bärlauch°, Salz<br>° aus zertifizierter Wildsammlung";
        assert_eq!(
            html_to_plain(html),
            "Bärlauch°, Salz\n° aus zertifizierter Wildsammlung"
        );
    }

    #[test]
    fn both_legends_below_each_other_keep_single_breaks() {
        // Bio und Wildsammlung können zusammen auftreten; dann stehen zwei
        // Legendenzeilen untereinander, ohne Leerzeile dazwischen.
        let html = "Bärlauch°*<br><br>* aus biologischer Landwirtschaft<br>° aus zertifizierter Wildsammlung";
        assert_eq!(
            html_to_plain(html),
            "Bärlauch°*\n* aus biologischer Landwirtschaft\n° aus zertifizierter Wildsammlung"
        );
    }

    #[test]
    fn self_closing_and_uppercase_br_count_too() {
        assert_eq!(html_to_plain("eins<BR/>zwei<br />drei"), "eins\nzwei\ndrei");
    }

    #[test]
    fn the_legend_line_carries_no_blank_line_before_it() {
        // Eine Leerzeile mitten im Zutaten-Abschnitt sähe aus wie ein neuer
        // Abschnitt und würde die Gliederung der Etikette verfälschen.
        let text = html_to_plain("Salz*<br><br>* aus biologischer Landwirtschaft");
        assert!(
            !text.contains("\n\n"),
            "the legend belongs directly under the list: {text:?}"
        );
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
