// Manual DEC-2 smoke check via WebDriver: the mono-product quality selector
// must appear when «Keine Zutatenliste» is ticked, and picking the Swiss Knospe
// must put the Knospe logo on the label preview.
mod common;

use common::recipes::{BioStatus, Config, RecipeIngredient};
use common::*;
use fantoccini::Locator;

#[tokio::test]
async fn mono_quality_selector_drives_the_knospe_logo() {
    let c = connect().await;
    goto_config(&c, Config::Knospe).await;

    // Fill the Sachbezeichnung so the preview renders a real label.
    if let Ok(el) = c
        .find(Locator::XPath(
            "//input[contains(@placeholder, 'Konfitüre') or contains(@placeholder, 'Sachbezeichnung')]",
        ))
        .await
    {
        let _ = el.click().await;
        let _ = el.send_keys("Weizenmehl").await;
    }

    // Tick «Keine Zutatenliste (Einzelzutat)». Click the checkbox itself; the
    // surrounding <label> is what FormField renders around it.
    let toggled = c
        .find(Locator::XPath(
            "//label[contains(normalize-space(.), 'Keine Zutatenliste')]//input[@type='checkbox']",
        ))
        .await;
    let toggled = match toggled {
        Ok(el) => el.click().await.is_ok(),
        Err(_) => false,
    };
    assert!(toggled, "could not find the «Keine Zutatenliste» checkbox");
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // The quality selector must now be present (DEC-2).
    let has_selector = c
        .find(Locator::XPath("//input[@name='mono_quality']"))
        .await
        .is_ok();
    assert!(
        has_selector,
        "mono-product quality selector missing after enabling «Keine Zutatenliste»"
    );

    // Choose «Bio (Knospe)»; the Swiss Knospe is the default variant.
    let picked = c
        .find(Locator::XPath("//input[@name='mono_quality'][1]"))
        .await;
    let picked = match picked {
        Ok(el) => el.click().await.is_ok(),
        Err(_) => false,
    };
    assert!(picked, "could not select «Bio (Knospe)»");
    tokio::time::sleep(std::time::Duration::from_millis(700)).await;

    assert!(
        has_bio_suisse_cross(&c).await,
        "Swiss Knospe must show the cross logo on the label"
    );

    let _ = c.close().await;
}

// DEC-6: the green Bio badge must follow the recipe, not the «Rezeptur prüfen»
// button. Seeding the recipe without pressing the button is the whole point, so
// this cannot reuse `seed_recipe_via_ui`.
#[tokio::test]
async fn bio_badge_appears_without_pressing_rezeptur_pruefen() {
    let c = connect().await;
    goto_config(&c, Config::Bio).await;

    // Empty form: no badge yet.
    assert!(
        !has_bio_qualified_badge(&c).await,
        "the badge must not show on an empty recipe"
    );

    set_sachbezeichnung(&c, "Haferflocken").await;
    add_full_ingredient(
        &c,
        &RecipeIngredient {
            name: "Hafer",
            grams: 1000.0,
            origin: Some("CH"),
            bio: BioStatus::BioCh,
        },
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(700)).await;

    // Deliberately NOT pressing «Rezeptur vollständig».
    assert!(
        has_bio_qualified_badge(&c).await,
        "a qualifying Bio recipe must show the badge without «Rezeptur prüfen»"
    );

    let _ = c.close().await;
}

// DEC-11: the Bio-V «Bio» quality must offer wild collection, with the
// Bio-Verordnung wording, and it must end up on the label.
#[tokio::test]
async fn biov_offers_wildsammlung_with_the_bio_wording() {
    let c = connect().await;
    goto_config(&c, Config::Bio).await;
    set_sachbezeichnung(&c, "Bärlauchpesto").await;

    add_full_ingredient(
        &c,
        &RecipeIngredient {
            name: "Bärlauch",
            grams: 150.0,
            origin: Some("CH"),
            bio: BioStatus::BioChWildsammlung,
        },
    )
    .await;
    add_full_ingredient(
        &c,
        &RecipeIngredient {
            name: "Rapsöl",
            grams: 850.0,
            origin: Some("CH"),
            bio: BioStatus::BioCh,
        },
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(700)).await;

    assert_label_contains(
        &c,
        "aus biologisch zertifizierter Wildsammlung",
        "biov / wildsammlung legend",
    )
    .await;

    let _ = c.close().await;
}

// DEC-9: the food DB already knows Dicarbonat/Salz/Wasser are non-agricultural.
// The quality must be preselected AND locked, like the allergen checkbox.
#[tokio::test]
async fn db_non_agricultural_ingredient_locks_the_quality() {
    let c = connect().await;
    goto_config(&c, Config::Knospe).await;

    assert!(open_add_ingredient(&c).await, "could not open the ingredient dialog");
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;

    // Type a DB ingredient that is flagged non-agricultural.
    if let Some(input) = first_accent_input(&c).await {
        let _ = input.click().await;
        let _ = input.send_keys("Dicarbonat").await;
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
        let _ = input.send_keys("\u{E007}").await; // Enter
    }
    tokio::time::sleep(std::time::Duration::from_millis(800)).await;

    let state = c
        .execute(
            r##"
            const radios = Array.from(document.querySelectorAll("dialog[open] input[name='bio_category']"));
            const rows = radios.map(r => {
                const label = r.closest('label');
                return {
                    text: label ? label.innerText.trim() : '',
                    checked: r.checked,
                    disabled: r.disabled,
                };
            });
            return JSON.stringify(rows);
            "##,
            vec![],
        )
        .await
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    assert!(
        state.contains("Nicht-landwirtschaftlich"),
        "quality options not rendered: {}",
        state
    );
    // Every option must be locked, and the non-agricultural one preselected.
    assert!(
        !state.contains("\"disabled\":false"),
        "quality radios must be disabled for a DB non-agricultural ingredient: {}",
        state
    );
    let checked_non_agri = c
        .execute(
            r##"
            const radios = Array.from(document.querySelectorAll("dialog[open] input[name='bio_category']"));
            const checked = radios.find(r => r.checked);
            if (!checked) return false;
            const label = checked.closest('label');
            return !!label && /Nicht-landwirtschaftlich/.test(label.innerText);
            "##,
            vec![],
        )
        .await
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    assert!(
        checked_non_agri,
        "«Nicht-landwirtschaftlich» must be preselected: {}",
        state
    );

    let _ = c.close().await;
}

