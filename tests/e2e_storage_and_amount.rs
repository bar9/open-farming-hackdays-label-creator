// R-14 / R-15 (Livetesting 09.11.2026): Aufbewahrungshinweise per Knopf,
// Abschnitt «Verpackungseinheit» mit erklärendem Satz.
//
// Sequentiell laufen lassen: `cargo test --test e2e_storage_and_amount -- --test-threads=1`

mod common;

use common::recipes::Config;
use common::*;
use std::time::Duration;

async fn storage_textarea_value(c: &fantoccini::Client) -> String {
    c.execute(
        r#"
        const picker = document.querySelector('[data-testid=storage-hint-picker]');
        if (!picker) return null;
        const ta = picker.parentElement.querySelector('textarea');
        return ta ? ta.value : null;
        "#,
        vec![],
    )
    .await
    .ok()
    .and_then(|v| v.as_str().map(|s| s.to_string()))
    .unwrap_or_default()
}

#[tokio::test]
async fn storage_hint_buttons_fill_the_field_and_the_label() {
    let c = connect().await;
    goto_config(&c, Config::Lebensmittelrecht).await;
    accept_disclaimer(&c).await;
    set_sachbezeichnung(&c, "Konfitüre").await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    assert!(
        click_button_by_text(&c, "kühl und trocken lagern").await,
        "a button for «kühl und trocken lagern» must be offered"
    );
    assert!(click_button_by_text(&c, "lichtgeschützt lagern").await);
    // Zweiter Klick auf denselben Hinweis erzeugt keine doppelte Zeile.
    assert!(click_button_by_text(&c, "kühl und trocken lagern").await);
    tokio::time::sleep(Duration::from_millis(400)).await;

    let value = storage_textarea_value(&c).await;
    assert_eq!(value, "kühl und trocken lagern\nlichtgeschützt lagern");

    assert_label_contains(&c, "kühl und trocken lagern", "storage hint on label").await;
    assert_label_contains(&c, "lichtgeschützt lagern", "second storage hint on label").await;

    // Die Erklärung am Knopf (Eiprodukte) gehört nicht auf die Etikette.
    assert!(click_button_by_text(&c, "bei max. 4 °C lagern").await);
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert_label_not_contains(&c, "Eiprodukte", "chip hint must stay off the label").await;

    assert_no_errors(&c, "storage hints").await;
    c.close().await.ok();
}

#[tokio::test]
async fn the_amount_section_says_it_is_about_one_package() {
    let c = connect().await;
    goto_config(&c, Config::Lebensmittelrecht).await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let state = c
        .execute(
            r#"
            const intro = document.querySelector('[data-testid=amount-price-intro]');
            const heading = intro ? intro.parentElement.querySelector('h4') : null;
            return JSON.stringify({
                heading: heading ? heading.textContent : '',
                intro: intro ? intro.textContent : '',
            });
            "#,
            vec![],
        )
        .await
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    assert!(
        state.contains("Verpackungseinheit"),
        "heading renamed: {state}"
    );
    assert!(
        state.contains("einzelne Packung"),
        "intro sentence shown: {state}"
    );

    assert_no_errors(&c, "amount intro").await;
    c.close().await.ok();
}
