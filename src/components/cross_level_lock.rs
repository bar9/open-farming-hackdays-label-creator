use dioxus::prelude::*;
use rust_i18n::t;

/// Greys out a composite field control when the same attribute is already defined on
/// a sub-ingredient (`locked`). A simple, non-interactive tooltip explains why; the
/// field is read-only here — change it on the sub-ingredient itself. When not locked
/// the control renders untouched (editable).
#[derive(Props, Clone, PartialEq)]
pub struct CrossLevelLockProps {
    pub locked: bool,
    /// The field control to render (greyed when locked).
    pub children: Element,
}

pub fn CrossLevelLock(props: CrossLevelLockProps) -> Element {
    if !props.locked {
        return rsx! { {props.children} };
    }
    rsx! {
        // Outer element captures hover (for the tooltip); inner greys + disables the control.
        div {
            class: "tooltip tooltip-top block w-full",
            "data-tip": t!("cross_level.defined_on_subingredients").to_string(),
            div { class: "opacity-50 pointer-events-none", {props.children} }
        }
    }
}
