use dioxus::prelude::*;
use rust_i18n::t;

#[component]
pub fn Bio() -> Element {
    rsx! {
        div {
            class: "flex items-center justify-center min-h-screen bg-base-200",
            div {
                class: "text-center",
                h1 {
                    class: "text-5xl font-bold mb-4",
                    {t!("splash.title")}
                }
                p {
                    class: "text-xl text-base-content/70",
                    {t!("splash.under_construction")}
                }
            }
        }
    }
}