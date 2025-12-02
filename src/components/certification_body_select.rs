use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct CertificationBodySelectProps {
    pub bound_value: Signal<String>,
}

pub fn CertificationBodySelect(mut props: CertificationBodySelectProps) -> Element {
    rsx! {
        select {
            class: "select select-bordered w-full",
            value: props.bound_value.read().clone(),
            onchange: move |e| {
                props.bound_value.set(e.value());
            },
            option { value: "", "Bitte wählen..." }
            option { value: "CH-BIO-006", "CH-BIO-006 (bio.inspecta AG)" }
            option { value: "CH-BIO-086", "CH-BIO-086 (Bio Test Agro AG (BTA))" }
            option { value: "CH-BIO-038", "CH-BIO-038 (ProCert AG)" }
            option { value: "CH-BIO-004", "CH-BIO-004 (Ecocert Swiss AG)" }
        }
    }
}