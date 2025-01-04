use dioxus::prelude::*;
use crate::Validations;

#[derive(Props, Clone, PartialEq)]
pub struct ValidationDisplayProps {
    paths: Vec<String>,
    children: Element,
}

pub fn ValidationDisplay(props: ValidationDisplayProps) -> Element {

    let validations_context= use_context::<Validations>();
    let validation_entries = (&*validations_context.0.read()).clone();
    let relevant_validation_entries = validation_entries.iter().filter(|(k, _v)| props.paths.contains(&**k))
        .collect::<Vec<_>>();

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
                    for (_path, msg) in relevant_validation_entries {
                        li {class: "p-1", "{msg}"}
                    }
                }
            }
        }
    }
}
