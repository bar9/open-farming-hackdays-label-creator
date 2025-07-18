#![allow(non_snake_case)]

use dioxus::prelude::*;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
use rust_i18n::i18n;
use crate::routes::Route;

mod layout;
mod shared;

mod model;
mod components;
pub mod core;
mod rules;
mod nl2br;
mod form;
mod routes;

mod pages;

i18n!();

fn main() {
    // Try to restore saved language from localStorage, default to de-CH
    let locale = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .and_then(|storage| storage.get_item("locale").ok())
        .flatten()
        .unwrap_or_else(|| "de-CH".to_string());
    
    rust_i18n::set_locale(&locale);
    
    launch(|| {
        rsx! {
            Router::<Route> {}
        }
    })
}


