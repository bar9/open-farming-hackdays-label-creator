use dioxus::prelude::*;
use rust_i18n::t;

/// The TWINT payment link behind the QR code, taken from the TWINT code PDF in
/// `requirements/`. The QR code in that PDF is a print sheet of three identical
/// cut-out copies, so a single code serves every language; only the surrounding
/// text is translated.
const TWINT_URL: &str = "https://www.twint.ch/info-qr-code/?sbsQrCode=02%3A1ed6e732316e43afb1031cd85a1d3d46%23c344143a2fca43e71780599699babd5b42d1d3b0%23";

/// "Support us" page. Layout mirrors the FAQ/Impressum page: a centered `prose`
/// column inside the shared container, with a back link at the bottom.
///
/// The QR code is rendered from an SVG traced off the TWINT code PDF, so it
/// stays crisp at any size. It doubles as a link for anyone reading on the same
/// device they would pay from, where scanning your own screen is impossible.
pub fn Support() -> Element {
    rsx! {
        div { class: "container mx-auto p-8",
            div { class: "max-w-4xl mx-auto",
                div { class: "prose m-auto pt-4 px-4 sm:px-6 lg:px-8",
                    h1 { {t!("support.title").to_string()} }
                    p { {t!("support.intro").to_string()} }
                    p { {t!("support.qr_hint").to_string()} }

                    // The donation card mirrors one cut-out of the TWINT code
                    // PDF: QR code, "Declarino.ch" caption, TWINT logo.
                    div { class: "not-prose flex flex-col items-center gap-4 my-8",
                        a {
                            href: TWINT_URL,
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "flex flex-col items-center gap-3 bg-white p-6 rounded-lg border border-base-300",
                            img {
                                src: asset!("assets/twint-qr.svg"),
                                alt: t!("support.qr_alt").to_string(),
                                class: "w-48 h-48 sm:w-56 sm:h-56",
                            }
                            span { class: "text-sm font-medium text-black", "Declarino.ch" }
                            img {
                                src: asset!("assets/logos/twint.png"),
                                alt: "TWINT",
                                class: "h-10",
                            }
                        }
                        a {
                            href: TWINT_URL,
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "link link-blue hover:link-primary",
                            {t!("support.twint_link").to_string()}
                        }
                    }

                    p { {t!("support.thanks").to_string()} }
                }
                div { class: "mt-8",
                    Link {
                        to: crate::routes::Route::SplashScreen {},
                        class: "btn btn-primary",
                        {t!("nav.zurueck").to_string()}
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TWINT_URL;

    const LOCALES: [(&str, &str); 3] = [
        ("de-CH", include_str!("../../locales/de-CH.yml")),
        ("fr-CH", include_str!("../../locales/fr-CH.yml")),
        ("it-CH", include_str!("../../locales/it-CH.yml")),
    ];

    /// Every language must ship the full set of support keys, otherwise the page
    /// renders raw keys like `support.title` in that language.
    #[test]
    fn every_locale_has_all_support_keys() {
        let keys = [
            "title",
            "intro",
            "qr_hint",
            "qr_alt",
            "twint_link",
            "thanks",
        ];

        for (locale, source) in LOCALES {
            let doc: serde_yaml::Value =
                serde_yaml::from_str(source).unwrap_or_else(|e| panic!("{locale} is not valid YAML: {e}"));

            let support = doc
                .get("support")
                .unwrap_or_else(|| panic!("{locale} has no support section"));

            for key in keys {
                let value = support
                    .get(key)
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| panic!("{locale} is missing support.{key}"));
                assert!(
                    !value.trim().is_empty(),
                    "{locale} has an empty support.{key}"
                );
            }

            // The footer label lives under `app` next to impressum/faq.
            let footer_label = doc
                .get("app")
                .and_then(|app| app.get("support"))
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("{locale} is missing app.support for the footer link"));
            assert!(
                !footer_label.trim().is_empty(),
                "{locale} has an empty app.support footer label"
            );
        }
    }

    /// The QR code asset must stay the code from the TWINT PDF. The SVG is a
    /// traced copy, so this pins the payload it was generated from: if someone
    /// swaps the asset or edits the URL, the two drift apart silently and
    /// donations would go nowhere.
    #[test]
    fn twint_url_points_at_the_declarino_twint_code() {
        assert!(
            TWINT_URL.starts_with("https://www.twint.ch/info-qr-code/?sbsQrCode="),
            "the TWINT link is no longer a TWINT QR-code URL: {TWINT_URL}"
        );
        // The merchant/code identifiers decoded from the PDF.
        assert!(
            TWINT_URL.contains("1ed6e732316e43afb1031cd85a1d3d46"),
            "the TWINT link lost the Declarino code id"
        );
        assert!(
            TWINT_URL.contains("c344143a2fca43e71780599699babd5b42d1d3b0"),
            "the TWINT link lost the Declarino code signature"
        );
    }

    /// The shipped SVG must be a 49x49 module QR code (the version traced from
    /// the PDF) drawn in black on white, so it stays scannable.
    #[test]
    fn qr_asset_is_a_scannable_svg() {
        let svg = include_str!("../../assets/twint-qr.svg");

        assert!(svg.starts_with("<svg"), "the QR asset is not an SVG");
        // 49 modules plus a 2 module quiet zone on each side.
        assert!(
            svg.contains("viewBox=\"0 0 53 53\""),
            "the QR asset no longer has the traced 49x49 module geometry"
        );
        assert!(
            svg.contains("fill=\"#000\"") && svg.contains("fill=\"#fff\""),
            "the QR asset must stay black on white to remain scannable"
        );
        assert!(
            svg.contains("shape-rendering=\"crispEdges\""),
            "the QR asset needs crisp edges so modules do not blur when scaled"
        );
    }
}
