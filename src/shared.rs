use crate::verdicts::Verdicts;
use dioxus::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct Validations(pub Memo<HashMap<String, Vec<String>>>);

#[derive(Clone, Copy)]
pub struct Conditionals(pub Memo<HashMap<String, bool>>);

/// The typed rule-engine decisions (TD-1). New UI code should read this
/// instead of the string-keyed `Conditionals`.
#[derive(Clone, Copy)]
pub struct VerdictsContext(pub Memo<Verdicts>);

impl Conditionals {
    /// Whether the rule engine set this conditional.
    ///
    /// Absent and `false` mean the same thing to every caller, so this collapses
    /// the `get(k).unwrap_or(&false) == &true` dance that was written out at
    /// each of the ~18 use sites in the label preview.
    pub fn is_set(&self, key: &str) -> bool {
        *self.0.read().get(key).unwrap_or(&false)
    }
}

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
