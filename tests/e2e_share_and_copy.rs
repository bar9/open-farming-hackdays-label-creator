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

use common::recipes::{BioStatus, Config, RecipeIngredient};
use common::*;
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

/// Liest den Text, den der Kopier-Knopf in die Zwischenablage legt.
///
/// Aus der Zwischenablage zurückzulesen verweigert Chrome ohne Nutzergeste.
/// Die App legt zum Kopieren kurzzeitig ein `<textarea>` an (siehe
/// `services::label_text::copy_to_clipboard`); wird `appendChild` überwacht,
/// lässt sich genau der Text abgreifen, der auch kopiert wird.
async fn capture_copied_text(c: &fantoccini::Client, button: &str) -> String {
    c.execute(
        r#"
        window.__captured = null;
        const orig = document.body.appendChild.bind(document.body);
        document.body.appendChild = function(node) {
            if (node && node.tagName === 'TEXTAREA') { window.__captured = node.value; }
            return orig(node);
        };
        return null;
        "#,
        vec![],
    )
    .await
    .expect("install capture");

    click_button_by_text(c, button).await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    c.execute("return window.__captured;", vec![])
        .await
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default()
}

#[tokio::test]
async fn the_label_copy_button_yields_plain_text_with_line_breaks() {
    let c = connect().await;
    seed_small_recipe(&c).await;

    // Der Knopf steht unter der Etikette, nicht in der Kopfzeile.
    assert!(
        button_by_text(&c, "Etikette kopieren").await.is_some(),
        "a button «Etikette kopieren» must sit below the label"
    );

    let copied = capture_copied_text(&c, "Etikette kopieren").await;

    assert!(
        copied.contains("Bergkäse"),
        "the copied text must contain the Sachbezeichnung: {copied:?}"
    );
    assert!(
        copied.contains("Rohmilch"),
        "the copied text must contain the ingredients: {copied:?}"
    );
    assert!(
        !copied.contains("<b>") && !copied.contains("&amp;") && !copied.contains("&#x27;"),
        "the copied text must be plain, not HTML: {copied:?}"
    );
    assert!(
        copied.contains("\n\n"),
        "sections need a blank line between them: {copied:?}"
    );
    assert!(
        !copied.contains("\n\n\n"),
        "no runs of blank lines in the copied text: {copied:?}"
    );
    // Die Badges der Vorschau sind Hinweise, keine Etiketteninhalte.
    assert!(
        !copied.contains("Zutatenliste") && !copied.contains("Herstelleradresse"),
        "placeholder badges must not end up in the copied label: {copied:?}"
    );

    assert_no_errors(&c, "label copy").await;
    c.close().await.ok();
}

