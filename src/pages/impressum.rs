use dioxus::prelude::*;
use crate::components::Impressum as ImpressumComponent;
use rust_i18n::t;

pub fn Impressum() -> Element {
    rsx! {
        div { class: "container mx-auto p-8",
            div { class: "max-w-4xl mx-auto",
                h1 { class: "text-4xl mb-8", "Impressum" }
                ImpressumComponent {}
                div { class: "mt-8",
                    Link {
                        to: crate::routes::Route::SplashScreen {},
                        class: "btn btn-primary",
                        "{t!(\"nav.zurueck\")}"
                    }
                }
            }
        }
    }
}