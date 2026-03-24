use dioxus::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct Validations(pub Memo<HashMap<String, Vec<String>>>);

#[derive(Clone, Copy)]
pub struct Conditionals(pub Memo<HashMap<String, bool>>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Configuration {
    Conventional,
    Bio,
    Knospe,
}

impl Configuration {
    pub fn theme_key(&self) -> &'static str {
        match self {
            Configuration::Conventional => "themes.swiss",
            Configuration::Bio => "themes.bio",
            Configuration::Knospe => "themes.knospe",
        }
    }

    pub fn has_certification_body(&self) -> bool {
        matches!(self, Configuration::Bio | Configuration::Knospe)
    }

    pub fn certification_body_help_key(&self) -> Option<&'static str> {
        match self {
            Configuration::Bio => Some("help.certification_body_bio"),
            Configuration::Knospe => Some("help.certification_body_knospe"),
            _ => None,
        }
    }
}

/// Post-process HTML to open external links in a new tab.
pub fn externalize_links(html: &str) -> String {
    html.replace("<a href=\"http", "<a target=\"_blank\" rel=\"noopener noreferrer\" href=\"http")
}

pub fn restore_params_from_session_storage() -> Option<String> {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.session_storage() {
            if let Ok(Some(saved_params)) = storage.get_item("pre_route_params") {
                let _ = storage.remove_item("pre_route_params");
                return Some(saved_params);
            }
        }
    }
    None
}
