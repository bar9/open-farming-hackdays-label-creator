use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct PureWrapProps {
    children: Element,
}
pub fn ThemeLayout<'a>(props: PureWrapProps) -> Element {
    {
        rsx! {
            div {
                class: "min-h-screen",
                "data-theme": "garden",
                main { class: "grid grid-cols-1 md:grid-cols-2 flex-grow",
                    {props.children}
                }
            }
        }
    }
}
