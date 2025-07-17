use dioxus::prelude::*;
use crate::routes::Route;
use web_sys::window;
use crate::components::icons;
use rust_i18n::t;

#[derive(Clone)]
pub struct CopyLinkContext {
    pub query_string: Option<String>,
}

impl Default for CopyLinkContext {
    fn default() -> Self {
        Self {
            query_string: None,
        }
    }
}

#[component]
pub fn SplitLayout() -> Element {
    let copy_link_context = use_context::<Signal<CopyLinkContext>>();
    let current_route = use_route::<Route>();
    
    rsx! {
        document::Stylesheet {
            href: asset!("assets/tailwind.css")
        }
        div {
            key: "split-layout",
            class: "min-h-screen flex flex-col",
            "data-theme": "lemonade",
            header {
                class: "bg-base-200 p-4 shadow-md",
                div {
                    class: "flex justify-between items-center",
                    Link {
                        to: Route::SplashScreen {},
                        class: "text-2xl font-bold hover:text-primary transition-colors",
                        "Label Creator"
                    }
                    nav {
                        class: "flex gap-4 items-center",
                        {
                            let context = copy_link_context.read();
                            if let Some(query_string) = &context.query_string {
                                let query_string_clone = query_string.clone();
                                rsx! {
                                    button {
                                        class: "btn btn-primary btn-sm",
                                        onclick: move |_| {
                                            if let Some(window) = window() {
                                                let navigator = window.navigator();
                                                let clipboard = navigator.clipboard();
                                                if let Ok(href) = window.location().href() {
                                                    let text = format!("{href}{query_string_clone}");
                                                    let _ = clipboard.write_text(&text);
                                                }
                                            }
                                        },
                                        icons::Clipboard {}
                                        "{t!(\"nav.linkKopieren\")}"
                                    }
                                }
                            } else {
                                rsx! { span {} }
                            }
                        }
                        
                        div {
                            class: "dropdown dropdown-end",
                            div {
                                tabindex: "0",
                                role: "button",
                                class: "btn btn-ghost btn-sm",
                                "Konfiguration "
                                svg {
                                    class: "w-4 h-4 ml-1",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                        d: "M19 9l-7 7-7-7"
                                    }
                                }
                            }
                            ul {
                                tabindex: "0",
                                class: "dropdown-content menu bg-base-100 rounded-box z-[1] w-72 p-2 shadow-lg",
                                li {
                                    Link {
                                        to: Route::Swiss {},
                                        class: format!("flex items-center gap-3 p-2 rounded-lg hover:bg-base-200 {}",
                                            if matches!(current_route, Route::Swiss { .. }) { "bg-primary/20 text-primary" } else { "" }),
                                        div {
                                            class: "w-8 h-8 flex items-center justify-center bg-red-50 rounded",
                                            svg {
                                                class: "w-6 h-6",
                                                view_box: "0 0 32 32",
                                                rect { width: "32", height: "32", fill: "#FF0000" }
                                                rect { x: "13", y: "6", width: "6", height: "20", fill: "white" }
                                                rect { x: "6", y: "13", width: "20", height: "6", fill: "white" }
                                            }
                                        }
                                        div {
                                            class: "flex flex-col",
                                            span { class: "font-medium", "CH-Lebensmittelrecht" }
                                            span { class: "text-sm text-base-content/70", "Schweizer Lebensmittelverordnung" }
                                        }
                                    }
                                }
                                li {
                                    Link {
                                        to: Route::Bio {},
                                        class: format!("flex items-center gap-3 p-2 rounded-lg hover:bg-base-200 {}",
                                            if matches!(current_route, Route::Bio { .. }) { "bg-primary/20 text-primary" } else { "" }),
                                        div {
                                            class: "w-8 h-8 flex items-center justify-center bg-green-100 rounded",
                                            svg {
                                                class: "w-6 h-6",
                                                view_box: "0 0 32 32",
                                                rect { width: "32", height: "32", fill: "#FF0000" }
                                                rect { x: "13", y: "6", width: "6", height: "20", fill: "white" }
                                                rect { x: "6", y: "13", width: "20", height: "6", fill: "white" }
                                            }
                                        }
                                        div {
                                            class: "flex flex-col",
                                            span { class: "font-medium", "Bio-Verordnung" }
                                            span { class: "text-sm text-base-content/70", "Biologische Produkte" }
                                        }
                                    }
                                }
                                li {
                                    Link {
                                        to: Route::Knospe {},
                                        class: format!("flex items-center gap-3 p-2 rounded-lg hover:bg-base-200 {}",
                                            if matches!(current_route, Route::Knospe { .. }) { "bg-primary/20 text-primary" } else { "" }),
                                        div {
                                            class: "w-8 h-8 flex items-center justify-center bg-green-50 rounded",
                                            svg {
                                                class: "w-6 h-6",
                                                view_box: "-2.27216241 -2.27216241 98.57944282 80.28307182",
                                                xmlns: "http://www.w3.org/2000/svg",
                                                path {
                                                    d: "m 24.38225,28.0565 c 0,-12.39625 10.04875,-22.44375 22.445,-22.44375 12.395,0 22.44375,10.0475 22.44375,22.44375 0,12.395 -10.04875,22.44375 -22.44375,22.44375 -12.39625,0 -22.445,-10.04875 -22.445,-22.44375",
                                                    style: "fill:#30a32d;fill-opacity:1;fill-rule:nonzero;stroke:none"
                                                }
                                                path {
                                                    d: "m 47.154,8.39425 c 0,0 -18.9025,11.39375 -6.43875,30.80375 0,0 -7.20625,-3.63125 -8.79375,-14.2175 -0.0475,-0.4375 -0.565,-0.95875 -0.93875,0.105 -3.7425,10.69125 1.84875,24.12 17.10875,23.4725 15.38875,-0.65375 18.88,-18.325 10.955,-29.23875 -0.28625,-0.4475 -0.7,-0.56375 -0.4625,0.2975 1.53875,5.57875 -1.02375,12.12375 -1.93875,13.605 C 61.249,18.10925 48.17025,8.518 47.154,8.39425",
                                                    style: "fill:#ffffff;fill-opacity:1;fill-rule:nonzero;stroke:none"
                                                }
                                            }
                                        }
                                        div {
                                            class: "flex flex-col",
                                            span { class: "font-medium", "Bio Knospe" }
                                            span { class: "text-sm text-base-content/70", "Bio Suisse Knospe" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            main { 
                key: "split-main",
                class: "grid grid-cols-1 md:grid-cols-2 flex-1 min-h-0",
                Outlet::<Route> {}
            }
            footer {
                class: "bg-base-200 p-4 text-center text-sm mt-auto",
                div {
                    class: "flex justify-between items-center",
                    span {
                        "© 2025 Swiss Food Label Creator"
                    }
                    div {
                        class: "flex gap-4 items-center",
                        span {
                            "Version 0.3.9 vom 13.06.2025"
                        }
                        Link {
                            to: Route::Impressum {},
                            class: "link link-blue hover:link-primary",
                            "Impressum"
                        }
                        a {
                            class: "link link-blue hover:link-primary",
                            href: "https://github.com/bar9/open-farming-hackdays-label-creator/wiki/Release-notes",
                            "Release Notes"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn FullLayout() -> Element {
    use_context_provider(|| Signal::new(CopyLinkContext::default()));
    
    rsx! {
        document::Stylesheet {
            href: asset!("assets/tailwind.css")
        }
        div {
            key: "full-layout",
            class: "min-h-screen",
            "data-theme": "lemonade",
            main { 
                key: "full-main",
                class: "",
                Outlet::<Route> {}
            }
        }
    }
}