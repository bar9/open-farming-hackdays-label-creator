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

/// The FAQ page is reachable from the footer, renders its content from the
/// locale files, and follows the Impressum layout.
#[tokio::test]
async fn faq_page_renders_from_the_locales_in_every_language() {
    let c = connect().await;

    let expected = [
        ("de-CH", "Häufige Fragen", "Ersetzt Declarino eine rechtliche Beratung?"),
        ("fr-CH", "Questions fréquentes", "Declarino remplace-t-il un conseil juridique ?"),
        ("it-CH", "Domande frequenti", "Declarino sostituisce una consulenza legale?"),
    ];

    for (locale, title, first_question) in expected {
        goto_with_locale(&c, locale, "faq").await;
        let body = c
            .find(Locator::Css("body"))
            .await
            .expect("body")
            .text()
            .await
            .expect("body text");

        assert!(body.contains(title), "{locale} FAQ is missing the title {title:?}");
        assert!(
            body.contains(first_question),
            "{locale} FAQ is missing its first question {first_question:?}"
        );
        // Both example answers must be present, so questions and answers stay paired.
        assert!(
            body.matches("Declarino").count() >= 2,
            "{locale} FAQ seems to render only part of the entries"
        );
        assert!(
            !body.contains("faq.") && !body.contains("nav."),
            "{locale} FAQ leaked a raw translation key"
        );
    }

    assert_no_errors(&c, "faq page").await;
    goto_with_locale(&c, "de-CH", "faq").await;
    let _ = c.close().await;
}

/// The footer link must exist on both footers (splash screen and the app
/// layout) and point at the FAQ route.
#[tokio::test]
async fn footer_links_to_the_faq_page() {
    let c = connect().await;

    for route in ["", "lebensmittelrecht"] {
        goto_with_locale(&c, "de-CH", route).await;
        // Links rendered by the router are <a href>; read them via JS so both
        // the Link component and plain anchors are covered.
        let found = c
            .execute(
                "return Array.from(document.querySelectorAll('footer a')).some(a => (a.getAttribute('href')||'').endsWith('/faq'));",
                vec![],
            )
            .await
            .expect("query footer links");
        assert_eq!(
            found.as_bool(),
            Some(true),
            "route {route:?} has no footer link to /faq"
        );
    }

    // Clicking it actually navigates.
    goto_with_locale(&c, "de-CH", "").await;
    c.execute(
        "Array.from(document.querySelectorAll('footer a')).find(a => (a.getAttribute('href')||'').endsWith('/faq')).click();",
        vec![],
    )
    .await
    .expect("click faq link");
    tokio::time::sleep(Duration::from_millis(1500)).await;
    let url = c.current_url().await.expect("current url");
    assert!(
        url.as_str().ends_with("/faq"),
        "clicking the footer link did not open the FAQ, landed on {url}"
    );

    let _ = c.close().await;
}

/// The "support us" page renders its translated text and the TWINT QR code in
/// every language. The QR code itself is language independent (the TWINT code
/// PDF ships three identical cut-outs), so only the copy around it changes.
#[tokio::test]
async fn support_page_renders_qr_and_translations_in_every_language() {
    let c = connect().await;

    let expected = [
        ("de-CH", "Unterstütze Declarino", "TWINT"),
        ("fr-CH", "Soutenir Declarino", "TWINT"),
        ("it-CH", "Sostieni Declarino", "TWINT"),
    ];

    for (locale, title, twint) in expected {
        goto_with_locale(&c, locale, "support").await;
        let body = c
            .find(Locator::Css("body"))
            .await
            .expect("body")
            .text()
            .await
            .expect("body text");

        assert!(
            body.contains(title),
            "{locale} support page is missing the title {title:?}"
        );
        assert!(
            body.contains(twint),
            "{locale} support page never mentions {twint}"
        );
        assert!(
            !body.contains("support."),
            "{locale} support page leaked a raw translation key"
        );

        // The QR image must be present, loaded, and linked to the TWINT payment URL.
        let qr_ok = c
            .execute(
                "const i = document.querySelector('img[src*=\"twint-qr\"]');\
                 return !!i && i.complete && i.naturalWidth > 0;",
                vec![],
            )
            .await
            .expect("query qr image");
        assert_eq!(
            qr_ok.as_bool(),
            Some(true),
            "{locale} support page does not show a loaded TWINT QR code"
        );

        let linked = c
            .execute(
                "return Array.from(document.querySelectorAll('a')).some(a => (a.getAttribute('href')||'').includes('twint.ch'));",
                vec![],
            )
            .await
            .expect("query twint link");
        assert_eq!(
            linked.as_bool(),
            Some(true),
            "{locale} support page has no TWINT link fallback"
        );
    }

    assert_no_errors(&c, "support page").await;
    goto_with_locale(&c, "de-CH", "support").await;
    let _ = c.close().await;
}

/// The support link must exist on both footers (splash screen and the app
/// layout) and navigate to the support route.
#[tokio::test]
async fn footer_links_to_the_support_page() {
    let c = connect().await;

    for route in ["", "lebensmittelrecht"] {
        goto_with_locale(&c, "de-CH", route).await;
        let found = c
            .execute(
                "return Array.from(document.querySelectorAll('footer a')).some(a => (a.getAttribute('href')||'').endsWith('/support'));",
                vec![],
            )
            .await
            .expect("query footer links");
        assert_eq!(
            found.as_bool(),
            Some(true),
            "route {route:?} has no footer link to /support"
        );
    }

    goto_with_locale(&c, "de-CH", "").await;
    c.execute(
        "Array.from(document.querySelectorAll('footer a')).find(a => (a.getAttribute('href')||'').endsWith('/support')).click();",
        vec![],
    )
    .await
    .expect("click support link");
    tokio::time::sleep(Duration::from_millis(1500)).await;
    let url = c.current_url().await.expect("current url");
    assert!(
        url.as_str().ends_with("/support"),
        "clicking the footer link did not open the support page, landed on {url}"
    );

    let _ = c.close().await;
}
