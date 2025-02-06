use dioxus::prelude::*;
use crate::components::FieldHelp;

#[derive(Props, Clone, PartialEq)]
pub struct FormFieldProps {
    #[props(into)]
    label: String,
    #[props(into)]
    help: Option<String>,
    children: Element,
}
pub fn FormField(props: FormFieldProps) -> Element {
    rsx! {
        div {
            class: "flex gap-2 flex-col",
            label {
                class: "flex items-center",
                "{props.label}"
                {rsx!{
                    FieldHelp {
                        label: props.label,
                        help: props.help.unwrap_or_default()
                    }
                }}
            }
            {props.children}
        }
    }
}