// The counterpart: a free-text ingredient is not in the DB, so the choice must
// stay editable.
#[tokio::test]
async fn custom_ingredient_keeps_the_quality_editable() {
    let c = connect().await;
    goto_config(&c, Config::Knospe).await;

    assert!(open_add_ingredient(&c).await, "could not open the ingredient dialog");
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;

    if let Some(input) = first_accent_input(&c).await {
        let _ = input.click().await;
        let _ = input.send_keys("Grossmutters Geheimzutat").await;
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
        let _ = input.send_keys("\u{E007}").await;
    }
    tokio::time::sleep(std::time::Duration::from_millis(800)).await;

    let any_enabled = c
        .execute(
            r##"
            const radios = Array.from(document.querySelectorAll("dialog[open] input[name='bio_category']"));
            return radios.length > 0 && radios.every(r => !r.disabled);
            "##,
            vec![],
        )
        .await
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    assert!(any_enabled, "a free-text ingredient must keep its quality editable");

    let _ = c.close().await;
}

// DEC-5: in Bio-V the qualities must form one contiguous radio group whose
// order never changes, with dependent fields below it — like Knospe. This
// asserts the actual DOM order before and after each selection, which is the
// only way to catch "the options jump around".
#[tokio::test]
async fn biov_quality_options_keep_their_order_on_every_selection() {
    let c = connect().await;
    goto_config(&c, Config::Bio).await;

    assert!(open_add_ingredient(&c).await, "could not open the ingredient dialog");
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;

    if let Some(input) = first_accent_input(&c).await {
        let _ = input.click().await;
        let _ = input.send_keys("Hafer").await;
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
        let _ = input.send_keys("\u{E007}").await;
    }
    tokio::time::sleep(std::time::Duration::from_millis(700)).await;

    // Document order of the quality labels.
    let order_script = r##"
        const radios = Array.from(document.querySelectorAll("dialog[open] input[name='bio_v_category']"));
        return radios.map(r => {
            const l = r.closest('label');
            return l ? l.innerText.trim() : '';
        }).join('|');
        "##;

    let baseline = c
        .execute(order_script, vec![])
        .await
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();
    assert!(
        baseline.matches('|').count() == 2,
        "expected exactly three Bio-V qualities, got: {}",
        baseline
    );

    // Select each quality in turn; the order must never change.
    for label in ["Bio", "Nicht-biologisch", "Nicht-landwirtschaftlich"] {
        let xpath = format!(
            "//dialog[@open]//label[normalize-space(.)='{}']//input[@name='bio_v_category']",
            label
        );
        if let Ok(el) = c.find(Locator::XPath(&xpath)).await {
            let _ = el.click().await;
            tokio::time::sleep(std::time::Duration::from_millis(400)).await;
        }
        let now = c
            .execute(order_script, vec![])
            .await
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();
        assert_eq!(
            now, baseline,
            "quality order changed after selecting «{}»",
            label
        );
    }

    // Dependent fields must sit BELOW the whole group: with «Bio» selected, the
    // Umstellbetrieb checkbox must come after the last quality radio.
    let xpath = "//dialog[@open]//label[normalize-space(.)='Bio']//input[@name='bio_v_category']";
    if let Ok(el) = c.find(Locator::XPath(xpath)).await {
        let _ = el.click().await;
        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
    }
    let dependent_is_below = c
        .execute(
            r##"
            const radios = Array.from(document.querySelectorAll("dialog[open] input[name='bio_v_category']"));
            const last = radios[radios.length - 1];
            const labels = Array.from(document.querySelectorAll('dialog[open] label'));
            const dep = labels.find(l => /Umstellbetrieb/.test(l.innerText));
            if (!last || !dep) return false;
            // DOCUMENT_POSITION_FOLLOWING === 4
            return (last.compareDocumentPosition(dep) & 4) !== 0;
            "##,
            vec![],
        )
        .await
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    assert!(
        dependent_is_below,
        "the Umstellbetrieb checkbox must appear below the whole quality group"
    );

    let _ = c.close().await;
}

// DEC-10: a Knospe-eligible recipe must show « Bio» after the Sachbezeichnung,
// as Bio-V already does.
#[tokio::test]
async fn knospe_recipe_appends_bio_to_the_sachbezeichnung() {
    let c = connect().await;
    goto_config(&c, Config::Knospe).await;
    set_sachbezeichnung(&c, "Konfitüre").await;

    add_full_ingredient(
        &c,
        &RecipeIngredient {
            name: "Himbeeren",
            grams: 600.0,
            origin: Some("CH"),
            bio: BioStatus::BioKnospe,
        },
    )
    .await;
    add_full_ingredient(
        &c,
        &RecipeIngredient {
            name: "Zucker",
            grams: 400.0,
            origin: Some("CH"),
            bio: BioStatus::BioKnospe,
        },
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(700)).await;

    assert_label_contains(&c, "Konfitüre Bio", "knospe / bio suffix").await;

    let _ = c.close().await;
}
