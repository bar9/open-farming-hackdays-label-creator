use crate::model::{lookup_allergen, lookup_agricultural, Country};
#[allow(unused_imports)]
use crate::rules::{Rule, RuleDef};
use crate::category_service::{is_fish_category, is_beef_category, is_meat_category, is_egg_category, is_honey_category, is_dairy_category, is_insect_category, is_plant_category};
use rust_i18n::t;
use serde::{Deserialize, Deserializer, Serialize};
use std::cmp::PartialEq;
use std::collections::{HashMap, HashSet};
use std::mem;

/// Custom deserializer for origins field that handles backwards compatibility.
/// Accepts either a single Country (old format) or Vec<Country> (new format).
fn deserialize_origins<'de, D>(deserializer: D) -> Result<Option<Vec<Country>>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Visitor, SeqAccess};

    struct OriginsVisitor;

    impl<'de> Visitor<'de> for OriginsVisitor {
        type Value = Option<Vec<Country>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("null, a single country, or an array of countries")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(OriginsInnerVisitor)
        }
    }

    struct OriginsInnerVisitor;

    impl<'de> Visitor<'de> for OriginsInnerVisitor {
        type Value = Option<Vec<Country>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a single country or an array of countries")
        }

        // Handle single country as string (e.g., "CH")
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            // Parse country code directly
            let country = parse_country_code(value).ok_or_else(|| {
                de::Error::unknown_variant(value, &["valid country code"])
            })?;
            Ok(Some(vec![country]))
        }

        // Handle array of countries
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut countries = Vec::new();
            while let Some(country) = seq.next_element()? {
                countries.push(country);
            }
            if countries.is_empty() {
                Ok(None)
            } else {
                Ok(Some(countries))
            }
        }

        // Handle map format (single country serialized as object)
        fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            let country: Country = Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))?;
            Ok(Some(vec![country]))
        }
    }

    deserializer.deserialize_option(OriginsVisitor)
}

/// Helper function to parse a country code string into a Country enum
fn parse_country_code(value: &str) -> Option<Country> {
    match value {
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
    }
}

#[derive(Clone, Default)]
pub struct Input {
    pub(crate) ingredients: Vec<Ingredient>,
    pub total: Option<f64>,
    pub certification_body: Option<String>,
    pub rezeptur_vollstaendig: bool,
}

impl Input {
    pub fn scale(&mut self, factor: f64) {
        for ingredient in self.ingredients.iter_mut() {
            ingredient.scale_recursive(factor);
        }
    }
}

#[derive(PartialEq)]
pub struct Output {
    pub success: bool,
    pub label: String,
    pub total_amount: f64,
    pub validation_messages: HashMap<String, Vec<String>>,
    pub conditional_elements: HashMap<String, bool>,
}

pub struct Calculator {
    pub(crate) rule_defs: Vec<RuleDef>,
}

fn calculate_swiss_agricultural_percentage(ingredients: &[Ingredient]) -> f64 {
    let leaves: Vec<&Ingredient> = ingredients.iter().flat_map(|i| i.leaves()).collect();

    let total_agricultural_amount: f64 = leaves
        .iter()
        .filter(|ingredient| ingredient.is_agricultural())
        .map(|ingredient| ingredient.amount)
        .sum();

    if total_agricultural_amount == 0.0 {
        return 0.0;
    }

    let swiss_agricultural_amount: f64 = leaves
        .iter()
        .filter(|ingredient| ingredient.is_agricultural())
        .filter(|ingredient| ingredient.origins.as_ref().is_some_and(|o| o.contains(&Country::CH)))
        .map(|ingredient| ingredient.amount)
        .sum();

    (swiss_agricultural_amount / total_agricultural_amount) * 100.0
}

fn calculate_bio_swiss_agricultural_percentage(ingredients: &[Ingredient]) -> f64 {
    let leaves: Vec<&Ingredient> = ingredients.iter().flat_map(|i| i.leaves()).collect();

    let total_bio_agricultural_amount: f64 = leaves
        .iter()
        .filter(|ingredient| ingredient.is_agricultural())
        .filter(|ingredient| ingredient.is_bio.unwrap_or(false))
        .map(|ingredient| ingredient.amount)
        .sum();

    if total_bio_agricultural_amount == 0.0 {
        return 0.0;
    }

    let swiss_bio_agricultural_amount: f64 = leaves
        .iter()
        .filter(|ingredient| ingredient.is_agricultural())
        .filter(|ingredient| ingredient.is_bio.unwrap_or(false))
        .filter(|ingredient| ingredient.origins.as_ref().is_some_and(|o| o.contains(&Country::CH)))
        .map(|ingredient| ingredient.amount)
        .sum();

    (swiss_bio_agricultural_amount / total_bio_agricultural_amount) * 100.0
}

fn calculate_knospe_certified_percentage(ingredients: &[Ingredient]) -> f64 {
    let leaves: Vec<&Ingredient> = ingredients.iter().flat_map(|i| i.leaves()).collect();

    let total_agricultural_amount: f64 = leaves
        .iter()
        .filter(|ingredient| ingredient.is_agricultural())
        .map(|ingredient| ingredient.amount)
        .sum();

    if total_agricultural_amount == 0.0 {
        return 100.0; // No agricultural ingredients means 100% compliance (only water/salt)
    }

    let knospe_certified_amount: f64 = leaves
        .iter()
        .filter(|ingredient| ingredient.is_agricultural())
        .filter(|ingredient| ingredient.is_bio.unwrap_or(false))
        .map(|ingredient| ingredient.amount)
        .sum();

    (knospe_certified_amount / total_agricultural_amount) * 100.0
}

fn calculate_bio_ch_certified_percentage(ingredients: &[Ingredient]) -> f64 {
    let leaves: Vec<&Ingredient> = ingredients.iter().flat_map(|i| i.leaves()).collect();

    let total_agricultural_amount: f64 = leaves
        .iter()
        .filter(|ingredient| ingredient.is_agricultural())
        .map(|ingredient| ingredient.amount)
        .sum();

    if total_agricultural_amount == 0.0 {
        return 100.0;
    }

    let bio_ch_certified_amount: f64 = leaves
        .iter()
        .filter(|ingredient| ingredient.is_agricultural())
        .filter(|ingredient| ingredient.bio_ch.unwrap_or(false))
        .filter(|ingredient| !ingredient.aus_umstellbetrieb.unwrap_or(false))
        .map(|ingredient| ingredient.amount)
        .sum();

    (bio_ch_certified_amount / total_agricultural_amount) * 100.0
}

