// E2E smoke tests via fantoccini.
//
// Prerequisites (run in two separate terminals before `make e2e`):
//   1. `make dev`              — Dioxus dev server on :8080
//   2. `chromedriver --port=4444` (or `geckodriver --port 4444`)
//
// Goal: visit each route, fail if any JS error / unhandled rejection /
// console.error appeared. Rust panics in WASM surface as one of these.

mod common;

use common::*;
use fantoccini::Locator;
use std::time::Duration;

#[tokio::test]
async fn no_panics_on_main_routes() {
    let c = connect().await;

    let routes = ["", "lebensmittelrecht", "bio", "knospe", "impressum"];
    let mut failures: Vec<(String, Vec<String>)> = Vec::new();
    for r in routes {
        goto(&c, r).await;
        let errs = read_errors(&c).await;
        if !errs.is_empty() {
            failures.push((r.to_string(), errs));
        }
    }

    // Bonus: simple interaction smoke on the main form route
    goto(&c, "lebensmittelrecht").await;
    if let Ok(input) = c
        .wait()
        .at_most(Duration::from_secs(3))
        .for_element(Locator::Css("input[type='text'], input:not([type])"))
        .await
    {
        let _ = input.send_keys("Test Produkt").await;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    let interaction_errs = read_errors(&c).await;
    if !interaction_errs.is_empty() {
        failures.push(("lebensmittelrecht (interaction)".into(), interaction_errs));
    }

    c.close().await.ok();

    assert!(
        failures.is_empty(),
        "JS errors detected:\n{}",
        failures
            .iter()
            .map(|(r, e)| format!("  [{}]\n    {}", r, e.join("\n    ")))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
