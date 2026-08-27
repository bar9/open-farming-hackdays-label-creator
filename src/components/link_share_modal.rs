use crate::services::url_shortener;
use crate::services::url_shortener::Provider;
use dioxus::prelude::*;
use rust_i18n::t;
use wasm_bindgen::JsCast;
use web_sys::{js_sys, window, HtmlTextAreaElement};

#[derive(Clone, Copy, PartialEq)]
pub enum LinkType {
    Full,
    Short,
}

#[component]
pub fn LinkShareModal(show: Signal<bool>, url: String) -> Element {
    // Kurz-Link ist die Vorgabe: Seit der Dienst unter declarino.ch läuft, ist
    // er schnell, zuverlässig und speichert nichts bei Dritten. Der volle Link
    // bleibt einen Klick entfernt, für Fälle wie Archivierung oder wenn jemand
    // den Inhalt im Link selbst behalten will.
    let mut link_type = use_signal(|| LinkType::Short);
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

    // Der volle Link als Signal, damit die Closures unten kopierbar bleiben
    // (ein `String` liesse sich nur einmal in eine `move`-Closure ziehen).
    let full_url = use_signal(|| url.clone());
    let url_for_shorten = url.clone();

    // Was im Eingabefeld steht und kopiert wird.
    let displayed_url = move || match link_type() {
        LinkType::Full => Some(full_url()),
        // Solange gekürzt wird, gibt es noch nichts anzuzeigen.
        LinkType::Short => short_url(),
    };

    let start_shortening = move || {
        let url_to_shorten = url_for_shorten.clone();
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

    // Beim Öffnen sofort kürzen. Der frühere Ablauf verlangte einen
    // zusätzlichen Klick auf "Link kürzen"; da der Dienst nun zur Anwendung
    // gehört, ist dieser Zwischenschritt nur noch Reibung.
    use_effect(move || {
        if show() && short_url().is_none() && !is_shortening() && shorten_error().is_none() {
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

            // Use simple textarea fallback method
            let mut success = false;
            if let Some(window) = window() {
                if let Some(document) = window.document() {
                    if let Ok(textarea) = document.create_element("textarea") {
                        if let Ok(textarea) = textarea.dyn_into::<HtmlTextAreaElement>() {
                            textarea.set_value(&url_to_copy);
                            textarea
                                .set_attribute(
                                    "style",
                                    "position: fixed; left: -999999px; top: -999999px;",
                                )
                                .ok();

                            if let Some(body) = document.body() {
                                if let Ok(node) = textarea.clone().dyn_into::<web_sys::Node>() {
                                    body.append_child(&node).ok();
                                    textarea.select();

                                    // Use JavaScript to copy
                                    let _ = js_sys::eval("document.execCommand('copy')");
                                    success = true;

                                    body.remove_child(&node).ok();
                                }
                            }
                        }
                    }
                }
            }

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
                                    link_type.set(LinkType::Short);
                                }
                            }
                            span { class: "label-text", {t!("link_type_short").to_string()} }
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
                                value: current_url,
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
                            }

                            if copy_success() {
                                div {
                                    class: "text-success text-sm mt-2",
                                    {t!("link_copied_success").to_string()}
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
