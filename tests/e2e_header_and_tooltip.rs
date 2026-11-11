// DEC-17 / DEC-18 (Bio Suisse testing feedback, September 2026).
//
// DEC-18: the header shows the full BIO SUISSE logo (as on the splash card)
//         for the Knospe configuration, not just the bud.
// DEC-17: the hint on the disabled «Speichern» button («Menge fehlt») must be
//         readable, i.e. not cut off at the right edge of the dialog.
//
// Run serially against `make dev` + chromedriver:
// `cargo test --test e2e_header_and_tooltip -- --test-threads=1`.
mod common;

use common::recipes::Config;
use common::*;
use std::time::Duration;

#[tokio::test]
async fn knospe_header_shows_the_full_bio_suisse_logo() {
    let c = connect().await;
    goto_config(&c, Config::Knospe).await;

    // The organisation logo is the only SVG with this viewBox that carries the
    // lettering; the header must contain it, outside the label preview.
    let found: bool = c
        .execute(
            r#"
            const svgs = [...document.querySelectorAll('svg')]
              .filter(s => (s.getAttribute('viewBox') || '').startsWith('-2.27216241')
                           && !s.closest('div.bg-white.rounded-lg.shadow-lg')
                           && !s.closest('dialog'));
            return svgs.some(s => {
              const r = s.getBoundingClientRect();
              return r.top < 120 && r.height > 0;
            });
            "#,
            vec![],
        )
        .await
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    assert!(
        found,
        "expected the BIO SUISSE logo in the header on /knospe"
    );

    assert_no_errors(&c, "knospe_header_shows_the_full_bio_suisse_logo").await;
    let _ = c.close().await;
}

#[tokio::test]
async fn disabled_save_hint_stays_inside_the_dialog() {
    let c = connect().await;
    goto_config(&c, Config::Lebensmittelrecht).await;
    assert!(open_add_ingredient(&c).await, "ingredient dialog");
    tokio::time::sleep(Duration::from_millis(400)).await;

    // A name without amount: the save button is disabled and carries the hint.
    let input = first_accent_input(&c).await.expect("name input");
    input.click().await.unwrap();
    input.send_keys("Testzutat").await.unwrap();
    tokio::time::sleep(Duration::from_millis(700)).await;
    input.send_keys("\u{E007}").await.unwrap();
    tokio::time::sleep(Duration::from_millis(600)).await;

    // daisyUI renders the tooltip from `data-tip` as a ::before box whose size
    // is computed even while hidden; compare that box with the dialog box.
    c.find(fantoccini::Locator::Css(
        "dialog[open] span.tooltip[data-tip]",
    ))
    .await
    .expect("disabled save button with a tooltip");

    let geometry: serde_json::Value = c
        .execute(
            r#"
            const t = document.querySelector('dialog[open] span.tooltip[data-tip]');
            const box = document.querySelector('dialog[open] .modal-box') || document.querySelector('dialog[open]');
            const tr = t.getBoundingClientRect();
            const br = box.getBoundingClientRect();
            const cs = getComputedStyle(t, '::before');
            const w = parseFloat(cs.width) || 0;
            // tooltip-end: the bubble is anchored at the wrapper's right edge
            // and grows to the left.
            const right = tr.right;
            const left = right - w;
            return { text: t.getAttribute('data-tip'), left, right,
                     boxLeft: br.left, boxRight: br.right, width: w,
                     cls: t.className };
            "#,
            vec![],
        )
        .await
        .unwrap();
    let left = geometry["left"].as_f64().unwrap_or(-1.0);
    let right = geometry["right"].as_f64().unwrap_or(-1.0);
    let box_left = geometry["boxLeft"].as_f64().unwrap_or(0.0);
    let box_right = geometry["boxRight"].as_f64().unwrap_or(0.0);
    assert!(
        geometry["text"].as_str().is_some_and(|s| !s.is_empty()),
        "the disabled save button must explain what is missing: {geometry}"
    );
    assert!(
        geometry["cls"]
            .as_str()
            .is_some_and(|s| s.contains("tooltip-end")),
        "the hint must grow to the left (tooltip-end): {geometry}"
    );
    assert!(
        geometry["width"].as_f64().unwrap_or(0.0) > 0.0,
        "tooltip box not measurable: {geometry}"
    );
    assert!(
        left >= box_left - 1.0 && right <= box_right + 1.0,
        "the hint is cut off by the dialog: {geometry}"
    );

    let _ = c.close().await;
}
