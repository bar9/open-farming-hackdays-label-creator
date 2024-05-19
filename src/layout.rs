use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct PureWrapProps {
    children: Element,
}
pub fn ThemeLayout<'a>(props: PureWrapProps) -> Element {
    let mut active_theme = use_signal(|| "garden");

    {
        rsx! {
            div {
                class: "min-h-screen",
                "data-theme": "{active_theme}",
                main { class: "grid grid-cols-1 md:grid-cols-2 gap-12 flex-grow",
                    {props.children}
                }
            }
        }
    }
}
