// Der geteilte Link muss die Rezeptur wirklich tragen.
//
// Er tat es zeitweise nicht: Das Teilen-Fenster haengt im Layout und entsteht
// schon beim Start der Anwendung. Seinen Link hielt es in einem `use_signal`
// fest, und das merkt sich allein den ersten Wert. Also blieb der Link fuer
// immer der des leeren Formulars, mit `product_title=` und `ingredients=`.
// Weil der Kurz-Link das Ziel verbirgt, sah man es erst beim Oeffnen.
//
// Geprueft wird deshalb der ganze Weg und nicht nur die Beschriftung: teilen,
// den Link oeffnen, die Etikette wiederfinden, erneut teilen.
//
// Sequentiell laufen lassen: `cargo test --test e2e_share_roundtrip -- --test-threads=1`

mod common;

use common::recipes::Config;
use common::*;
use std::time::Duration;

/// Oeffnet den Teilen-Dialog und liest den *vollstaendigen* Link.
///
/// Der Kurz-Link wuerde nur `https://da.gd/...` zeigen und damit gerade das
/// verbergen, worum es hier geht.
async fn full_share_link(c: &fantoccini::Client) -> String {
    for _ in 0..20 {
        if click_button_by_text(c, "Teilen").await {
            tokio::time::sleep(Duration::from_millis(600)).await;

            let _ = c
                .execute(
                    r#"
                    for (const l of document.querySelectorAll('label')) {
                        if (l.textContent.includes('ollst')) {
                            const r = l.querySelector('input[type=radio]');
                            if (r) { r.click(); return 1; }
                        }
                    }
                    return 0;
                    "#,
                    vec![],
                )
                .await;
            tokio::time::sleep(Duration::from_millis(400)).await;

            for _ in 0..15 {
                if let Some(u) = read_share_url(c).await {
                    return u;
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    String::new()
}

#[tokio::test]
async fn a_shared_link_carries_the_recipe_and_restores_it() {
    let c = connect().await;
    goto_config(&c, Config::Lebensmittelrecht).await;
    set_sachbezeichnung(&c, "Bergkäse").await;
    set_product_title(&c, "Alpzauber").await;
    add_simple_ingredient(&c, "Rohmilch", 800).await;
    tokio::time::sleep(Duration::from_millis(700)).await;

    let link = full_share_link(&c).await;
    assert!(
        link.contains("Bergk") && link.contains("Alpzauber"),
        "the shared link must carry the recipe, not the empty form: {link}"
    );
    assert!(
        link.contains("Rohmilch"),
        "the shared link must carry the ingredients: {link}"
    );

    // Der eigentliche Beweis: der Link fuellt das Formular wieder.
    c.goto(&link).await.expect("open the shared link");
    tokio::time::sleep(mount_delay()).await;
    tokio::time::sleep(Duration::from_millis(1500)).await;

    let html = label_html(&c).await;
    assert!(
        html.contains("Bergk") && html.contains("Alpzauber") && html.contains("Rohmilch"),
        "the opened link must fill the form again: {html}"
    );

    // Und weiterreichen laesst er sich auch: der zweite Link ist nicht leer,
    // sonst waere die Kette nach einmal Oeffnen gerissen.
    let link2 = full_share_link(&c).await;
    assert!(
        link2.contains("Bergk") && link2.contains("Rohmilch"),
        "re-sharing after a restore must keep the recipe: {link2}"
    );

    assert_no_errors(&c, "share roundtrip").await;
    c.close().await.ok();
}
