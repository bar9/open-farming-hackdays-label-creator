use crate::faq::{self, FaqItem};
use dioxus::prelude::*;
use rust_i18n::t;

/// FAQ page. Layout mirrors the Impressum/Datenschutz page: a centered
/// `prose` column inside the shared container, with a back link at the bottom.
pub fn Faq() -> Element {
    let items = faq::items();

    rsx! {
        div { class: "container mx-auto p-8",
            div { class: "max-w-4xl mx-auto",
                div { class: "prose m-auto pt-4 px-4 sm:px-6 lg:px-8",
                    h1 { {faq::title()} }
                    p { {faq::intro()} }

                    for (idx, item) in items.iter().enumerate() {
                        match item {
                            FaqItem::Heading(heading) => rsx! {
                                h2 { key: "faq-{idx}", {heading.clone()} }
                            },
                            FaqItem::Entry(entry) => rsx! {
                                div { key: "faq-{idx}",
                                    h3 { {entry.question.clone()} }
                                    p { {entry.answer.clone()} }
                                }
                            },
                        }
                    }
                }
                div { class: "mt-8",
                    Link {
                        to: crate::routes::Route::SplashScreen {},
                        class: "btn btn-primary",
                        {t!("nav.zurueck").to_string()}
                    }
                }
            }
        }
    }
}