/// Determines if a product is a Monoprodukt (single agricultural ingredient)
fn is_mono_product(ingredients: &[Ingredient]) -> bool {
    ingredients.iter()
        .flat_map(|i| i.leaves())
        .filter(|i| i.is_agricultural())
        .count() == 1
}

/// Check if any leaf ingredient has aus_umstellbetrieb set
fn has_umstellbetrieb_ingredients(ingredients: &[Ingredient]) -> bool {
    ingredients.iter()
        .flat_map(|i| i.leaves())
        .any(|i| i.aus_umstellbetrieb.unwrap_or(false))
}

/// Calculate the percentage of an ingredient relative to the total amount
fn calculate_ingredient_percentage(ingredient_amount: f64, total_amount: f64) -> f64 {
    (ingredient_amount / total_amount) * 100.0
}

/// Format percentage for display, showing "<1%" instead of "0%" for very small percentages
fn format_percentage(percentage: f64) -> String {
    let rounded = percentage.round() as u8;
    if rounded == 0 && percentage > 0.0 {
        "<1%".to_string()
    } else {
        format!("{}%", rounded)
    }
}

impl Calculator {
    pub(crate) fn new() -> Self {
        Calculator { rule_defs: vec![] }
    }

    /// Debug logging method to display all rules as a table in browser console
    /// Shows all rules with their active status, type, and description
    #[cfg(target_arch = "wasm32")]
    fn log_active_rules(&self) {
        use js_sys::{Array, Object, Reflect};

        let table_data = Array::new();

        for rule in RuleDef::all_rules() {
            let row = Object::new();
            let is_active = self
                .rule_defs
                .iter()
                .any(|r| std::mem::discriminant(r) == std::mem::discriminant(&rule));

            let _ = Reflect::set(
                &row,
                &"Aktiv".into(),
                &(if is_active { "✅" } else { "❌" }).into(),
            );
            let _ = Reflect::set(&row, &"Regel".into(), &format!("{:?}", rule).into());
            let _ = Reflect::set(&row, &"Typ".into(), &format!("{:?}", rule.get_type()).into());
            let _ = Reflect::set(&row, &"Beschreibung".into(), &rule.get_description().into());

            table_data.push(&row);
        }

        web_sys::console::log_1(
            &format!(
                "📋 Regel-Übersicht ({} von {} aktiv)",
                self.rule_defs.len(),
                RuleDef::all_rules().len()
            )
            .into(),
        );
        web_sys::console::table_1(&table_data);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn log_active_rules(&self) {
        // No-op for non-wasm targets
    }

    /// Debug logging method to log individual rule processing
    #[cfg(target_arch = "wasm32")]
    fn log_rule_processing(&self, rule: &RuleDef, processing_type: &str, additional_info: Option<&str>) {
        let info = if let Some(info) = additional_info {
            format!(" - {}", info)
        } else {
            String::new()
        };

        let message = format!(
            "🔄 Processing [{}] {:?}: {}{}",
            processing_type,
            rule,
            rule.get_description(),
            info
        );
        web_sys::console::log_1(&message.into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn log_rule_processing(&self, _rule: &RuleDef, _processing_type: &str, _additional_info: Option<&str>) {
        // No-op for non-wasm targets
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Ingredient {
    pub name: String,
    pub is_allergen: bool,
    pub amount: f64,
    #[serde(default)]
    pub unit: AmountUnit,
    #[serde(default, skip_serializing)]
    pub sub_components: Option<Vec<SubIngredient>>,
    #[serde(default)]
    pub children: Option<Vec<Ingredient>>,
    pub is_namensgebend: Option<bool>,
    /// Multiple origin countries (LIV Art. 16 Abs. 2)
    /// Backwards compatible: deserializes both single Country and Vec<Country>
    #[serde(default, deserialize_with = "deserialize_origins", alias = "origin")]
    pub origins: Option<Vec<Country>>,
    #[serde(default = "default_is_agricultural")]
    pub is_agricultural: bool,
    pub is_bio: Option<bool>,
    pub category: Option<String>,
    pub aufzucht_ort: Option<Country>,
    pub schlachtungs_ort: Option<Country>,
    pub fangort: Option<Country>,
    pub bio_ch: Option<bool>,
    // Erlaubte Ausnahmen für nicht-bio/nicht-knospe Zutaten
    pub erlaubte_ausnahme_bio: Option<bool>,
    pub erlaubte_ausnahme_bio_details: Option<String>,
    pub erlaubte_ausnahme_knospe: Option<bool>,
    pub erlaubte_ausnahme_knospe_details: Option<String>,
    /// Ausgewählte Verarbeitungsschritte (Bio Suisse)
    pub processing_steps: Option<Vec<String>>,
    /// Backwards compatibility field for old URLs
    #[serde(default)]
    pub aus_umstellbetrieb: Option<bool>,
    /// When true, this node's own fields are authoritative even if children exist.
    /// When false/None (default), values are computed from children.
    #[serde(default)]
    pub override_children: Option<bool>,
}

fn default_is_agricultural() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub enum AmountUnit {
    #[default]
    Gram,
    Milliliter,
}

impl AmountUnit {
    pub fn translation_key(&self) -> &'static str {
        match self {
            AmountUnit::Gram => "units.g",
            AmountUnit::Milliliter => "units.ml",
        }
    }
}

impl Ingredient {
    pub fn from_name_amount(name: String, amount: f64) -> Self {
        Self {
            name: name.clone(),
            is_allergen: lookup_allergen(&name),
            is_agricultural: lookup_agricultural(&name),
            amount,
            unit: AmountUnit::default(),
            sub_components: None,
            children: None,
            is_namensgebend: None,
            origins: None,
            is_bio: None,
            category: None,
            aufzucht_ort: None,
            schlachtungs_ort: None,
            fangort: None,
            bio_ch: None,
            erlaubte_ausnahme_bio: None,
            erlaubte_ausnahme_bio_details: None,
            erlaubte_ausnahme_knospe: None,
            erlaubte_ausnahme_knospe_details: None,
            processing_steps: None,
            aus_umstellbetrieb: None,
            override_children: None,
        }
    }

    pub fn is_agricultural(&self) -> bool {
        self.is_agricultural
    }

    pub fn composite_name(&self) -> String {
        let mut name = String::new();
        name.push_str(&self.name);
        if let Some(children) = &self.children {
            if !children.is_empty() {
                name.push_str(" (");
                name.push_str(
                    &children
                        .iter()
                        .map(|child| child.composite_name())
                        .collect::<Vec<String>>()
                        .join(", "),
                );
                name.push(')');
            }
        }
        name
    }

    pub fn composites(&self) -> String {
        let mut output = String::new();
        if let Some(children) = &self.children {
            if !children.is_empty() {
                output.push_str(" (");
                output.push_str(
                    &children
                        .iter()
                        .map(|child| {
                            let escaped_name = html_escape(&child.name);
                            let mut base_name = if child.is_allergen {
                                format!("<b>{}</b>", escaped_name)
                            } else {
                                escaped_name
                            };
                            // Recurse into children's children
                            base_name.push_str(&child.composites());
                            // Add processing steps
                            if let Some(steps) = &child.processing_steps {
                                if !steps.is_empty() {
                                    let steps_text = steps.iter().map(|s| html_escape(s)).collect::<Vec<_>>().join(", ");
                                    base_name = format!("{}, {}", base_name, steps_text);
                                }
                            }
                            // Append origin if present (skip NoOriginRequired)
                            if let Some(origins) = &child.origins {
                                let valid: Vec<&Country> = origins
                                    .iter()
                                    .filter(|o| !matches!(o, Country::NoOriginRequired))
                                    .collect();
                                if !valid.is_empty() {
                                    let codes: Vec<&str> = valid.iter().map(|o| o.country_code()).collect();
                                    base_name = format!("{} ({})", base_name, codes.join(", "));
                                }
                            }
                            base_name
                        })
                        .collect::<Vec<String>>()
                        .join(", "),
                );
                output.push(')');
            }
        }
        output
    }

    /// Migrate old sub_components to new children field (for v1 backwards compatibility)
    pub fn migrate_sub_components(&mut self) {
        if let Some(subs) = self.sub_components.take() {
            if !subs.is_empty() && self.children.is_none() {
                self.children = Some(subs.into_iter().map(|sub| Ingredient {
                    name: sub.name,
                    is_allergen: sub.is_allergen,
                    origins: sub.origin.map(|o| vec![o]),
                    ..Default::default()
                }).collect());
            }
        }
    }

    /// Recursively scale amounts by a factor
    pub fn scale_recursive(&mut self, factor: f64) {
        self.amount *= factor;
        if let Some(children) = &mut self.children {
            for child in children {
                child.scale_recursive(factor);
            }
        }
    }

    /// Is this node a leaf (no children, override active, or children are qualitative-only)?
    /// Qualitative-only children (all amounts zero) means the parent is the authoritative
    /// source for calculations, while children are display-only (for the composites label).
    fn is_leaf(&self) -> bool {
        self.override_children.unwrap_or(false)
            || self.children.as_ref().is_none_or(|c| {
                c.is_empty() || c.iter().all(|child| child.amount == 0.0)
            })
    }

    /// Effective amount: own value if leaf/override, sum of children otherwise
    pub fn computed_amount(&self) -> f64 {
        if self.is_leaf() {
            self.amount
        } else {
            self.children.as_ref().unwrap()
                .iter().map(|c| c.computed_amount()).sum()
        }
    }

    /// Effective bio status: own if leaf/override, all-children-bio otherwise
    pub fn computed_bio_status(&self) -> Option<bool> {
        if self.is_leaf() {
            self.is_bio
        } else {
            let children = self.children.as_ref().unwrap();
            if children.iter().any(|c| c.computed_bio_status().is_some()) {
                Some(children.iter().all(|c| c.computed_bio_status().unwrap_or(false)))
            } else {
                None
            }
        }
    }

    /// Effective bio_ch status: same logic as bio
    pub fn computed_bio_ch_status(&self) -> Option<bool> {
        if self.is_leaf() {
            self.bio_ch
        } else {
            let children = self.children.as_ref().unwrap();
            if children.iter().any(|c| c.computed_bio_ch_status().is_some()) {
                Some(children.iter().all(|c| c.computed_bio_ch_status().unwrap_or(false)))
            } else {
                None
            }
        }
    }

    /// Effective origins: own if leaf/override, union of children's otherwise
    pub fn computed_origins(&self) -> Option<Vec<Country>> {
        if self.is_leaf() {
            self.origins.clone()
        } else {
            let all: HashSet<Country> = self.children.as_ref().unwrap()
                .iter()
                .filter_map(|c| c.computed_origins())
                .flatten()
                .collect();
            if all.is_empty() { None } else { Some(all.into_iter().collect()) }
        }
    }

    /// Collect leaf-level ingredients for percentage calculations.
    /// If override is set, treat this node as a leaf.
    pub fn leaves(&self) -> Vec<&Ingredient> {
        if self.is_leaf() {
            vec![self]
        } else {
            self.children.as_ref().unwrap()
                .iter().flat_map(|c| c.leaves()).collect()
        }
    }
}

impl Default for Ingredient {
    fn default() -> Self {
        Self {
            name: String::new(),
            is_allergen: false,
            amount: 0.,
            unit: AmountUnit::default(),
            sub_components: None,
            children: None,
            is_namensgebend: None,
            origins: None,
            is_agricultural: true,
            is_bio: None,
            category: None,
            aufzucht_ort: None,
            schlachtungs_ort: None,
            fangort: None,
            bio_ch: None,
            erlaubte_ausnahme_bio: None,
            erlaubte_ausnahme_bio_details: None,
            erlaubte_ausnahme_knospe: None,
            erlaubte_ausnahme_knospe_details: None,
            processing_steps: None,
            aus_umstellbetrieb: None,
            override_children: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SubIngredient {
    pub name: String,
    pub is_allergen: bool,
    pub origin: Option<Country>,
}

/// HTML-escape a string to prevent XSS when rendered via dangerous_inner_html.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

struct OutputFormatter {
    ingredient: Ingredient,
    RuleDefs: Vec<RuleDef>,
    total_amount: f64,
    agricultural_ingredient_count: usize,
}

impl PartialEq for RuleDef {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}

impl OutputFormatter {
    pub fn from(ingredient: Ingredient, total_amount: f64, RuleDefs: Vec<RuleDef>, agricultural_ingredient_count: usize) -> Self {
        Self {
            ingredient,
            total_amount,
            RuleDefs,
            agricultural_ingredient_count,
        }
    }

    pub fn format(&self) -> String {
        let escaped_name = html_escape(&self.ingredient.name);
        let mut output = match self.ingredient.is_allergen {
            true => format!("<b>{}</b>", escaped_name),
            false => escaped_name,
        };

        // Umstellbetrieb-Stern (**) vor Bio-Stern (*) prüfen
        let is_umstellbetrieb = self.ingredient.aus_umstellbetrieb.unwrap_or(false);
        let is_bio_ingredient = self.ingredient.is_bio == Some(true) || self.ingredient.bio_ch == Some(true);
        let has_bio_input_rule = self.RuleDefs.contains(&RuleDef::Bio_Knospe_EingabeIstBio)
            || self.RuleDefs.contains(&RuleDef::Bio_PartialBioMarking);
        let suppress_asterisk = self.RuleDefs.contains(&RuleDef::Bio_AllAgriAreBio);

        if has_bio_input_rule && is_umstellbetrieb {
            // Umstellbetrieb ingredients get ** instead of *
            output = format!("{}**", output);
        } else if has_bio_input_rule && is_bio_ingredient && !suppress_asterisk {
            output = format!("{}*", output);
        }

        if self
            .RuleDefs.contains(&RuleDef::AP1_2_ProzentOutputNamensgebend)
        {
            if let Some(true) = self.ingredient.is_namensgebend {
                let percentage = self.ingredient.amount / self.total_amount * 100.;

                // LIV Anhang 7: Bei >100% alternatives Format verwenden
                if percentage > 100.0 {
                    let grams_per_100g = percentage.round() as u32;
                    output = format!(
                        "{} ({})",
                        output,
                        t!("label.liv_anhang7_format", grams = grams_per_100g)
                    )
                } else {
                    output = format!(
                        "{} {}",
                        output,
                        format_percentage(percentage)
                    )
                }
            }
        }
        if self
            .RuleDefs.contains(&RuleDef::AP2_1_ZusammegesetztOutput)
            && self.ingredient.children.as_ref().is_some_and(|c| !c.is_empty())
        {
            output = format! {"{}{}", output, self.ingredient.composites()};
        }
        // Verarbeitungsschritte ausgeben (nach Zutatname/Subkomponenten, vor Herkunft)
        if let Some(steps) = &self.ingredient.processing_steps {
            if !steps.is_empty() {
                let steps_text = steps.iter().map(|s| html_escape(s)).collect::<Vec<_>>().join(", ");
                output = format!("{}, {}", output, steps_text);
            }
        }
        // Handle Knospe-specific rules first (they take precedence)
        let has_knospe_100_rule = self
            .RuleDefs.contains(&RuleDef::Knospe_100_Percent_CH_NoOrigin);
        let has_knospe_90_99_rule = self
            .RuleDefs.contains(&RuleDef::Knospe_90_99_Percent_CH_ShowOrigin);
        let has_knospe_under90_rule = self
            .RuleDefs.contains(&RuleDef::Knospe_Under90_Percent_CH_IngredientRules);

        if has_knospe_100_rule {
            // Rule A: 100% Swiss agricultural ingredients - no origin display
            // Do nothing, origin already not displayed by default
        } else if has_knospe_90_99_rule {
            // Rule B: 90-99.99% Swiss agricultural ingredients - show origin for Swiss ingredients only
            if self.ingredient.origins.as_ref().is_some_and(|o| o.contains(&Country::CH)) {
                output = format!("{} (CH)", output);
            }
        } else if has_knospe_under90_rule {
            // Rule C: <90% Swiss agricultural ingredients - show origin based on specific ingredient criteria
            let percentage = calculate_ingredient_percentage(self.ingredient.amount, self.total_amount);
            let is_mono_product = self.agricultural_ingredient_count == 1;

            if should_show_origin_knospe_under90(&self.ingredient, percentage, self.total_amount, is_mono_product) {
                if let Some(origins) = &self.ingredient.origins {
                    let valid_origins: Vec<&Country> = origins
                        .iter()
                        .filter(|o| !matches!(o, Country::NoOriginRequired))
                        .collect();
                    if !valid_origins.is_empty() {
                        let country_codes: Vec<&str> = valid_origins.iter().map(|o| o.country_code()).collect();
                        output = format!("{} ({})", output, country_codes.join(", "));
                    }
                }
            }
        } else {
            // Check for beef-specific origin display first
            if self.RuleDefs.contains(&RuleDef::AP7_4_RindfleischHerkunftDetails) {
                if let Some(category) = &self.ingredient.category {
                    if is_beef_category(category) {
                        let mut beef_origin_parts = Vec::new();

                        if let Some(aufzucht_ort) = &self.ingredient.aufzucht_ort {
                            beef_origin_parts.push(t!("birthplace", country = aufzucht_ort.country_code()).to_string());
                        }

                        if let Some(schlachtungs_ort) = &self.ingredient.schlachtungs_ort {
                            beef_origin_parts.push(t!("slaughtered_in", country = schlachtungs_ort.country_code()).to_string());
                        }

                        if !beef_origin_parts.is_empty() {
                            output = format!("{} ({})", output, beef_origin_parts.join(", "));
                        }
                    }
                }
            }
            // Check for fish-specific origin display
            if self.RuleDefs.contains(&RuleDef::AP7_5_FischFangort) {
                if let Some(category) = &self.ingredient.category {
                    if is_fish_category(category) {
                        if let Some(fangort) = &self.ingredient.fangort {
                            output = format!("{} ({})", output, fangort.country_code());
                        }
                    }
                }
            }
            // Add country of origin display for traditional herkunft rules (only if no Knospe rules apply)
            if self
                .RuleDefs
                .iter()
                .any(|x| *x == RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent || *x == RuleDef::AP7_3_HerkunftFleischUeber20Prozent || *x == RuleDef::Knospe_AlleZutatenHerkunft)
            {
                if let Some(origins) = &self.ingredient.origins {
                    // Filter out "NoOriginRequired" and join multiple origins
                    let valid_origins: Vec<&Country> = origins
                        .iter()
                        .filter(|o| !matches!(o, Country::NoOriginRequired))
                        .collect();
                    if !valid_origins.is_empty() {
                        let country_codes: Vec<&str> = valid_origins.iter().map(|o| o.country_code()).collect();
                        output = format!("{} ({})", output, country_codes.join(", "));
                    }
                }
            }
        }
        output
    }
}

impl Calculator {
    pub fn registerRuleDefs(&mut self, rule_defs: Vec<RuleDef>) {
        self.rule_defs = rule_defs;
    }

    // Optional RuleRegistry integration methods
    pub fn from_registry_config(config: crate::shared::Configuration) -> Self {
        use crate::rules::RuleRegistry;
        let registry = RuleRegistry::new();
        let rules = registry
            .get_rules_for_config(&config)
            .cloned()
            .unwrap_or_default();
        Calculator { rule_defs: rules }
    }

    pub fn get_rule_descriptions(&self) -> Vec<(&crate::rules::RuleDef, &'static str)> {
        use crate::rules::Rule;
        self.rule_defs
            .iter()
            .map(|rule| (rule, rule.get_description()))
            .collect()
    }

    pub fn execute(&self, input: Input) -> Output {
        // Debug logging: Show active rules
        self.log_active_rules();

        let mut validation_messages = HashMap::new();
        let mut conditionals = HashMap::new();

        // Calculate total amount first (needed for validations)
        let mut total_amount = input.ingredients.iter().map(|x| x.amount).sum();
        if self
            .rule_defs.contains(&RuleDef::AP1_4_ManuelleEingabeTotal)
        {
            if let Some(tot) = input.total {
                total_amount = tot;
            }
        }

        // validations
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&"📋 Validation Rules".into());
        for ruleDef in &self.rule_defs {
            // Ingredient validations only run when recipe is marked as complete
            if input.rezeptur_vollstaendig {
                if let RuleDef::AP1_1_ZutatMengeValidierung = ruleDef {
                    self.log_rule_processing(ruleDef, "VALIDATION", Some("Checking ingredient amounts > 0"));
                    validate_amount(&input.ingredients, &mut validation_messages)
                }
                if let RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent = ruleDef {
                    self.log_rule_processing(ruleDef, "VALIDATION", Some(&format!("Checking origin for ingredients >50% of {}g total", total_amount)));
                    validate_origin(&input.ingredients, total_amount, &mut validation_messages);
                }
                if let RuleDef::AP7_3_HerkunftFleischUeber20Prozent = ruleDef {
                    self.log_rule_processing(ruleDef, "VALIDATION", Some(&format!("Checking meat origin for ingredients >20% of {}g total", total_amount)));
                    validate_meat_origin(&input.ingredients, total_amount, &mut validation_messages);
                }
                if let RuleDef::AP7_4_RindfleischHerkunftDetails = ruleDef {
                    self.log_rule_processing(ruleDef, "VALIDATION", Some("Checking beef origin details (birthplace/slaughter)"));
                    validate_beef_origin_details(&input.ingredients, &mut validation_messages);
                }
                if let RuleDef::AP7_5_FischFangort = ruleDef {
                    self.log_rule_processing(ruleDef, "VALIDATION", Some("Checking fish catch location"));
                    validate_fish_catch_location(&input.ingredients, &mut validation_messages);
                }
                if let RuleDef::Knospe_AlleZutatenHerkunft = ruleDef {
                    self.log_rule_processing(ruleDef, "VALIDATION", Some("Checking origin for ALL ingredients (Bio/Knospe)"));
                    validate_all_ingredients_origin(&input.ingredients, &mut validation_messages)
                }
                if let RuleDef::Knospe_Under90_Percent_CH_IngredientRules = ruleDef {
                    self.log_rule_processing(ruleDef, "VALIDATION", Some("Checking Knospe <90% specific ingredient origin requirements"));
                    validate_knospe_under90_origin(&input.ingredients, total_amount, &mut validation_messages);
                }
            }
            // Non-ingredient validations always run
            if let RuleDef::Bio_Knospe_ZertifizierungsstellePflicht = ruleDef {
                self.log_rule_processing(ruleDef, "VALIDATION", Some("Checking mandatory certification body for Bio/Knospe"));
                validate_certification_body(&input.certification_body, &mut validation_messages);
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let total_errors: usize = validation_messages.values().map(|v| v.len()).sum();
            web_sys::console::log_1(&format!("📊 Validation results: {} fields with {} total errors", validation_messages.len(), total_errors).into());

            if !validation_messages.is_empty() {
                web_sys::console::log_1(&"❌ Validation errors by field:".into());
                for (field, messages) in &validation_messages {
                    for message in messages {
                        web_sys::console::log_1(&format!("  {} → {}", field, message).into());
                    }
                }
            }
        }

        // conditionals
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&"🎛️ Conditional Display Rules".into());
        for ruleDef in &self.rule_defs {
            if let RuleDef::AP1_3_EingabeNamensgebendeZutat = ruleDef {
                self.log_rule_processing(ruleDef, "CONDITIONAL", Some("Enabling name-giving ingredient input"));
                conditionals.insert(String::from("namensgebende_zutat"), true);
            }
            if let RuleDef::Bio_Knospe_EingabeIstBio = ruleDef {
                self.log_rule_processing(ruleDef, "CONDITIONAL", Some("Enabling bio certification input"));
                conditionals.insert(String::from("is_bio_eingabe"), true);
            }
        }
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!("🎛️ Conditional elements: {} enabled", conditionals.len()).into());

        let mut sorted_ingredients = input.ingredients.clone();
        sorted_ingredients.sort_by(|y, x| x.computed_amount().partial_cmp(&y.computed_amount()).unwrap());

        if self
            .rule_defs.contains(&RuleDef::AP1_4_ManuelleEingabeTotal)
        {
            self.log_rule_processing(&RuleDef::AP1_4_ManuelleEingabeTotal, "CONDITIONAL", Some("Enabling manual total input"));
            conditionals.insert(String::from("manuelles_total"), true);
        }

        // Determine which ingredients require country of origin display
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&"🌍 Origin Requirement Rules".into());
        let has_50_percent_rule = self
            .rule_defs.contains(&RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent);
        let has_bio_knospe_rule = self
            .rule_defs.contains(&RuleDef::Knospe_AlleZutatenHerkunft);

        // Handle Knospe-specific percentage-based rules
        let has_knospe_100_rule = self
            .rule_defs.contains(&RuleDef::Knospe_100_Percent_CH_NoOrigin);
        let has_knospe_90_99_rule = self
            .rule_defs.contains(&RuleDef::Knospe_90_99_Percent_CH_ShowOrigin);
        let has_knospe_under90_rule = self
            .rule_defs.contains(&RuleDef::Knospe_Under90_Percent_CH_IngredientRules);

        // Calculate percentage of Swiss agricultural ingredients for Knospe rules
        let mut actual_knospe_rule: Option<RuleDef> = None;
        if has_knospe_100_rule || has_knospe_90_99_rule || has_knospe_under90_rule {
            // Use bio-specific calculation if Bio_Knospe_EingabeIstBio rule is active
            let has_bio_rule = self.rule_defs.contains(&RuleDef::Bio_Knospe_EingabeIstBio);
            let swiss_percentage = if has_bio_rule {
                calculate_bio_swiss_agricultural_percentage(&input.ingredients)
            } else {
                calculate_swiss_agricultural_percentage(&input.ingredients)
            };

            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&format!("🇨🇭 Swiss agricultural percentage: {:.1}% (bio-specific: {})", swiss_percentage, has_bio_rule).into());

            if swiss_percentage >= 100.0 && has_knospe_100_rule {
                actual_knospe_rule = Some(RuleDef::Knospe_100_Percent_CH_NoOrigin);
                self.log_rule_processing(&RuleDef::Knospe_100_Percent_CH_NoOrigin, "OUTPUT", Some("100% Swiss ingredients - no origin display needed"));
                #[cfg(target_arch = "wasm32")]
                web_sys::console::log_1(&"✅ Knospe Rule A: 100% Swiss agricultural ingredients - origin display disabled".into());
            } else if swiss_percentage >= 90.0 && has_knospe_90_99_rule {
                actual_knospe_rule = Some(RuleDef::Knospe_90_99_Percent_CH_ShowOrigin);
                self.log_rule_processing(&RuleDef::Knospe_90_99_Percent_CH_ShowOrigin, "OUTPUT", Some(&format!("{:.1}% Swiss ingredients - show origin for Swiss", swiss_percentage)));
                #[cfg(target_arch = "wasm32")]
                web_sys::console::log_1(&format!("✅ Knospe Rule B: {:.1}% Swiss agricultural ingredients - show origin for Swiss only", swiss_percentage).into());
            } else if swiss_percentage < 90.0 && has_knospe_under90_rule {
                actual_knospe_rule = Some(RuleDef::Knospe_Under90_Percent_CH_IngredientRules);
                self.log_rule_processing(&RuleDef::Knospe_Under90_Percent_CH_IngredientRules, "OUTPUT", Some(&format!("{:.1}% Swiss ingredients - use ingredient-specific rules", swiss_percentage)));
                #[cfg(target_arch = "wasm32")]
                web_sys::console::log_1(&format!("✅ Knospe Rule C: {:.1}% Swiss agricultural ingredients - ingredient-specific origin rules", swiss_percentage).into());
            }
        }

        // Handle Bio Suisse logo display for Knospe configuration
        if self.rule_defs.contains(&RuleDef::Knospe_ShowBioSuisseLogo) {
            self.log_rule_processing(&RuleDef::Knospe_ShowBioSuisseLogo, "OUTPUT", Some("Determining Bio Suisse logo display based on Knospe certification"));

            // First check if ALL agricultural ingredients are Knospe-certified (required for any logo)
            let knospe_percentage = calculate_knospe_certified_percentage(&input.ingredients);

            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&format!("🌾 Knospe certified percentage: {:.1}%", knospe_percentage).into());

            // Only show a Knospe logo if 100% of agricultural ingredients are Knospe-certified
            if knospe_percentage >= 100.0 {
                // Now determine which logo variant based on Swiss percentage
                // Use bio-specific calculation if Bio_Knospe_EingabeIstBio rule is active
                let has_bio_rule = self.rule_defs.contains(&RuleDef::Bio_Knospe_EingabeIstBio);
                let swiss_percentage = if has_bio_rule {
                    calculate_bio_swiss_agricultural_percentage(&input.ingredients)
                } else {
                    calculate_swiss_agricultural_percentage(&input.ingredients)
                };

                #[cfg(target_arch = "wasm32")]
                web_sys::console::log_1(&format!("🇨🇭 Swiss percentage of Knospe ingredients: {:.1}%", swiss_percentage).into());

                if swiss_percentage >= 90.0 {
                    // Knospe with Swiss cross (>= 90% Swiss)
                    conditionals.insert(String::from("bio_suisse_regular"), true);
                } else {
                    // Knospe without Swiss cross (< 90% Swiss, including 0% Swiss)
                    conditionals.insert(String::from("bio_suisse_no_cross"), true);
                }
            } else {
                #[cfg(target_arch = "wasm32")]
                web_sys::console::log_1(&format!("⚠️ Not all ingredients are Knospe-certified ({:.1}%), no logo will be shown", knospe_percentage).into());
            }
        }

        // Bio-V/CH Bio: Add "Bio" to Sachbezeichnung when >= 95% Bio-CH certified
        let bio_ch_percentage = if self.rule_defs.contains(&RuleDef::Bio_ShowBioSachbezeichnung) {
            let pct = calculate_bio_ch_certified_percentage(&input.ingredients);

            if pct >= 95.0 {
                conditionals.insert(String::from("bio_sachbezeichnung_suffix"), true);
                conditionals.insert(String::from("bio_marketing_allowed"), true);
            } else if pct > 0.0 {
                conditionals.insert(String::from("bio_marketing_not_allowed"), true);
            }
            // Umstellbetrieb handling for Bio Sachbezeichnung
            if has_umstellbetrieb_ingredients(&input.ingredients) {
                if is_mono_product(&input.ingredients) {
                    // Monoprodukt with umstellbetrieb: keep suffix, add hinweis
                    conditionals.insert(String::from("umstellbetrieb_hinweis"), true);
                } else {
                    // Composite with umstellbetrieb: remove suffix, block marketing
                    conditionals.remove("bio_sachbezeichnung_suffix");
                    conditionals.remove("bio_marketing_allowed");
                    conditionals.insert(String::from("bio_marketing_not_allowed"), true);
                }
            }

            pct
        } else {
            0.0
        };

        let has_meat_rule = self
            .rule_defs.contains(&RuleDef::AP7_3_HerkunftFleischUeber20Prozent);

        if has_50_percent_rule || has_bio_knospe_rule || has_meat_rule {
            let mut has_any_herkunft_required = false;
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&"🌍 Analyzing origin requirements for each ingredient:".into());

            for (index, ingredient) in input.ingredients.iter().enumerate() {
                let mut requires_herkunft = false;
                let mut reasons = Vec::new();
                let percentage = calculate_ingredient_percentage(ingredient.amount, total_amount);

                // Check if >50% rule applies
                if has_50_percent_rule && percentage > 50.0 {
                    requires_herkunft = true;
                    reasons.push(format!(">50% ({:.1}%)", percentage));
                }

                // Check if meat rule applies (meat ingredients >20%)
                if has_meat_rule && percentage > 20.0 {
                    if let Some(category) = &ingredient.category {
                        if is_meat_category(category) {
                            requires_herkunft = true;
                            reasons.push(format!("meat >20% ({:.1}%)", percentage));
                        }
                    }
                }

                // Check if Bio/Knospe rule applies (requires origin for ALL ingredients)
                if has_bio_knospe_rule {
                    requires_herkunft = true;
                    reasons.push("Bio/Knospe".to_string());
                }

                #[cfg(target_arch = "wasm32")]
                if requires_herkunft {
                    web_sys::console::log_1(&format!("  ✅ {} ({:.1}%): Origin required - {}", ingredient.name, percentage, reasons.join(", ")).into());
                } else {
                    web_sys::console::log_1(&format!("  ⚪ {} ({:.1}%): No origin required", ingredient.name, percentage).into());
                }

                if requires_herkunft {
                    conditionals.insert(format!("herkunft_benoetigt_{}", index), true);
                    has_any_herkunft_required = true;
                }
            }

            if has_any_herkunft_required {
                conditionals.insert(String::from("herkunft_benoetigt_ueber_50_prozent"), true);
            }
        }

        // End origin requirement logging

        // Prepare rule_defs for OutputFormatter, including the specific Knospe rule
        let mut output_rules = self.rule_defs.clone();
        if let Some(knospe_rule) = actual_knospe_rule {
            // Remove the generic Knospe rules and add the specific one
            output_rules.retain(|rule| !matches!(rule, RuleDef::Knospe_100_Percent_CH_NoOrigin | RuleDef::Knospe_90_99_Percent_CH_ShowOrigin));
            output_rules.push(knospe_rule);
        }

        // Inject Bio marking mode rules (only for Bio config with Bio_ShowBioSachbezeichnung)
        if self.rule_defs.contains(&RuleDef::Bio_ShowBioSachbezeichnung) {
            if bio_ch_percentage >= 95.0 {
                output_rules.push(RuleDef::Bio_AllAgriAreBio);
            } else if bio_ch_percentage > 0.0 {
                output_rules.push(RuleDef::Bio_PartialBioMarking);
            }
        }

        // Final summary logging
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::log_1(&"📈 Final Results".into());
            web_sys::console::log_1(&format!("✅ Label generation complete - {} ingredients processed", sorted_ingredients.len()).into());
            web_sys::console::log_1(&format!("📋 {} validation messages", validation_messages.len()).into());
            web_sys::console::log_1(&format!("🎛️ {} conditional elements enabled", conditionals.len()).into());
            web_sys::console::log_1(&format!("⚖️ Total amount: {}g", total_amount).into());
        }

        // Prüfe ob Bio-Zutaten oder Umstellbetrieb vorhanden sind (für Legende)
        let has_bio_rules = output_rules.contains(&RuleDef::Bio_Knospe_EingabeIstBio)
            || output_rules.contains(&RuleDef::Bio_AllAgriAreBio)
            || output_rules.contains(&RuleDef::Bio_PartialBioMarking);
        let has_bio_ingredients = has_bio_rules && sorted_ingredients.iter().any(|ing|
            ing.is_bio == Some(true) || ing.bio_ch == Some(true)
        );
        let has_umstellbetrieb = has_bio_rules && sorted_ingredients.iter().any(|ing|
            ing.aus_umstellbetrieb.unwrap_or(false)
        );

        // Count agricultural ingredients for Monoprodukt detection in OutputFormatter
        let agricultural_ingredient_count = sorted_ingredients.iter()
            .flat_map(|i| i.leaves())
            .filter(|i| i.is_agricultural())
            .count();

        // Generiere Zutatenliste
        let ingredients_label = sorted_ingredients
            .into_iter()
            .map(|item| OutputFormatter::from(item, total_amount, output_rules.clone(), agricultural_ingredient_count))
            .map(|fmt| fmt.format())
            .collect::<Vec<_>>()
            .join(", ");

        // Legende anhängen basierend auf Bio-Modus
        let mut label = if output_rules.contains(&RuleDef::Bio_AllAgriAreBio) && has_bio_ingredients {
            // AllBio mode: no asterisks, "Alle landwirtschaftlichen..." legend
            format!("{}<br><br>{}", ingredients_label, t!("bio_legend.alle_landwirtschaftlichen"))
        } else if output_rules.contains(&RuleDef::Bio_PartialBioMarking) && has_bio_ingredients {
            // PartialBio mode: asterisks on bio ingredients, percentage legend
            let rounded = bio_ch_percentage.round() as u32;
            format!("{}<br><br>* {}", ingredients_label, t!("bio_legend.x_prozent_bio", percentage = rounded))
        } else if has_bio_ingredients {
            // Knospe fallback: simple * legend
            format!("{}<br><br>* {}", ingredients_label, t!("bio_legend.aus_biologischer_landwirtschaft"))
        } else {
            ingredients_label
        };

        // Append Umstellbetrieb legend if any umstellbetrieb ingredients present
        if has_umstellbetrieb {
            label = format!("{}<br>** {}", label, t!("bio_legend.aus_umstellung"));
        }

        Output {
            success: true,
            label,
            total_amount,
            validation_messages,
            conditional_elements: conditionals,
        }
    }
}

