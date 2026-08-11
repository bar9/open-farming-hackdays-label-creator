// Verifies that user-visible strings come from the locale files rather than
// hardcoded German. The country dropdowns are the interesting case: they used
// to hardcode ~250 German country names, so switching the UI language left them
// German while everything else translated.
//
// Prerequisites: `make dev` on :8080 and chromedriver/geckodriver on :4444.

mod common;

use common::*;
use fantoccini::Locator;
use std::time::Duration;

/// Set the stored locale, then load a route so the app boots in that language.
async fn goto_with_locale(c: &fantoccini::Client, locale: &str, path: &str) {
    goto(c, path).await;
    c.execute(
        "localStorage.setItem('locale', arguments[0]);",
        vec![serde_json::json!(locale)],
    )
    .await
    .expect("could not set locale");
    goto(c, path).await;
    tokio::time::sleep(mount_delay()).await;
}

/// All option texts of every `select` on the page.
async fn option_texts(c: &fantoccini::Client) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(options) = c.find_all(Locator::Css("option")).await {
        for o in options {
            if let Ok(t) = o.text().await {
                out.push(t.trim().to_string());
            }
        }
    }
    out
}

/// Open the ingredient modal, which is where the origin country select lives.
/// The button label is translated, so all three variants are tried.
async fn open_ingredient_modal(c: &fantoccini::Client) {
    accept_disclaimer(c).await;
    for label in [
        "Zutat hinzufügen",
        "Ajouter un ingrédient",
        "Aggiungi ingrediente",
    ] {
        if click_button_by_text(c, label).await {
            break;
        }
    }
    tokio::time::sleep(Duration::from_millis(1200)).await;
}

#[tokio::test]
async fn country_dropdown_is_translated_per_locale() {
    let c = connect().await;

    // German baseline: the German names must still be there.
    goto_with_locale(&c, "de-CH", "lebensmittelrecht").await;
    open_ingredient_modal(&c).await;
    let de = option_texts(&c).await;
    assert!(
        de.iter().any(|t| t == "Deutschland"),
        "de-CH should list 'Deutschland', got a sample of {:?}",
        &de.iter().take(20).collect::<Vec<_>>()
    );

    // French: the same dropdown must show French names, not German ones.
    goto_with_locale(&c, "fr-CH", "lebensmittelrecht").await;
    open_ingredient_modal(&c).await;
    let fr = option_texts(&c).await;
    assert!(
        fr.iter().any(|t| t == "Allemagne"),
        "fr-CH should list 'Allemagne', got a sample of {:?}",
        &fr.iter().take(20).collect::<Vec<_>>()
    );
    assert!(
        !fr.iter().any(|t| t == "Deutschland"),
        "fr-CH still shows the German country name 'Deutschland'"
    );

    // Italian.
    goto_with_locale(&c, "it-CH", "lebensmittelrecht").await;
    open_ingredient_modal(&c).await;
    let it = option_texts(&c).await;
    assert!(
        it.iter().any(|t| t == "Germania"),
        "it-CH should list 'Germania', got a sample of {:?}",
        &it.iter().take(20).collect::<Vec<_>>()
    );
    assert!(
        !it.iter().any(|t| t == "Deutschland"),
        "it-CH still shows the German country name 'Deutschland'"
    );

    // Leave the browser on the default language for following tests.
    goto_with_locale(&c, "de-CH", "lebensmittelrecht").await;
    assert_no_errors(&c, "country dropdown locales").await;
    let _ = c.close().await;
}

/// No option in any locale may render a raw translation key (`countries.XX`),
/// which is what rust-i18n emits for a missing entry.
#[tokio::test]
async fn dropdowns_never_render_raw_translation_keys() {
    let c = connect().await;

    for locale in ["de-CH", "fr-CH", "it-CH"] {
        goto_with_locale(&c, locale, "lebensmittelrecht").await;
        open_ingredient_modal(&c).await;
        let texts = option_texts(&c).await;
        let leaked: Vec<&String> = texts
            .iter()
            .filter(|t| t.contains("countries.") || t.contains("origin.") || t.contains("label."))
            .collect();
        assert!(
            leaked.is_empty(),
            "{locale} leaked raw translation keys: {leaked:?}"
        );
        assert!(
            texts.len() > 100,
            "{locale} rendered only {} options, expected the full country list",
            texts.len()
        );
    }

    goto_with_locale(&c, "de-CH", "lebensmittelrecht").await;
    let _ = c.close().await;
}
