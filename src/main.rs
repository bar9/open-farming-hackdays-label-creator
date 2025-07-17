#![allow(non_snake_case)]

use dioxus::prelude::*;
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
    rust_i18n::set_locale("de-CH");
    // launch(app);
    launch(|| {
        rsx! {
            Router::<Route> {}
        }
    })
}


