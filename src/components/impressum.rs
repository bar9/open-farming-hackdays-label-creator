use crate::shared::externalize_links;
use dioxus::prelude::*;
use markdown::to_html;
use rust_i18n::t;

#[component]
pub fn Impressum() -> Element {
    let text = t!("impressum");

    // Die Datenschutzerklärung liegt bei PrivacyBee und wird als iframe
    // eingebettet. Die Sprache folgt der UI-Sprache statt fest auf Deutsch zu
    // stehen: sonst liest ein französisch- oder italienischsprachiger Nutzer
    // die Erklärung auf Deutsch, also genau das Dokument, das er verstehen
    // müsste. Liefert PrivacyBee eine Sprache nicht aus, fällt der Dienst
    // seinerseits auf seine Vorgabe zurück; die Seite bleibt in jedem Fall
    // funktionsfähig.
    let privacy_lang = match rust_i18n::locale().as_ref() {
        "fr-CH" => "fr",
        "it-CH" => "it",
        _ => "de",
    };
    let privacy_src = format!(
        "https://app.privacybee.io/v/clldi3nqp2313020rttms8eh7y?lang={privacy_lang}&type=dsg"
    );

    rsx! {
        div { class: "prose m-auto pt-4 px-4 sm:px-6 lg:px-8", dangerous_inner_html: externalize_links(&to_html(&text)) }
        div { class: "prose m-auto",
            iframe {
                src: "{privacy_src}",
                style: "width: 100%; height: 1000px; border: 0"
            }
        }
    }
}
