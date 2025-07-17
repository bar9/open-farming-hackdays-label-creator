use dioxus::prelude::*;
use rust_i18n::t;

#[component]
pub fn Knospe() -> Element {
    rsx! {
        div {
            class: "flex items-center justify-center h-full bg-base-200",
            div {
                class: "text-center",
                h1 {
                    class: "text-5xl font-bold mb-4",
                    {t!("Bio Knospe")}
                }
                p {
                    class: "text-xl text-base-content/70",
                    {t!("under construction")}
                }
            }
        }
    }
}