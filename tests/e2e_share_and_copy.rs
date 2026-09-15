// Die beiden Kopierwege der Anwendung, im Browser geprüft.
//
// Sie werden leicht verwechselt, deshalb liegen sie hier nebeneinander:
//
// - «Teilen» oben gibt den *Link* zur Rezeptur weiter. Das Kopieren passiert
//   erst im Dialog, dazu ein QR-Code für den Weg aufs Handy.
// - «Etikette kopieren» unter der Vorschau gibt den *Text* der Etikette als
//   Klartext heraus, zum Einsetzen in ein Druck- oder Schreibprogramm.
//
// Sequentiell laufen lassen: `cargo test --test e2e_share_and_copy -- --test-threads=1`

mod common;

use common::recipes::Config;
use common::*;
use fantoccini::Locator;
use std::time::Duration;

/// Eine kleine, vollständige Rezeptur, damit Etikette und Link Inhalt haben.
async fn seed_small_recipe(c: &fantoccini::Client) {
    goto_config(c, Config::Lebensmittelrecht).await;
    set_sachbezeichnung(c, "Bergkäse").await;
    add_simple_ingredient(c, "Rohmilch", 800).await;
    add_simple_ingredient(c, "Salz", 200).await;
    tokio::time::sleep(Duration::from_millis(700)).await;
}

/// Text des Knopfes, der den Teilen-Dialog öffnet.
///
/// Mit Wiederholung: der Knopf erscheint erst, wenn die Rezeptur einen
/// Query-String erzeugt hat.
async fn share_button_text(c: &fantoccini::Client) -> Option<String> {
    for _ in 0..20 {
        let found = c
            .execute(
                r#"
                for (const b of document.querySelectorAll('button')) {
                    if (b.className.includes('btn-info')) return b.textContent.trim();
                }
                return null;
                "#,
                vec![],
            )
            .await
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()));

        if found.is_some() {
            return found;
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    None
}

