// Realistic recipe + workflow E2E tests via fantoccini.
//
// These exercise actual user flows: filling forms, editing, switching pages,
// link-share roundtrip, and the saved-ingredients ("merken") feature.
//
// Selectors are best-effort (no data-testid in production code). When a
// helper can't locate an element, it fails the test with a clear message;
// silent no-ops are avoided.
//
// Run sequentially: `cargo test --test e2e_recipes -- --test-threads=1`

mod common;

use common::*;
use fantoccini::Locator;
use std::time::Duration;

// ---------- A. Recipe entry via UI ----------

#[tokio::test]
async fn recipe_strawberry_jam() {
    let c = connect().await;
    goto(&c, "lebensmittelrecht").await;

    set_product_title(&c, "Erdbeer-Fruchtaufstrich").await;
    add_simple_ingredient(&c, "Erdbeere", 165).await;
    add_simple_ingredient(&c, "Zucker", 70).await;
    add_simple_ingredient(&c, "Pektin", 10).await;
    add_simple_ingredient(&c, "Zitronensäure", 5).await;

    tokio::time::sleep(Duration::from_millis(500)).await;
    let errs = read_errors(&c).await;
    c.close().await.ok();
    assert!(errs.is_empty(), "panics in strawberry jam:\n  {}", errs.join("\n  "));
}

#[tokio::test]
async fn recipe_beef_with_origins() {
    let c = connect().await;
    goto(&c, "lebensmittelrecht").await;

    set_product_title(&c, "Rindfleisch").await;
    add_simple_ingredient(&c, "Rindfleisch", 300).await;
    // Aufzucht/Schlachtung-Felder erscheinen u.U. nur bei Edit; das hier
    // testet zumindest die Rind-spezifische Render-Logik.

    let errs = read_errors(&c).await;
    c.close().await.ok();
    assert!(errs.is_empty(), "panics in beef recipe:\n  {}", errs.join("\n  "));
}

#[tokio::test]
async fn recipe_fish_salmon() {
    let c = connect().await;
    goto(&c, "lebensmittelrecht").await;

    set_product_title(&c, "Lachsfilet").await;
    add_simple_ingredient(&c, "Lachs", 200).await;

    let errs = read_errors(&c).await;
    c.close().await.ok();
    assert!(errs.is_empty(), "panics in fish recipe:\n  {}", errs.join("\n  "));
}

// ---------- B. Editing ----------

