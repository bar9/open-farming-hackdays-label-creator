use dioxus::prelude::*;
use rust_i18n::t;

#[derive(Props, Clone, PartialEq)]
pub struct CertificationBodySelectProps {
    pub bound_value: Signal<String>,
}

pub fn CertificationBodySelect(mut props: CertificationBodySelectProps) -> Element {
    rsx! {
        select {
            class: "select select-bordered w-full",
            required: true,
            value: props.bound_value.read().clone(),
            onchange: move |e| {
                props.bound_value.set(e.value());
            },
            option { value: "", {t!("certification_body.please_select")} }
            option { value: "CH-BIO-006", {t!("certification_body.bio_006")} }
            option { value: "CH-BIO-086", {t!("certification_body.bio_086")} }
            option { value: "CH-BIO-038", {t!("certification_body.bio_038")} }
            option { value: "CH-BIO-004", {t!("certification_body.bio_004")} }
        }
    }
}