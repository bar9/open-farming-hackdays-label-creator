use crate::components::icons;
use dioxus::prelude::*;
use markdown::{to_html_with_options, Options};
use rust_i18n::t;

#[derive(Props, Clone, PartialEq)]
pub struct FieldHelpProps {
    #[props(into)]
    label: String,
    #[props(into)]
    help: String,
}
pub fn FieldHelp(props: FieldHelpProps) -> Element {
    let mut is_open = use_signal(|| false);
    if props.help.is_empty() {
        rsx! {}
    } else {
        rsx! {
            button {
                class: "btn btn-xs ml-2",
                onkeydown: move |evt| if evt.key() == Key::Escape {is_open.set(false); },
                onclick: move |_| is_open.toggle(),
                icons::Info{}
            }
            if is_open() {
                div {
                    // class: "fixed inset-0 bg-black bg-opacity-50 backdrop-blur-md",
                    dialog { open: "{is_open}", class: "modal",
                    div { class: "modal-box",
                        h3 { class: "font-bold text-lg", dangerous_inner_html: to_html_with_options(&props.label, &Options::gfm()).unwrap() }
                        div { class: "prose", dangerous_inner_html: to_html_with_options(&props.help, &Options::gfm()).unwrap() }
                        div { class: "modal-action",
                            form { method: "dialog",
                                button {
                                    class: "btn btn-sm",
                                    onclick: move |_| is_open.toggle(),
                                    "{t!(\"nav.schliessen\")}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