fn validate_amount(ingredients: &[Ingredient], validation_messages: &mut HashMap<String, Vec<String>>) {
    for (i, ingredient) in ingredients.iter().enumerate() {
        if ingredient.amount <= 0. {
            validation_messages.entry(format!("ingredients[{}][amount]", i))
                .or_default()
                .push(t!("validation.amount_greater_than_zero").to_string());
        }
    }
}

fn validate_origin(
    ingredients: &[Ingredient],
    total_amount: f64,
    validation_messages: &mut HashMap<String, Vec<String>>,
) {
    for (i, ingredient) in ingredients.iter().enumerate() {
        let percentage = calculate_ingredient_percentage(ingredient.amount, total_amount);
        let has_origin = ingredient.origins.as_ref().is_some_and(|v| !v.is_empty());
        if percentage > 50.0 && !has_origin {
            validation_messages.entry(format!("ingredients[{}][origin]", i))
                .or_default()
                .push(t!("validation.origin_required_over_50_percent").to_string());
        }
    }
}

// Functions are already imported above, no need to re-export

// Use centralized category service functions

/// Determines if an ingredient should show origin for Knospe <90% CH rules
/// Based on specific Knospe criteria for ingredient types and percentages
fn should_show_origin_knospe_under90(ingredient: &Ingredient, percentage: f64, _total_amount: f64, is_mono_product: bool) -> bool {
    // For monoproducts (single ingredient products), always show origin
    if is_mono_product {
        return true;
    }

    // Name-giving ingredients (namensgebende Zutat) always show origin for Knospe
    if ingredient.is_namensgebend == Some(true) {
        return true;
    }

    // Swiss ingredients with at least 10% share
    if ingredient.origins.as_ref().is_some_and(|o| o.contains(&Country::CH)) && percentage >= 10.0 {
        return true;
    }

    // Plant ingredients with more than 50% share
    if let Some(category) = &ingredient.category {
        if is_plant_category(category) && percentage > 50.0 {
            return true;
        }
    }

    // Eggs/Honey/Fish/Other aquacultures with more than 10% share
    if let Some(category) = &ingredient.category {
        if (is_egg_category(category) ||
            is_honey_category(category) ||
            is_fish_category(category)) && percentage > 10.0 {
            return true;
        }
    }

    // Milk/Dairy/Meat/Insects always show origin (question in requirements:
    // "gilt das auch bei solchen Zutaten, oder nur wenn das ganze Produkt zB ein Milchprodukt ist?"
    // I'm interpreting this as: these ingredient types always need origin)
    if let Some(category) = &ingredient.category {
        if is_dairy_category(category) ||
           is_meat_category(category) ||
           is_insect_category(category) {
            return true;
        }
    }

    false
}

