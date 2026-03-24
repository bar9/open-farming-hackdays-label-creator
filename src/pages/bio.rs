use crate::pages::label_page::LabelPage;
use crate::shared::Configuration;
use dioxus::prelude::*;

pub fn Bio() -> Element {
    rsx! { LabelPage { configuration: Configuration::Bio } }
}
