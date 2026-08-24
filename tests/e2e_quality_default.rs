// DEC-14: the «Nicht-biologisch» default must survive «Speichern und nächste
// Zutat».
//
// The food DB marks entries like Salz as non-agricultural and locks the quality
// to «Nicht-landwirtschaftlich» (DEC-9). That choice used to leak: it was never
// released, so the next ingredient started out non-agricultural instead of on
// the «Nicht-biologisch» default.
//
// Run serially against a served build + chromedriver:
// `cargo test --test e2e_quality_default -- --test-threads=1`.
mod common;

use common::recipes::Config;
use common::*;
use fantoccini::Locator;
use std::time::Duration;

/// Label of the checked quality radio in the open dialog.
async fn checked_quality(c: &fantoccini::Client) -> String {
    c.execute(
        r#"
        const labels = [...document.querySelectorAll("dialog[open] label")]
          .filter(l => l.querySelector("input[type=radio]"));
        const hit = labels.find(l => l.querySelector("input[type=radio]").checked);
        return hit ? hit.textContent.trim() : "(none checked)";
        "#,
        vec![],
    )
    .await
    .unwrap()
    .as_str()
    .unwrap_or("?")
    .to_string()
}

/// Type a name into the open dialog and commit it.
async fn enter_name(c: &fantoccini::Client, name: &str) {
    let input = first_accent_input(c).await.expect("name input");
    input.click().await.unwrap();
    input.send_keys(name).await.unwrap();
    tokio::time::sleep(Duration::from_millis(700)).await;
    input.send_keys("\u{E007}").await.unwrap();
    tokio::time::sleep(Duration::from_millis(600)).await;
}

#[tokio::test]
async fn non_agricultural_lock_does_not_leak_into_the_next_ingredient() {
    let c = connect().await;
    goto_config(&c, Config::Knospe).await;

    assert!(open_add_ingredient(&c).await, "ingredient dialog");
    tokio::time::sleep(Duration::from_millis(400)).await;

    // «Salz» is non-agricultural in the food DB, so the DB picks the quality.
    enter_name(&c, "Salz").await;
    assert_eq!(
        checked_quality(&c).await,
        "Nicht-landwirtschaftlich",
        "premise: the food DB should lock Salz to non-agricultural"
    );

    if let Ok(num) = c.find(Locator::Css("dialog[open] input[type='number']")).await {
        num.click().await.unwrap();
        num.send_keys("10").await.unwrap();
    }
    tokio::time::sleep(Duration::from_millis(300)).await;

    assert!(
        click_button_by_text(&c, "Speichern und nächste").await,
        "«Speichern und nächste Zutat» button"
    );
    tokio::time::sleep(Duration::from_millis(900)).await;

    // The next ingredient must start on the default again.
    assert_eq!(
        checked_quality(&c).await,
        "Nicht-biologisch",
        "the DB-driven «Nicht-landwirtschaftlich» must not survive save-and-next"
    );

    assert_no_errors(&c, "quality default after save-and-next").await;
    let _ = c.close().await;
}

#[tokio::test]
async fn renaming_away_from_a_locked_db_entry_restores_the_default() {
    let c = connect().await;
    goto_config(&c, Config::Knospe).await;

    assert!(open_add_ingredient(&c).await, "ingredient dialog");
    tokio::time::sleep(Duration::from_millis(400)).await;

    enter_name(&c, "Salz").await;
    assert_eq!(
        checked_quality(&c).await,
        "Nicht-landwirtschaftlich",
        "premise: Salz is locked by the food DB"
    );

    // Rename to free text: the DB no longer answers the question.
    let input = first_accent_input(&c).await.expect("name input");
    input.click().await.unwrap();
    input.send_keys("\u{E009}a").await.unwrap(); // ctrl+a
    input.send_keys("Freitext Zutat").await.unwrap();
    tokio::time::sleep(Duration::from_millis(900)).await;

    assert_eq!(
        checked_quality(&c).await,
        "Nicht-biologisch",
        "renaming off a locked DB entry should release the DB's choice"
    );

    let _ = c.close().await;
}

// The release must not undo a choice the user made themselves.
#[tokio::test]
async fn a_manual_non_agricultural_choice_is_kept() {
    let c = connect().await;
    goto_config(&c, Config::Knospe).await;

    assert!(open_add_ingredient(&c).await, "ingredient dialog");
    tokio::time::sleep(Duration::from_millis(400)).await;

    // Free-text ingredient: the DB has no opinion, the user picks.
    enter_name(&c, "Freitext Zutat").await;
    let radio = c
        .find(Locator::XPath(
            "//dialog[@open]//label[contains(normalize-space(.), 'Nicht-landwirtschaftlich')]//input[@type='radio']",
        ))
        .await
        .expect("non-agricultural radio");
    radio.click().await.unwrap();
    tokio::time::sleep(Duration::from_millis(500)).await;

    assert_eq!(
        checked_quality(&c).await,
        "Nicht-landwirtschaftlich",
        "a manual choice must stick"
    );

    // Typing more of the same free text must not reset it.
    let input = first_accent_input(&c).await.expect("name input");
    input.click().await.unwrap();
    input.send_keys(" extra").await.unwrap();
    tokio::time::sleep(Duration::from_millis(800)).await;

    assert_eq!(
        checked_quality(&c).await,
        "Nicht-landwirtschaftlich",
        "editing a free-text name must not undo the user's own choice"
    );

    let _ = c.close().await;
}