fn validate_meat_origin(
    ingredients: &[Ingredient],
    total_amount: f64,
    validation_messages: &mut HashMap<String, Vec<String>>,
) {
    for (i, ingredient) in ingredients.iter().enumerate() {
        let percentage = calculate_ingredient_percentage(ingredient.amount, total_amount);
        if percentage > 20.0 {
            // Check if this ingredient is meat-based using the category
            if let Some(category) = &ingredient.category {
                let has_origin = ingredient.origins.as_ref().is_some_and(|v| !v.is_empty());
                if is_meat_category(category) && !has_origin {
                    validation_messages.entry(format!("ingredients[{}][origin]", i))
                        .or_default()
                        .push(t!("validation.origin_required_meat_over_20").to_string());
                }
            }
        }
    }
}

fn validate_all_ingredients_origin(
    ingredients: &[Ingredient],
    validation_messages: &mut HashMap<String, Vec<String>>,
) {
    for (i, ingredient) in ingredients.iter().enumerate() {
        let has_origin = ingredient.origins.as_ref().is_some_and(|v| !v.is_empty());
        if !has_origin {
            validation_messages.entry(format!("ingredients[{}][origin]", i))
                .or_default()
                .push(t!("validation.origin_required_knospe").to_string());
        }
    }
}

fn validate_certification_body(
    certification_body: &Option<String>,
    validation_messages: &mut HashMap<String, Vec<String>>,
) {
    match certification_body {
        None => {
            validation_messages.entry("certification_body".to_string())
                .or_default()
                .push(t!("validation.certification_body_required").to_string());
        }
        Some(s) if s.is_empty() => {
            validation_messages.entry("certification_body".to_string())
                .or_default()
                .push(t!("validation.certification_body_required").to_string());
        }
        Some(s) => {
            if !s.starts_with("CH-BIO-") {
                validation_messages.entry("certification_body".to_string())
                    .or_default()
                    .push(t!("validation.certification_body_format").to_string());
            }
        }
    }
}

