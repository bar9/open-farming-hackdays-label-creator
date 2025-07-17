use dioxus::prelude::*;
use rust_i18n::t;
use crate::layout::ThemeContext;

#[component]
pub fn Bio() -> Element {
    let mut theme_context = use_context::<Signal<ThemeContext>>();
    
    use_effect(move || {
        theme_context.write().theme = "bio".to_string();
    });
    
    rsx! {
        div {
            class: "flex items-center justify-center h-full bg-base-200",
            div {
                class: "text-center",
                h1 {
                    class: "text-5xl font-bold mb-4",
                    {t!("Bio")}
                }
                p {
                    class: "text-xl text-base-content/70",
                    {t!("under construction")}
                }
            }
        }
    }
}