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

/// Returns the inline SVG markup of the brand logo for the active UI locale.
///
/// The logo is inlined (rather than an `<img>`) so the leaf mark, which uses
/// `fill="currentColor"`, picks up the active theme's primary color — drive it
/// by putting `text-primary` on the wrapping element. The wordmark/tagline keep
/// their own dark fills. Language switches trigger a full page reload, so this is
/// recomputed fresh on each load — no reactivity needed.
#[cfg(not(feature = "hidebio"))]
pub fn locale_logo() -> &'static str {
    let svg = match rust_i18n::locale().as_ref() {
        "fr-CH" => include_str!("../assets/declarino-logo-fr-CH.svg"),
        "it-CH" => include_str!("../assets/declarino-logo-it-CH.svg"),
        _ => include_str!("../assets/declarino-logo-de-CH.svg"),
    };
    // Drop the XML prolog/comment so it injects cleanly as HTML innerHTML.
    match svg.find("<svg") {
        Some(i) => &svg[i..],
        None => svg,
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

#[derive(Clone, Default)]
pub struct DisclaimerContext {
    pub accepted: bool,
}

#[component]
pub fn SplitLayout() -> Element {
    let copy_link_context = use_context::<Signal<CopyLinkContext>>();
    let theme_context = use_context::<Signal<ThemeContext>>();
    let disclaimer_context = use_context::<Signal<DisclaimerContext>>();
    let current_route = use_route::<Route>();
    let mut show_link_modal = use_signal(|| false);
    let mut show_warning = use_signal(|| false);
    let mut target_route = use_signal(|| Option::<Route>::None);
    let nav = use_navigator();

    rsx! {
        div {
            key: "{&theme_context.read().theme}",
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
                            {
                                #[cfg(not(feature = "hidebio"))]
                                { rsx! { span { class: "inline-block h-16 text-primary", "aria-label": "Declarino", dangerous_inner_html: locale_logo() } } }
                                #[cfg(feature = "hidebio")]
                                { rsx! { {t!("app.title").to_string()} } }
                            }
                        }
                        div {
                            class: "dropdown dropdown-end",
                            div {
                                tabindex: "0",
                                role: "button",
                                class: "btn btn-ghost btn-sm",
                                {
                                    match &current_route {
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
                                            {t!("routes.swiss").to_string()}
                                        },
                                        #[cfg(not(feature = "hidebio"))]
                                        Route::Bio { .. } => rsx! {
                                            div {
                                                class: "w-5 h-5 mr-2 flex flex-col items-center justify-center bg-green-100 rounded",
                                                span { class: "text-green-700 font-bold leading-none", style: "font-size:6px", "CH" }
                                                span { class: "text-green-700 font-bold leading-none", style: "font-size:6px", "BIO" }
                                            }
                                            {t!("routes.bio").to_string()}
                                        },
                                        #[cfg(not(feature = "hidebio"))]
                                        Route::Knospe { .. } => rsx! {
                                            div {
                                                class: "w-4 h-4 mr-2",
                                                svg {
                                                    class: "w-4 h-4",
                                                    view_box: "22 3 50 50",
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
                                            {t!("routes.knospe").to_string()}
                                        },
                                        _ => rsx! { {t!("routes.configuration").to_string()} },
                                    }
                                }
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
                                            span { class: "font-medium", {t!("routes.swiss").to_string()} }
                                            span { class: "text-sm text-base-content/70", {t!("routes.swiss_desc").to_string()} }
                                        }
                                    }
                                }
                                if !cfg!(feature = "hidebio") {
                                    li {
                                        button {
                                            class: "flex items-center gap-3 p-2 rounded-lg hover:bg-base-200 w-full text-left",
                                            onclick: {
                                                move |_| {
                                                    #[cfg(not(feature = "hidebio"))]
                                                    {
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
                                                span { class: "font-medium", {t!("routes.bio").to_string()} }
                                                span { class: "text-sm text-base-content/70", {t!("routes.bio_desc").to_string()} }
                                            }
                                        }
                                    }
                                }
                                if !cfg!(feature = "hidebio") {
                                    li {
                                        button {
                                            class: "flex items-center gap-3 p-2 rounded-lg hover:bg-base-200 w-full text-left",
                                            onclick: {
                                                move |_| {
                                                    #[cfg(not(feature = "hidebio"))]
                                                    {
                                                        target_route.set(Some(Route::Knospe {}));
                                                        show_warning.set(true);
                                                    }
                                                }
                                            },
                                            div {
                                                class: "w-8 h-8 flex items-center justify-center bg-green-50 rounded",
                                                svg {
                                                    class: "w-6 h-6",
                                                    view_box: "22 3 50 50",
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
                                                span { class: "font-medium", {t!("routes.knospe").to_string()} }
                                                span { class: "text-sm text-base-content/70", {t!("routes.knospe_desc").to_string()} }
                                            }
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
                            let disclaimer_accepted = disclaimer_context.read().accepted;
                            if let Some(_query_string) = &context.query_string {
                                rsx! {
                                    div {
                                        class: if !disclaimer_accepted { "tooltip tooltip-bottom" } else { "" },
                                        "data-tip": if !disclaimer_accepted { t!("disclaimer.button_tooltip").to_string() } else { String::new() },
                                        button {
                                            class: format!("btn btn-info btn-sm {}", if !disclaimer_accepted { "btn-disabled" } else { "" }),
                                            onclick: move |_| {
                                                if disclaimer_context.read().accepted {
                                                    show_link_modal.set(true);
                                                }
                                            },
                                            icons::Clipboard {}
                                            "{t!(\"nav.linkKopieren\").to_string()}"
                                        }
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
                                    "fr-CH" => t!("languages.fr").to_string(),
                                    "it-CH" => t!("languages.it").to_string(),
                                    _ => t!("languages.de").to_string(),
                                }}
                                " "
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
                                        {t!("languages.de").to_string()}
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
                                        {t!("languages.fr").to_string()}
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
                                        {t!("languages.it").to_string()}
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
                        {t!("version.version").to_string()} " " {env!("CARGO_PKG_VERSION")} " " {t!("version.from").to_string()} " " {
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
                        {t!("app.impressum").to_string()}
                    }
                    a {
                        class: "link link-blue hover:link-primary",
                        href: "https://github.com/bar9/open-farming-hackdays-label-creator/wiki/Release-notes",
                        target: "_blank",
                        {t!("app.release_notes").to_string()}
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
                        {t!("warnings.title").to_string()}
                    }
                    p {
                        class: "py-4",
                        {t!("warnings.data_loss_on_switch").to_string()}
                    }
                    div {
                        class: "modal-action",
                        button {
                            class: "btn btn-ghost",
                            onclick: move |_| {
                                show_warning.set(false);
                                target_route.set(None);
                            },
                            {t!("buttons.cancel").to_string()}
                        }
                        button {
                            class: "btn btn-primary",
                            onclick: move |_| {
                                show_warning.set(false);
                                if let Some(route) = target_route() {
                                    nav.push(route);
                                }
                            },
                            {t!("buttons.ok").to_string()}
                        }
                    }
                }
            }
        }
    }
}

#[allow(deprecated)]
#[component]
pub fn FullLayout() -> Element {
    use_context_provider(|| Signal::new(CopyLinkContext::default()));
    use_context_provider(|| Signal::new(ThemeContext::default()));
    // Restore disclaimer acceptance from localStorage
    let disclaimer_accepted = window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .and_then(|storage| storage.get_item("disclaimer_accepted").ok())
        .flatten()
        .map(|v| v == "true")
        .unwrap_or(false);
    use_context_provider(|| Signal::new(DisclaimerContext { accepted: disclaimer_accepted }));

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
