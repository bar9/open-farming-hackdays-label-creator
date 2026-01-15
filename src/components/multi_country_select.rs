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

pub fn MultiCountrySelect(props: MultiCountrySelectProps) -> Element {
    let values = props.values.clone().unwrap_or_default();

    // Handler to add a new country
    let add_country = {
        let values = values.clone();
        let onchange = props.onchange.clone();
        move |country_str: String| {
            let country = match country_str.as_str() {
                "" => None,
                "CH" => Some(Country::CH),
                "EU" => Some(Country::EU),
                "NoOriginRequired" => Some(Country::NoOriginRequired),
                "AD" => Some(Country::AD), "AE" => Some(Country::AE), "AF" => Some(Country::AF),
                "AG" => Some(Country::AG), "AI" => Some(Country::AI), "AL" => Some(Country::AL),
                "AM" => Some(Country::AM), "AO" => Some(Country::AO), "AQ" => Some(Country::AQ),
                "AR" => Some(Country::AR), "AS" => Some(Country::AS), "AT" => Some(Country::AT),
                "AU" => Some(Country::AU), "AW" => Some(Country::AW), "AX" => Some(Country::AX),
                "AZ" => Some(Country::AZ), "BA" => Some(Country::BA), "BB" => Some(Country::BB),
                "BD" => Some(Country::BD), "BE" => Some(Country::BE), "BF" => Some(Country::BF),
                "BG" => Some(Country::BG), "BH" => Some(Country::BH), "BI" => Some(Country::BI),
                "BJ" => Some(Country::BJ), "BL" => Some(Country::BL), "BM" => Some(Country::BM),
                "BN" => Some(Country::BN), "BO" => Some(Country::BO), "BQ" => Some(Country::BQ),
                "BR" => Some(Country::BR), "BS" => Some(Country::BS), "BT" => Some(Country::BT),
                "BV" => Some(Country::BV), "BW" => Some(Country::BW), "BY" => Some(Country::BY),
                "BZ" => Some(Country::BZ), "CA" => Some(Country::CA), "CC" => Some(Country::CC),
                "CD" => Some(Country::CD), "CF" => Some(Country::CF), "CG" => Some(Country::CG),
                "CI" => Some(Country::CI), "CK" => Some(Country::CK), "CL" => Some(Country::CL),
                "CM" => Some(Country::CM), "CN" => Some(Country::CN), "CO" => Some(Country::CO),
                "CR" => Some(Country::CR), "CU" => Some(Country::CU), "CV" => Some(Country::CV),
                "CW" => Some(Country::CW), "CX" => Some(Country::CX), "CY" => Some(Country::CY),
                "CZ" => Some(Country::CZ), "DE" => Some(Country::DE), "DJ" => Some(Country::DJ),
                "DK" => Some(Country::DK), "DM" => Some(Country::DM), "DO" => Some(Country::DO),
                "DZ" => Some(Country::DZ), "EC" => Some(Country::EC), "EE" => Some(Country::EE),
                "EG" => Some(Country::EG), "EH" => Some(Country::EH), "ER" => Some(Country::ER),
                "ES" => Some(Country::ES), "ET" => Some(Country::ET), "FI" => Some(Country::FI),
                "FJ" => Some(Country::FJ), "FK" => Some(Country::FK), "FM" => Some(Country::FM),
                "FO" => Some(Country::FO), "FR" => Some(Country::FR), "GA" => Some(Country::GA),
                "GB" => Some(Country::GB), "GD" => Some(Country::GD), "GE" => Some(Country::GE),
                "GF" => Some(Country::GF), "GG" => Some(Country::GG), "GH" => Some(Country::GH),
                "GI" => Some(Country::GI), "GL" => Some(Country::GL), "GM" => Some(Country::GM),
                "GN" => Some(Country::GN), "GP" => Some(Country::GP), "GQ" => Some(Country::GQ),
                "GR" => Some(Country::GR), "GS" => Some(Country::GS), "GT" => Some(Country::GT),
                "GU" => Some(Country::GU), "GW" => Some(Country::GW), "GY" => Some(Country::GY),
                "HK" => Some(Country::HK), "HM" => Some(Country::HM), "HN" => Some(Country::HN),
                "HR" => Some(Country::HR), "HT" => Some(Country::HT), "HU" => Some(Country::HU),
                "ID" => Some(Country::ID), "IE" => Some(Country::IE), "IL" => Some(Country::IL),
                "IM" => Some(Country::IM), "IN" => Some(Country::IN), "IO" => Some(Country::IO),
                "IQ" => Some(Country::IQ), "IR" => Some(Country::IR), "IS" => Some(Country::IS),
                "IT" => Some(Country::IT), "JE" => Some(Country::JE), "JM" => Some(Country::JM),
                "JO" => Some(Country::JO), "JP" => Some(Country::JP), "KE" => Some(Country::KE),
                "KG" => Some(Country::KG), "KH" => Some(Country::KH), "KI" => Some(Country::KI),
                "KM" => Some(Country::KM), "KN" => Some(Country::KN), "KP" => Some(Country::KP),
                "KR" => Some(Country::KR), "KW" => Some(Country::KW), "KY" => Some(Country::KY),
                "KZ" => Some(Country::KZ), "LA" => Some(Country::LA), "LB" => Some(Country::LB),
                "LC" => Some(Country::LC), "LI" => Some(Country::LI), "LK" => Some(Country::LK),
                "LR" => Some(Country::LR), "LS" => Some(Country::LS), "LT" => Some(Country::LT),
                "LU" => Some(Country::LU), "LV" => Some(Country::LV), "LY" => Some(Country::LY),
                "MA" => Some(Country::MA), "MC" => Some(Country::MC), "MD" => Some(Country::MD),
                "ME" => Some(Country::ME), "MF" => Some(Country::MF), "MG" => Some(Country::MG),
                "MH" => Some(Country::MH), "MK" => Some(Country::MK), "ML" => Some(Country::ML),
                "MM" => Some(Country::MM), "MN" => Some(Country::MN), "MO" => Some(Country::MO),
                "MP" => Some(Country::MP), "MQ" => Some(Country::MQ), "MR" => Some(Country::MR),
                "MS" => Some(Country::MS), "MT" => Some(Country::MT), "MU" => Some(Country::MU),
                "MV" => Some(Country::MV), "MW" => Some(Country::MW), "MX" => Some(Country::MX),
                "MY" => Some(Country::MY), "MZ" => Some(Country::MZ), "NA" => Some(Country::NA),
                "NC" => Some(Country::NC), "NE" => Some(Country::NE), "NF" => Some(Country::NF),
                "NG" => Some(Country::NG), "NI" => Some(Country::NI), "NL" => Some(Country::NL),
                "NO" => Some(Country::NO), "NP" => Some(Country::NP), "NR" => Some(Country::NR),
                "NU" => Some(Country::NU), "NZ" => Some(Country::NZ), "OM" => Some(Country::OM),
                "PA" => Some(Country::PA), "PE" => Some(Country::PE), "PF" => Some(Country::PF),
                "PG" => Some(Country::PG), "PH" => Some(Country::PH), "PK" => Some(Country::PK),
                "PL" => Some(Country::PL), "PM" => Some(Country::PM), "PN" => Some(Country::PN),
                "PR" => Some(Country::PR), "PS" => Some(Country::PS), "PT" => Some(Country::PT),
                "PW" => Some(Country::PW), "PY" => Some(Country::PY), "QA" => Some(Country::QA),
                "RE" => Some(Country::RE), "RO" => Some(Country::RO), "RS" => Some(Country::RS),
                "RU" => Some(Country::RU), "RW" => Some(Country::RW), "SA" => Some(Country::SA),
                "SB" => Some(Country::SB), "SC" => Some(Country::SC), "SD" => Some(Country::SD),
                "SE" => Some(Country::SE), "SG" => Some(Country::SG), "SH" => Some(Country::SH),
                "SI" => Some(Country::SI), "SJ" => Some(Country::SJ), "SK" => Some(Country::SK),
                "SL" => Some(Country::SL), "SM" => Some(Country::SM), "SN" => Some(Country::SN),
                "SO" => Some(Country::SO), "SR" => Some(Country::SR), "SS" => Some(Country::SS),
                "ST" => Some(Country::ST), "SV" => Some(Country::SV), "SX" => Some(Country::SX),
                "SY" => Some(Country::SY), "SZ" => Some(Country::SZ), "TC" => Some(Country::TC),
                "TD" => Some(Country::TD), "TF" => Some(Country::TF), "TG" => Some(Country::TG),
                "TH" => Some(Country::TH), "TJ" => Some(Country::TJ), "TK" => Some(Country::TK),
                "TL" => Some(Country::TL), "TM" => Some(Country::TM), "TN" => Some(Country::TN),
                "TO" => Some(Country::TO), "TR" => Some(Country::TR), "TT" => Some(Country::TT),
                "TV" => Some(Country::TV), "TW" => Some(Country::TW), "TZ" => Some(Country::TZ),
                "UA" => Some(Country::UA), "UG" => Some(Country::UG), "UM" => Some(Country::UM),
                "US" => Some(Country::US), "UY" => Some(Country::UY), "UZ" => Some(Country::UZ),
                "VA" => Some(Country::VA), "VC" => Some(Country::VC), "VE" => Some(Country::VE),
                "VG" => Some(Country::VG), "VI" => Some(Country::VI), "VN" => Some(Country::VN),
                "VU" => Some(Country::VU), "WF" => Some(Country::WF), "WS" => Some(Country::WS),
                "YE" => Some(Country::YE), "YT" => Some(Country::YT), "ZA" => Some(Country::ZA),
                "ZM" => Some(Country::ZM), "ZW" => Some(Country::ZW),
                _ => None,
            };

            if let Some(c) = country {
                let mut new_values = values.clone();
                // Only add if not already present
                if !new_values.contains(&c) {
                    new_values.push(c);
                    if new_values.is_empty() {
                        onchange.call(None);
                    } else {
                        onchange.call(Some(new_values));
                    }
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
                            let values_for_remove = props.values.clone().unwrap_or_default();
                            let onchange_for_remove = props.onchange.clone();
                            rsx! {
                                span {
                                    key: "{idx}",
                                    class: "badge badge-lg gap-1",
                                    "{country_clone.flag_emoji()} {country_clone.display_name()}"
                                    button {
                                        class: "btn btn-xs btn-ghost btn-circle",
                                        r#type: "button",
                                        title: "{t!(\"origin.remove_country\")}",
                                        onclick: move |_| {
                                            let mut new_values: Vec<Country> = values_for_remove
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

            // Dropdown to add new country
            select {
                class: "select select-bordered w-full",
                value: "",
                onchange: move |e| {
                    add_country(e.value());
                },

                option { value: "", "{t!(\"origin.add_country\")}" }

                // Regions section
                optgroup { label: "{t!(\"origin.regions_header\")}",
                    option { value: "CH", "Schweiz" }
                    option { value: "EU", "EU" }
                    option { value: "NoOriginRequired", "Keine Herkunftsangabe benötigt" }
                }

                // Common European countries
                optgroup { label: "{t!(\"origin.countries_header\")}",
                    option { value: "DE", "Deutschland" }
                    option { value: "FR", "Frankreich" }
                    option { value: "IT", "Italien" }
                    option { value: "AT", "Österreich" }

                    // All ISO countries (only if include_all_countries is true)
                    if props.include_all_countries {
                        option { value: "AD", "Andorra" }
                        option { value: "AE", "Vereinigte Arabische Emirate" }
                        option { value: "AF", "Afghanistan" }
                        option { value: "AG", "Antigua und Barbuda" }
                        option { value: "AI", "Anguilla" }
                        option { value: "AL", "Albanien" }
                        option { value: "AM", "Armenien" }
                        option { value: "AO", "Angola" }
                        option { value: "AQ", "Antarktis" }
                        option { value: "AR", "Argentinien" }
                        option { value: "AS", "Amerikanisch-Samoa" }
                        option { value: "AU", "Australien" }
                        option { value: "AW", "Aruba" }
                        option { value: "AX", "Åland" }
                        option { value: "AZ", "Aserbaidschan" }
                        option { value: "BA", "Bosnien und Herzegowina" }
                        option { value: "BB", "Barbados" }
                        option { value: "BD", "Bangladesch" }
                        option { value: "BE", "Belgien" }
                        option { value: "BF", "Burkina Faso" }
                        option { value: "BG", "Bulgarien" }
                        option { value: "BH", "Bahrain" }
                        option { value: "BI", "Burundi" }
                        option { value: "BJ", "Benin" }
                        option { value: "BL", "Saint-Barthélemy" }
                        option { value: "BM", "Bermuda" }
                        option { value: "BN", "Brunei" }
                        option { value: "BO", "Bolivien" }
                        option { value: "BQ", "Bonaire, Sint Eustatius und Saba" }
                        option { value: "BR", "Brasilien" }
                        option { value: "BS", "Bahamas" }
                        option { value: "BT", "Bhutan" }
                        option { value: "BV", "Bouvetinsel" }
                        option { value: "BW", "Botswana" }
                        option { value: "BY", "Belarus" }
                        option { value: "BZ", "Belize" }
                        option { value: "CA", "Kanada" }
                        option { value: "CC", "Kokosinseln" }
                        option { value: "CD", "Demokratische Republik Kongo" }
                        option { value: "CF", "Zentralafrikanische Republik" }
                        option { value: "CG", "Republik Kongo" }
                        option { value: "CI", "Elfenbeinküste" }
                        option { value: "CK", "Cookinseln" }
                        option { value: "CL", "Chile" }
                        option { value: "CM", "Kamerun" }
                        option { value: "CN", "China" }
                        option { value: "CO", "Kolumbien" }
                        option { value: "CR", "Costa Rica" }
                        option { value: "CU", "Kuba" }
                        option { value: "CV", "Kap Verde" }
                        option { value: "CW", "Curaçao" }
                        option { value: "CX", "Weihnachtsinsel" }
                        option { value: "CY", "Zypern" }
                        option { value: "CZ", "Tschechien" }
                        option { value: "DJ", "Dschibuti" }
                        option { value: "DK", "Dänemark" }
                        option { value: "DM", "Dominica" }
                        option { value: "DO", "Dominikanische Republik" }
                        option { value: "DZ", "Algerien" }
                        option { value: "EC", "Ecuador" }
                        option { value: "EE", "Estland" }
                        option { value: "EG", "Ägypten" }
                        option { value: "EH", "Westsahara" }
                        option { value: "ER", "Eritrea" }
                        option { value: "ES", "Spanien" }
                        option { value: "ET", "Äthiopien" }
                        option { value: "FI", "Finnland" }
                        option { value: "FJ", "Fidschi" }
                        option { value: "FK", "Falklandinseln" }
                        option { value: "FM", "Mikronesien" }
                        option { value: "FO", "Färöer" }
                        option { value: "GA", "Gabun" }
                        option { value: "GB", "Vereinigtes Königreich" }
                        option { value: "GD", "Grenada" }
                        option { value: "GE", "Georgien" }
                        option { value: "GF", "Französisch-Guayana" }
                        option { value: "GG", "Guernsey" }
                        option { value: "GH", "Ghana" }
                        option { value: "GI", "Gibraltar" }
                        option { value: "GL", "Grönland" }
                        option { value: "GM", "Gambia" }
                        option { value: "GN", "Guinea" }
                        option { value: "GP", "Guadeloupe" }
                        option { value: "GQ", "Äquatorialguinea" }
                        option { value: "GR", "Griechenland" }
                        option { value: "GS", "Südgeorgien und die Südlichen Sandwichinseln" }
                        option { value: "GT", "Guatemala" }
                        option { value: "GU", "Guam" }
                        option { value: "GW", "Guinea-Bissau" }
                        option { value: "GY", "Guyana" }
                        option { value: "HK", "Hongkong" }
                        option { value: "HM", "Heard und McDonaldinseln" }
                        option { value: "HN", "Honduras" }
                        option { value: "HR", "Kroatien" }
                        option { value: "HT", "Haiti" }
                        option { value: "HU", "Ungarn" }
                        option { value: "ID", "Indonesien" }
                        option { value: "IE", "Irland" }
                        option { value: "IL", "Israel" }
                        option { value: "IM", "Isle of Man" }
                        option { value: "IN", "Indien" }
                        option { value: "IO", "Britisches Territorium im Indischen Ozean" }
                        option { value: "IQ", "Irak" }
                        option { value: "IR", "Iran" }
                        option { value: "IS", "Island" }
                        option { value: "JE", "Jersey" }
                        option { value: "JM", "Jamaika" }
                        option { value: "JO", "Jordanien" }
                        option { value: "JP", "Japan" }
                        option { value: "KE", "Kenia" }
                        option { value: "KG", "Kirgisistan" }
                        option { value: "KH", "Kambodscha" }
                        option { value: "KI", "Kiribati" }
                        option { value: "KM", "Komoren" }
                        option { value: "KN", "St. Kitts und Nevis" }
                        option { value: "KP", "Nordkorea" }
                        option { value: "KR", "Südkorea" }
                        option { value: "KW", "Kuwait" }
                        option { value: "KY", "Kaimaninseln" }
                        option { value: "KZ", "Kasachstan" }
                        option { value: "LA", "Laos" }
                        option { value: "LB", "Libanon" }
                        option { value: "LC", "St. Lucia" }
                        option { value: "LI", "Liechtenstein" }
                        option { value: "LK", "Sri Lanka" }
                        option { value: "LR", "Liberia" }
                        option { value: "LS", "Lesotho" }
                        option { value: "LT", "Litauen" }
                        option { value: "LU", "Luxemburg" }
                        option { value: "LV", "Lettland" }
                        option { value: "LY", "Libyen" }
                        option { value: "MA", "Marokko" }
                        option { value: "MC", "Monaco" }
                        option { value: "MD", "Moldau" }
                        option { value: "ME", "Montenegro" }
                        option { value: "MF", "Saint-Martin" }
                        option { value: "MG", "Madagaskar" }
                        option { value: "MH", "Marshallinseln" }
                        option { value: "MK", "Nordmazedonien" }
                        option { value: "ML", "Mali" }
                        option { value: "MM", "Myanmar" }
                        option { value: "MN", "Mongolei" }
                        option { value: "MO", "Macau" }
                        option { value: "MP", "Nördliche Marianen" }
                        option { value: "MQ", "Martinique" }
                        option { value: "MR", "Mauretanien" }
                        option { value: "MS", "Montserrat" }
                        option { value: "MT", "Malta" }
                        option { value: "MU", "Mauritius" }
                        option { value: "MV", "Malediven" }
                        option { value: "MW", "Malawi" }
                        option { value: "MX", "Mexiko" }
                        option { value: "MY", "Malaysia" }
                        option { value: "MZ", "Mosambik" }
                        option { value: "NA", "Namibia" }
                        option { value: "NC", "Neukaledonien" }
                        option { value: "NE", "Niger" }
                        option { value: "NF", "Norfolkinsel" }
                        option { value: "NG", "Nigeria" }
                        option { value: "NI", "Nicaragua" }
                        option { value: "NL", "Niederlande" }
                        option { value: "NO", "Norwegen" }
                        option { value: "NP", "Nepal" }
                        option { value: "NR", "Nauru" }
                        option { value: "NU", "Niue" }
                        option { value: "NZ", "Neuseeland" }
                        option { value: "OM", "Oman" }
                        option { value: "PA", "Panama" }
                        option { value: "PE", "Peru" }
                        option { value: "PF", "Französisch-Polynesien" }
                        option { value: "PG", "Papua-Neuguinea" }
                        option { value: "PH", "Philippinen" }
                        option { value: "PK", "Pakistan" }
                        option { value: "PL", "Polen" }
                        option { value: "PM", "Saint-Pierre und Miquelon" }
                        option { value: "PN", "Pitcairninseln" }
                        option { value: "PR", "Puerto Rico" }
                        option { value: "PS", "Palästina" }
                        option { value: "PT", "Portugal" }
                        option { value: "PW", "Palau" }
                        option { value: "PY", "Paraguay" }
                        option { value: "QA", "Katar" }
                        option { value: "RE", "Réunion" }
                        option { value: "RO", "Rumänien" }
                        option { value: "RS", "Serbien" }
                        option { value: "RU", "Russland" }
                        option { value: "RW", "Ruanda" }
                        option { value: "SA", "Saudi-Arabien" }
                        option { value: "SB", "Salomonen" }
                        option { value: "SC", "Seychellen" }
                        option { value: "SD", "Sudan" }
                        option { value: "SE", "Schweden" }
                        option { value: "SG", "Singapur" }
                        option { value: "SH", "St. Helena" }
                        option { value: "SI", "Slowenien" }
                        option { value: "SJ", "Svalbard und Jan Mayen" }
                        option { value: "SK", "Slowakei" }
                        option { value: "SL", "Sierra Leone" }
                        option { value: "SM", "San Marino" }
                        option { value: "SN", "Senegal" }
                        option { value: "SO", "Somalia" }
                        option { value: "SR", "Suriname" }
                        option { value: "SS", "Südsudan" }
                        option { value: "ST", "São Tomé und Príncipe" }
                        option { value: "SV", "El Salvador" }
                        option { value: "SX", "Sint Maarten" }
                        option { value: "SY", "Syrien" }
                        option { value: "SZ", "Eswatini" }
                        option { value: "TC", "Turks- und Caicosinseln" }
                        option { value: "TD", "Tschad" }
                        option { value: "TF", "Französische Süd- und Antarktisgebiete" }
                        option { value: "TG", "Togo" }
                        option { value: "TH", "Thailand" }
                        option { value: "TJ", "Tadschikistan" }
                        option { value: "TK", "Tokelau" }
                        option { value: "TL", "Osttimor" }
                        option { value: "TM", "Turkmenistan" }
                        option { value: "TN", "Tunesien" }
                        option { value: "TO", "Tonga" }
                        option { value: "TR", "Türkei" }
                        option { value: "TT", "Trinidad und Tobago" }
                        option { value: "TV", "Tuvalu" }
                        option { value: "TW", "Taiwan" }
                        option { value: "TZ", "Tansania" }
                        option { value: "UA", "Ukraine" }
                        option { value: "UG", "Uganda" }
                        option { value: "UM", "Amerikanische Überseeinseln" }
                        option { value: "US", "Vereinigte Staaten" }
                        option { value: "UY", "Uruguay" }
                        option { value: "UZ", "Usbekistan" }
                        option { value: "VA", "Vatikanstadt" }
                        option { value: "VC", "St. Vincent und die Grenadinen" }
                        option { value: "VE", "Venezuela" }
                        option { value: "VG", "Britische Jungferninseln" }
                        option { value: "VI", "Amerikanische Jungferninseln" }
                        option { value: "VN", "Vietnam" }
                        option { value: "VU", "Vanuatu" }
                        option { value: "WF", "Wallis und Futuna" }
                        option { value: "WS", "Samoa" }
                        option { value: "YE", "Jemen" }
                        option { value: "YT", "Mayotte" }
                        option { value: "ZA", "Südafrika" }
                        option { value: "ZM", "Sambia" }
                        option { value: "ZW", "Simbabwe" }
                    }
                }
            }
        }
    }
}