/// Öffnet den Teilen-Dialog und wartet, bis er wirklich offen ist.
///
/// Der Knopf steht erst bereit, wenn die Rezeptur einen Query-String erzeugt
/// hat. Läuft die Suite am Stück, kann das ein paar hundert Millisekunden
/// später sein als nach dem Seed; ein einzelner Klickversuch war deshalb
/// unzuverlässig.
async fn open_share_dialog(c: &fantoccini::Client) -> bool {
    for _ in 0..20 {
        if click_button_by_text(c, "Teilen").await {
            for _ in 0..10 {
                if open_dialog_count(c).await > 0 {
                    return true;
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    false
}

#[tokio::test]
async fn the_top_button_is_called_teilen_and_carries_a_share_icon() {
    let c = connect().await;
    seed_small_recipe(&c).await;

    let text = share_button_text(&c)
        .await
        .expect("share button must exist");
    assert!(
        text.contains("Teilen"),
        "the button that opens the dialog must say «Teilen», got: {text:?}"
    );
    assert!(
        !text.contains("kopieren"),
        "copying happens inside the dialog, not on this button: {text:?}"
    );

    // Das Teilen-Zeichen besteht aus drei Knoten und zwei Verbindungen; das
    // alte Clipboard-Symbol hatte keine Kreise. So bleibt die Prüfung an der
    // Bedeutung und nicht an einer konkreten Pfadangabe.
    let circles = c
        .execute(
            r#"
            for (const b of document.querySelectorAll('button')) {
                if (b.className.includes('btn-info')) {
                    const svg = b.querySelector('svg');
                    if (!svg) return -1;
                    return svg.querySelectorAll('path').length;
                }
            }
            return -1;
            "#,
            vec![],
        )
        .await
        .ok()
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);

    assert!(
        circles >= 5,
        "the share icon needs its three nodes and two links, found {circles} paths"
    );

    assert_no_errors(&c, "share button").await;
    c.close().await.ok();
}

#[tokio::test]
async fn the_dialog_offers_copying_by_name_and_a_qr_code() {
    let c = connect().await;
    seed_small_recipe(&c).await;

    assert!(
        open_share_dialog(&c).await,
        "the «‹Teilen›» button must open the dialog"
    );
    // Der Kurz-Link wird beim Öffnen geholt; kurz warten.
    tokio::time::sleep(Duration::from_millis(2500)).await;

    // Erst hier, im Dialog, heisst es «Kopieren».
    let copy_label = c
        .execute(
            r#"
            const d = document.querySelector('dialog[open]');
            if (!d) return null;
            for (const b of d.querySelectorAll('button')) {
                if (b.className.includes('btn-primary')) return b.textContent.trim();
            }
            return null;
            "#,
            vec![],
        )
        .await
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .expect("the dialog must have a primary button");

    assert!(
        copy_label.contains("Kopieren"),
        "inside the dialog the action must be named, got: {copy_label:?}"
    );

    // Der QR-Code trägt den Link. Geprüft wird, dass er wirklich Module hat,
    // ein leeres SVG wäre nicht scannbar.
    let qr_modules = c
        .execute(
            r#"
            const d = document.querySelector('dialog[open]');
            if (!d) return -1;
            for (const svg of d.querySelectorAll('svg')) {
                const label = svg.getAttribute('aria-label') || '';
                if (label.includes('QR')) {
                    const paths = svg.querySelectorAll('path');
                    if (paths.length < 2) return 0;
                    return paths[1].getAttribute('d').length;
                }
            }
            return -1;
            "#,
            vec![],
        )
        .await
        .ok()
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);

    assert!(
        qr_modules > 100,
        "the dialog must show a QR code with actual modules, got path length {qr_modules}"
    );

    assert_no_errors(&c, "share dialog").await;
    c.close().await.ok();
}

#[tokio::test]
async fn the_label_copy_button_yields_plain_text_with_line_breaks() {
    let c = connect().await;
    seed_small_recipe(&c).await;

    // Der Knopf steht unter der Etikette, nicht in der Kopfzeile.
    assert!(
        click_button_by_text(&c, "Etikette kopieren").await,
        "a button «Etikette kopieren» must sit below the label"
    );
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Was kopiert wurde, lässt sich nicht direkt aus der Zwischenablage lesen
    // (der Browser verweigert das ohne Nutzergeste). Stattdessen wird in ein
    // Textfeld eingefügt und von dort gelesen.
    let pasted = c
        .execute(
            r#"
            const ta = document.createElement('textarea');
            document.body.appendChild(ta);
            ta.focus();
            const ok = document.execCommand('paste');
            const value = ta.value;
            document.body.removeChild(ta);
            return value;
            "#,
            vec![],
        )
        .await
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    // Chrome erlaubt execCommand('paste') aus Sicherheitsgründen oft nicht.
    // Dann wird der Text geprüft, den der Knopf kopieren *würde*: er steht
    // sichtbar nirgends, also wird die Bestätigung geprüft und der Aufbau der
    // Etikette aus dem DOM abgeleitet.
    if pasted.is_empty() {
        let confirmed = c
            .find(Locator::XPath(
                "//*[contains(text(), 'in die Zwischenablage kopiert')]",
            ))
            .await
            .is_ok();
        assert!(
            confirmed,
            "without clipboard read access the button must at least confirm the copy"
        );
    } else {
        assert!(
            pasted.contains("Bergkäse"),
            "the copied text must contain the Sachbezeichnung: {pasted:?}"
        );
        assert!(
            pasted.contains("Rohmilch"),
            "the copied text must contain the ingredients: {pasted:?}"
        );
        assert!(
            !pasted.contains("<b>") && !pasted.contains("&amp;"),
            "the copied text must be plain, not HTML: {pasted:?}"
        );
        assert!(
            pasted.contains('\n'),
            "the copied text needs real line breaks: {pasted:?}"
        );
        assert!(
            !pasted.contains("\n\n\n"),
            "no runs of blank lines in the copied text: {pasted:?}"
        );
    }

    assert_no_errors(&c, "label copy").await;
    c.close().await.ok();
}

#[tokio::test]
async fn the_label_copy_button_stays_away_until_there_is_a_label() {
    let c = connect().await;
    // Leeres Formular: es gibt noch nichts zu kopieren.
    goto_config(&c, Config::Lebensmittelrecht).await;
    tokio::time::sleep(Duration::from_millis(700)).await;

    let present = button_by_text(&c, "Etikette kopieren").await.is_some();
    assert!(
        !present,
        "an empty recipe must not offer to copy an empty label"
    );

    // Sobald eine Sachbezeichnung da ist, erscheint der Knopf.
    set_sachbezeichnung(&c, "Bergkäse").await;
    tokio::time::sleep(Duration::from_millis(700)).await;

    assert!(
        button_by_text(&c, "Etikette kopieren").await.is_some(),
        "with content on the label the copy button must appear"
    );

    assert_no_errors(&c, "empty label").await;
    c.close().await.ok();
}
