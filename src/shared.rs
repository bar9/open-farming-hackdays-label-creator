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
