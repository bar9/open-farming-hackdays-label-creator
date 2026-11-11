// K1/K2 (11.11.2026): conservative Knospe logo with missing weights, and the
// ingredient overview agrees with the editor modal. Seeds recipes through a
// shared link so the exact tree (weights, origins, qualities) is under test.
//
// Sequentiell: `cargo test --test e2e_conservative_logo -- --test-threads=1`

mod common;

use common::recipes::Config;
use common::*;
use std::time::Duration;

async fn open_shared_knospe(c: &fantoccini::Client, query: &str) {
    goto(c, &format!("knospe?{}", query)).await;
    tokio::time::sleep(mount_delay()).await;
    tokio::time::sleep(Duration::from_millis(2500)).await;
}

fn leaf(prefix: &str, name: &str, grams: f64, origin: &str) -> String {
    format!(
        "&{p}[name]={name}&{p}[is_allergen]=false&{p}[amount]={grams}&{p}[unit]=Gram\
&{p}[is_agricultural]=true&{p}[is_bio]=true&{p}[origins][0]={origin}",
        p = prefix
    )
}

/// Füllung 500 g (CH-Frucht ohne Gewicht, DE-Frucht ohne Gewicht) + Zucker 500 g CH.
/// Before K1 the weightless children dropped out and the label got the cross.
fn weightless_mixed() -> String {
    let mut q = String::from(
        "v=2&ingredients[0][name]=Füllung&ingredients[0][is_allergen]=false\
&ingredients[0][amount]=500.0&ingredients[0][unit]=Gram&ingredients[0][is_agricultural]=true",
    );
    q += &leaf("ingredients[0][children][0]", "Himbeere", 0.0, "CH");
    q += &leaf("ingredients[0][children][1]", "Brombeere", 0.0, "DE");
    q += &leaf("ingredients[1]", "Zucker", 500.0, "CH");
    q += "&rezeptur_vollstaendig=true&certification_body=CH-BIO-006";
    q
}

/// Same, but every child is Swiss: the cross is still earned.
fn weightless_all_ch() -> String {
    let mut q = String::from(
        "v=2&ingredients[0][name]=Füllung&ingredients[0][is_allergen]=false\
&ingredients[0][amount]=500.0&ingredients[0][unit]=Gram&ingredients[0][is_agricultural]=true",
    );
    q += &leaf("ingredients[0][children][0]", "Himbeere", 0.0, "CH");
    q += &leaf("ingredients[0][children][1]", "Brombeere", 0.0, "CH");
    q += &leaf("ingredients[1]", "Zucker", 500.0, "CH");
    q += "&rezeptur_vollstaendig=true&certification_body=CH-BIO-006";
    q
}

async fn any_knospe_logo(c: &fantoccini::Client) -> bool {
    c.execute(
        "return document.querySelectorAll('div.absolute.top-2.right-2 svg').length > 0;",
        vec![],
    )
    .await
    .ok()
    .and_then(|v| v.as_bool())
    .unwrap_or(false)
}

/// The composite row in the ingredient overview: does its Knospe icon carry
/// the Swiss cross? (KnospeCompactCh vs KnospeCompactNoCross, by aria/title.)
async fn overview_icon_alt(c: &fantoccini::Client, name: &str) -> String {
    c.execute(
        r#"
        for (const row of document.querySelectorAll('div.grid.grid-cols-3')) {
            if (!row.textContent.includes(arguments[0])) continue;
            const el = row.querySelector('[aria-label*="Knospe"], img[alt*="Knospe"], svg[aria-label*="Knospe"], [title*="Knospe"]');
            if (el) return el.getAttribute('aria-label') || el.getAttribute('alt') || el.getAttribute('title') || 'icon';
            return 'none';
        }
        return 'row-missing';
        "#,
        vec![serde_json::json!(name)],
    )
    .await
    .ok()
    .and_then(|v| v.as_str().map(|s| s.to_string()))
    .unwrap_or_default()
}

#[tokio::test]
async fn weightless_foreign_child_denies_the_swiss_cross_on_the_label() {
    let c = connect().await;
    goto_config(&c, Config::Knospe).await;
    open_shared_knospe(&c, &weightless_mixed()).await;

    assert!(
        any_knospe_logo(&c).await,
        "premise: the recipe is fully Knospe, a logo shows"
    );
    assert!(
        !has_bio_suisse_cross(&c).await,
        "a weightless DE ingredient must deny the Swiss cross"
    );
    let icon = overview_icon_alt(&c, "Füllung").await;
    assert!(
        !icon.contains("Schweizer"),
        "overview icon of the composite must not carry the cross: {icon}"
    );

    assert_no_errors(&c, "conservative logo").await;
    c.close().await.ok();
}

#[tokio::test]
async fn weightless_all_swiss_children_keep_the_swiss_cross() {
    let c = connect().await;
    goto_config(&c, Config::Knospe).await;
    open_shared_knospe(&c, &weightless_all_ch()).await;

    assert!(
        has_bio_suisse_cross(&c).await,
        "all Swiss, weights or not: the cross is earned"
    );
    let icon = overview_icon_alt(&c, "Füllung").await;
    assert!(
        icon.contains("Schweizer"),
        "overview icon agrees with the label: {icon}"
    );

    assert_no_errors(&c, "all swiss weightless").await;
    c.close().await.ok();
}

/// Modal and overview give the same answer for the composite (K2).
#[tokio::test]
async fn modal_and_overview_agree_on_the_composite_quality() {
    let c = connect().await;
    goto_config(&c, Config::Knospe).await;
    open_shared_knospe(&c, &weightless_mixed()).await;

    assert!(
        open_ingredient_edit_by_name(&c, "Füllung").await,
        "the composite must open in the editor"
    );
    tokio::time::sleep(Duration::from_millis(800)).await;
    let modal = c
        .execute(
            "const d=document.querySelector('dialog[open]'); return d ? d.innerText : '';",
            vec![],
        )
        .await
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();
    assert!(
        modal.contains("Herkunft unbekannt"),
        "mixed CH/DE without weights: the modal must say unknown, not Schweiz/Import: {}",
        &modal.chars().take(600).collect::<String>()
    );
    assert!(!modal.contains("Herkunft Schweiz ·") && !modal.contains("· Herkunft Schweiz"));

    assert_no_errors(&c, "modal vs overview").await;
    c.close().await.ok();
}
