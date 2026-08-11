use crate::model::Country;
use dioxus::prelude::*;
use rust_i18n::t;

#[derive(Props, Clone, PartialEq)]
pub struct MultiCountrySelectProps {
    /// Current selected countries
    pub values: Option<Vec<Country>>,
    /// Callback when selection changes
    pub onchange: EventHandler<Option<Vec<Country>>>,
    /// Whether to include all ISO countries or just basic options
    #[props(default = true)]
    pub include_all_countries: bool,
}

/// Regions listed above the country list.
const REGIONS: [Country; 3] = [Country::CH, Country::EU, Country::NoOriginRequired];
/// Countries offered before the full ISO list, for quick access.
const COMMON_COUNTRIES: [Country; 4] = [Country::DE, Country::FR, Country::IT, Country::AT];

pub fn MultiCountrySelect(props: MultiCountrySelectProps) -> Element {
    let values = props.values.clone().unwrap_or_default();

    // Handler to add a new country
    let add_country = {
        let values = values.clone();
        let onchange = props.onchange;
        move |country_str: String| {
            if let Some(country) = Country::from_code(&country_str) {
                let mut new_values = values.clone();
                if !new_values.contains(&country) {
                    new_values.push(country);
                }
                if new_values.is_empty() {
                    onchange.call(None);
                } else {
                    onchange.call(Some(new_values));
                }
            }
        }
    };

    rsx! {
        div { class: "space-y-2",
            // Display selected countries as tags
            if !values.is_empty() {
                div { class: "flex flex-wrap gap-2",
                    for (idx, country) in values.iter().enumerate() {
                        {
                            let country_clone = country.clone();
                            let label = format!("{} {}", country_clone.flag_emoji(), country_clone.localized_name());
                            let values_for_remove = props.values.clone().unwrap_or_default();
                            let onchange_for_remove = props.onchange;
                            rsx! {
                                span {
                                    key: "{idx}",
                                    class: "badge badge-lg gap-1",
                                    "{label}"
                                    span { class: "tooltip tooltip-bottom", "data-tip": t!("origin.remove_country").to_string(),
                                        button {
                                            class: "btn btn-xs btn-ghost btn-circle",
                                            r#type: "button",
                                            onclick: move |_| {
                                                let new_values: Vec<Country> = values_for_remove
                                                    .iter()
                                                    .filter(|c| *c != &country_clone)
                                                    .cloned()
                                                    .collect();
                                                if new_values.is_empty() {
                                                    onchange_for_remove.call(None);
                                                } else {
                                                    onchange_for_remove.call(Some(new_values));
                                                }
                                            },
                                            "×"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Dropdown to add new country
            select {
                class: "select select-bordered w-full",
                value: "",
                onchange: move |e| {
                    add_country(e.value());
                },

                option { value: "", {t!("origin.add_country").to_string()} }

                // Regions section
                optgroup { label: "{t!(\"origin.regions_header\").to_string()}",
                    for country in REGIONS {
                        option {
                            key: "region-{country.country_code()}",
                            value: match country { Country::NoOriginRequired => "NoOriginRequired", c => c.country_code() },
                            {country.localized_name()}
                        }
                    }
                }

                // Common European countries
                optgroup { label: "{t!(\"origin.countries_header\").to_string()}",
                    for country in COMMON_COUNTRIES {
                        option {
                            key: "common-{country.country_code()}",
                            value: "{country.country_code()}",
                            {country.localized_name()}
                        }
                    }

                    // All ISO countries (only if include_all_countries is true)
                    if props.include_all_countries {
                        for country in Country::ISO_COUNTRIES {
                            option {
                                key: "iso-{country.country_code()}",
                                value: "{country.country_code()}",
                                {country.localized_name()}
                            }
                        }
                    }
                }
            }
        }
    }
}
