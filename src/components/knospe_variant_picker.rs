use crate::model::Country;
use dioxus::prelude::*;
use rust_i18n::t;

/// The four Knospe logo variants. The key is the stable identifier used by the
/// setters and by the `bio_labels.*` translation keys. The suffix `_ch` means
/// Swiss origin, the prefix `umstellung` means conversion farm.
pub const KNOSPE_VARIANTS: [&str; 4] = [
    "knospe_ch",
    "knospe_import",
    "umstellung_ch",
    "umstellung_import",
];

/// Which variant is currently selected, derived from the two signals that
/// actually store the state: Swiss origin and Umstellbetrieb.
pub fn knospe_variant_key(origins_have_ch: bool, umstellung: bool) -> &'static str {
    match (origins_have_ch, umstellung) {
        (true, false) => "knospe_ch",
        (false, false) => "knospe_import",
        (true, true) => "umstellung_ch",
        (false, true) => "umstellung_import",
    }
}

/// Does this variant mean Umstellung (conversion farm)?
pub fn variant_is_umstellung(variant: &str) -> bool {
    variant.starts_with("umstellung")
}

/// The origin implied by a variant, given the origins currently selected.
/// Swiss variants force CH; import variants keep any non-CH selection and
/// otherwise fall back to the generic `Import` origin.
pub fn variant_origins(variant: &str, current: Option<Vec<Country>>) -> Option<Vec<Country>> {
    if variant.ends_with("_ch") {
        Some(vec![Country::CH])
    } else {
        current
            .filter(|o| !o.is_empty() && !o.contains(&Country::CH))
            .or(Some(vec![Country::Import]))
    }
}

/// Caption for a variant. The translation keys are spelled out literally so
/// `cargo test --test locale_parity` (and a plain grep) still finds them.
fn variant_label(variant: &str) -> String {
    match variant {
        "knospe_ch" => t!("bio_labels.knospe_ch").to_string(),
        "knospe_import" => t!("bio_labels.knospe_import").to_string(),
        "umstellung_ch" => t!("bio_labels.umstellung_ch").to_string(),
        _ => t!("bio_labels.umstellung_import").to_string(),
    }
}

/// The logo row that picks WHICH Knospe applies: artwork is Knospe mit/ohne
/// Kreuz resp. Umstellungsknospe, the caption carries the Schweiz/Import split
/// (Variante b, Testing 25.06.2026).
///
/// Used for both leaf and composite ingredients; the only difference is the
/// setter passed in, so the markup lives here once.
#[component]
pub fn KnospeVariantPicker(
    /// Currently selected variant key, see `knospe_variant_key`.
    selected: String,
    /// Extra classes for the grid, e.g. spacing that differs per call site.
    #[props(default = String::new())]
    grid_class: String,
    on_select: EventHandler<&'static str>,
) -> Element {
    rsx! {
        div { class: "grid grid-cols-2 sm:grid-cols-4 gap-2 {grid_class}",
            for key in KNOSPE_VARIANTS {
                {
                    let is_selected = selected == key;
                    let umstellung = variant_is_umstellung(key);
                    let ch = key.ends_with("_ch");
                    let label = variant_label(key);
                    rsx! {
                        button {
                            r#type: "button",
                            class: if is_selected { "flex flex-col items-center gap-1 p-2 rounded-lg border-2 border-primary bg-primary/5" } else { "flex flex-col items-center gap-1 p-2 rounded-lg border-2 border-base-300 hover:border-base-content/30" },
                            onclick: move |_| { on_select.call(key); },
                            div { class: "h-16 flex items-center",
                                if umstellung && ch {
                                    crate::components::icons::UmstellungsknospeRegular {}
                                } else if umstellung {
                                    crate::components::icons::UmstellungsknospeNoCross {}
                                } else if ch {
                                    crate::components::icons::BioSuisseRegular {}
                                } else {
                                    crate::components::icons::BioSuisseNoCross {}
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
