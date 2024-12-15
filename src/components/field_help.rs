use dioxus::prelude::*;
use crate::components::InfoIcon;

#[derive(Props, Clone, PartialEq)]
pub struct FieldHelpProps {
    #[props(into)]
    label: String,
    help: Element,
}
pub fn FieldHelp(props: FieldHelpProps) -> Element {
    let mut is_open = use_signal(|| false);
    if props.help.is_err() {
        rsx!{}
    } else {
        rsx! {
            button {
                class: "btn btn-xs ml-2",
                onkeydown: move |evt| {
                    match evt.key() {
                        Key::Escape => {
                            is_open.set(false);
                        }
                        _ => {}
                    }
                },
                onclick: move |_| is_open.toggle(),
                InfoIcon {}
            }
            if is_open() { div { class: "fixed inset-0 bg-black bg-opacity-50 backdrop-blur-md" } }
            dialog { open: "{is_open}", class: "modal",
                div { class: "modal-box bg-base-100 backdrop-blur-3xl",
                    h3 { class: "font-bold text-lg", "{props.label}" }
                    {props.help}
                    div { class: "modal-action",
                        form { method: "dialog",
                            button {
                                class: "btn btn-sm",
                                onclick: move |_| is_open.toggle(),
                                "× Schliessen"
                            }
                        }
                    }
                }
            }
        }
    }
}