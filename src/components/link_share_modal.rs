use crate::services::label_text;
use crate::services::qr_code;
use crate::services::url_shortener;
use crate::services::url_shortener::Provider;
use dioxus::prelude::*;
use rust_i18n::t;

#[derive(Clone, Copy, PartialEq)]
pub enum LinkType {
    Full,
    Short,
}

#[component]
pub fn LinkShareModal(show: Signal<bool>, url: String) -> Element {
    // Der vollständige Link ist die Vorgabe (R-18, Entscheid 11.11.2026). Er
    // entsteht im Browser und schickt nichts an einen Server. Der Kurz-Link
    // speichert die ganze Rezeptur samt Adresse auf declarino.ch; das passiert
    // erst, wenn die Nutzerin ihn ausdrücklich wählt. Vorher (0.8.20) wurde
    // schon beim Öffnen gekürzt, was der Aussage «keine Speicherung in einer
    // Cloud» widersprach, die Testerinnen gerade gelobt hatten.
    let mut link_type = use_signal(|| LinkType::Full);
    let mut is_copying = use_signal(|| false);
    let mut copy_success = use_signal(|| false);
    let mut short_url = use_signal(|| None::<String>);
    let mut is_shortening = use_signal(|| false);
    let mut shorten_error = use_signal(|| None::<String>);
    // Welcher Dienst den Link erzeugt hat. Normalerweise declarino.ch; fällt
    // der aus (oder lehnt das Ziel ab, etwa bei localhost), springt ein
    // Fremddienst ein — dann muss der Hinweis das auch sagen, statt weiter
    // declarino.ch zu versprechen.
    let mut short_provider = use_signal(|| None::<Provider>);
    // Rückmeldung zum QR-Knopf (R-13): kopiert, heruntergeladen oder Fehler.
    let mut qr_feedback = use_signal(|| None::<qr_code::QrCopyOutcome>);

    // Der volle Link als Signal, damit die Closures unten kopierbar bleiben
    // (ein `String` liesse sich nur einmal in eine `move`-Closure ziehen).
    //
    // Nachgefuehrt, nicht nur einmal gesetzt: Das Fenster haengt im Layout und
    // entsteht schon beim Start der Anwendung, zu einem Zeitpunkt also, an dem
    // das Formular noch leer ist. `use_signal` merkt sich aber allein den
    // ersten Wert. Der Link blieb dadurch fuer immer der des leeren Formulars,
    // und wer teilte, verschickte eine leere Etikette.
    let mut full_url = use_signal(|| url.clone());
    use_effect(use_reactive!(|url| {
        if full_url.peek().as_str() != url.as_str() {
            full_url.set(url.clone());
            // Der Kurz-Link zeigte auf die vorige Fassung; er muss neu geholt
            // werden, statt weiter auf veraltete Daten zu verweisen.
            short_url.set(None);
            short_provider.set(None);
            shorten_error.set(None);
        }
    }));

    // Was im Eingabefeld steht und kopiert wird.
    //
    // Solange der Kurz-Link noch unterwegs ist, steht der volle Link im Feld,
    // statt gar keines zu zeigen. Vorher war das Modal in dieser Zeitspanne
    // (und dauerhaft, wenn der Dienst das Ziel ablehnt, etwa bei localhost)
    // ohne jedes kopierbare Feld: der Nutzer wollte teilen und stand vor einem
    // Kasten ohne Link. Sobald der Kurz-Link da ist, ersetzt er ihn.
    let displayed_url = move || match link_type() {
        LinkType::Full => Some(full_url()),
        LinkType::Short => short_url().or_else(|| Some(full_url())),
    };

    let start_shortening = move || {
        // Aus dem Signal, nicht aus der beim Bauen eingefangenen Fassung:
        // sonst kuerzt der Dienst weiterhin den Link des leeren Formulars.
        let url_to_shorten = full_url();
        spawn(async move {
            is_shortening.set(true);
            shorten_error.set(None);

            // Anbieter-Kette (siehe services::url_shortener): der eigene
            // Endpunkt zuerst, Fremddienste nur als Rückfallebene.
            match url_shortener::shorten(&url_to_shorten).await {
                Ok(link) => {
                    short_url.set(Some(link.url));
                    short_provider.set(Some(link.provider));
                    shorten_error.set(None);
                }
                Err(e) => {
                    tracing::error!("Failed to shorten URL: {}", e);
                    shorten_error.set(Some(t!("link_shorten_error").to_string()));
                    // Beim Scheitern den vollen Link zeigen, statt ein leeres
                    // Feld: der Nutzer wollte teilen, nicht kürzen.
                    link_type.set(LinkType::Full);
                }
            }

            is_shortening.set(false);
        });
    };

    // Gekürzt wird nur, wenn der Kurz-Link gewählt ist, nie schon beim Öffnen.
    // Ändert sich die Rezeptur bei gewähltem Kurz-Link, wird neu gekürzt.
    use_effect(move || {
        if show()
            && link_type() == LinkType::Short
            && short_url().is_none()
            && !is_shortening()
            && shorten_error().is_none()
        {
            start_shortening();
        }
    });

    let copy_to_clipboard = move |_| {
        let Some(url_to_copy) = displayed_url() else {
            return;
        };

        spawn(async move {
            is_copying.set(true);
            copy_success.set(false);

            // Gemeinsam mit dem Knopf unter der Etikette (services::label_text).
            let success = label_text::copy_to_clipboard(&url_to_copy);

            if success {
                copy_success.set(true);
                gloo::timers::future::TimeoutFuture::new(2000).await;
                copy_success.set(false);
            }

            is_copying.set(false);
        });
    };

    // Beim Wechsel der Auswahl die Erfolgsmeldung zurücksetzen, sonst stünde
    // "kopiert" neben einem Link, der gar nicht kopiert wurde.
    use_effect(move || {
        let _ = link_type();
        copy_success.set(false);
    });

    rsx! {
        dialog {
            class: "modal",
            open: show(),
            div {
                class: "modal-box w-11/12 max-w-2xl",
                h3 {
                    class: "font-bold text-lg mb-4",
                    {t!("link_share_title").to_string()}
                }

                div {
                    class: "form-control mb-4",
                    div {
                        class: "flex gap-4 mb-4",
                        label {
                            class: "label cursor-pointer flex items-center gap-2",
                            input {
                                r#type: "radio",
                                name: "link-type",
                                class: "radio radio-primary",
                                checked: link_type() == LinkType::Full,
                                onchange: move |_| {
                                    link_type.set(LinkType::Full);
                                }
                            }
                            span { class: "label-text", {t!("link_type_full").to_string()} }
                        }
                        label {
                            class: "label cursor-pointer flex items-center gap-2",
                            input {
                                r#type: "radio",
                                name: "link-type",
                                class: "radio radio-primary",
                                checked: link_type() == LinkType::Short,
                                onchange: move |_| {
                                    // Ausdrückliche Wahl: ein früherer Fehlschlag
                                    // soll einen neuen Versuch nicht blockieren.
                                    shorten_error.set(None);
                                    link_type.set(LinkType::Short);
                                }
                            }
                            span { class: "label-text", {t!("link_type_short").to_string()} }
                        }
                    }

                    if link_type() == LinkType::Full {
                        div {
                            class: "text-sm text-base-content/70 mb-4",
                            {t!("link_full_disclaimer").to_string()}
                        }
                    }

                    if link_type() == LinkType::Short {
                        div {
                            class: "text-sm text-base-content/70 mb-4",
                            match short_provider() {
                                // Regelfall: eigener Dienst.
                                Some(Provider::Declarino) | None => {
                                    t!("link_short_disclaimer").to_string()
                                }
                                // Rückfallebene: transparent machen, wo der
                                // Link nun liegt.
                                Some(other) => {
                                    t!("link_short_disclaimer_provider", provider = other.host())
                                        .to_string()
                                }
                            }
                        }
                    }

                    if is_shortening() {
                        div {
                            class: "flex items-center gap-2 text-sm text-base-content/70 mb-4",
                            span { class: "loading loading-spinner loading-sm" }
                            {t!("shortening").to_string()}
                        }
                    }

                    if let Some(error_msg) = shorten_error() {
                        div {
                            class: "alert alert-warning mb-4",
                            svg {
                                class: "w-6 h-6 shrink-0 stroke-current",
                                fill: "none",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                                }
                            }
                            span { {error_msg} }
                        }
                    }

                    if let Some(current_url) = displayed_url() {
                        div {
                            class: "flex gap-2",
                            input {
                                r#type: "text",
                                class: "input input-bordered flex-1",
                                value: current_url.clone(),
                                readonly: true,
                                disabled: is_shortening(),
                            }
                            button {
                                class: "btn btn-primary",
                                disabled: is_copying() || is_shortening(),
                                onclick: copy_to_clipboard,
                                if is_copying() {
                                    span { class: "loading loading-spinner" }
                                } else if copy_success() {
                                    svg {
                                        class: "w-6 h-6",
                                        fill: "none",
                                        stroke: "currentColor",
                                        stroke_width: "2",
                                        view_box: "0 0 24 24",
                                        path {
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            d: "M5 13l4 4L19 7"
                                        }
                                    }
                                } else {
                                    svg {
                                        class: "w-6 h-6",
                                        fill: "none",
                                        stroke: "currentColor",
                                        stroke_width: "2",
                                        view_box: "0 0 24 24",
                                        path {
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            d: "M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
                                        }
                                    }
                                }
                                // Der Knopf oben heisst jetzt «Teilen», also
                                // muss der eigentliche Kopiervorgang hier
                                // benannt sein. Vorher stand hier nur ein
                                // Symbol, dessen Bedeutung man raten musste.
                                span { class: "ml-2", {t!("link_copy_button").to_string()} }
                            }

                            if copy_success() {
                                div {
                                    class: "text-success text-sm mt-2",
                                    {t!("link_copied_success").to_string()}
                                }
                            }
                        }

                        // QR-Code: holt den Link auf ein Handy, ohne ihn
                        // abzutippen. Fuer einen vollstaendigen Link mit ganzer
                        // Rezeptur kann er zu lang werden, dann bleibt das Feld
                        // oben der Weg (siehe services::qr_code).
                        if link_type() == LinkType::Full && qr_code::svg_path(&current_url).is_none() {
                            // Der vollständige Link trägt die ganze Rezeptur
                            // und passt oft nicht in einen QR-Code. Dann sagen,
                            // wie man doch zu einem kommt, statt still keinen
                            // zu zeigen.
                            div {
                                class: "mt-6 text-sm text-base-content/70 text-center",
                                {t!("link_qr_needs_short").to_string()}
                            }
                        } else if let Some(qr) = qr_code::svg_path(&current_url) {
                            div {
                                class: "mt-6 flex flex-col items-center gap-2",
                                div {
                                    class: "text-sm font-medium",
                                    {t!("link_qr_title").to_string()}
                                }
                                svg {
                                    class: "w-48 h-48 bg-white p-2 rounded border border-base-300",
                                    view_box: "0 0 {qr.size} {qr.size}",
                                    role: "img",
                                    "aria-label": t!("link_qr_alt").to_string(),
                                    "shape-rendering": "crispEdges",
                                    path { fill: "#ffffff", d: "M0,0h{qr.size}v{qr.size}h-{qr.size}z" }
                                    path { fill: "#000000", d: "{qr.path}" }
                                }
                                div {
                                    class: "text-xs text-base-content/70 text-center",
                                    {t!("link_qr_hint").to_string()}
                                }
                                // R-13: Bild kopieren oder herunterladen, ohne
                                // Snipping Tool (Livetesting 09.11.2026).
                                div {
                                    class: "flex gap-2 mt-1",
                                    button {
                                        r#type: "button",
                                        class: "btn btn-sm btn-outline",
                                        onclick: {
                                            let svg = qr.standalone_svg(8);
                                            move |_| {
                                                let svg = svg.clone();
                                                spawn(async move {
                                                    let outcome = qr_code::copy_png(&svg, "declarino-qr-code.png").await;
                                                    qr_feedback.set(Some(outcome));
                                                    gloo::timers::future::TimeoutFuture::new(3000).await;
                                                    qr_feedback.set(None);
                                                });
                                            }
                                        },
                                        {t!("link_qr_copy").to_string()}
                                    }
                                    button {
                                        r#type: "button",
                                        class: "btn btn-sm btn-outline",
                                        onclick: {
                                            let svg = qr.standalone_svg(8);
                                            move |_| {
                                                let svg = svg.clone();
                                                spawn(async move {
                                                    let outcome = qr_code::download_png(&svg, "declarino-qr-code.png").await;
                                                    qr_feedback.set(Some(outcome));
                                                    gloo::timers::future::TimeoutFuture::new(3000).await;
                                                    qr_feedback.set(None);
                                                });
                                            }
                                        },
                                        {t!("link_qr_download").to_string()}
                                    }
                                }
                                match qr_feedback() {
                                    Some(qr_code::QrCopyOutcome::Copied) => rsx! {
                                        div { class: "text-success text-sm", {t!("link_qr_copied").to_string()} }
                                    },
                                    Some(qr_code::QrCopyOutcome::Downloaded) => rsx! {
                                        div { class: "text-success text-sm", {t!("link_qr_downloaded").to_string()} }
                                    },
                                    Some(qr_code::QrCopyOutcome::Failed) => rsx! {
                                        div { class: "text-warning text-sm", {t!("link_qr_failed").to_string()} }
                                    },
                                    None => rsx! {},
                                }
                            }
                        }
                    }
                }

                div {
                    class: "modal-action",
                    button {
                        class: "btn",
                        onclick: move |_| show.set(false),
                        {t!("close").to_string()}
                    }
                }
            }
            form {
                method: "dialog",
                class: "modal-backdrop",
                onclick: move |_| show.set(false),
                button { {t!("buttons.close").to_string()} }
            }
        }
    }
}
