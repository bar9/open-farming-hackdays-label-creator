// DEC-15: «Namensgebende Zutat» must keep its place when an ingredient is
// turned into a composite. It used to be rendered after both the leaf and the
// composite branch, so in composite mode the derived Allergen and Herkunft
// fields pushed it to the bottom of the dialog.
//
// Run serially against a served build + chromedriver:
// `cargo test --test e2e_namensgebend_layout -- --test-threads=1`.
mod common;

use common::recipes::Config;
use common::*;
use std::time::Duration;

/// Vertical position of a field label inside the open dialog, or -1.
async fn field_y(c: &fantoccini::Client, needle: &str) -> f64 {
    c.execute(
        &format!(
            r#"
            const dlg = document.querySelector("dialog[open]");
            if (!dlg) return -1;
            for (const el of dlg.querySelectorAll("label, .label-text")) {{
              const t = el.textContent.trim();
              if (t.length < 60 && t.includes({needle})) {{
                const r = el.getBoundingClientRect();
                if (r.height > 0) return Math.round(r.top);
              }}
            }}
            return -1;
            "#,
            needle = serde_json::to_string(needle).unwrap()
        ),
        vec![],
    )
    .await
    .unwrap()
    .as_f64()
    .unwrap_or(-1.0)
}

async fn open_named_ingredient(c: &fantoccini::Client, name: &str) {
    assert!(open_add_ingredient(c).await, "ingredient dialog");
    tokio::time::sleep(Duration::from_millis(400)).await;
    let input = first_accent_input(c).await.expect("name input");
    input.click().await.unwrap();
    input.send_keys(name).await.unwrap();
    tokio::time::sleep(Duration::from_millis(700)).await;
    input.send_keys("\u{E007}").await.unwrap();
    tokio::time::sleep(Duration::from_millis(600)).await;
}

async fn enable_composite(c: &fantoccini::Client) {
    let r = c
        .execute(
            r#"
            const dlg = document.querySelector("dialog[open]");
            const hit = [...dlg.querySelectorAll("label")]
              .find(l => /Zusammengesetzte Zutat/.test(l.textContent));
            const box = hit && hit.querySelector("input[type=checkbox]");
            if (!box) return false;
            box.click();
            return true;
            "#,
            vec![],
        )
        .await
        .unwrap();
    assert_eq!(r.as_bool(), Some(true), "composite toggle");
    tokio::time::sleep(Duration::from_millis(900)).await;
}

#[tokio::test]
async fn namensgebend_stays_above_allergen_and_herkunft_in_composite_mode() {
    let c = connect().await;
    goto_config(&c, Config::Lebensmittelrecht).await;
    open_named_ingredient(&c, "Testzutat").await;

    // Leaf mode: the field sits above Herkunft.
    let leaf_namensgebend = field_y(&c, "Namensgebende Zutat").await;
    let leaf_herkunft = field_y(&c, "Herkunft").await;
    assert!(
        leaf_namensgebend > 0.0,
        "premise: the name-giving field should be visible in leaf mode"
    );
    assert!(
        leaf_namensgebend < leaf_herkunft,
        "leaf mode: Namensgebend ({leaf_namensgebend}) should sit above Herkunft ({leaf_herkunft})"
    );

    enable_composite(&c).await;

    let namensgebend = field_y(&c, "Namensgebende Zutat").await;
    let allergen = field_y(&c, "Allergen").await;
    let herkunft = field_y(&c, "Herkunft").await;
    assert!(
        namensgebend > 0.0,
        "the name-giving field should still be visible for a composite"
    );
    assert!(
        namensgebend < allergen,
        "composite: Namensgebend ({namensgebend}) should stay above Allergen ({allergen})"
    );
    assert!(
        namensgebend < herkunft,
        "composite: Namensgebend ({namensgebend}) should stay above Herkunft ({herkunft})"
    );

    assert_no_errors(&c, "namensgebend layout").await;
    let _ = c.close().await;
}

// Rendering it in two places must not produce two checkboxes.
#[tokio::test]
async fn namensgebend_renders_exactly_once_per_mode() {
    let c = connect().await;
    goto_config(&c, Config::Lebensmittelrecht).await;
    open_named_ingredient(&c, "Testzutat").await;

    let count_script = r#"
        const dlg = document.querySelector("dialog[open]");
        return [...dlg.querySelectorAll("label")]
          .filter(l => /Namensgebende Zutat/.test(l.textContent)
                       && l.querySelector("input[type=checkbox]")).length;
        "#;

    let leaf = c.execute(count_script, vec![]).await.unwrap();
    assert_eq!(
        leaf.as_i64(),
        Some(1),
        "leaf mode should render the name-giving checkbox exactly once"
    );

    enable_composite(&c).await;

    let composite = c.execute(count_script, vec![]).await.unwrap();
    assert_eq!(
        composite.as_i64(),
        Some(1),
        "composite mode should render the name-giving checkbox exactly once"
    );

    let _ = c.close().await;
}
