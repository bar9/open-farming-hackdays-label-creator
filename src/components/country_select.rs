use crate::model::Country;
use dioxus::prelude::*;
use rust_i18n::t;

#[derive(Props, Clone, PartialEq)]
pub struct CountrySelectProps {
    /// Current selected country value
    pub value: Option<Country>,
    /// Callback when selection changes
    pub onchange: EventHandler<Option<Country>>,
    /// Additional CSS classes
    #[props(default = "select select-bordered w-full")]
    pub class: &'static str,
    /// Whether to include all ISO countries or just basic options
    #[props(default = true)]
    pub include_all_countries: bool,
}

/// Countries offered before the full ISO list, for quick access.
const COMMON_COUNTRIES: [Country; 4] = [Country::DE, Country::FR, Country::IT, Country::AT];

pub fn CountrySelect(props: CountrySelectProps) -> Element {
    rsx! {
        select {
            class: "{props.class}",
            value: match props.value.as_ref() {
                Some(country) => format!("{:?}", country),
                None => "".to_string(),
            },
            onchange: move |e| {
                props.onchange.call(Country::from_code(e.value().as_str()));
            },

            // Basic options (always shown)
            option { value: "", selected: props.value.is_none(), {t!("country_select.please_choose").to_string()} }
            option { value: "CH", selected: matches!(props.value.as_ref(), Some(Country::CH)), {Country::CH.localized_name()} }
            option { value: "EU", selected: matches!(props.value.as_ref(), Some(Country::EU)), {Country::EU.localized_name()} }
            option {
                value: "NoOriginRequired",
                selected: matches!(props.value.as_ref(), Some(Country::NoOriginRequired)),
                {Country::NoOriginRequired.localized_name()}
            }

            // Common European countries (always shown for simplified mode)
            for country in COMMON_COUNTRIES {
                option {
                    key: "common-{country.country_code()}",
                    value: "{country.country_code()}",
                    selected: props.value.as_ref() == Some(&country),
                    {country.localized_name()}
                }
            }

            // All ISO countries (only if include_all_countries is true)
            if props.include_all_countries {
                for country in Country::ISO_COUNTRIES {
                    option {
                        key: "iso-{country.country_code()}",
                        value: "{country.country_code()}",
                        selected: props.value.as_ref() == Some(&country),
                        {country.localized_name()}
                    }
                }
            }
        }
    }
}