fn validate_beef_origin_details(
    ingredients: &[Ingredient],
    validation_messages: &mut HashMap<String, Vec<String>>,
) {
    for (i, ingredient) in ingredients.iter().enumerate() {
        // Check if this ingredient is beef-based using the category
        if let Some(category) = &ingredient.category {
            if is_beef_category(category) {
                // Validate aufzucht_ort (birthplace/where it lived)
                if ingredient.aufzucht_ort.is_none() {
                    validation_messages.entry(format!("ingredients[{}][aufzucht_ort]", i))
                        .or_default()
                        .push(t!("validation.beef_breeding_location_required").to_string());
                }

                // Validate schlachtungs_ort (slaughter location)
                if ingredient.schlachtungs_ort.is_none() {
                    validation_messages.entry(format!("ingredients[{}][schlachtungs_ort]", i))
                        .or_default()
                        .push(t!("validation.beef_slaughter_location_required").to_string());
                }
            }
        }
    }
}

fn validate_fish_catch_location(
    ingredients: &[Ingredient],
    validation_messages: &mut HashMap<String, Vec<String>>,
) {
    for (i, ingredient) in ingredients.iter().enumerate() {
        // Check if this ingredient is fish-based using the category
        if let Some(category) = &ingredient.category {
            if is_fish_category(category) {
                // Validate fangort (catch location)
                if ingredient.fangort.is_none() {
                    validation_messages.entry(format!("ingredients[{}][fangort]", i))
                        .or_default()
                        .push(t!("validation.fish_catch_location_required").to_string());
                }
            }
        }
    }
}

