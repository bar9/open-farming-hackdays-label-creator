use crate::built_info;
use crate::components::icons;
use crate::components::{LinkShareModal, SavedIngredientsManager};
use crate::routes::Route;
use dioxus::prelude::*;
use rust_i18n::t;
use web_sys::window;

#[derive(Clone, Default)]
pub struct CopyLinkContext {
    pub query_string: Option<String>,
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
    let mut show_link_modal = use_signal(|| false);
    let mut show_warning = use_signal(|| false);
    let mut target_route = use_signal(|| Option::<Route>::None);
    let nav = use_navigator();

    rsx! {
        document::Stylesheet {
            href: asset!("assets/tailwind.css")
        }
        div {
            key: "split-layout",
            class: "min-h-screen flex flex-col",
            "data-theme": "{theme_context.read().theme}",
            header {
                class: "bg-base-200 p-4",
                div {
                    class: "flex justify-between items-center",
                    div {
                        class: "flex items-center gap-4",
                        Link {
                            to: Route::SplashScreen {},
                            class: "text-2xl font-bold hover:text-primary transition-colors",
                            {t!("app.title")}
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
                                            class: "w-4 h-4 mr-2 flex flex-col items-center justify-center bg-green-100 rounded",
                                            span { class: "text-green-700 font-bold text-[6px] leading-none", "CH" }
                                            span { class: "text-green-700 font-bold text-[6px] leading-none", "BIO" }
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
                                                path {
                                                    d: "m 81.13737,15.08862 c -0.05,-1.94 -0.06,-3 -0.3225,-5.3875 -0.7825,0.85875 -1.635,1.78875 -2.6625,2.69 -0.19875,-1.6875 -0.53125,-3.255 -0.72,-4.3025 -1.36125,1.4125 -1.945,2.02 -3.08625,3.12 0.16875,1.05875 0.3725,2.2425 0.4675,3.7375 -1.1425,0.8525 -2.25875,1.6025 -3.3125,2.2625 0.15375,1.83 0.2125,2.81125 0.325,4.45625 0.26,-0.14875 1.68125,-1.09625 3.275,-2.125 0.0225,1.6725 0.07,3.2375 0.006,4.605 0.335,-0.195 2.23,-1.1825 3.50875,-2.48 0.18375,-1.3875 0.0788,-2.93 0.015,-4.61 1.35875,-0.945 2.2625,-1.5925 2.50625,-1.96625",
                                                    style: "fill:#ffffff;fill-opacity:1;fill-rule:nonzero;stroke:none"
                                                }
                                                path {
                                                    d: "m 81.13887,14.92375 c -0.3725,0.42125 -1.41375,1.20625 -2.66875,2.085 0.0925,1.6025 0.1,3.165 -0.0437,4.49 -1.1725,1.2625 -2.80375,2.12 -3.11375,2.31125 0.0325,-1.35375 0.008,-2.96875 -0.0637,-4.625 -1.61,1.0575 -2.97625,1.92 -3.22625,2.0675 -0.10125,-1.57125 -0.135,-2.135 -0.28125,-3.88125 1.005,-0.64 2.12,-1.4175 3.24375,-2.26125 -0.13,-1.46125 -0.29125,-2.795 -0.47625,-3.82125 1.04875,-1.07125 1.595,-1.58125 2.84375,-2.95625 0.2075,1.05 0.47125,2.5825 0.6975,4.30375 1.05375,-0.92 2.02125,-1.8425 2.79375,-2.7025 -0.615,-5.57 -1.97,-9.42375 -1.9725,-9.435 L 78.69522,0 78.43647,0.45875 c 0,0.001 -0.0825,0.1475 -0.245,0.41625 -1.0725,1.7925 -5.6,8.955 -11.68375,13.43875 2.72875,3.8975 4.33125,8.635 4.33125,13.74375 0,1.61 -0.16125,3.185 -0.465,4.7075 0.85125,-0.4425 1.74375,-0.915 2.68125,-1.42 1.8625,-1.08375 3.345,-1.84 4.55,-2.98625 1.20625,-1.14375 2.1025,-2.66875 2.7775,-5.21625 l 0.002,-0.006 0.001,-0.009 c 0.54625,-2.72 0.75375,-5.4 0.75375,-7.91125 0,-0.0987 -0.001,-0.195 -0.001,-0.2925 M 25.27785,38.6574 c -8.3625,4.5675 -6.28,9.895 -0.0613,12.86125 7.335,3.50125 21.9925,5.2225 29.05125,2.47 C 41.27155,54.6899 29.6553,50.1274 25.2778,38.6574",
                                                    style: "fill:#e2001a;fill-opacity:1;fill-rule:nonzero;stroke:none"
                                                }
                                                path {
                                                    d: "m 86.74887,75.42725 0,-13.755 7.20875,0 0,1.0325 -6.03875,0 0,5.085 5.63,0 0,1.0325 -5.63,0 0,5.5725 6.11625,0 0,1.0325 -7.28625,0 z m -7.14012,0.23437 c -1.7725,0 -3.33125,-0.74 -4.01375,-2.455 -0.2325,-0.585 -0.29125,-1.18875 -0.33,-1.8125 l 1.16875,0 c 0.0575,1.93 1.24625,3.215 3.23375,3.215 1.5975,0 3.07875,-0.99375 3.07875,-2.70875 0,-0.895 -0.41,-1.51875 -1.09125,-2.045 -0.89625,-0.70125 -2.1825,-1.15 -3.215,-1.65625 -0.4675,-0.21375 -0.935,-0.42875 -1.36375,-0.74125 -0.87625,-0.6225 -1.36375,-1.51875 -1.36375,-2.61 0,-2.22125 1.94875,-3.41 3.97375,-3.41 1.735,0 3.25375,0.83875 3.74125,2.5725 0.0975,0.35125 0.1175,0.70125 0.13625,1.0525 l -1.15,0 c -0.0387,-0.41 -0.11625,-0.83875 -0.31125,-1.20875 -0.4875,-0.95375 -1.48125,-1.36375 -2.5125,-1.36375 -1.345,0 -2.67,0.8375 -2.67,2.3 0,3.0775 7.05375,2.435 7.05375,7.0325 0,2.49375 -2.04625,3.83875 -4.365,3.83875 m -10.77563,0 c -1.7725,0 -3.33125,-0.74 -4.01375,-2.455 -0.23375,-0.585 -0.2925,-1.18875 -0.33125,-1.8125 l 1.16875,0 c 0.0587,1.93 1.2475,3.215 3.235,3.215 1.5975,0 3.07875,-0.99375 3.07875,-2.70875 0,-0.895 -0.41,-1.51875 -1.09125,-2.045 -0.8975,-0.70125 -2.1825,-1.15 -3.215,-1.65625 -0.4675,-0.21375 -0.935,-0.42875 -1.36375,-0.74125 -0.8775,-0.6225 -1.36375,-1.51875 -1.36375,-2.61 0,-2.22125 1.94875,-3.41 3.97375,-3.41 1.735,0 3.25375,0.83875 3.74125,2.5725 0.0975,0.35125 0.11625,0.70125 0.13625,1.0525 l -1.15,0 c -0.0387,-0.41 -0.11625,-0.83875 -0.31125,-1.20875 -0.4875,-0.95375 -1.48125,-1.36375 -2.5125,-1.36375 -1.345,0 -2.67,0.8375 -2.67,2.3 0,3.0775 7.0525,2.435 7.0525,7.0325 0,2.49375 -2.045,3.83875 -4.36375,3.83875 m -8.67125,-13.99 1.2075,0 0,13.755 -1.2075,0 0,-13.755 z m -3.66425,10.6385 c -0.39,2.3575 -2.37625,3.35125 -4.5975,3.35125 -1.89,0 -3.83875,-0.76 -4.4625,-2.70875 -0.195,-0.58375 -0.21375,-1.2075 -0.21375,-1.83125 l 0,-9.44875 1.16875,0 c 0,3.0775 -0.02,6.13625 -0.02,9.21625 0,0.77875 0.0788,1.5775 0.4675,2.27875 0.62375,1.09 1.87125,1.44125 3.06,1.44125 0.68125,0 1.46125,-0.175 2.065,-0.48625 1.6175,-0.83875 1.4225,-2.59125 1.4225,-4.13125 l 0,-8.31875 1.18875,0 0,9.41 c 0,0.41 -0.02,0.81875 -0.0788,1.2275 m -16.2315,3.3515 c -1.7725,0 -3.33125,-0.74 -4.01375,-2.455 -0.23375,-0.585 -0.2925,-1.18875 -0.33125,-1.8125 l 1.16875,0 c 0.0587,1.93 1.2475,3.215 3.235,3.215 1.5975,0 3.07875,-0.99375 3.07875,-2.70875 0,-0.895 -0.41,-1.51875 -1.09125,-2.045 -0.89625,-0.70125 -2.1825,-1.15 -3.215,-1.65625 -0.4675,-0.21375 -0.93625,-0.42875 -1.36375,-0.74125 -0.8775,-0.6225 -1.36375,-1.51875 -1.36375,-2.61 0,-2.22125 1.9475,-3.41 3.97375,-3.41 1.73375,0 3.25375,0.83875 3.74125,2.5725 0.0975,0.35125 0.1175,0.70125 0.13625,1.0525 l -1.14875,0 c -0.04,-0.41 -0.1175,-0.83875 -0.3125,-1.20875 -0.4875,-0.95375 -1.48125,-1.36375 -2.5125,-1.36375 -1.345,0 -2.67,0.8375 -2.67,2.3 0,3.0775 7.05375,2.435 7.05375,7.0325 0,2.49375 -2.04625,3.83875 -4.365,3.83875 M 27.27357,64.18 c -2.41625,0 -3.1575,2.35875 -3.1575,4.36625 0,2.06625 0.72125,4.36625 3.19625,4.36625 2.3975,0 3.1575,-2.28125 3.1575,-4.3075 0,-2.08625 -0.6825,-4.425 -3.19625,-4.425 m 0.0188,11.55875 c -4.2875,0 -6.9575,-2.9825 -6.9575,-7.1925 0,-4.24875 2.65,-7.1925 6.9775,-7.1925 4.2875,0 6.95875,3.0025 6.95875,7.1925 0,4.21 -2.69,7.1925 -6.97875,7.1925 m -12.88175,-14.07213 3.64625,0 0,13.76 -3.64625,0 0,-13.76 z m -8.29937,7.9125 -2.54375,0 0,3.1575 2.68,0 c 1.13125,0 2.18375,-0.19375 2.18375,-1.55875 0,-1.4625 -1.15,-1.59875 -2.32,-1.59875 m -0.17625,-5.2425 -2.3675,0 0,2.68875 2.11625,0 c 1.0125,0 2.085,-0.0975 2.085,-1.38375 0,-1.13 -0.8975,-1.305 -1.83375,-1.305 m 4.26875,10.33 c -1.285,0.72125 -3.09875,0.76 -4.54125,0.76 l -5.6625,0 0,-13.76 5.64375,0 c 1.735,0 3.89875,0.0388 5.06625,1.55875 0.44875,0.565 0.645,1.30625 0.645,2.02625 0,1.385 -0.74125,2.32 -2.0475,2.7675 1.69625,0.37125 2.84625,1.54125 2.84625,3.33375 0,1.38375 -0.74125,2.65125 -1.95,3.31375",
                                                    style: "fill:#30a32d;fill-opacity:1;fill-rule:nonzero;stroke:none"
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
                                    button {
                                        class: format!("flex items-center gap-3 p-2 rounded-lg hover:bg-base-200 w-full text-left {}",
                                            if matches!(current_route.clone(), Route::Swiss { .. }) { "bg-primary/20 text-primary" } else { "" }),
                                        onclick: {
                                            let route = current_route.clone();
                                            move |_| {
                                                if !matches!(route, Route::Swiss { .. }) {
                                                    target_route.set(Some(Route::Swiss {}));
                                                    show_warning.set(true);
                                                }
                                            }
                                        },
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
                                    button {
                                        class: format!("flex items-center gap-3 p-2 rounded-lg hover:bg-base-200 w-full text-left {}",
                                            if matches!(current_route.clone(), Route::Bio { .. }) { "bg-primary/20 text-primary" } else { "" }),
                                        onclick: {
                                            let route = current_route.clone();
                                            move |_| {
                                                if !matches!(route, Route::Bio { .. }) {
                                                    target_route.set(Some(Route::Bio {}));
                                                    show_warning.set(true);
                                                }
                                            }
                                        },
                                        div {
                                            class: "w-8 h-8 flex flex-col items-center justify-center bg-green-100 rounded",
                                            span { class: "text-green-700 font-bold text-xs leading-none", "CH" }
                                            span { class: "text-green-700 font-bold text-xs leading-none", "BIO" }
                                        }
                                        div {
                                            class: "flex flex-col",
                                            span { class: "font-medium", {t!("routes.bio")} }
                                            span { class: "text-sm text-base-content/70", {t!("routes.bio_desc")} }
                                        }
                                    }
                                }
                                li {
                                    button {
                                        class: format!("flex items-center gap-3 p-2 rounded-lg hover:bg-base-200 w-full text-left {}",
                                            if matches!(current_route.clone(), Route::Knospe { .. }) { "bg-primary/20 text-primary" } else { "" }),
                                        onclick: {
                                            let route = current_route.clone();
                                            move |_| {
                                                if !matches!(route, Route::Knospe { .. }) {
                                                    target_route.set(Some(Route::Knospe {}));
                                                    show_warning.set(true);
                                                }
                                            }
                                        },
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
                                                path {
                                                    d: "m 81.13737,15.08862 c -0.05,-1.94 -0.06,-3 -0.3225,-5.3875 -0.7825,0.85875 -1.635,1.78875 -2.6625,2.69 -0.19875,-1.6875 -0.53125,-3.255 -0.72,-4.3025 -1.36125,1.4125 -1.945,2.02 -3.08625,3.12 0.16875,1.05875 0.3725,2.2425 0.4675,3.7375 -1.1425,0.8525 -2.25875,1.6025 -3.3125,2.2625 0.15375,1.83 0.2125,2.81125 0.325,4.45625 0.26,-0.14875 1.68125,-1.09625 3.275,-2.125 0.0225,1.6725 0.07,3.2375 0.006,4.605 0.335,-0.195 2.23,-1.1825 3.50875,-2.48 0.18375,-1.3875 0.0788,-2.93 0.015,-4.61 1.35875,-0.945 2.2625,-1.5925 2.50625,-1.96625",
                                                    style: "fill:#ffffff;fill-opacity:1;fill-rule:nonzero;stroke:none"
                                                }
                                                path {
                                                    d: "m 81.13887,14.92375 c -0.3725,0.42125 -1.41375,1.20625 -2.66875,2.085 0.0925,1.6025 0.1,3.165 -0.0437,4.49 -1.1725,1.2625 -2.80375,2.12 -3.11375,2.31125 0.0325,-1.35375 0.008,-2.96875 -0.0637,-4.625 -1.61,1.0575 -2.97625,1.92 -3.22625,2.0675 -0.10125,-1.57125 -0.135,-2.135 -0.28125,-3.88125 1.005,-0.64 2.12,-1.4175 3.24375,-2.26125 -0.13,-1.46125 -0.29125,-2.795 -0.47625,-3.82125 1.04875,-1.07125 1.595,-1.58125 2.84375,-2.95625 0.2075,1.05 0.47125,2.5825 0.6975,4.30375 1.05375,-0.92 2.02125,-1.8425 2.79375,-2.7025 -0.615,-5.57 -1.97,-9.42375 -1.9725,-9.435 L 78.69522,0 78.43647,0.45875 c 0,0.001 -0.0825,0.1475 -0.245,0.41625 -1.0725,1.7925 -5.6,8.955 -11.68375,13.43875 2.72875,3.8975 4.33125,8.635 4.33125,13.74375 0,1.61 -0.16125,3.185 -0.465,4.7075 0.85125,-0.4425 1.74375,-0.915 2.68125,-1.42 1.8625,-1.08375 3.345,-1.84 4.55,-2.98625 1.20625,-1.14375 2.1025,-2.66875 2.7775,-5.21625 l 0.002,-0.006 0.001,-0.009 c 0.54625,-2.72 0.75375,-5.4 0.75375,-7.91125 0,-0.0987 -0.001,-0.195 -0.001,-0.2925 M 25.27785,38.6574 c -8.3625,4.5675 -6.28,9.895 -0.0613,12.86125 7.335,3.50125 21.9925,5.2225 29.05125,2.47 C 41.27155,54.6899 29.6553,50.1274 25.2778,38.6574",
                                                    style: "fill:#e2001a;fill-opacity:1;fill-rule:nonzero;stroke:none"
                                                }
                                                path {
                                                    d: "m 86.74887,75.42725 0,-13.755 7.20875,0 0,1.0325 -6.03875,0 0,5.085 5.63,0 0,1.0325 -5.63,0 0,5.5725 6.11625,0 0,1.0325 -7.28625,0 z m -7.14012,0.23437 c -1.7725,0 -3.33125,-0.74 -4.01375,-2.455 -0.2325,-0.585 -0.29125,-1.18875 -0.33,-1.8125 l 1.16875,0 c 0.0575,1.93 1.24625,3.215 3.23375,3.215 1.5975,0 3.07875,-0.99375 3.07875,-2.70875 0,-0.895 -0.41,-1.51875 -1.09125,-2.045 -0.89625,-0.70125 -2.1825,-1.15 -3.215,-1.65625 -0.4675,-0.21375 -0.935,-0.42875 -1.36375,-0.74125 -0.87625,-0.6225 -1.36375,-1.51875 -1.36375,-2.61 0,-2.22125 1.94875,-3.41 3.97375,-3.41 1.735,0 3.25375,0.83875 3.74125,2.5725 0.0975,0.35125 0.1175,0.70125 0.13625,1.0525 l -1.15,0 c -0.0387,-0.41 -0.11625,-0.83875 -0.31125,-1.20875 -0.4875,-0.95375 -1.48125,-1.36375 -2.5125,-1.36375 -1.345,0 -2.67,0.8375 -2.67,2.3 0,3.0775 7.05375,2.435 7.05375,7.0325 0,2.49375 -2.04625,3.83875 -4.365,3.83875 m -10.77563,0 c -1.7725,0 -3.33125,-0.74 -4.01375,-2.455 -0.23375,-0.585 -0.2925,-1.18875 -0.33125,-1.8125 l 1.16875,0 c 0.0587,1.93 1.2475,3.215 3.235,3.215 1.5975,0 3.07875,-0.99375 3.07875,-2.70875 0,-0.895 -0.41,-1.51875 -1.09125,-2.045 -0.8975,-0.70125 -2.1825,-1.15 -3.215,-1.65625 -0.4675,-0.21375 -0.935,-0.42875 -1.36375,-0.74125 -0.8775,-0.6225 -1.36375,-1.51875 -1.36375,-2.61 0,-2.22125 1.94875,-3.41 3.97375,-3.41 1.735,0 3.25375,0.83875 3.74125,2.5725 0.0975,0.35125 0.11625,0.70125 0.13625,1.0525 l -1.15,0 c -0.0387,-0.41 -0.11625,-0.83875 -0.31125,-1.20875 -0.4875,-0.95375 -1.48125,-1.36375 -2.5125,-1.36375 -1.345,0 -2.67,0.8375 -2.67,2.3 0,3.0775 7.0525,2.435 7.0525,7.0325 0,2.49375 -2.045,3.83875 -4.36375,3.83875 m -8.67125,-13.99 1.2075,0 0,13.755 -1.2075,0 0,-13.755 z m -3.66425,10.6385 c -0.39,2.3575 -2.37625,3.35125 -4.5975,3.35125 -1.89,0 -3.83875,-0.76 -4.4625,-2.70875 -0.195,-0.58375 -0.21375,-1.2075 -0.21375,-1.83125 l 0,-9.44875 1.16875,0 c 0,3.0775 -0.02,6.13625 -0.02,9.21625 0,0.77875 0.0788,1.5775 0.4675,2.27875 0.62375,1.09 1.87125,1.44125 3.06,1.44125 0.68125,0 1.46125,-0.175 2.065,-0.48625 1.6175,-0.83875 1.4225,-2.59125 1.4225,-4.13125 l 0,-8.31875 1.18875,0 0,9.41 c 0,0.41 -0.02,0.81875 -0.0788,1.2275 m -16.2315,3.3515 c -1.7725,0 -3.33125,-0.74 -4.01375,-2.455 -0.23375,-0.585 -0.2925,-1.18875 -0.33125,-1.8125 l 1.16875,0 c 0.0587,1.93 1.2475,3.215 3.235,3.215 1.5975,0 3.07875,-0.99375 3.07875,-2.70875 0,-0.895 -0.41,-1.51875 -1.09125,-2.045 -0.89625,-0.70125 -2.1825,-1.15 -3.215,-1.65625 -0.4675,-0.21375 -0.93625,-0.42875 -1.36375,-0.74125 -0.8775,-0.6225 -1.36375,-1.51875 -1.36375,-2.61 0,-2.22125 1.9475,-3.41 3.97375,-3.41 1.73375,0 3.25375,0.83875 3.74125,2.5725 0.0975,0.35125 0.1175,0.70125 0.13625,1.0525 l -1.14875,0 c -0.04,-0.41 -0.1175,-0.83875 -0.3125,-1.20875 -0.4875,-0.95375 -1.48125,-1.36375 -2.5125,-1.36375 -1.345,0 -2.67,0.8375 -2.67,2.3 0,3.0775 7.05375,2.435 7.05375,7.0325 0,2.49375 -2.04625,3.83875 -4.365,3.83875 M 27.27357,64.18 c -2.41625,0 -3.1575,2.35875 -3.1575,4.36625 0,2.06625 0.72125,4.36625 3.19625,4.36625 2.3975,0 3.1575,-2.28125 3.1575,-4.3075 0,-2.08625 -0.6825,-4.425 -3.19625,-4.425 m 0.0188,11.55875 c -4.2875,0 -6.9575,-2.9825 -6.9575,-7.1925 0,-4.24875 2.65,-7.1925 6.9775,-7.1925 4.2875,0 6.95875,3.0025 6.95875,7.1925 0,4.21 -2.69,7.1925 -6.97875,7.1925 m -12.88175,-14.07213 3.64625,0 0,13.76 -3.64625,0 0,-13.76 z m -8.29937,7.9125 -2.54375,0 0,3.1575 2.68,0 c 1.13125,0 2.18375,-0.19375 2.18375,-1.55875 0,-1.4625 -1.15,-1.59875 -2.32,-1.59875 m -0.17625,-5.2425 -2.3675,0 0,2.68875 2.11625,0 c 1.0125,0 2.085,-0.0975 2.085,-1.38375 0,-1.13 -0.8975,-1.305 -1.83375,-1.305 m 4.26875,10.33 c -1.285,0.72125 -3.09875,0.76 -4.54125,0.76 l -5.6625,0 0,-13.76 5.64375,0 c 1.735,0 3.89875,0.0388 5.06625,1.55875 0.44875,0.565 0.645,1.30625 0.645,2.02625 0,1.385 -0.74125,2.32 -2.0475,2.7675 1.69625,0.37125 2.84625,1.54125 2.84625,3.33375 0,1.38375 -0.74125,2.65125 -1.95,3.31375",
                                                    style: "fill:#30a32d;fill-opacity:1;fill-rule:nonzero;stroke:none"
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
                    }
                    nav {
                        class: "flex gap-4 items-center",
                        {
                            let context = copy_link_context.read();
                            if let Some(_query_string) = &context.query_string {
                                rsx! {
                                    button {
                                        class: "btn btn-info btn-sm",
                                        onclick: move |_| {
                                            show_link_modal.set(true);
                                        },
                                        icons::Clipboard {}
                                        "{t!(\"nav.linkKopieren\")}"
                                    }
                                }
                            } else {
                                rsx! { span {} }
                            }
                        }
                        
                        // Saved ingredients manager button
                        SavedIngredientsManager {}

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

        // Link share modal
        {
            let context = copy_link_context.read();
            if let Some(query_string) = &context.query_string {
                let full_url = if let Some(window) = window() {
                    if let Ok(href) = window.location().href() {
                        // Remove existing query parameters from href before appending new ones
                        let base_url = if let Some(question_mark_pos) = href.find('?') {
                            &href[..question_mark_pos]
                        } else {
                            &href
                        };
                        format!("{}{}", base_url, query_string)
                    } else {
                        query_string.clone()
                    }
                } else {
                    query_string.clone()
                };

                rsx! {
                    LinkShareModal {
                        show: show_link_modal,
                        url: full_url
                    }
                }
            } else {
                rsx! {}
            }
        }

        // Warning Dialog
        if show_warning() {
            div {
                class: "modal modal-open",
                div {
                    class: "modal-box",
                    h3 {
                        class: "font-bold text-lg",
                        "Warnung"
                    }
                    p {
                        class: "py-4",
                        "Ihre Daten gehen verloren, wenn Sie in einen anderen Standard wechseln"
                    }
                    div {
                        class: "modal-action",
                        button {
                            class: "btn btn-ghost",
                            onclick: move |_| {
                                show_warning.set(false);
                                target_route.set(None);
                            },
                            "Abbrechen"
                        }
                        button {
                            class: "btn btn-primary",
                            onclick: move |_| {
                                show_warning.set(false);
                                if let Some(route) = target_route() {
                                    nav.push(route);
                                }
                            },
                            "OK"
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
