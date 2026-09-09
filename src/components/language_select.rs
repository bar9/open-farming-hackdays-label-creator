use dioxus::prelude::*;
use rust_i18n::t;

/// Switch the UI language: remember the choice in localStorage and reload, so
/// every already-rendered string comes back translated.
fn switch_locale(locale: &'static str) {
    rust_i18n::set_locale(locale);
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("locale", locale);
        }
        let _ = window.location().reload();
    }
}

/// Name of the active language, for the closed dropdown.
fn current_language_label() -> String {
    match rust_i18n::locale().as_ref() {
        "fr-CH" => t!("languages.fr").to_string(),
        "it-CH" => t!("languages.it").to_string(),
        _ => t!("languages.de").to_string(),
    }
}

/// The de/fr/it language dropdown in the header. Identical on the splash screen
/// and in the app layout, so it lives here once.
#[component]
pub fn LanguageSelect() -> Element {
    rsx! {
        div {
            class: "dropdown dropdown-end",
            div {
                tabindex: "0",
                role: "button",
                class: "btn btn-ghost btn-sm",
                {current_language_label()}
                " "
                svg {
                    class: "w-4 h-4 ml-1",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M19 9l-7 7-7-7"
                    }
                }
            }
            ul {
                tabindex: "0",
                class: "dropdown-content menu bg-base-100 rounded-box z-[1] w-20 p-2 shadow-lg",
                li {
                    button {
                        class: "btn btn-ghost btn-sm justify-start",
                        onclick: move |_| switch_locale("de-CH"),
                        {t!("languages.de").to_string()}
                    }
                }
                li {
                    button {
                        class: "btn btn-ghost btn-sm justify-start",
                        onclick: move |_| switch_locale("fr-CH"),
                        {t!("languages.fr").to_string()}
                    }
                }
                li {
                    button {
                        class: "btn btn-ghost btn-sm justify-start",
                        onclick: move |_| switch_locale("it-CH"),
                        {t!("languages.it").to_string()}
                    }
                }
            }
        }
    }
}
