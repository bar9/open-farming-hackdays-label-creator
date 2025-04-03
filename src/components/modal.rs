use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ModalProps {
    open: Signal<bool>,
    title: String,
    children: Element,
}

pub fn Modal(mut props: ModalProps) -> Element {
    rsx! {
        if *props.open.read() {
            div {
                onkeydown: move |evt: KeyboardEvent| if evt.key() == Key::Escape { props.open.set(false); },
                class: "fixed inset-0 bg-black bg-opacity-50 backdrop-blur-md",
                dialog {
                    open: "{props.open}", class: "modal modal-open z-50",
                    div {
                        class: "modal-box bg-base-100 backdrop-blur-3xl",
                        h3 {
                            class: "font-bold text-lg", "{props.title}" }
                        {props.children}
                        div {
                            class: "modal-action",
                            form {
                                method: "dialog",
                                button {
                                    class: "btn btn-sm",
                                    onclick: move |_| props.open.set(false),
                                    "× Schliessen"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}