#[tokio::test]
async fn the_copied_label_keeps_multi_line_blocks_together() {
    // Die Herstellerangaben sind mehrzeilig und dürfen innerhalb nicht durch
    // Leerzeilen auseinandergerissen werden.
    let c = connect().await;
    goto_config(&c, Config::Lebensmittelrecht).await;
    let query = "v=2&product_title=Hausmarke&product_subtitle=Bergk%C3%A4se\
&amount_type=Weight&weight_unit=g&amount[Single]=250&price[Single]=1250\
&producer_name=Hof%20Muster&producer_address=Dorfstrasse%201&producer_zip=3000\
&producer_city=Bern&producer_phone=031%20111%2022%2033&producer_email=hof@muster.ch\
&producer_website=www.muster.ch&production_country=Schweiz";
    goto(&c, &format!("lebensmittelrecht?{query}")).await;
    tokio::time::sleep(mount_delay()).await;
    tokio::time::sleep(Duration::from_millis(700)).await;

    let copied = capture_copied_text(&c, "Etikette kopieren").await;

    assert!(
        copied.contains("Hof Muster\nDorfstrasse 1\n3000 Bern"),
        "the producer address must stay one block with single breaks: {copied:?}"
    );
    assert!(
        copied.contains("Hausmarke\nBergkäse"),
        "product name and Sachbezeichnung belong together: {copied:?}"
    );
    assert!(
        copied.contains("Tel: 031 111 22 33") && copied.contains("E-Mail: hof@muster.ch"),
        "contact lines must be part of the copied label: {copied:?}"
    );

    assert_no_errors(&c, "multi-line blocks").await;
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

/// Die Fussnotenlegende der Bio-Zutaten braucht eine eigene Zeile.
///
/// In der Vorschau trennt ein `<br>` die Legende von der Zutatenliste. Wurde
/// das beim Umwandeln in Klartext verschluckt, stand dort
/// «Bärlauch** aus biologischer Landwirtschaft»: das Fussnotenzeichen der
/// letzten Zutat verschmolz mit dem der Legende zu einem doppelten Stern, der
/// eine zweite Fussnote vortäuscht. Auf einer Etikette ist das eine falsche
/// Angabe, kein Schönheitsfehler.
#[tokio::test]
async fn the_footnote_legend_stands_on_its_own_line() {
    let c = connect().await;
    goto_config(&c, Config::Knospe).await;
    set_sachbezeichnung(&c, "Bärlauchpesto").await;
    add_full_ingredient(
        &c,
        &RecipeIngredient {
            name: "Weizenmehl",
            grams: 600.0,
            origin: Some("CH"),
            bio: BioStatus::BioCh,
        },
    )
    .await;
    add_full_ingredient(
        &c,
        &RecipeIngredient {
            name: "Bärlauch",
            grams: 400.0,
            origin: Some("CH"),
            bio: BioStatus::BioCh,
        },
    )
    .await;
    tokio::time::sleep(Duration::from_millis(1500)).await;

    let copied = capture_copied_text(&c, "Etikette kopieren").await;

    assert!(
        copied.contains("Bärlauch*\n* aus biologischer Landwirtschaft"),
        "the legend needs its own line below the list: {copied:?}"
    );
    assert!(
        !copied.contains("**"),
        "a doubled asterisk would pretend a second footnote: {copied:?}"
    );

    assert_no_errors(&c, "footnote legend").await;
    c.close().await.ok();
}

/// Eigene Zeilenumbrüche in den Freitextfeldern müssen erhalten bleiben.
///
/// «Weitere Angaben» und «Lagerhinweis» dürfen mehrzeilig sein; die Vorschau
/// bildet das ab. Im kopierten Text müssen dieselben Umbrüche stehen, und
/// zwar als einfache: eine Leerzeile dazwischen sähe aus wie ein neuer
/// Abschnitt der Etikette.
#[tokio::test]
async fn multi_line_free_text_keeps_its_own_breaks() {
    let c = connect().await;
    goto_config(&c, Config::Lebensmittelrecht).await;
    let query = "v=2&product_subtitle=Bergk%C3%A4se&amount_type=Weight&weight_unit=g\
&amount[Single]=250&additional_info=Rohmilchk%C3%A4se%0AMindestens%206%20Monate%20gereift\
&storage_info=K%C3%BChl%20lagern%0ANach%20dem%20%C3%96ffnen%20rasch%20verbrauchen\
&producer_name=Hof%20Muster&production_country=Schweiz";
    goto(&c, &format!("lebensmittelrecht?{query}")).await;
    tokio::time::sleep(mount_delay()).await;
    tokio::time::sleep(Duration::from_millis(700)).await;

    let copied = capture_copied_text(&c, "Etikette kopieren").await;

    assert!(
        copied.contains("Rohmilchkäse\nMindestens 6 Monate gereift"),
        "line breaks typed into «Weitere Angaben» must survive: {copied:?}"
    );
    assert!(
        copied.contains("Kühl lagern\nNach dem Öffnen rasch verbrauchen"),
        "line breaks typed into the storage hint must survive: {copied:?}"
    );
    // Beide Freitexte gehören in denselben Abschnitt, wie in der Vorschau.
    assert!(
        copied.contains("Mindestens 6 Monate gereift\nKühl lagern"),
        "both free texts form one block: {copied:?}"
    );

    assert_no_errors(&c, "multi-line free text").await;
    c.close().await.ok();
}

/// Die Bio-Zertifizierungsstelle gehört in den kopierten Text.
///
/// Sie ist eine Pflichtangabe der Bio-Verordnung und steht in der Vorschau
/// als eigener Block unter der Etikette. Fehlte sie im Klartext, wäre eine
/// daraus gedruckte Etikette unvollständig.
#[tokio::test]
async fn the_certification_body_is_part_of_the_copied_label() {
    let c = connect().await;
    goto_config(&c, Config::Bio).await;
    let query = "v=2&product_subtitle=Bergk%C3%A4se&amount_type=Weight&weight_unit=g\
&amount[Single]=250&producer_name=Hof%20Muster&production_country=Schweiz\
&certification_body=bio.inspecta%20AG";
    goto(&c, &format!("bio?{query}")).await;
    tokio::time::sleep(mount_delay()).await;

    // Auf den Knopf warten, statt auf eine feste Zeit zu hoffen: die
    // Bio-Route baut die Vorschau spürbar später auf als die anderen.
    let mut ready = false;
    for _ in 0..20 {
        if button_by_text(&c, "Etikette kopieren").await.is_some() {
            ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    assert!(ready, "the copy button must appear on the Bio route");

    let copied = capture_copied_text(&c, "Etikette kopieren").await;

    assert!(
        copied.contains("bio.inspecta AG"),
        "the certification body is mandatory on an organic label: {copied:?}"
    );
    // Eigener Abschnitt, wie in der Vorschau: davor eine Leerzeile.
    assert!(
        copied.contains("\n\nBio-Zertifizierung: bio.inspecta AG"),
        "it belongs in its own section: {copied:?}"
    );

    assert_no_errors(&c, "certification body").await;
    c.close().await.ok();
}

/// Auch der vollständige Link muss im Dialog brauchbar bleiben.
///
/// Er trägt die ganze Rezeptur im Query-String und wird dadurch lang. Passt
/// er noch in einen QR-Code, muss dieser den Link tragen; passt er nicht mehr
/// hinein, darf kein leeres oder kaputtes Bild stehenbleiben. In beiden
/// Fällen bleibt das Textfeld der verlässliche Weg.
#[tokio::test]
async fn the_full_link_keeps_the_dialog_usable() {
    let c = connect().await;
    seed_small_recipe(&c).await;

    assert!(
        open_share_dialog(&c).await,
        "the «Teilen» button must open the dialog"
    );
    tokio::time::sleep(Duration::from_millis(2500)).await;

    // Auf «Vollständiger Link» umschalten.
    let switched = c
        .execute(
            r#"
            const d = document.querySelector('dialog[open]');
            if (!d) return false;
            const radios = [...d.querySelectorAll('input[type=radio]')];
            if (radios.length < 2) return false;
            radios[1].click();
            return true;
            "#,
            vec![],
        )
        .await
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    assert!(switched, "the dialog must offer the full link as an option");
    tokio::time::sleep(Duration::from_millis(1200)).await;

    let state = c
        .execute(
            r#"
            const d = document.querySelector('dialog[open]');
            const input = d.querySelector('input[type=text]');
            let qrLength = -1;
            for (const svg of d.querySelectorAll('svg')) {
                if ((svg.getAttribute('aria-label') || '').includes('QR')) {
                    const paths = svg.querySelectorAll('path');
                    qrLength = paths.length > 1 ? paths[1].getAttribute('d').length : 0;
                }
            }
            return JSON.stringify({
                link: input ? input.value : '',
                qr: qrLength,
            });
            "#,
            vec![],
        )
        .await
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    // Das Textfeld trägt in jedem Fall den vollen Link mit der Rezeptur.
    assert!(
        state.contains("product_subtitle") || state.contains("Bergk"),
        "the full link must carry the recipe in its query string: {state}"
    );
    // Entweder ein Code mit echten Modulen oder gar keiner; ein leeres SVG
    // wäre eine Attrappe, die zum Scannen einlädt und nichts liefert.
    assert!(
        !state.contains("\"qr\":0"),
        "an empty QR image must not be shown: {state}"
    );

    assert_no_errors(&c, "full link dialog").await;
    c.close().await.ok();
}

// Der Erfolg des Kopierens ist sonst nur am Knopf sichtbar, während der Blick
// auf der Vorschau liegt. Die Etikette selbst quittiert ihn deshalb kurz und
// nimmt die Quittung danach wieder zurück; ein dauerhafter Rahmen wäre Dekor.
#[tokio::test]
async fn the_label_itself_acknowledges_a_successful_copy() {
    let c = connect().await;
    seed_small_recipe(&c).await;

    let flashing = |c: &fantoccini::Client| {
        let c = c.clone();
        async move {
            c.execute(
                "return document.querySelectorAll('.label-copied-flash').length;",
                vec![],
            )
            .await
            .ok()
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
        }
    };

    assert_eq!(
        flashing(&c).await,
        0,
        "the label must be calm before anything was copied"
    );

    click_button_by_text(&c, "Etikette kopieren").await;
    tokio::time::sleep(Duration::from_millis(300)).await;

    assert_eq!(
        flashing(&c).await,
        1,
        "the label must acknowledge the copy right after the click"
    );

    // Die Quittung am Knopf dauert zwei Sekunden; danach ist auch die
    // Etikette wieder ruhig.
    tokio::time::sleep(Duration::from_millis(2200)).await;
    assert_eq!(
        flashing(&c).await,
        0,
        "the acknowledgement must fade instead of staying on the label"
    );

    assert_no_errors(&c, "label copy effect").await;
    c.close().await.ok();
}
