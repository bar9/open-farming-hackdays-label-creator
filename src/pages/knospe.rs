use crate::pages::label_page::LabelPage;
use crate::shared::Configuration;
use dioxus::prelude::*;

pub fn Knospe() -> Element {
    rsx! { LabelPage { configuration: Configuration::Knospe } }
}
