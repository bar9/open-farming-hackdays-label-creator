use dioxus::prelude::*;

pub fn ThemeLayout<'a>(cx: Scope<'a, PureWrapProps<'a>>) -> Element {
    let active_theme = use_state(cx, || "cupcake");
    let themes = vec![
        // "dark",
        "cupcake",
        // "bumblebee",
        // "emerald",
        // "corporate",
        // "synthwave",
        // "retro",
        // "cyberpunk",
        // "valentine",
        // "halloween",
        // "garden",
        // "forest",
        // "aqua",
        // "lofi",
        // "pastel",
        // "fantasy",
        // "wireframe",
        // "black",
        // "luxury",
        // "dracula",
        // "cmyk",
        // "autumn",
        // "business",
        // "acid",
        // "lemonade",
        // "night",
        // "coffee",
        // "winter",
        // "dim",
        // "nord",
        // "sunset",
    ];

    render! {
        div {
            class: "min-h-screen",
            "data-theme": "{active_theme}",
            div {
                class: "container",
            &cx.props.children
            }
        }
    }
}

#[derive(Props)]
pub struct PureWrapProps<'a> {
    children: Element<'a>,
}