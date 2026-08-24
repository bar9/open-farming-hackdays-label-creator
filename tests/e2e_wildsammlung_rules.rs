// DEC-16: two narrowings of existing behaviour.
//
// 1. «Alle landwirtschaftlichen Zutaten stammen aus biologischer Landwirtschaft»
//    only makes sense when the recipe actually has agricultural ingredients. A
//    product made purely from certified wild collection has none.
// 2. The 10% °-marking for wild collection is a Bio-Suisse rule; the Bio-V
//    prints the step inline instead (its own wording stays, DEC-11).
//
// Run serially against a served build + chromedriver:
// `cargo test --test e2e_wildsammlung_rules -- --test-threads=1`.
mod common;

use common::recipes::{BioStatus, Config, RecipeIngredient};
use common::*;
use std::time::Duration;

#[tokio::test]
async fn biov_wild_collection_is_not_marked_with_a_degree_sign() {
    let c = connect().await;
    goto_config(&c, Config::Bio).await;
    set_sachbezeichnung(&c, "Bärlauchpesto").await;

    // 150 g of 1000 g is 15%, i.e. above the old 10% threshold.
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
    tokio::time::sleep(Duration::from_millis(800)).await;

    let label = label_html(&c).await;
    assert!(
        label.contains("aus biologisch zertifizierter Wildsammlung"),
        "the Bio-V wording must still appear; label: {}",
        label
    );
    assert!(
        !label.contains('°'),
        "the 10% ° marking is Knospe-only (DEC-16); label: {}",
        label
    );

    assert_no_errors(&c, "biov wild collection").await;
    let _ = c.close().await;
}

#[tokio::test]
async fn knospe_keeps_the_degree_marking_for_wild_collection() {
    let c = connect().await;
    goto_config(&c, Config::Knospe).await;
    set_sachbezeichnung(&c, "Bärlauchpesto").await;

    add_full_ingredient(
        &c,
        &RecipeIngredient {
            name: "Bärlauch",
            grams: 150.0,
            origin: Some("CH"),
            bio: BioStatus::BioKnospe,
        },
    )
    .await;
    add_full_ingredient(
        &c,
        &RecipeIngredient {
            name: "Rapsöl",
            grams: 850.0,
            origin: Some("CH"),
            bio: BioStatus::BioKnospe,
        },
    )
    .await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Tick wild collection on the first ingredient.
    assert!(
        open_ingredient_edit_by_name(&c, "Bärlauch").await,
        "could not reopen the Bärlauch card"
    );
    tokio::time::sleep(Duration::from_millis(500)).await;
    if let Ok(el) = c
        .find(fantoccini::Locator::XPath(
            "//dialog[@open]//label[contains(normalize-space(.), 'Wildsammlung')]//input[@type='checkbox']",
        ))
        .await
    {
        let _ = el.click().await;
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    let _ = click_button_by_text(&c, "Speichern").await;
    tokio::time::sleep(Duration::from_millis(800)).await;

    let label = label_html(&c).await;
    assert!(
        label.contains('°'),
        "Knospe keeps the 10% ° marking; label: {}",
        label
    );

    let _ = c.close().await;
}
