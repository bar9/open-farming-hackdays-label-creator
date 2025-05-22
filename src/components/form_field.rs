use dioxus::prelude::*;
use crate::components::FieldHelp;

#[derive(Props, Clone, PartialEq)]
pub struct FormFieldProps {
    #[props(into)]
    label: String,
    #[props(into)]
    help: Option<String>,
    #[props(into, default=false)]
    required: bool,
    children: Element,
}
pub fn FormField(props: FormFieldProps) -> Element {
    rsx! {
        div {
            class: "flex gap-2 flex-col",
            label {
                class: "flex items-center text-left",
                if props.required {
                    span {class: "text-red-300", "* "}
                }
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
