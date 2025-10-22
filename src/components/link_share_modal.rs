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
    let mut link_type = use_signal(|| LinkType::Full);
    let mut is_copying = use_signal(|| false);
    let mut copy_success = use_signal(|| false);
    let mut short_url = use_signal(|| None::<String>);
    let mut is_shortening = use_signal(|| false);
    let mut show_shorten_button = use_signal(|| true);

    let url_clone1 = url.clone();
    let url_clone2 = url.clone();

    let copy_to_clipboard = move |_| {
        let display_url = match link_type() {
            LinkType::Full => url_clone1.clone(),
            LinkType::Short => short_url().unwrap_or_else(|| url_clone1.clone()),
        };

        spawn(async move {
            is_copying.set(true);
            copy_success.set(false);

            let url_to_copy = display_url;

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

    let shorten_url_action = move |_| {
        let url_to_shorten = url_clone2.clone();
        spawn(async move {
            is_shortening.set(true);
            show_shorten_button.set(false);

            // Using TinyURL as a reliable URL shortener
            let encoded_url = urlencoding::encode(&url_to_shorten);
            let tinyurl_api = format!("https://tinyurl.com/api-create.php?url={}", encoded_url);

            match gloo::net::http::Request::get(&tinyurl_api).send().await {
                Ok(response) => {
                    if let Ok(shortened) = response.text().await {
                        short_url.set(Some(shortened));
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to shorten URL: {:?}", e);
                    // Reset button on error
                    show_shorten_button.set(true);
                }
            }

            is_shortening.set(false);
        });
    };

    // Reset short URL and button when switching back to full link
    use_effect(move || {
        if link_type() == LinkType::Full {
            short_url.set(None);
            show_shorten_button.set(true);
        }
    });

    rsx! {
        dialog {
            class: "modal",
            open: show(),
            div {
                class: "modal-box w-11/12 max-w-2xl",
                h3 {
                    class: "font-bold text-lg mb-4",
                    {t!("link_share_title")}
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
                            span { class: "label-text", {t!("link_type_full")} }
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
                            span { class: "label-text", {t!("link_type_short")} }
                        }
                    }

                    if link_type() == LinkType::Short {
                        div {
                            class: "text-sm text-base-content/70 mb-4",
                            {t!("link_short_disclaimer")}
                        }

                        if short_url().is_none() && show_shorten_button() {
                            div {
                                class: "mb-4",
                                button {
                                    class: "btn btn-warning btn-sm",
                                    disabled: is_shortening(),
                                    onclick: shorten_url_action,
                                    if is_shortening() {
                                        span { class: "loading loading-spinner loading-sm" }
                                        {t!("shortening")}
                                    } else {
                                        {t!("shorten_url_button")}
                                    }
                                }
                            }
                        }
                    }

                    if link_type() == LinkType::Full || short_url().is_some() {
                        div {
                            class: "flex gap-2",
                            input {
                                r#type: "text",
                                class: "input input-bordered flex-1",
                                value: match link_type() {
                                    LinkType::Full => url.clone(),
                                    LinkType::Short => short_url().unwrap_or_else(|| url.clone()),
                                },
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
                                    {t!("link_copied_success")}
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
                        {t!("close")}
                    }
                }
            }
            form {
                method: "dialog",
                class: "modal-backdrop",
                onclick: move |_| show.set(false),
                button { {t!("buttons.close")} }
            }
        }
    }
}
