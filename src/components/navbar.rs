use dioxus::prelude::*;
use crate::pages::swiss::Configuration;
use crate::rules::RuleDef;

#[derive(Props, Clone, PartialEq)]
pub struct NavbarProps {
    rules: Vec<RuleDef>,
    config: Signal<Configuration>,
}

#[component]
#[allow(unused_variables)]
pub fn Navbar(props: NavbarProps) -> Element {
    todo!()
    // rsx! {
    //     div { class: "navbar bg-base-100",
    //         div { class: "",
    //             ul { class: "menu menu-horizontal px-1",
    //                 li {
    //                     details {
    //                         summary { "Konfiguration" }
    //                         ul { class: "p-2",
    //                             // todo
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //         div { class: "",
    //             a { class: "btn", "aktive Regeln" }
    //         }
    //     }
    // }
}