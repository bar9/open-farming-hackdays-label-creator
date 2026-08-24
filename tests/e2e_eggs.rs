// DEC-13: a pack of eggs is declared by piece count, not by a weight-based
// Grundpreis. When the Sachbezeichnung names eggs, the Grundpreis field turns
// into «Anzahl Eier» with the unit «Stück», and Abtropfgewicht disappears.
//
// Run serially against a served build + chromedriver, like the other e2e
// suites: `cargo test --test e2e_eggs -- --test-threads=1`.
mod common;

use common::recipes::Config;
use common::*;
use fantoccini::Locator;
use std::time::Duration;

/// Visible field labels of the amount/price section.
async fn form_labels(c: &fantoccini::Client) -> Vec<String> {
    let v = c
        .execute(
            r#"
            return [...document.querySelectorAll("label, .label, span")]
              .map(e => e.textContent.trim())
              .filter(t => t.length > 0 && t.length < 60);
            "#,
            vec![],
        )
        .await
        .unwrap();
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|s| s.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

async fn has_label(c: &fantoccini::Client, needle: &str) -> bool {
    form_labels(c).await.iter().any(|l| l.contains(needle))
}

/// Replace the Sachbezeichnung. `set_sachbezeichnung` only appends, which
/// would produce "KonfitüreEier" when switching between products.
async fn replace_sachbezeichnung(c: &fantoccini::Client, value: &str) {
    let el = input_by_placeholder(c, "Himbeerkonfitüre")
        .await
        .expect("Sachbezeichnung input");
    el.click().await.unwrap();
    // Ctrl+A then type over the selection.
    el.send_keys("\u{E009}a").await.unwrap();
    el.send_keys(value).await.unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
}

#[tokio::test]
async fn egg_sachbezeichnung_switches_grundpreis_to_piece_count() {
    let c = connect().await;
    goto_config(&c, Config::Lebensmittelrecht).await;

    // Baseline: a normal product shows Grundpreis and offers Abtropfgewicht.
    replace_sachbezeichnung(&c, "Konfitüre").await;
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert!(
        has_label(&c, "Grundpreis").await,
        "a normal product should show the Grundpreis field"
    );
    assert!(
        has_label(&c, "Abtropfgewicht").await,
        "a normal product should offer Abtropfgewicht"
    );
    assert!(
        !has_label(&c, "Anzahl Eier").await,
        "a normal product must not show the egg count field"
    );

    // Eggs: Grundpreis becomes «Anzahl Eier» and Abtropfgewicht disappears.
    replace_sachbezeichnung(&c, "Eier").await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(
        has_label(&c, "Anzahl Eier").await,
        "«Eier» should turn the Grundpreis field into «Anzahl Eier»"
    );
    assert!(
        has_label(&c, "Stück").await,
        "the egg count field should carry the unit «Stück»"
    );
    assert!(
        !has_label(&c, "Grundpreis").await,
        "«Eier» should replace the Grundpreis field"
    );
    assert!(
        !has_label(&c, "Abtropfgewicht").await,
        "«Eier» should hide Abtropfgewicht"
    );

    assert_no_errors(&c, "egg mode").await;
    let _ = c.close().await;
}

#[tokio::test]
async fn egg_count_reaches_the_label() {
    let c = connect().await;
    goto_config(&c, Config::Lebensmittelrecht).await;
    set_sachbezeichnung(&c, "Eier").await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // The egg count input sits next to the «Stück» badge.
    let input = c
        .find(Locator::XPath(
            "//*[contains(@class,'badge') and contains(normalize-space(.), 'Stück')]\
/preceding::input[@type='number'][1]",
        ))
        .await
        .expect("egg count input");
    input.click().await.unwrap();
    input.send_keys("6").await.unwrap();
    tokio::time::sleep(Duration::from_millis(600)).await;

    let label = label_html(&c).await;
    assert!(
        label.contains("6") && label.contains("Stück"),
        "label should declare the piece count; label: {}",
        label
    );

    assert_no_errors(&c, "egg count on label").await;
    let _ = c.close().await;
}

// A product that merely contains egg is still sold by weight.
#[tokio::test]
async fn egg_containing_products_keep_the_normal_fields() {
    let c = connect().await;
    goto_config(&c, Config::Lebensmittelrecht).await;
    set_sachbezeichnung(&c, "Eierlikör").await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    assert!(
        has_label(&c, "Grundpreis").await,
        "«Eierlikör» is sold by weight and keeps the Grundpreis field"
    );
    assert!(
        !has_label(&c, "Anzahl Eier").await,
        "«Eierlikör» must not switch to the egg count field"
    );

    let _ = c.close().await;
}
