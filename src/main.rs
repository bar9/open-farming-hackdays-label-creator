#![allow(non_snake_case)]

use dioxus::prelude::*;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
use crate::routes::Route;
use rust_i18n::i18n;

mod layout;
mod shared;

mod api;
mod category_service;
mod components;
pub mod core;
mod form;
mod model;
mod nl2br;
mod persistence;
pub mod processing_service;
mod routes;
mod rules;
mod services;

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
