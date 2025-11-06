use crate::components::FieldHelp;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct FormFieldProps {
    #[props(into)]
    label: String,
    #[props(into)]
    help: Option<String>,
    #[props(into, default = false)]
    required: bool,
    #[props(into, default = false)]
    inline_checkbox: bool,
    children: Element,
}
pub fn FormField(props: FormFieldProps) -> Element {
    if props.inline_checkbox {
        rsx! {
            div {
                class: "flex gap-2 flex-col",
                label {
                    class: "label cursor-pointer justify-start gap-2 items-center",
                    {props.children}
                    span {
                        class: "label-text font-semibold",
                        if props.required {
                            span {class: "text-red-300", "* "}
                        }
                        "{props.label}"
                    }
                    {rsx!{
                        FieldHelp {
                            label: props.label,
                            help: props.help.unwrap_or_default()
                        }
                    }}
                }
            }
        }
    } else {
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
}
