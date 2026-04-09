use crate::pages::label_page::LabelPage;
use crate::shared::Configuration;
use dioxus::prelude::*;

pub fn Swiss() -> Element {
    rsx! { LabelPage { configuration: Configuration::Conventional } }
}
