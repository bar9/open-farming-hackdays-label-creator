use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct PureWrapProps {
    children: Element,
}
pub fn ThemeLayout<'a>(props: PureWrapProps) -> Element {
    let mut active_theme = use_signal(|| "emerald");
    let themes = vec![
        "dark", "cupcake", "bumblebee", "emerald", "corporate", "synthwave",
        "retro", "cyberpunk", "valentine", "halloween", "garden", "forest",
        "aqua", "lofi", "pastel", "fantasy", "wireframe", "black", "luxury",
        "dracula", "cmyk", "autumn", "business", "acid", "lemonade", "night",
        "coffee", "winter", "dim", "nord", "sunset",
    ];

    {rsx! {
        div {
            class: "min-h-screen",
            "data-theme": "{active_theme}",
            div { class: "flex flex-col",
                div { class: "flex-1",
                    div { class: "navbar bg-base-100",
                        ul { class: "menu menu-horizontal px-8",
                            li {
                                details {
                                    summary {
                                        "Choose a theme..."
                                    }
                                    ul { class: "bg-base-100 rounded-t-none",
                                        for theme in themes {
                                            li { a { onclick: move |_| active_theme.set(theme), {theme} } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                main { class: "grid grid-cols-1 md:grid-cols-2 gap-12 flex-grow",
                    {props.children}
                }
            }
        }
    }}
}
