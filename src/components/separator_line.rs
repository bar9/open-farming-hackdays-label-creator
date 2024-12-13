use dioxus::prelude::*;
pub fn SeparatorLine() -> Element {
    rsx! {
        hr { class: "border-1 border-dashed border-neutral-400 my-2" }
    }
}
