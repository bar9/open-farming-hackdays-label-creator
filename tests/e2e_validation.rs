// Validation flow tests (MT §1.1 paths, §2.2, §3.4, §3.5).
//
// Each test seeds a baseline, mutates one field, asserts the validation
// banner appears, restores, asserts cleared. These are the highest
// bug-catching value tests in the suite — they exercise the Signal/Memo
// update path that has historically been a regression source.
//
// Run sequentially: `cargo test --test e2e_validation -- --test-threads=1`

mod common;

use common::recipes::*;
use common::*;

// ---------- §1.1 — validation appears + clears for missing origin ----------
//
// Seed Rindshackbraten with Rindfleisch missing its origin. Press
// "Rezeptur vollständig" (which gates the validation rules per
// `core.rs:795`). Verify the AP7_1 origin-required banner appears.
// Then open the edit card, set the origin to CH, and verify the banner
// disappears.

#[tokio::test]
async fn validation_clears_when_origin_restored() {
    let c = connect().await;
    goto_config(&c, RINDSHACKBRATEN.config).await;

    // Same recipe as RINDSHACKBRATEN but with Rindfleisch.origin = None
    // so AP7_1 (Herkunft >50% required) fires after Rezeptur vollständig.
    let recipe = Recipe {
        config: Config::Lebensmittelrecht,
        product_name: "Rindshackbraten — kein Origin",
        sachbezeichnung: "Rindshackbraten",
        certification_body: None,
        ingredients: &[
            RecipeIngredient { name: "Rindfleisch", grams: 350.0, origin: None,        bio: BioStatus::Conventional },
            RecipeIngredient { name: "Zwiebel",     grams: 80.0,  origin: Some("CH"), bio: BioStatus::Conventional },
            RecipeIngredient { name: "Salz",        grams: 5.0,   origin: None,       bio: BioStatus::Conventional },
        ],
    };
    seed_recipe_via_ui(&c, &recipe).await;
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;

    let body: String = c
        .execute("return document.body.innerText;", vec![])
        .await
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();
    assert!(
        body.contains("Herkunft"),
        "expected origin-required validation banner after Rezeptur vollständig (Rindfleisch is 80% of total — AP7_1 should fire). body excerpt:\n{}",
        &body.chars().take(2000).collect::<String>()
    );

    // Add origin via edit card and verify the banner disappears.
    let opened = open_ingredient_edit_by_name(&c, "Rindfleisch").await;
    assert!(opened, "could not open Rindfleisch edit card");
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;
    set_origin_in_open_dialog(&c, "CH").await;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    click_button_by_text(&c, "Speichern").await;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let body_after: String = c
        .execute("return document.body.innerText;", vec![])
        .await
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();
    // The strict assertion is "no origin-required validation message
    // referencing Rindfleisch". Sub-string scan: if "Herkunft" still
    // appears alongside the validation key fragment, the rule is still
    // active. Form-level "Herkunft" labels (e.g. "Herkunft *" on the
    // input) appear regardless and don't indicate failure — so we
    // require the more specific "erforderlich"/"benötigt" or rule key.
    assert!(
        !(body_after.contains("erforderlich") || body_after.contains("benötigt")),
        "expected origin-required validation banner to clear after setting Rindfleisch origin. body excerpt:\n{}",
        &body_after.chars().take(2000).collect::<String>()
    );

    assert_no_errors(&c, "validation_clears_when_origin_restored").await;
    let _ = c.close().await;
}

// ---------- §2.2 — Bio cert body required ----------

#[tokio::test]
async fn bio_cert_body_required() {
    let c = connect().await;

    // Seed a Bio recipe but skip the certification body.
    let recipe = Recipe {
        config: Config::Bio,
        product_name: "Bio Test ohne CH-BIO",
        sachbezeichnung: "Bio Test",
        certification_body: None,
        ingredients: &[
            RecipeIngredient {
                name: "Erdbeeren",
                grams: 100.0,
                origin: Some("CH"),
                bio: BioStatus::BioCh,
            },
        ],
    };
    goto_config(&c, recipe.config).await;
    seed_recipe_via_ui(&c, &recipe).await;

    // The form-level validation flag `Bio_Knospe_ZertifizierungsstellePflicht`
    // surfaces a validation message somewhere in the body.
    let body: String = c
        .execute("return document.body.innerText;", vec![])
        .await
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();
    assert!(
        body.contains("Zertifizierung") || body.contains("zertifizierung"),
        "expected certification body validation message. body:\n{}",
        body
    );

    assert_no_errors(&c, "bio_cert_body_required").await;
    let _ = c.close().await;
}

// ---------- §3.5 — Knospe origin required for ALL ingredients ----------

#[tokio::test]
async fn knospe_origin_required_for_all() {
    let c = connect().await;

    // Knospe recipe with one ingredient missing origin.
    let recipe = Recipe {
        config: Config::Knospe,
        product_name: "Knospe Test",
        sachbezeichnung: "Knospe Test",
        certification_body: Some("CH-BIO-006"),
        ingredients: &[
            RecipeIngredient {
                name: "Weizenmehl",
                grams: 50.0,
                origin: Some("CH"),
                bio: BioStatus::BioKnospe,
            },
            RecipeIngredient {
                name: "Salz",
                grams: 1.0,
                origin: None, // Missing — Knospe_AlleZutatenHerkunft must flag it.
                bio: BioStatus::NichtLandwirtschaftlich,
            },
        ],
    };
    goto_config(&c, recipe.config).await;
    seed_recipe_via_ui(&c, &recipe).await;

    let body: String = c
        .execute("return document.body.innerText;", vec![])
        .await
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();
    assert!(
        body.contains("Herkunft"),
        "expected origin-required validation when an ingredient lacks origin in Knospe config. body:\n{}",
        body
    );

    assert_no_errors(&c, "knospe_origin_required_for_all").await;
    let _ = c.close().await;
}

