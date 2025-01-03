use dioxus::prelude::*;
use crate::Configuration;
use crate::rules::RuleDef;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Props, Clone, PartialEq)]
pub struct NavbarProps {
    rules: Vec<RuleDef>,
    config: Signal<Configuration>,
}

#[component]
pub fn Navbar(props: NavbarProps) -> Element {
    rsx! {
        div { class: "navbar bg-base-100",
            div { class: "",
                ul { class: "menu menu-horizontal px-1",
                    li {
                        details {
                            summary { "Konfiguration" }
                            ul { class: "p-2",
                                // todo
                            }
                        }
                    }
                }
            }
            div { class: "",
                a { class: "btn", "aktive Regeln" }
            }
        }
    }
}