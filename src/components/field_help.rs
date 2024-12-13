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
    if props.help.is_none() {
        None
    } else {
        rsx! {
            button {
                class: "btn btn-xs ml-2",
                onkeydown: move |evt| {
                    match evt.key() {
                        Key::Escape => {
                            is_open.set(false);
                        },
                        _ => {}
                    }
                },
                onclick: move |_| is_open.toggle(),
                InfoIcon{}
            }
            dialog {
                open: "{is_open}",
                class: "modal",
                div {
                    class: "modal-box bg-base-100",
                    h3 { class:"font-bold text-lg", "{props.label}" }
                    {props.help}
                    div {class: "modal-action",
                        form {method: "dialog",
                            button {class:"btn btn-sm",
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