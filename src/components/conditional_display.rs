use crate::shared::Conditionals;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ConditionalDisplayProps {
    path: String,
    children: Element,
}

pub fn ConditionalDisplay(props: ConditionalDisplayProps) -> Element {
    let conditional_context = use_context::<Conditionals>();

    // Create a derived memo to ensure reactivity
    let do_display = use_memo(move || {
        let conditionals = conditional_context.0.read();
        *conditionals.get(&props.path).unwrap_or(&false)
    });

    if do_display() {
        rsx! {
            {props.children}
        }
    } else {
        rsx! {}
    }
}