// ---------- §3.4 — Knospe fail path: change radio Bio (Knospe) → Bio ----------

#[tokio::test]
async fn knospe_fail_path_radio_change() {
    let c = connect().await;
    goto_config(&c, SCHOGGI_COOKIE_BSK.config).await;
    seed_recipe_via_ui(&c, &SCHOGGI_COOKIE_BSK).await;

    // Open Bratbutter's card and switch its bio radio from "Bio (Knospe)" to "Bio".
    // (Recipe data uses "Bratbutter" — the food_db-exact name — so the typeahead
    // selects the right entry; see tests/common/recipes.rs::SCHOGGI_COOKIE_BSK.)
    let opened = open_ingredient_edit_by_name(&c, "Bratbutter").await;
    assert!(opened, "could not open Bratbutter edit card");
    let xpath = "//dialog[@open]//label[contains(normalize-space(.), 'Bio') and not(contains(normalize-space(.), 'Knospe'))]";
    if let Ok(el) = c.find(fantoccini::Locator::XPath(xpath)).await {
        let _ = el.click().await;
    }
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    click_button_by_text(&c, "Speichern").await;
    click_button_by_text(&c, "Schliessen").await;
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;

    // Knospe-fail warning banner: text from de-CH translation
    // "knospe_marketing_not_allowed" or similar phrasing.
    let body: String = c
        .execute("return document.body.innerText;", vec![])
        .await
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();
    assert!(
        body.contains("Knospe") && (body.contains("NICHT") || body.contains("nicht")),
        "expected knospe-not-allowed warning when Butter is no longer Knospe-bio. body:\n{}",
        body
    );

    assert_no_errors(&c, "knospe_fail_path_radio_change").await;
    let _ = c.close().await;
}

// ---------- §3.x — Knospe origin lock (CH) vs Knospe Import ----------
//
// New in the per-attribute aggregation work: selecting "Bio (Knospe)" locks
// origin to CH (origin control replaced by a locked CH badge); selecting
// "Bio (Knospe) Import" leaves origin editable.

#[tokio::test]
async fn knospe_origin_locks_to_ch_and_import_unlocks() {
    use std::time::Duration;
    let c = connect().await;
    goto_config(&c, Config::Knospe).await;
    assert!(open_add_ingredient(&c).await, "could not open genesis add-ingredient modal");
    tokio::time::sleep(Duration::from_millis(400)).await;

    // Enter a name + amount so the bio/quality block renders.
    if let Some(input) = first_accent_input(&c).await {
        let _ = input.click().await;
        let _ = input.send_keys("Weizenmehl").await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        let _ = input.send_keys("\u{E007}").await; // Enter commits exact match / custom
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    if let Ok(num) = c.find(fantoccini::Locator::Css("dialog[open] input[type='number']")).await {
        let _ = num.click().await;
        let _ = num.send_keys("100").await;
    }
    tokio::time::sleep(Duration::from_millis(200)).await;

    let dialog_text = |c: &fantoccini::Client| {
        let c = c.clone();
        async move {
            c.execute("const d=document.querySelector('dialog[open]'); return d?d.innerText:'';", vec![])
                .await.ok().and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default()
        }
    };

    // Plain "Bio (Knospe)" → origin locked to CH: static CH badge, no editable
    // country picker (the "Land hinzufügen" option of MultiCountrySelect is absent).
    let knospe_xpath = "//dialog[@open]//label[contains(normalize-space(.), 'Bio (Knospe)') and not(contains(normalize-space(.), 'Import'))]";
    if let Ok(el) = c.find(fantoccini::Locator::XPath(knospe_xpath)).await {
        let _ = el.click().await;
    }
    tokio::time::sleep(Duration::from_millis(400)).await;
    let after_knospe = dialog_text(&c).await;
    assert!(
        !after_knospe.contains("Land hinzufügen"),
        "plain Knospe must lock origin to CH (no editable country picker). dialog:\n{}",
        after_knospe
    );

    // Variante b: click the "Knospe Import" logo → origin editable again (picker reappears).
    let import_xpath = "//dialog[@open]//button[.//span[contains(normalize-space(.), 'Knospe Import')]]";
    if let Ok(el) = c.find(fantoccini::Locator::XPath(import_xpath)).await {
        let _ = el.click().await;
    }
    tokio::time::sleep(Duration::from_millis(400)).await;
    let after_import = dialog_text(&c).await;
    assert!(
        after_import.contains("Land hinzufügen"),
        "Knospe Import must leave origin editable (country picker present). dialog:\n{}",
        after_import
    );

    assert_no_errors(&c, "knospe_origin_locks_to_ch_and_import_unlocks").await;
    let _ = c.close().await;
}
