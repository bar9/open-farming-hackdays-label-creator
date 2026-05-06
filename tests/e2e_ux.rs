// UX layout assertions via fantoccini.
//
// These tests are not about Rust panics — they enforce structural UX rules:
// modals shouldn't have scrollbars when the viewport has room to spare,
// fixed-size containers shouldn't crop their content, etc.
//
// Run sequentially: `cargo test --test e2e_ux -- --test-threads=1`

mod common;

use common::*;
use fantoccini::Locator;
use std::time::Duration;

const DESKTOP_W: u32 = 1920;
const DESKTOP_H: u32 = 1080;
const TALL_W: u32 = 1280;
const TALL_H: u32 = 1600;
const LAPTOP_W: u32 = 1280;
const LAPTOP_H: u32 = 800;

#[tokio::test]
async fn genesis_modal_sized_well_desktop() {
    let c = connect().await;
    set_window_size(&c, DESKTOP_W, DESKTOP_H).await;
    goto(&c, "lebensmittelrecht").await;

    open_add_ingredient(&c).await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let issues = check_modal_sizing(&c).await;
    c.close().await.ok();

    assert!(
        issues.is_empty(),
        "genesis modal at {DESKTOP_W}x{DESKTOP_H}:\n  - {}",
        issues.join("\n  - ")
    );
}

#[tokio::test]
async fn genesis_modal_sized_well_tall_viewport() {
    let c = connect().await;
    set_window_size(&c, TALL_W, TALL_H).await;
    goto(&c, "lebensmittelrecht").await;

    open_add_ingredient(&c).await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let issues = check_modal_sizing(&c).await;
    c.close().await.ok();

    assert!(
        issues.is_empty(),
        "genesis modal at {TALL_W}x{TALL_H} (tall viewport):\n  - {}",
        issues.join("\n  - ")
    );
}

#[tokio::test]
async fn link_share_modal_sized_well() {
    let c = connect().await;
    set_window_size(&c, DESKTOP_W, DESKTOP_H).await;
    goto(&c, "lebensmittelrecht").await;
    set_product_title(&c, "ShareSizingTest").await;
    add_simple_ingredient(&c, "Mehl", 100).await;

    click_button_by_text(&c, "Link kopieren").await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let issues = check_modal_sizing(&c).await;
    c.close().await.ok();

    assert!(
        issues.is_empty(),
        "link share modal:\n  - {}",
        issues.join("\n  - ")
    );
}

#[tokio::test]
async fn saved_ingredients_manager_sized_well() {
    let c = connect().await;
    set_window_size(&c, DESKTOP_W, DESKTOP_H).await;
    goto(&c, "lebensmittelrecht").await;

    // Seed several entries to make the table have content
    let json = r#"[
        {"ingredient": {"name": "Bouillon-A", "is_allergen": false, "amount": 9.0, "is_agricultural": true,
            "children": [
                {"name": "Salz", "is_allergen": false, "amount": 5.0, "is_agricultural": false, "origins": ["CH"]},
                {"name": "Pfeffer", "is_allergen": false, "amount": 4.0, "is_agricultural": true, "origins": ["DE"]}
            ]}},
        {"ingredient": {"name": "Bouillon-B", "is_allergen": false, "amount": 9.0, "is_agricultural": true,
            "children": [
                {"name": "Tomate", "is_allergen": false, "amount": 5.0, "is_agricultural": true, "origins": ["IT"]}
            ]}},
        {"ingredient": {"name": "Bouillon-C", "is_allergen": false, "amount": 9.0, "is_agricultural": true,
            "children": [
                {"name": "Kräuter", "is_allergen": false, "amount": 3.0, "is_agricultural": true, "origins": ["FR"]}
            ]}}
    ]"#;
    seed_saved_ingredient_json(&c, json).await;
    reload(&c).await;

    click_button_by_text(&c, "Gespeicherte Zutaten").await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let issues = check_modal_sizing(&c).await;
    c.close().await.ok();

    assert!(
        issues.is_empty(),
        "saved-ingredients manager:\n  - {}",
        issues.join("\n  - ")
    );
}

#[tokio::test]
async fn genesis_modal_sized_well_laptop() {
    let c = connect().await;
    set_window_size(&c, LAPTOP_W, LAPTOP_H).await;
    goto(&c, "lebensmittelrecht").await;

    open_add_ingredient(&c).await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let issues = check_modal_sizing(&c).await;
    c.close().await.ok();

    assert!(
        issues.is_empty(),
        "genesis modal at {LAPTOP_W}x{LAPTOP_H} (laptop):\n  - {}",
        issues.join("\n  - ")
    );
}

#[tokio::test]
async fn edit_ingredient_modal_sized_well() {
    let c = connect().await;
    set_window_size(&c, DESKTOP_W, DESKTOP_H).await;
    goto(&c, "lebensmittelrecht").await;
    set_product_title(&c, "EditSizingTest").await;
    add_simple_ingredient(&c, "Mehl", 100).await;

    // Click the first edit button (icon-only, inside a table row)
    if let Ok(btn) = c
        .find(Locator::XPath(
            "//table//button[.//svg] | //tbody//button[.//svg]",
        ))
        .await
    {
        let _ = btn.click().await;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    let issues = check_modal_sizing(&c).await;
    c.close().await.ok();

    assert!(
        issues.is_empty(),
        "edit ingredient modal:\n  - {}",
        issues.join("\n  - ")
    );
}
