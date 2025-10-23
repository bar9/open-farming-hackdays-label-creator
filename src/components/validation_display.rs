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
                class: if !relevant_validation_entries().is_empty() { "border border-red-500 rounded-md" } else { "" },
                {props.children}
            }
            if !relevant_validation_entries().is_empty() {
                div {
                    class: "bg-red-50 border border-red-200 rounded-md p-3 mt-2",
                    for (index, (_path, msg)) in relevant_validation_entries().iter().enumerate() {
                        div {
                            class: if index > 0 { "mt-2 pt-2 border-t border-red-200" } else { "" },
                            div {
                                class: "flex items-start gap-2 text-red-700 text-sm",
                                div {
                                    class: "flex-shrink-0 w-5 h-5 bg-red-500 text-white rounded-full flex items-center justify-center text-xs font-bold mt-0.5",
                                    "!"
                                }
                                div {
                                    class: "flex-1",
                                    "{msg}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
