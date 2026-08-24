// Verifies the Wildsammlung checkbox sits inside the quality block, next to
// the other checkboxes, instead of alone at the bottom of the modal.
mod common;

use common::recipes::Config;
use common::*;
use fantoccini::Locator;
use std::time::Duration;

/// Order of the checkbox/radio labels inside the open dialog, top to bottom.
async fn dialog_control_labels(c: &fantoccini::Client) -> Vec<String> {
    let els = c
        .find_all(Locator::XPath(
            "//dialog[@open]//label[.//input[@type='checkbox' or @type='radio']]",
        ))
        .await
        .unwrap();
    let mut out = Vec::new();
    for el in els {
        if let Ok(t) = el.text().await {
            let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
            if !t.is_empty() {
                out.push(t);
            }
        }
    }
    out
}

/// Vertical gap in px between the Wildsammlung control and the one above it.
async fn wildsammlung_gap(c: &fantoccini::Client) -> f64 {
    c.execute(
        r#"
        const labels = [...document.querySelectorAll("dialog[open] label")]
          .filter(l => l.querySelector("input[type=checkbox], input[type=radio]"));
        const idx = labels.findIndex(l => l.textContent.includes("Wildsammlung"));
        if (idx <= 0) return -1;
        const prev = labels[idx - 1].getBoundingClientRect();
        const wild = labels[idx].getBoundingClientRect();
        return wild.top - prev.bottom;
        "#,
        vec![],
    )
    .await
    .unwrap()
    .as_f64()
    .unwrap()
}

/// Open the add-ingredient dialog and commit a name, leaving the dialog open
/// on the quality section. Mirrors the first half of `add_full_ingredient`.
async fn open_dialog_with_name(c: &fantoccini::Client, name: &str) {
    assert!(open_add_ingredient(c).await, "add-ingredient dialog");
    tokio::time::sleep(Duration::from_millis(400)).await;
    let input = first_accent_input(c).await.expect("name input");
    input.click().await.unwrap();
    input.send_keys(name).await.unwrap();
    tokio::time::sleep(Duration::from_millis(500)).await;
    input.send_keys("\u{E007}").await.unwrap(); // Enter commits the name
    tokio::time::sleep(Duration::from_millis(400)).await;
}

#[tokio::test]
async fn biov_wildsammlung_is_grouped_with_the_other_checkboxes() {
    let c = connect().await;
    goto_config(&c, Config::Bio).await;
    set_sachbezeichnung(&c, "Bärlauchpesto").await;

    open_dialog_with_name(&c, "Bärlauch").await;
    // Pick the «Bio» quality, which is what reveals Wildsammlung.
    let radio = c
        .find(Locator::XPath(
            "//dialog[@open]//label[normalize-space(.)='Bio']",
        ))
        .await
        .unwrap();
    radio.click().await.unwrap();
    tokio::time::sleep(Duration::from_millis(400)).await;

    let labels = dialog_control_labels(&c).await;
    let wild = labels
        .iter()
        .position(|l| l.contains("Wildsammlung"))
        .unwrap_or_else(|| panic!("no Wildsammlung control; labels: {:?}", labels));
    let umstell = labels
        .iter()
        .position(|l| l.contains("Umstellbetrieb"))
        .unwrap_or_else(|| panic!("no Umstellbetrieb control; labels: {:?}", labels));

    // Grouped means: directly after Umstellbetrieb, and last in the group.
    assert_eq!(
        wild,
        umstell + 1,
        "Wildsammlung should follow Umstellbetrieb directly; labels: {:?}",
        labels
    );

    // And visually adjacent rather than pushed to the bottom of the modal.
    let gap = wildsammlung_gap(&c).await;
    assert!(
        (0.0..40.0).contains(&gap),
        "Wildsammlung should sit next to the previous checkbox, gap was {}px",
        gap
    );

    let _ = c.close().await;
}

#[tokio::test]
async fn knospe_wildsammlung_is_grouped_with_the_quality_block() {
    let c = connect().await;
    goto_config(&c, Config::Knospe).await;
    set_sachbezeichnung(&c, "Bärlauchpesto").await;

    open_dialog_with_name(&c, "Bärlauch").await;
    let radio = c
        .find(Locator::XPath(
            "//dialog[@open]//label[contains(normalize-space(.), 'Bio (Knospe)')]",
        ))
        .await
        .unwrap();
    radio.click().await.unwrap();
    tokio::time::sleep(Duration::from_millis(400)).await;

    let labels = dialog_control_labels(&c).await;
    assert!(
        labels.iter().any(|l| l.contains("Wildsammlung")),
        "Knospe quality should offer Wildsammlung; labels: {:?}",
        labels
    );

    // It must render above the Herkunft field of the same block, i.e. inside
    // the quality block rather than after all the other sections.
    let above_herkunft = c
        .execute(
            r#"
            const wild = [...document.querySelectorAll("dialog[open] label")]
              .find(l => l.textContent.includes("Wildsammlung"));
            const herkunft = [...document.querySelectorAll("dialog[open] *")]
              .find(e => e.children.length === 0 && e.textContent.trim() === "Herkunft");
            if (!wild || !herkunft) return false;
            return wild.getBoundingClientRect().top < herkunft.getBoundingClientRect().top;
            "#,
            vec![],
        )
        .await
        .unwrap();
    assert_eq!(
        above_herkunft.as_bool(),
        Some(true),
        "Wildsammlung should sit inside the Knospe quality block, above Herkunft"
    );

    let _ = c.close().await;
}
