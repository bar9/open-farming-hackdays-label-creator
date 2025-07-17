use dioxus::prelude::*;
use crate::routes::Route;

#[component]
pub fn SplitLayout() -> Element {
    rsx! {
        document::Stylesheet {
            href: asset!("assets/tailwind.css")
        }
        div {
            class: "min-h-screen",
            "data-theme": "lemonade",
            main { class: "grid grid-cols-1 md:grid-cols-2 flex-grow",
                Outlet::<Route> {}
            }
        }
    }
}

#[component]
pub fn FullLayout() -> Element {
    rsx! {
        document::Stylesheet {
            href: asset!("assets/tailwind.css")
        }
        div {
            class: "min-h-screen",
            "data-theme": "lemonade",
            main { class: "",
                Outlet::<Route> {}
            }
        }
    }
}