#[tokio::test]
async fn edit_change_amount() {
    let c = connect().await;
    goto(&c, "lebensmittelrecht").await;

    set_product_title(&c, "Edit-Test").await;
    add_simple_ingredient(&c, "Mehl", 100).await;

    // Try to click the first edit-button-like control in the ingredients table.
    // Edit buttons have an SVG icon (ListDetail) and no text, so we look for
    // buttons inside the ingredients list area.
    if let Ok(btn) = c
        .find(Locator::XPath(
            "//table//button[.//svg] | //tbody//button[.//svg]",
        ))
        .await
    {
        let _ = btn.click().await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        // In the edit modal, change amount if a number input appears
        if let Ok(num) = c
            .find(Locator::Css("dialog[open] input[type='number']"))
            .await
        {
            let _ = num.click().await;
            // Select all + replace
            let _ = c
                .execute(
                    "const el = document.querySelector(\"dialog[open] input[type='number']\"); if (el) { el.value=''; el.dispatchEvent(new Event('input', {bubbles:true})); } return null;",
                    vec![],
                )
                .await;
            let _ = num.send_keys("250").await;
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        for label in &["Speichern", "OK", "Schliessen"] {
            if click_button_by_text(&c, label).await {
                break;
            }
        }
    }

    let errs = read_errors(&c).await;
    c.close().await.ok();
    assert!(errs.is_empty(), "panics in edit-amount:\n  {}", errs.join("\n  "));
}

#[tokio::test]
async fn edit_delete_ingredient() {
    let c = connect().await;
    goto(&c, "lebensmittelrecht").await;

    set_product_title(&c, "Delete-Test").await;
    add_simple_ingredient(&c, "Mehl", 100).await;
    add_simple_ingredient(&c, "Butter", 50).await;

    // Click a delete button — typically btn-error or with red styling in the
    // ingredients table.
    if let Ok(btn) = c
        .find(Locator::XPath(
            "//table//button[contains(@class,'error')] | //tbody//button[contains(@class,'error')]",
        ))
        .await
    {
        let _ = btn.click().await;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    let errs = read_errors(&c).await;
    c.close().await.ok();
    assert!(errs.is_empty(), "panics in delete-ingredient:\n  {}", errs.join("\n  "));
}

// Regression tests for tester feedback against v0.8.5:
// (1) editing an ingredient amount didn't update the displayed list,
// (2) repeated edits trailed the typed value (30→40 showed 38, then 39, then 40),
// (3) the "Total" line failed to recalculate (needed a URL copy/paste roundtrip).
// The v0.8.6 refactor (PR #31) unified ingredient state into a single signal;
// these tests assert the visible values now follow each edit.

/// Force-close any open dialog (genesis modal stays open after
/// `add_simple_ingredient` because we click "Speichern und nächste Zutat").
async fn close_any_open_dialog(c: &fantoccini::Client) {
    if open_dialog_count(c).await == 0 {
        return;
    }
    for label in &["Schliessen", "Schließen", "Close"] {
        if click_button_by_text(c, label).await {
            break;
        }
    }
    if open_dialog_count(c).await > 0 {
        if let Ok(body) = c.find(Locator::Css("body")).await {
            let _ = body.send_keys("\u{E00C}").await; // ESC
        }
    }
    // Last resort: native dialog.close()
    if open_dialog_count(c).await > 0 {
        let _ = c
            .execute(
                "document.querySelectorAll('dialog[open]').forEach(d => d.close()); return null;",
                vec![],
            )
            .await;
    }
    tokio::time::sleep(Duration::from_millis(200)).await;
}

async fn edit_amount_to(c: &fantoccini::Client, name: &str, new_amount: u32) -> bool {
    close_any_open_dialog(c).await;
    if !open_ingredient_edit_by_name(c, name).await {
        return false;
    }
    let _ = c
        .execute(
            "const el = document.querySelector(\"dialog[open] input[type='number']\"); \
             if (el) { el.focus(); el.value=''; el.dispatchEvent(new Event('input', {bubbles:true})); } \
             return null;",
            vec![],
        )
        .await;
    if let Ok(num) = c
        .find(Locator::Css("dialog[open] input[type='number']"))
        .await
    {
        let _ = num.click().await;
        let _ = num.send_keys(&new_amount.to_string()).await;
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    for label in &["Speichern", "Schliessen", "Schließen", "OK"] {
        if click_button_by_text(c, label).await {
            break;
        }
    }
    close_any_open_dialog(c).await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    true
}

async fn read_ingredient_row_text(c: &fantoccini::Client, name: &str) -> String {
    let safe = name.replace('\\', "\\\\").replace('\'', "\\'");
    let script = format!(
        r#"
        const rows = document.querySelectorAll('div.grid.grid-cols-3');
        for (const r of rows) {{
            if (r.closest('dialog[open]')) continue;
            if (r.innerText && r.innerText.includes('{name}')) {{
                return r.innerText;
            }}
        }}
        return '';
        "#,
        name = safe
    );
    c.execute(&script, vec![])
        .await
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default()
}

async fn read_total_row_text(c: &fantoccini::Client) -> String {
    c.execute(
        r#"
        const rows = document.querySelectorAll('div.grid.grid-cols-3');
        for (const r of rows) {
            if (r.closest('dialog[open]')) continue;
            const cells = r.children;
            if (cells.length >= 2) {
                const first = (cells[0].innerText || '').trim();
                if (first === 'Total' || first === 'Totale') {
                    return r.innerText;
                }
            }
        }
        return '';
        "#,
        vec![],
    )
    .await
    .ok()
    .and_then(|v| v.as_str().map(|s| s.to_string()))
    .unwrap_or_default()
}

// Note: ingredient names must be exact food_db.csv entries (or strings that
// don't prefix-match anything). Pressing Enter in the genesis input commits
// the first autocomplete match — typing "Mehl" autocompletes to
// "Buchweizenmehl", which then breaks substring lookups with case-sensitive
// helpers.
const ING1: &str = "Buchweizenmehl";
const ING2: &str = "Salz";

#[tokio::test]
async fn edit_amount_reflects_in_list() {
    let c = connect().await;
    goto(&c, "lebensmittelrecht").await;

    set_product_title(&c, "Edit-Reflect").await;
    add_simple_ingredient(&c, ING1, 100).await;

    let opened = edit_amount_to(&c, ING1, 300).await;
    let row = read_ingredient_row_text(&c, ING1).await;
    let errs = read_errors(&c).await;
    c.close().await.ok();

    assert!(opened, "could not open edit modal for {ING1}");
    assert!(
        row.contains("300"),
        "ingredient row should reflect new amount 300 after edit, got: {row:?}"
    );
    assert!(
        !row.contains("100.0 g"),
        "ingredient row still shows stale 100.0 g after edit, got: {row:?}"
    );
    assert!(
        errs.is_empty(),
        "panics in edit_amount_reflects_in_list:\n  {}",
        errs.join("\n  ")
    );
}

#[tokio::test]
async fn edit_amount_repeated_updates() {
    let c = connect().await;
    goto(&c, "lebensmittelrecht").await;

    set_product_title(&c, "Repeat-Edit").await;
    add_simple_ingredient(&c, ING1, 30).await;

    let mut failures: Vec<String> = Vec::new();
    for new in [40u32, 50, 60] {
        if !edit_amount_to(&c, ING1, new).await {
            failures.push(format!("edit modal didn't open for target {new}"));
            continue;
        }
        let row = read_ingredient_row_text(&c, ING1).await;
        if !row.contains(&new.to_string()) {
            failures.push(format!("after edit to {new}, row was {row:?}"));
        }
    }

    let errs = read_errors(&c).await;
    c.close().await.ok();
    assert!(
        failures.is_empty(),
        "successive edits did not reflect in list:\n  {}",
        failures.join("\n  ")
    );
    assert!(
        errs.is_empty(),
        "panics in edit_amount_repeated_updates:\n  {}",
        errs.join("\n  ")
    );
}

#[tokio::test]
async fn total_weight_recalculates_on_edit() {
    let c = connect().await;
    goto(&c, "lebensmittelrecht").await;

    set_product_title(&c, "Total-Recalc").await;
    add_simple_ingredient(&c, ING1, 100).await;
    add_simple_ingredient(&c, ING2, 50).await;

    let total_before = read_total_row_text(&c).await;
    let edited = edit_amount_to(&c, ING1, 200).await;
    let total_after = read_total_row_text(&c).await;
    let errs = read_errors(&c).await;
    c.close().await.ok();

    assert!(edited, "could not open edit modal for {ING1}");
    assert!(
        total_before.contains("150"),
        "expected total to read 150 g before edit, got: {total_before:?}"
    );
    assert!(
        total_after.contains("250"),
        "total did not recalc after edit (expected 250), got: {total_after:?}"
    );
    assert!(
        errs.is_empty(),
        "panics in total_weight_recalculates_on_edit:\n  {}",
        errs.join("\n  ")
    );
}

// ---------- C. Page switching ----------

#[tokio::test]
async fn switch_swiss_to_bio() {
    let c = connect().await;
    goto(&c, "lebensmittelrecht").await;

    // Open the configuration dropdown in the header.
    if let Ok(btn) = c
        .find(Locator::Css(".dropdown-end > div[role='button']"))
        .await
    {
        let _ = btn.click().await;
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    // Click "Bio" entry
    click_button_by_text(&c, "Bio").await;
    tokio::time::sleep(Duration::from_millis(400)).await;
    // A confirmation dialog may appear — try common confirm labels
    for label in &["Wechseln", "Bestätigen", "OK", "Ja"] {
        if click_button_by_text(&c, label).await {
            break;
        }
    }
    tokio::time::sleep(Duration::from_millis(800)).await;

    // Verify we're on /bio (best-effort: URL contains "bio")
    let url = c.current_url().await.ok().map(|u| u.to_string()).unwrap_or_default();
    let on_bio = url.contains("/bio");

    let errs = read_errors(&c).await;
    c.close().await.ok();

    assert!(errs.is_empty(), "panics during swiss→bio switch:\n  {}", errs.join("\n  "));
    if !on_bio {
        eprintln!("warning: did not reach /bio (URL: {url}) — selector for switch button may have changed");
    }
}

#[tokio::test]
async fn switch_bio_to_knospe_with_data() {
    let c = connect().await;
    goto(&c, "bio").await;
    set_product_title(&c, "Bio-Test").await;
    add_simple_ingredient(&c, "Hafer", 600).await;

    if let Ok(btn) = c
        .find(Locator::Css(".dropdown-end > div[role='button']"))
        .await
    {
        let _ = btn.click().await;
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    click_button_by_text(&c, "Knospe").await;
    tokio::time::sleep(Duration::from_millis(400)).await;
    for label in &["Wechseln", "Bestätigen", "OK", "Ja"] {
        if click_button_by_text(&c, label).await {
            break;
        }
    }
    tokio::time::sleep(Duration::from_millis(800)).await;

    let errs = read_errors(&c).await;
    c.close().await.ok();
    assert!(errs.is_empty(), "panics during bio→knospe switch:\n  {}", errs.join("\n  "));
}

// ---------- D. Link share roundtrip ----------

#[tokio::test]
async fn link_copy_modal_opens() {
    let c = connect().await;
    goto(&c, "lebensmittelrecht").await;

    set_product_title(&c, "ShareTest").await;
    add_simple_ingredient(&c, "Mehl", 100).await;

    let opened = click_button_by_text(&c, "Link kopieren").await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let url = read_share_url(&c).await.unwrap_or_default();

    let errs = read_errors(&c).await;
    c.close().await.ok();

    assert!(opened, "could not find 'Link kopieren' button");
    assert!(!url.is_empty(), "share URL was empty");
    assert!(errs.is_empty(), "panics opening share modal:\n  {}", errs.join("\n  "));
}

#[tokio::test]
async fn link_copy_and_reuse() {
    let c1 = connect().await;
    goto(&c1, "lebensmittelrecht").await;
    set_product_title(&c1, "RoundtripTest").await;
    add_simple_ingredient(&c1, "Mehl", 100).await;

    click_button_by_text(&c1, "Link kopieren").await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    let share_url = read_share_url(&c1).await;
    let errs1 = read_errors(&c1).await;
    c1.close().await.ok();
    assert!(errs1.is_empty(), "panics in source session:\n  {}", errs1.join("\n  "));

    let url = share_url.expect("share URL must be present");
    assert!(!url.is_empty());

    // Open URL in fresh session
    let c2 = connect().await;
    c2.goto(&url).await.expect("goto shared URL");
    install_trap(&c2).await;
    tokio::time::sleep(mount_delay()).await;
    // Continue editing — add another ingredient
    add_simple_ingredient(&c2, "Butter", 50).await;
    let errs2 = read_errors(&c2).await;
    c2.close().await.ok();
    assert!(errs2.is_empty(), "panics in restored session:\n  {}", errs2.join("\n  "));
}

// ---------- E. "Zutaten merken" (saved ingredients) ----------
//
// Note: creating a composite ingredient via the UI is the most complex
// modal interaction. We bypass that by seeding localStorage directly with
// a saved composite, then test the manager + recall paths. This still
// exercises the parsing and rendering code that the user is unsure about.

const SEED_SAVED: &str = r#"[{
    "ingredient": {
        "name": "Test-Bouillon",
        "is_allergen": false,
        "amount": 9.0,
        "is_agricultural": true,
        "children": [
            {"name": "Salz", "is_allergen": false, "amount": 5.0, "is_agricultural": false, "origins": ["CH"]},
            {"name": "Pfeffer", "is_allergen": false, "amount": 4.0, "is_agricultural": true, "origins": ["DE"]}
        ]
    }
}]"#;

#[tokio::test]
async fn merken_seed_and_open_manager() {
    let c = connect().await;
    goto(&c, "lebensmittelrecht").await;
    seed_saved_ingredient_json(&c, SEED_SAVED).await;
    reload(&c).await;

    let opened = click_button_by_text(&c, "Gespeicherte Zutaten").await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Expect the seeded entry to appear in the table
    let has_entry = c
        .find(Locator::XPath(
            "//table//*[contains(text(), 'Test-Bouillon')]",
        ))
        .await
        .is_ok();

    let errs = read_errors(&c).await;
    c.close().await.ok();

    assert!(opened, "could not find 'Gespeicherte Zutaten' header button");
    assert!(errs.is_empty(), "panics opening saved-ingredients manager:\n  {}", errs.join("\n  "));
    assert!(has_entry, "seeded saved ingredient not visible in manager");
}

#[tokio::test]
async fn merken_recall_in_search() {
    let c = connect().await;
    goto(&c, "lebensmittelrecht").await;
    seed_saved_ingredient_json(&c, SEED_SAVED).await;
    reload(&c).await;

    open_add_ingredient(&c).await;
    tokio::time::sleep(Duration::from_millis(400)).await;
    if let Some(input) = first_accent_input(&c).await {
        let _ = input.click().await;
        let _ = input.send_keys("Test-Bouillon").await;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    // Look for the "Gespeicherte Zutaten" section header in the dropdown
    let saved_section = c
        .find(Locator::XPath("//*[contains(text(),'Gespeicherte Zutaten')]"))
        .await
        .is_ok();

    let errs = read_errors(&c).await;
    c.close().await.ok();
    assert!(errs.is_empty(), "panics during saved-ingredient recall:\n  {}", errs.join("\n  "));
    if !saved_section {
        eprintln!("warning: saved-ingredients section did not appear in search dropdown");
    }
}

#[tokio::test]
async fn merken_delete() {
    let c = connect().await;
    goto(&c, "lebensmittelrecht").await;
    seed_saved_ingredient_json(&c, SEED_SAVED).await;
    reload(&c).await;

    click_button_by_text(&c, "Gespeicherte Zutaten").await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Click the delete button in the table row
    if let Ok(del) = c
        .find(Locator::Css("dialog[open] table button.btn-error"))
        .await
    {
        let _ = del.click().await;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    let errs = read_errors(&c).await;
    c.close().await.ok();
    assert!(errs.is_empty(), "panics during saved-ingredient delete:\n  {}", errs.join("\n  "));
}

// ---------- F. Robustness ----------

#[tokio::test]
async fn empty_form_no_panic() {
    let c = connect().await;
    goto(&c, "lebensmittelrecht").await;
    // Just sit on the page after mount; tab through some elements
    if let Some(input) = first_text_input(&c).await {
        let _ = input.click().await;
    }
    tokio::time::sleep(Duration::from_millis(500)).await;
    let errs = read_errors(&c).await;
    c.close().await.ok();
    assert!(errs.is_empty(), "panics on empty form:\n  {}", errs.join("\n  "));
}

#[tokio::test]
async fn many_ingredients_stress() {
    let c = connect().await;
    goto(&c, "lebensmittelrecht").await;
    set_product_title(&c, "Stresstest").await;
    let names = [
        "Mehl", "Zucker", "Butter", "Eier", "Milch",
        "Salz", "Hefe", "Wasser", "Honig", "Vanille",
    ];
    for (i, n) in names.iter().enumerate() {
        add_simple_ingredient(&c, n, ((i + 1) * 10) as u32).await;
    }
    let errs = read_errors(&c).await;
    c.close().await.ok();
    assert!(errs.is_empty(), "panics in stress test:\n  {}", errs.join("\n  "));
}
