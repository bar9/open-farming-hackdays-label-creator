use crate::components::icons::{
    BioSuisseNoCross, BioSuisseRegular, UmstellungsknospeNoCross, UmstellungsknospeRegular,
};
use crate::components::FormField;
use crate::pages::label_page::MonoQuality;
use crate::shared::Configuration;
use dioxus::prelude::*;
use rust_i18n::t;

/// Quality selector for an Einzelzutat/Monoprodukt («Keine Zutatenliste»).
///
/// Without a recipe there is no ingredient pane to declare the quality in, yet
/// the label still depends on it: «Bio» in the Sachbezeichnung, the Knospe logo
/// variant, the Umstellungssatz (DEC-2). This mirrors the leaf-level choice of
/// `IngredientPane`, so users see the same options in both places:
///
/// - Bio-V: Bio / Nicht-biologisch / Nicht-landwirtschaftlich, plus the
///   Umstellbetrieb checkbox under Bio.
/// - Knospe: additionally the four Knospe logo variants (Schweiz/Import ×
///   Knospe/Umstellung), picked by artwork exactly as in the ingredient pane.
#[derive(Props, Clone, PartialEq)]
pub struct MonoQualitySelectProps {
    pub quality: Signal<MonoQuality>,
    pub configuration: Signal<Configuration>,
}

pub fn MonoQualitySelect(mut props: MonoQualitySelectProps) -> Element {
    let is_knospe = matches!(*props.configuration.read(), Configuration::Knospe);
    let current = *props.quality.read();

    // Top-level category. The four Knospe variants collapse into one "knospe"
    // category whose specific logo is picked in the artwork row below, matching
    // the ingredient pane's Variante-b layout.
    let category = match current {
        MonoQuality::KnospeCh
        | MonoQuality::KnospeImport
        | MonoQuality::UmstellungKnospeCh
        | MonoQuality::UmstellungKnospeImport => "knospe",
        MonoQuality::Bio | MonoQuality::BioUmstellung => "bio",
        MonoQuality::NichtLandwirtschaftlich => "nicht_lw",
        MonoQuality::Andere => "andere",
    };

    let mut set_category = move |cat: &str| {
        props.quality.set(match cat {
            // A fresh Knospe choice defaults to the Swiss Knospe, as in the pane.
            "knospe" => MonoQuality::KnospeCh,
            "bio" => MonoQuality::Bio,
            "nicht_lw" => MonoQuality::NichtLandwirtschaftlich,
            _ => MonoQuality::Andere,
        });
    };

    rsx! {
        div { class: "flex flex-col gap-1",
            if is_knospe {
                FormField {
                    help: Some(t!("help.bio_knospe").to_string()),
                    label: t!("bio_labels.bio_knospe").to_string(),
                    inline_checkbox: true,
                    input {
                        r#type: "radio",
                        name: "mono_quality",
                        class: "radio radio-primary",
                        checked: category == "knospe",
                        onchange: move |_| { set_category("knospe"); }
                    }
                }
            }
            FormField {
                help: Some(t!("help.bio_ch").to_string()),
                label: t!("bio_labels.bio_ch").to_string(),
                inline_checkbox: true,
                input {
                    r#type: "radio",
                    name: "mono_quality",
                    class: "radio radio-primary",
                    checked: category == "bio",
                    onchange: move |_| { set_category("bio"); }
                }
            }
            FormField {
                help: Some(t!("help.andere").to_string()),
                label: t!("bio_labels.andere").to_string(),
                inline_checkbox: true,
                input {
                    r#type: "radio",
                    name: "mono_quality",
                    class: "radio radio-primary",
                    checked: category == "andere",
                    onchange: move |_| { set_category("andere"); }
                }
            }
            FormField {
                help: Some(t!("help.nicht_landwirtschaftlich").to_string()),
                label: t!("bio_labels.nicht_landwirtschaftlich").to_string(),
                inline_checkbox: true,
                input {
                    r#type: "radio",
                    name: "mono_quality",
                    class: "radio radio-primary",
                    checked: category == "nicht_lw",
                    onchange: move |_| { set_category("nicht_lw"); }
                }
            }
            // Bio (Bio-V): the conversion-farm case is a checkbox, matching the pane.
            if category == "bio" {
                div { class: "border-t border-base-300 pt-2 mt-2",
                    FormField {
                        help: Some(t!("help.aus_umstellbetrieb").to_string()),
                        label: t!("bio_labels.aus_umstellbetrieb").to_string(),
                        inline_checkbox: true,
                        input {
                            r#type: "checkbox",
                            class: "checkbox checkbox-accent",
                            checked: current == MonoQuality::BioUmstellung,
                            onchange: move |evt| {
                                props.quality.set(if evt.data.value() == "true" {
                                    MonoQuality::BioUmstellung
                                } else {
                                    MonoQuality::Bio
                                });
                            }
                        }
                    }
                }
            }
            // Knospe: pick WHICH Knospe by its artwork, as in the ingredient pane.
            if category == "knospe" {
                div { class: "border-t border-base-300 pt-2 mt-2",
                    div { class: "grid grid-cols-2 sm:grid-cols-4 gap-2 mb-3",
                        for (variant, label) in [
                            (MonoQuality::KnospeCh, t!("bio_labels.knospe_ch").to_string()),
                            (MonoQuality::KnospeImport, t!("bio_labels.knospe_import").to_string()),
                            (MonoQuality::UmstellungKnospeCh, t!("bio_labels.umstellung_ch").to_string()),
                            (MonoQuality::UmstellungKnospeImport, t!("bio_labels.umstellung_import").to_string()),
                        ].into_iter() {
                            {
                                let selected = current == variant;
                                let umstellung = matches!(
                                    variant,
                                    MonoQuality::UmstellungKnospeCh | MonoQuality::UmstellungKnospeImport
                                );
                                let ch = matches!(
                                    variant,
                                    MonoQuality::KnospeCh | MonoQuality::UmstellungKnospeCh
                                );
                                rsx! {
                                    button {
                                        r#type: "button",
                                        class: if selected {
                                            "flex flex-col items-center gap-1 p-2 rounded-lg border-2 border-primary bg-primary/5"
                                        } else {
                                            "flex flex-col items-center gap-1 p-2 rounded-lg border-2 border-base-300 hover:border-base-content/30"
                                        },
                                        onclick: move |_| { props.quality.set(variant); },
                                        div { class: "h-16 flex items-center",
                                            if umstellung && ch {
                                                UmstellungsknospeRegular {}
                                            } else if umstellung {
                                                UmstellungsknospeNoCross {}
                                            } else if ch {
                                                BioSuisseRegular {}
                                            } else {
                                                BioSuisseNoCross {}
                                            }
                                        }
                                        span { class: "text-xs text-center leading-tight font-medium", "{label}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
