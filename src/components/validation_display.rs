use crate::shared::Validations;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ValidationDisplayProps {
    paths: Vec<String>,
    children: Element,
}

pub fn ValidationDisplay(props: ValidationDisplayProps) -> Element {
    let validations_context = use_context::<Validations>();

    // Create a derived memo to ensure reactivity
    let relevant_validation_entries = use_memo(move || {
        let validation_entries = validations_context.0.read();
        validation_entries
            .iter()
            .filter(|(k, _v)| props.paths.contains(&**k))
            .map(|(k, v)| (k.clone(), *v))
            .collect::<Vec<_>>()
    });

    rsx! {
        div {
            class: "flex flex-col",
            div {
                class: "border-red",
                {props.children}
            }
            div {
                class: "border-red bg-red-100",
                ul {
                    for (_path, msg) in relevant_validation_entries() {
                        li {class: "p-1", "{msg}"}
                    }
                }
            }
        }
    }
}
