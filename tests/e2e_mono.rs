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
