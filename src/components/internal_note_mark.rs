use crate::components::icons;
use dioxus::prelude::*;
use rust_i18n::t;

/// Symbolic marker for internal-only "note" fields whose value never appears on
/// the label. Renders a crossed-out eye with a fast hover tooltip explaining it.
/// Drop it next to any note field's label/control for a consistent signal.
#[component]
pub fn InternalNoteMark() -> Element {
    rsx! {
        span {
            class: "tooltip tooltip-left align-middle text-base-content/40",
            "data-tip": t!("bio_labels.erlaubte_ausnahme_details_note").to_string(),
            icons::EyeOff {}
        }
    }
}
