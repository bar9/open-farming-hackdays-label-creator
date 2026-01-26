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

pub fn CountrySelect(props: CountrySelectProps) -> Element {
    rsx! {
        select {
            class: "{props.class}",
            value: match props.value.as_ref() {
                Some(country) => format!("{:?}", country),
                None => "".to_string(),
            },
            onchange: move |e| {
                let country = match e.value().as_str() {
                    "" => None,
                    "CH" => Some(Country::CH),
                    "EU" => Some(Country::EU),
                    "NoOriginRequired" => Some(Country::NoOriginRequired),
                    // All ISO countries
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
                props.onchange.call(country);
            },

            // Basic options (always shown)
            option { value: "", selected: props.value.is_none(), {t!("country_select.please_choose").to_string()} }
            option { value: "CH", selected: matches!(props.value.as_ref(), Some(Country::CH)), "Schweiz" }
            option { value: "EU", selected: matches!(props.value.as_ref(), Some(Country::EU)), "EU" }
            option { value: "NoOriginRequired", selected: matches!(props.value.as_ref(), Some(Country::NoOriginRequired)), "Keine Herkunftsangabe benötigt" }

            // Common European countries (always shown for simplified mode)
            option { value: "DE", selected: matches!(props.value.as_ref(), Some(Country::DE)), "Deutschland" }
            option { value: "FR", selected: matches!(props.value.as_ref(), Some(Country::FR)), "Frankreich" }
            option { value: "IT", selected: matches!(props.value.as_ref(), Some(Country::IT)), "Italien" }
            option { value: "AT", selected: matches!(props.value.as_ref(), Some(Country::AT)), "Österreich" }

            // All ISO countries (only if include_all_countries is true)
            if props.include_all_countries {
                option { value: "AD", selected: matches!(props.value.as_ref(), Some(Country::AD)), "Andorra" }
                option { value: "AE", selected: matches!(props.value.as_ref(), Some(Country::AE)), "Vereinigte Arabische Emirate" }
                option { value: "AF", selected: matches!(props.value.as_ref(), Some(Country::AF)), "Afghanistan" }
                option { value: "AG", selected: matches!(props.value.as_ref(), Some(Country::AG)), "Antigua und Barbuda" }
                option { value: "AI", selected: matches!(props.value.as_ref(), Some(Country::AI)), "Anguilla" }
                option { value: "AL", selected: matches!(props.value.as_ref(), Some(Country::AL)), "Albanien" }
                option { value: "AM", selected: matches!(props.value.as_ref(), Some(Country::AM)), "Armenien" }
                option { value: "AO", selected: matches!(props.value.as_ref(), Some(Country::AO)), "Angola" }
                option { value: "AQ", selected: matches!(props.value.as_ref(), Some(Country::AQ)), "Antarktis" }
                option { value: "AR", selected: matches!(props.value.as_ref(), Some(Country::AR)), "Argentinien" }
                option { value: "AS", selected: matches!(props.value.as_ref(), Some(Country::AS)), "Amerikanisch-Samoa" }
                option { value: "AT", selected: matches!(props.value.as_ref(), Some(Country::AT)), "Österreich" }
                option { value: "AU", selected: matches!(props.value.as_ref(), Some(Country::AU)), "Australien" }
                option { value: "AW", selected: matches!(props.value.as_ref(), Some(Country::AW)), "Aruba" }
                option { value: "AX", selected: matches!(props.value.as_ref(), Some(Country::AX)), "Åland" }
                option { value: "AZ", selected: matches!(props.value.as_ref(), Some(Country::AZ)), "Aserbaidschan" }
                option { value: "BA", selected: matches!(props.value.as_ref(), Some(Country::BA)), "Bosnien und Herzegowina" }
                option { value: "BB", selected: matches!(props.value.as_ref(), Some(Country::BB)), "Barbados" }
                option { value: "BD", selected: matches!(props.value.as_ref(), Some(Country::BD)), "Bangladesch" }
                option { value: "BE", selected: matches!(props.value.as_ref(), Some(Country::BE)), "Belgien" }
                option { value: "BF", selected: matches!(props.value.as_ref(), Some(Country::BF)), "Burkina Faso" }
                option { value: "BG", selected: matches!(props.value.as_ref(), Some(Country::BG)), "Bulgarien" }
                option { value: "BH", selected: matches!(props.value.as_ref(), Some(Country::BH)), "Bahrain" }
                option { value: "BI", selected: matches!(props.value.as_ref(), Some(Country::BI)), "Burundi" }
                option { value: "BJ", selected: matches!(props.value.as_ref(), Some(Country::BJ)), "Benin" }
                option { value: "BL", selected: matches!(props.value.as_ref(), Some(Country::BL)), "Saint-Barthélemy" }
                option { value: "BM", selected: matches!(props.value.as_ref(), Some(Country::BM)), "Bermuda" }
                option { value: "BN", selected: matches!(props.value.as_ref(), Some(Country::BN)), "Brunei" }
                option { value: "BO", selected: matches!(props.value.as_ref(), Some(Country::BO)), "Bolivien" }
                option { value: "BQ", selected: matches!(props.value.as_ref(), Some(Country::BQ)), "Bonaire, Sint Eustatius und Saba" }
                option { value: "BR", selected: matches!(props.value.as_ref(), Some(Country::BR)), "Brasilien" }
                option { value: "BS", selected: matches!(props.value.as_ref(), Some(Country::BS)), "Bahamas" }
                option { value: "BT", selected: matches!(props.value.as_ref(), Some(Country::BT)), "Bhutan" }
                option { value: "BV", selected: matches!(props.value.as_ref(), Some(Country::BV)), "Bouvetinsel" }
                option { value: "BW", selected: matches!(props.value.as_ref(), Some(Country::BW)), "Botswana" }
                option { value: "BY", selected: matches!(props.value.as_ref(), Some(Country::BY)), "Belarus" }
                option { value: "BZ", selected: matches!(props.value.as_ref(), Some(Country::BZ)), "Belize" }
                option { value: "CA", selected: matches!(props.value.as_ref(), Some(Country::CA)), "Kanada" }
                option { value: "CC", selected: matches!(props.value.as_ref(), Some(Country::CC)), "Kokosinseln" }
                option { value: "CD", selected: matches!(props.value.as_ref(), Some(Country::CD)), "Demokratische Republik Kongo" }
                option { value: "CF", selected: matches!(props.value.as_ref(), Some(Country::CF)), "Zentralafrikanische Republik" }
                option { value: "CG", selected: matches!(props.value.as_ref(), Some(Country::CG)), "Republik Kongo" }
                option { value: "CI", selected: matches!(props.value.as_ref(), Some(Country::CI)), "Elfenbeinküste" }
                option { value: "CK", selected: matches!(props.value.as_ref(), Some(Country::CK)), "Cookinseln" }
                option { value: "CL", selected: matches!(props.value.as_ref(), Some(Country::CL)), "Chile" }
                option { value: "CM", selected: matches!(props.value.as_ref(), Some(Country::CM)), "Kamerun" }
                option { value: "CN", selected: matches!(props.value.as_ref(), Some(Country::CN)), "China" }
                option { value: "CO", selected: matches!(props.value.as_ref(), Some(Country::CO)), "Kolumbien" }
                option { value: "CR", selected: matches!(props.value.as_ref(), Some(Country::CR)), "Costa Rica" }
                option { value: "CU", selected: matches!(props.value.as_ref(), Some(Country::CU)), "Kuba" }
                option { value: "CV", selected: matches!(props.value.as_ref(), Some(Country::CV)), "Kap Verde" }
                option { value: "CW", selected: matches!(props.value.as_ref(), Some(Country::CW)), "Curaçao" }
                option { value: "CX", selected: matches!(props.value.as_ref(), Some(Country::CX)), "Weihnachtsinsel" }
                option { value: "CY", selected: matches!(props.value.as_ref(), Some(Country::CY)), "Zypern" }
                option { value: "CZ", selected: matches!(props.value.as_ref(), Some(Country::CZ)), "Tschechien" }
                option { value: "DE", selected: matches!(props.value.as_ref(), Some(Country::DE)), "Deutschland" }
                option { value: "DJ", selected: matches!(props.value.as_ref(), Some(Country::DJ)), "Dschibuti" }
                option { value: "DK", selected: matches!(props.value.as_ref(), Some(Country::DK)), "Dänemark" }
                option { value: "DM", selected: matches!(props.value.as_ref(), Some(Country::DM)), "Dominica" }
                option { value: "DO", selected: matches!(props.value.as_ref(), Some(Country::DO)), "Dominikanische Republik" }
                option { value: "DZ", selected: matches!(props.value.as_ref(), Some(Country::DZ)), "Algerien" }
                option { value: "EC", selected: matches!(props.value.as_ref(), Some(Country::EC)), "Ecuador" }
                option { value: "EE", selected: matches!(props.value.as_ref(), Some(Country::EE)), "Estland" }
                option { value: "EG", selected: matches!(props.value.as_ref(), Some(Country::EG)), "Ägypten" }
                option { value: "EH", selected: matches!(props.value.as_ref(), Some(Country::EH)), "Westsahara" }
                option { value: "ER", selected: matches!(props.value.as_ref(), Some(Country::ER)), "Eritrea" }
                option { value: "ES", selected: matches!(props.value.as_ref(), Some(Country::ES)), "Spanien" }
                option { value: "ET", selected: matches!(props.value.as_ref(), Some(Country::ET)), "Äthiopien" }
                option { value: "FI", selected: matches!(props.value.as_ref(), Some(Country::FI)), "Finnland" }
                option { value: "FJ", selected: matches!(props.value.as_ref(), Some(Country::FJ)), "Fidschi" }
                option { value: "FK", selected: matches!(props.value.as_ref(), Some(Country::FK)), "Falklandinseln" }
                option { value: "FM", selected: matches!(props.value.as_ref(), Some(Country::FM)), "Mikronesien" }
                option { value: "FO", selected: matches!(props.value.as_ref(), Some(Country::FO)), "Färöer" }
                option { value: "FR", selected: matches!(props.value.as_ref(), Some(Country::FR)), "Frankreich" }
                option { value: "GA", selected: matches!(props.value.as_ref(), Some(Country::GA)), "Gabun" }
                option { value: "GB", selected: matches!(props.value.as_ref(), Some(Country::GB)), "Vereinigtes Königreich" }
                option { value: "GD", selected: matches!(props.value.as_ref(), Some(Country::GD)), "Grenada" }
                option { value: "GE", selected: matches!(props.value.as_ref(), Some(Country::GE)), "Georgien" }
                option { value: "GF", selected: matches!(props.value.as_ref(), Some(Country::GF)), "Französisch-Guayana" }
                option { value: "GG", selected: matches!(props.value.as_ref(), Some(Country::GG)), "Guernsey" }
                option { value: "GH", selected: matches!(props.value.as_ref(), Some(Country::GH)), "Ghana" }
                option { value: "GI", selected: matches!(props.value.as_ref(), Some(Country::GI)), "Gibraltar" }
                option { value: "GL", selected: matches!(props.value.as_ref(), Some(Country::GL)), "Grönland" }
                option { value: "GM", selected: matches!(props.value.as_ref(), Some(Country::GM)), "Gambia" }
                option { value: "GN", selected: matches!(props.value.as_ref(), Some(Country::GN)), "Guinea" }
                option { value: "GP", selected: matches!(props.value.as_ref(), Some(Country::GP)), "Guadeloupe" }
                option { value: "GQ", selected: matches!(props.value.as_ref(), Some(Country::GQ)), "Äquatorialguinea" }
                option { value: "GR", selected: matches!(props.value.as_ref(), Some(Country::GR)), "Griechenland" }
                option { value: "GS", selected: matches!(props.value.as_ref(), Some(Country::GS)), "Südgeorgien und die Südlichen Sandwichinseln" }
                option { value: "GT", selected: matches!(props.value.as_ref(), Some(Country::GT)), "Guatemala" }
                option { value: "GU", selected: matches!(props.value.as_ref(), Some(Country::GU)), "Guam" }
                option { value: "GW", selected: matches!(props.value.as_ref(), Some(Country::GW)), "Guinea-Bissau" }
                option { value: "GY", selected: matches!(props.value.as_ref(), Some(Country::GY)), "Guyana" }
                option { value: "HK", selected: matches!(props.value.as_ref(), Some(Country::HK)), "Hongkong" }
                option { value: "HM", selected: matches!(props.value.as_ref(), Some(Country::HM)), "Heard und McDonaldinseln" }
                option { value: "HN", selected: matches!(props.value.as_ref(), Some(Country::HN)), "Honduras" }
                option { value: "HR", selected: matches!(props.value.as_ref(), Some(Country::HR)), "Kroatien" }
                option { value: "HT", selected: matches!(props.value.as_ref(), Some(Country::HT)), "Haiti" }
                option { value: "HU", selected: matches!(props.value.as_ref(), Some(Country::HU)), "Ungarn" }
                option { value: "ID", selected: matches!(props.value.as_ref(), Some(Country::ID)), "Indonesien" }
                option { value: "IE", selected: matches!(props.value.as_ref(), Some(Country::IE)), "Irland" }
                option { value: "IL", selected: matches!(props.value.as_ref(), Some(Country::IL)), "Israel" }
                option { value: "IM", selected: matches!(props.value.as_ref(), Some(Country::IM)), "Isle of Man" }
                option { value: "IN", selected: matches!(props.value.as_ref(), Some(Country::IN)), "Indien" }
                option { value: "IO", selected: matches!(props.value.as_ref(), Some(Country::IO)), "Britisches Territorium im Indischen Ozean" }
                option { value: "IQ", selected: matches!(props.value.as_ref(), Some(Country::IQ)), "Irak" }
                option { value: "IR", selected: matches!(props.value.as_ref(), Some(Country::IR)), "Iran" }
                option { value: "IS", selected: matches!(props.value.as_ref(), Some(Country::IS)), "Island" }
                option { value: "IT", selected: matches!(props.value.as_ref(), Some(Country::IT)), "Italien" }
                option { value: "JE", selected: matches!(props.value.as_ref(), Some(Country::JE)), "Jersey" }
                option { value: "JM", selected: matches!(props.value.as_ref(), Some(Country::JM)), "Jamaika" }
                option { value: "JO", selected: matches!(props.value.as_ref(), Some(Country::JO)), "Jordanien" }
                option { value: "JP", selected: matches!(props.value.as_ref(), Some(Country::JP)), "Japan" }
                option { value: "KE", selected: matches!(props.value.as_ref(), Some(Country::KE)), "Kenia" }
                option { value: "KG", selected: matches!(props.value.as_ref(), Some(Country::KG)), "Kirgisistan" }
                option { value: "KH", selected: matches!(props.value.as_ref(), Some(Country::KH)), "Kambodscha" }
                option { value: "KI", selected: matches!(props.value.as_ref(), Some(Country::KI)), "Kiribati" }
                option { value: "KM", selected: matches!(props.value.as_ref(), Some(Country::KM)), "Komoren" }
                option { value: "KN", selected: matches!(props.value.as_ref(), Some(Country::KN)), "St. Kitts und Nevis" }
                option { value: "KP", selected: matches!(props.value.as_ref(), Some(Country::KP)), "Nordkorea" }
                option { value: "KR", selected: matches!(props.value.as_ref(), Some(Country::KR)), "Südkorea" }
                option { value: "KW", selected: matches!(props.value.as_ref(), Some(Country::KW)), "Kuwait" }
                option { value: "KY", selected: matches!(props.value.as_ref(), Some(Country::KY)), "Kaimaninseln" }
                option { value: "KZ", selected: matches!(props.value.as_ref(), Some(Country::KZ)), "Kasachstan" }
                option { value: "LA", selected: matches!(props.value.as_ref(), Some(Country::LA)), "Laos" }
                option { value: "LB", selected: matches!(props.value.as_ref(), Some(Country::LB)), "Libanon" }
                option { value: "LC", selected: matches!(props.value.as_ref(), Some(Country::LC)), "St. Lucia" }
                option { value: "LI", selected: matches!(props.value.as_ref(), Some(Country::LI)), "Liechtenstein" }
                option { value: "LK", selected: matches!(props.value.as_ref(), Some(Country::LK)), "Sri Lanka" }
                option { value: "LR", selected: matches!(props.value.as_ref(), Some(Country::LR)), "Liberia" }
                option { value: "LS", selected: matches!(props.value.as_ref(), Some(Country::LS)), "Lesotho" }
                option { value: "LT", selected: matches!(props.value.as_ref(), Some(Country::LT)), "Litauen" }
                option { value: "LU", selected: matches!(props.value.as_ref(), Some(Country::LU)), "Luxemburg" }
                option { value: "LV", selected: matches!(props.value.as_ref(), Some(Country::LV)), "Lettland" }
                option { value: "LY", selected: matches!(props.value.as_ref(), Some(Country::LY)), "Libyen" }
                option { value: "MA", selected: matches!(props.value.as_ref(), Some(Country::MA)), "Marokko" }
                option { value: "MC", selected: matches!(props.value.as_ref(), Some(Country::MC)), "Monaco" }
                option { value: "MD", selected: matches!(props.value.as_ref(), Some(Country::MD)), "Moldau" }
                option { value: "ME", selected: matches!(props.value.as_ref(), Some(Country::ME)), "Montenegro" }
                option { value: "MF", selected: matches!(props.value.as_ref(), Some(Country::MF)), "Saint-Martin" }
                option { value: "MG", selected: matches!(props.value.as_ref(), Some(Country::MG)), "Madagaskar" }
                option { value: "MH", selected: matches!(props.value.as_ref(), Some(Country::MH)), "Marshallinseln" }
                option { value: "MK", selected: matches!(props.value.as_ref(), Some(Country::MK)), "Nordmazedonien" }
                option { value: "ML", selected: matches!(props.value.as_ref(), Some(Country::ML)), "Mali" }
                option { value: "MM", selected: matches!(props.value.as_ref(), Some(Country::MM)), "Myanmar" }
                option { value: "MN", selected: matches!(props.value.as_ref(), Some(Country::MN)), "Mongolei" }
                option { value: "MO", selected: matches!(props.value.as_ref(), Some(Country::MO)), "Macau" }
                option { value: "MP", selected: matches!(props.value.as_ref(), Some(Country::MP)), "Nördliche Marianen" }
                option { value: "MQ", selected: matches!(props.value.as_ref(), Some(Country::MQ)), "Martinique" }
                option { value: "MR", selected: matches!(props.value.as_ref(), Some(Country::MR)), "Mauretanien" }
                option { value: "MS", selected: matches!(props.value.as_ref(), Some(Country::MS)), "Montserrat" }
                option { value: "MT", selected: matches!(props.value.as_ref(), Some(Country::MT)), "Malta" }
                option { value: "MU", selected: matches!(props.value.as_ref(), Some(Country::MU)), "Mauritius" }
                option { value: "MV", selected: matches!(props.value.as_ref(), Some(Country::MV)), "Malediven" }
                option { value: "MW", selected: matches!(props.value.as_ref(), Some(Country::MW)), "Malawi" }
                option { value: "MX", selected: matches!(props.value.as_ref(), Some(Country::MX)), "Mexiko" }
                option { value: "MY", selected: matches!(props.value.as_ref(), Some(Country::MY)), "Malaysia" }
                option { value: "MZ", selected: matches!(props.value.as_ref(), Some(Country::MZ)), "Mosambik" }
                option { value: "NA", selected: matches!(props.value.as_ref(), Some(Country::NA)), "Namibia" }
                option { value: "NC", selected: matches!(props.value.as_ref(), Some(Country::NC)), "Neukaledonien" }
                option { value: "NE", selected: matches!(props.value.as_ref(), Some(Country::NE)), "Niger" }
                option { value: "NF", selected: matches!(props.value.as_ref(), Some(Country::NF)), "Norfolkinsel" }
                option { value: "NG", selected: matches!(props.value.as_ref(), Some(Country::NG)), "Nigeria" }
                option { value: "NI", selected: matches!(props.value.as_ref(), Some(Country::NI)), "Nicaragua" }
                option { value: "NL", selected: matches!(props.value.as_ref(), Some(Country::NL)), "Niederlande" }
                option { value: "NO", selected: matches!(props.value.as_ref(), Some(Country::NO)), "Norwegen" }
                option { value: "NP", selected: matches!(props.value.as_ref(), Some(Country::NP)), "Nepal" }
                option { value: "NR", selected: matches!(props.value.as_ref(), Some(Country::NR)), "Nauru" }
                option { value: "NU", selected: matches!(props.value.as_ref(), Some(Country::NU)), "Niue" }
                option { value: "NZ", selected: matches!(props.value.as_ref(), Some(Country::NZ)), "Neuseeland" }
                option { value: "OM", selected: matches!(props.value.as_ref(), Some(Country::OM)), "Oman" }
                option { value: "PA", selected: matches!(props.value.as_ref(), Some(Country::PA)), "Panama" }
                option { value: "PE", selected: matches!(props.value.as_ref(), Some(Country::PE)), "Peru" }
                option { value: "PF", selected: matches!(props.value.as_ref(), Some(Country::PF)), "Französisch-Polynesien" }
                option { value: "PG", selected: matches!(props.value.as_ref(), Some(Country::PG)), "Papua-Neuguinea" }
                option { value: "PH", selected: matches!(props.value.as_ref(), Some(Country::PH)), "Philippinen" }
                option { value: "PK", selected: matches!(props.value.as_ref(), Some(Country::PK)), "Pakistan" }
                option { value: "PL", selected: matches!(props.value.as_ref(), Some(Country::PL)), "Polen" }
                option { value: "PM", selected: matches!(props.value.as_ref(), Some(Country::PM)), "Saint-Pierre und Miquelon" }
                option { value: "PN", selected: matches!(props.value.as_ref(), Some(Country::PN)), "Pitcairninseln" }
                option { value: "PR", selected: matches!(props.value.as_ref(), Some(Country::PR)), "Puerto Rico" }
                option { value: "PS", selected: matches!(props.value.as_ref(), Some(Country::PS)), "Palästina" }
                option { value: "PT", selected: matches!(props.value.as_ref(), Some(Country::PT)), "Portugal" }
                option { value: "PW", selected: matches!(props.value.as_ref(), Some(Country::PW)), "Palau" }
                option { value: "PY", selected: matches!(props.value.as_ref(), Some(Country::PY)), "Paraguay" }
                option { value: "QA", selected: matches!(props.value.as_ref(), Some(Country::QA)), "Katar" }
                option { value: "RE", selected: matches!(props.value.as_ref(), Some(Country::RE)), "Réunion" }
                option { value: "RO", selected: matches!(props.value.as_ref(), Some(Country::RO)), "Rumänien" }
                option { value: "RS", selected: matches!(props.value.as_ref(), Some(Country::RS)), "Serbien" }
                option { value: "RU", selected: matches!(props.value.as_ref(), Some(Country::RU)), "Russland" }
                option { value: "RW", selected: matches!(props.value.as_ref(), Some(Country::RW)), "Ruanda" }
                option { value: "SA", selected: matches!(props.value.as_ref(), Some(Country::SA)), "Saudi-Arabien" }
                option { value: "SB", selected: matches!(props.value.as_ref(), Some(Country::SB)), "Salomonen" }
                option { value: "SC", selected: matches!(props.value.as_ref(), Some(Country::SC)), "Seychellen" }
                option { value: "SD", selected: matches!(props.value.as_ref(), Some(Country::SD)), "Sudan" }
                option { value: "SE", selected: matches!(props.value.as_ref(), Some(Country::SE)), "Schweden" }
                option { value: "SG", selected: matches!(props.value.as_ref(), Some(Country::SG)), "Singapur" }
                option { value: "SH", selected: matches!(props.value.as_ref(), Some(Country::SH)), "St. Helena" }
                option { value: "SI", selected: matches!(props.value.as_ref(), Some(Country::SI)), "Slowenien" }
                option { value: "SJ", selected: matches!(props.value.as_ref(), Some(Country::SJ)), "Svalbard und Jan Mayen" }
                option { value: "SK", selected: matches!(props.value.as_ref(), Some(Country::SK)), "Slowakei" }
                option { value: "SL", selected: matches!(props.value.as_ref(), Some(Country::SL)), "Sierra Leone" }
                option { value: "SM", selected: matches!(props.value.as_ref(), Some(Country::SM)), "San Marino" }
                option { value: "SN", selected: matches!(props.value.as_ref(), Some(Country::SN)), "Senegal" }
                option { value: "SO", selected: matches!(props.value.as_ref(), Some(Country::SO)), "Somalia" }
                option { value: "SR", selected: matches!(props.value.as_ref(), Some(Country::SR)), "Suriname" }
                option { value: "SS", selected: matches!(props.value.as_ref(), Some(Country::SS)), "Südsudan" }
                option { value: "ST", selected: matches!(props.value.as_ref(), Some(Country::ST)), "São Tomé und Príncipe" }
                option { value: "SV", selected: matches!(props.value.as_ref(), Some(Country::SV)), "El Salvador" }
                option { value: "SX", selected: matches!(props.value.as_ref(), Some(Country::SX)), "Sint Maarten" }
                option { value: "SY", selected: matches!(props.value.as_ref(), Some(Country::SY)), "Syrien" }
                option { value: "SZ", selected: matches!(props.value.as_ref(), Some(Country::SZ)), "Eswatini" }
                option { value: "TC", selected: matches!(props.value.as_ref(), Some(Country::TC)), "Turks- und Caicosinseln" }
                option { value: "TD", selected: matches!(props.value.as_ref(), Some(Country::TD)), "Tschad" }
                option { value: "TF", selected: matches!(props.value.as_ref(), Some(Country::TF)), "Französische Süd- und Antarktisgebiete" }
                option { value: "TG", selected: matches!(props.value.as_ref(), Some(Country::TG)), "Togo" }
                option { value: "TH", selected: matches!(props.value.as_ref(), Some(Country::TH)), "Thailand" }
                option { value: "TJ", selected: matches!(props.value.as_ref(), Some(Country::TJ)), "Tadschikistan" }
                option { value: "TK", selected: matches!(props.value.as_ref(), Some(Country::TK)), "Tokelau" }
                option { value: "TL", selected: matches!(props.value.as_ref(), Some(Country::TL)), "Osttimor" }
                option { value: "TM", selected: matches!(props.value.as_ref(), Some(Country::TM)), "Turkmenistan" }
                option { value: "TN", selected: matches!(props.value.as_ref(), Some(Country::TN)), "Tunesien" }
                option { value: "TO", selected: matches!(props.value.as_ref(), Some(Country::TO)), "Tonga" }
                option { value: "TR", selected: matches!(props.value.as_ref(), Some(Country::TR)), "Türkei" }
                option { value: "TT", selected: matches!(props.value.as_ref(), Some(Country::TT)), "Trinidad und Tobago" }
                option { value: "TV", selected: matches!(props.value.as_ref(), Some(Country::TV)), "Tuvalu" }
                option { value: "TW", selected: matches!(props.value.as_ref(), Some(Country::TW)), "Taiwan" }
                option { value: "TZ", selected: matches!(props.value.as_ref(), Some(Country::TZ)), "Tansania" }
                option { value: "UA", selected: matches!(props.value.as_ref(), Some(Country::UA)), "Ukraine" }
                option { value: "UG", selected: matches!(props.value.as_ref(), Some(Country::UG)), "Uganda" }
                option { value: "UM", selected: matches!(props.value.as_ref(), Some(Country::UM)), "Amerikanische Überseeinseln" }
                option { value: "US", selected: matches!(props.value.as_ref(), Some(Country::US)), "Vereinigte Staaten" }
                option { value: "UY", selected: matches!(props.value.as_ref(), Some(Country::UY)), "Uruguay" }
                option { value: "UZ", selected: matches!(props.value.as_ref(), Some(Country::UZ)), "Usbekistan" }
                option { value: "VA", selected: matches!(props.value.as_ref(), Some(Country::VA)), "Vatikanstadt" }
                option { value: "VC", selected: matches!(props.value.as_ref(), Some(Country::VC)), "St. Vincent und die Grenadinen" }
                option { value: "VE", selected: matches!(props.value.as_ref(), Some(Country::VE)), "Venezuela" }
                option { value: "VG", selected: matches!(props.value.as_ref(), Some(Country::VG)), "Britische Jungferninseln" }
                option { value: "VI", selected: matches!(props.value.as_ref(), Some(Country::VI)), "Amerikanische Jungferninseln" }
                option { value: "VN", selected: matches!(props.value.as_ref(), Some(Country::VN)), "Vietnam" }
                option { value: "VU", selected: matches!(props.value.as_ref(), Some(Country::VU)), "Vanuatu" }
                option { value: "WF", selected: matches!(props.value.as_ref(), Some(Country::WF)), "Wallis und Futuna" }
                option { value: "WS", selected: matches!(props.value.as_ref(), Some(Country::WS)), "Samoa" }
                option { value: "YE", selected: matches!(props.value.as_ref(), Some(Country::YE)), "Jemen" }
                option { value: "YT", selected: matches!(props.value.as_ref(), Some(Country::YT)), "Mayotte" }
                option { value: "ZA", selected: matches!(props.value.as_ref(), Some(Country::ZA)), "Südafrika" }
                option { value: "ZM", selected: matches!(props.value.as_ref(), Some(Country::ZM)), "Sambia" }
                option { value: "ZW", selected: matches!(props.value.as_ref(), Some(Country::ZW)), "Simbabwe" }
            }
        }
    }
}