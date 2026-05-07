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
            .flat_map(|(k, v)| v.iter().map(move |msg| (k.clone(), msg.clone())))
            .collect::<Vec<_>>()
    });

    let has_errors = !relevant_validation_entries().is_empty();
    let border_class = if has_errors { "border border-error rounded-md" } else { "" };

    rsx! {
        div {
            class: "flex flex-col",
            div {
                class: "{border_class}",
                {props.children}
            }
            if has_errors {
                div {
                    class: "bg-error/30 border border-error/40 rounded-md p-3 mt-2",
                    for (index, (_path, msg)) in relevant_validation_entries().iter().enumerate() {
                        div {
                            class: if index > 0 { "mt-2 pt-2 border-t border-error/40" } else { "" },
                            div {
                                class: "flex items-start gap-2 text-base-content text-sm",
                                div {
                                    class: "flex-shrink-0 w-5 h-5 bg-error text-error-content rounded-full flex items-center justify-center text-xs font-bold mt-0.5",
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