fn validate_knospe_under90_origin(
    ingredients: &[Ingredient],
    total_amount: f64,
    validation_messages: &mut HashMap<String, Vec<String>>,
) {
    let agricultural_count = ingredients.iter()
        .flat_map(|i| i.leaves())
        .filter(|i| i.is_agricultural())
        .count();

    for (i, ingredient) in ingredients.iter().enumerate() {
        let percentage = calculate_ingredient_percentage(ingredient.amount, total_amount);
        let is_mono_product = agricultural_count == 1;

        let requires_origin = should_show_origin_knospe_under90(ingredient, percentage, total_amount, is_mono_product);
        let has_origin = ingredient.origins.as_ref().is_some_and(|v| !v.is_empty());

        if requires_origin && !has_origin {
            let reason = if is_mono_product {
                t!("validation.knospe_mono_origin_required").to_string()
            } else if ingredient.is_namensgebend == Some(true) {
                t!("validation.knospe_name_giving_origin_required").to_string()
            } else if let Some(category) = &ingredient.category {
                if is_plant_category(category) && percentage > 50.0 {
                    t!("validation.knospe_plants_over_50_origin_required").to_string()
                } else if (is_egg_category(category) || is_honey_category(category) || is_fish_category(category)) && percentage > 10.0 {
                    t!("validation.knospe_egg_honey_fish_origin_required").to_string()
                } else if is_dairy_category(category) || is_meat_category(category) || is_insect_category(category) {
                    t!("validation.knospe_dairy_meat_insects_origin_required").to_string()
                } else if percentage >= 10.0 {
                    t!("validation.knospe_over_10_percent_origin_required").to_string()
                } else {
                    t!("validation.knospe_general_origin_required").to_string()
                }
            } else if percentage >= 10.0 {
                t!("validation.knospe_over_10_percent_origin_required").to_string()
            } else {
                t!("validation.knospe_general_origin_required").to_string()
            };

            validation_messages.entry(format!("ingredients[{}][origin]", i))
                .or_default()
                .push(reason);
        }
    }
}


#[cfg(test)]
mod tests;
