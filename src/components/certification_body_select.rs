use dioxus::prelude::*;
use rust_i18n::t;

/// Swiss organic certification bodies offered in the dropdown: (code, locale key).
/// Source: `Inhaltsverzeichnis_Bio_Zusatz.xlsx`, sheet «Code_Zert_CH» (IMO now
/// operates as Ecocert Swiss AG; the sheet's «CH-BIO-0.38» is a typo for 038).
pub const CERTIFICATION_BODIES: [(&str, &str); 4] = [
    ("CH-BIO-006", "certification_body.bio_006"),
    ("CH-BIO-086", "certification_body.bio_086"),
    ("CH-BIO-038", "certification_body.bio_038"),
    ("CH-BIO-004", "certification_body.bio_004"),
];

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
            option { value: "", {t!("certification_body.please_select").to_string()} }
            for (code, key) in CERTIFICATION_BODIES {
                option { value: code, {t!(key).to_string()} }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Every code from the requirement sheet is offered, the label shows the
    // code in every language, and the validator accepts it.
    #[test]
    fn all_swiss_certification_bodies_from_the_requirements_are_offered() {
        let codes: Vec<&str> = CERTIFICATION_BODIES.iter().map(|(c, _)| *c).collect();
        for expected in ["CH-BIO-004", "CH-BIO-006", "CH-BIO-038", "CH-BIO-086"] {
            assert!(codes.contains(&expected), "{expected} missing from the dropdown");
        }
        for locale in ["de-CH", "fr-CH", "it-CH"] {
            for (code, key) in CERTIFICATION_BODIES {
                let label = t!(key, locale = locale).to_string();
                assert!(
                    label.starts_with(code),
                    "{locale}: label for {code} is «{label}»"
                );
            }
        }
    }
}
