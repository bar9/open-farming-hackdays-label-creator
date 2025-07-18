use dioxus::prelude::*;
use crate::routes::Route;
use web_sys::window;
use crate::components::icons;
use rust_i18n::t;
use crate::built_info;

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

#[derive(Clone)]
pub struct ThemeContext {
    pub theme: String,
}

impl Default for ThemeContext {
    fn default() -> Self {
        Self {
            theme: "corporate".to_string(),
        }
    }
}

#[component]
pub fn SplitLayout() -> Element {
    let copy_link_context = use_context::<Signal<CopyLinkContext>>();
    let theme_context = use_context::<Signal<ThemeContext>>();
    let current_route = use_route::<Route>();
    
    rsx! {
        document::Stylesheet {
            href: asset!("assets/tailwind.css")
        }
        div {
            key: "split-layout",
            class: "min-h-screen flex flex-col",
            "data-theme": "{theme_context.read().theme}",
            header {
                class: "bg-base-200 p-4 shadow-md border-b border-base-300",
                div {
                    class: "flex justify-between items-center",
                    Link {
                        to: Route::SplashScreen {},
                        class: "text-2xl font-bold hover:text-primary transition-colors",
                        {t!("app.title")}
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
                                {match current_route {
                                    Route::Swiss { .. } => rsx! {
                                        div {
                                            class: "w-4 h-4 mr-2",
                                            svg {
                                                class: "w-4 h-4",
                                                view_box: "0 0 32 32",
                                                rect { width: "32", height: "32", fill: "#FF0000" }
                                                rect { x: "13", y: "6", width: "6", height: "20", fill: "white" }
                                                rect { x: "6", y: "13", width: "20", height: "6", fill: "white" }
                                            }
                                        }
                                        {t!("routes.swiss")}
                                    },
                                    Route::Bio { .. } => rsx! {
                                        div {
                                            class: "w-4 h-4 mr-2",
                                            svg {
                                                class: "w-4 h-4",
                                                view_box: "0 0 32 32",
                                                rect { width: "32", height: "32", fill: "#FF0000" }
                                                rect { x: "13", y: "6", width: "6", height: "20", fill: "white" }
                                                rect { x: "6", y: "13", width: "20", height: "6", fill: "white" }
                                            }
                                        }
                                        {t!("routes.bio")}
                                    },
                                    Route::Knospe { .. } => rsx! {
                                        div {
                                            class: "w-4 h-4 mr-2",
                                            svg {
                                                class: "w-4 h-4",
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
                                        {t!("routes.knospe")}
                                    },
                                    _ => rsx! {
                                        {t!("routes.configuration")}
                                    }
                                }}
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
                                            span { class: "font-medium", {t!("routes.swiss")} }
                                            span { class: "text-sm text-base-content/70", {t!("routes.swiss_desc")} }
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
                                            span { class: "font-medium", {t!("routes.bio")} }
                                            span { class: "text-sm text-base-content/70", {t!("routes.bio_desc")} }
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
                                            span { class: "font-medium", {t!("routes.knospe")} }
                                            span { class: "text-sm text-base-content/70", {t!("routes.knospe_desc")} }
                                        }
                                    }
                                }
                            }
                        }
                        
                        div {
                            class: "dropdown dropdown-end",
                            div {
                                tabindex: "0",
                                role: "button",
                                class: "btn btn-ghost btn-sm",
                                {match rust_i18n::locale().as_ref() {
                                    "fr-CH" => "FR ",
                                    "it-CH" => "IT ",
                                    _ => "DE ",
                                }}
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
                                class: "dropdown-content menu bg-base-100 rounded-box z-[1] w-20 p-2 shadow-lg",
                                li {
                                    button {
                                        class: "btn btn-ghost btn-sm justify-start",
                                        onclick: move |_| {
                                            rust_i18n::set_locale("de-CH");
                                            if let Some(window) = web_sys::window() {
                                                if let Ok(Some(storage)) = window.local_storage() {
                                                    let _ = storage.set_item("locale", "de-CH");
                                                }
                                                let _ = window.location().reload();
                                            }
                                        },
                                        "DE"
                                    }
                                }
                                li {
                                    button {
                                        class: "btn btn-ghost btn-sm justify-start",
                                        onclick: move |_| {
                                            rust_i18n::set_locale("fr-CH");
                                            if let Some(window) = web_sys::window() {
                                                if let Ok(Some(storage)) = window.local_storage() {
                                                    let _ = storage.set_item("locale", "fr-CH");
                                                }
                                                let _ = window.location().reload();
                                            }
                                        },
                                        "FR"
                                    }
                                }
                                li {
                                    button {
                                        class: "btn btn-ghost btn-sm justify-start",
                                        onclick: move |_| {
                                            rust_i18n::set_locale("it-CH");
                                            if let Some(window) = web_sys::window() {
                                                if let Ok(Some(storage)) = window.local_storage() {
                                                    let _ = storage.set_item("locale", "it-CH");
                                                }
                                                let _ = window.location().reload();
                                            }
                                        },
                                        "IT"
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
                class: "bg-base-200 p-4 text-center text-sm mt-auto border-t border-base-300",
                div {
                    class: "flex justify-center items-center gap-4",
                    span {
                        "Version " {env!("CARGO_PKG_VERSION")} " vom " {
                            // Convert UTC time string to a more readable format
                            let build_time = built_info::BUILT_TIME_UTC;
                            // Parse the RFC 2822 formatted string and format it as dd.mm.yyyy
                            if let Ok(datetime) = chrono::DateTime::parse_from_rfc2822(build_time) {
                                format!("{}", datetime.format("%d.%m.%Y"))
                            } else {
                                build_time.to_string()
                            }
                        }
                    }
                    Link {
                        to: Route::Impressum {},
                        class: "link link-blue hover:link-primary",
                        {t!("app.impressum")}
                    }
                    a {
                        class: "link link-blue hover:link-primary",
                        href: "https://github.com/bar9/open-farming-hackdays-label-creator/wiki/Release-notes",
                        {t!("app.release_notes")}
                    }
                }
            }
        }
    }
}

#[component]
pub fn FullLayout() -> Element {
    use_context_provider(|| Signal::new(CopyLinkContext::default()));
    use_context_provider(|| Signal::new(ThemeContext::default()));
    
    rsx! {
        document::Stylesheet {
            href: asset!("assets/tailwind.css")
        }
        div {
            key: "full-layout",
            class: "min-h-screen",
            "data-theme": "corporate",
            main { 
                key: "full-main",
                class: "",
                Outlet::<Route> {}
            }
        }
    }
}