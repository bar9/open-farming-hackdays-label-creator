use crate::verdicts::{BioBlockReason, BioVerdict, CheckState, KnospeBlockReason, KnospeLogo, KnospeVerdict, Verdicts};
use crate::model::{lookup_allergen, lookup_agricultural, Country};
use crate::rules::RuleDef;
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
        "Import" => Some(Country::Import),
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
    /// «Keine Zutatenliste (Einzelzutat)» — the product has no recipe at all,
    /// so the «Rezeptur prüfen» hints must stay silent (DEC-3).
    pub ignore_ingredients: bool,
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
    /// The typed rule-engine decisions (TD-1). The UI reads these directly;
    /// the legacy key→bool view is derived on demand via [`Output::conditionals`].
    pub verdicts: Verdicts,
}

impl Output {
    /// Legacy key→bool view of the verdicts.
    ///
    /// Exists for the test suite, which asserts against the flat contract
    /// (`c.get(keys::…)`) accumulated over the project's history — a dense net
    /// that is deliberately kept. Production code reads `verdicts`; nothing at
    /// runtime consumes this map anymore.
    pub fn conditionals(&self) -> HashMap<String, bool> {
        let mut map = HashMap::new();
        self.verdicts.write_conditionals(&mut map);
        map
    }
}

pub struct Calculator {
    pub(crate) rule_defs: Vec<RuleDef>,
}

/// Share (in percent) of the agricultural weight that satisfies `numerator`,
/// out of the agricultural weight that satisfies `denominator`.
///
/// All the Bio/Knospe percentages are this same shape: pick a subset of the
/// leaves, weigh it against another subset, guard the empty case. Writing it
/// once keeps the six rules below to their actual difference — the predicates —
/// and makes the `empty` fallback an explicit, per-rule decision rather than a
/// detail buried in copied code.
fn agricultural_share(
    ingredients: &[Ingredient],
    denominator: impl Fn(&Ingredient) -> bool,
    numerator: impl Fn(&Ingredient) -> bool,
    empty: f64,
) -> f64 {
    let leaves: Vec<&Ingredient> = ingredients.iter().flat_map(|i| i.leaves()).collect();

    let total: f64 = leaves
        .iter()
        .filter(|i| i.is_agricultural() && denominator(i))
        .map(|i| i.amount)
        .sum();

    if total == 0.0 {
        return empty;
    }

    let matching: f64 = leaves
        .iter()
        .filter(|i| i.is_agricultural() && denominator(i) && numerator(i))
        .map(|i| i.amount)
        .sum();

    (matching / total) * 100.0
}

/// Swiss share of the agricultural weight.
fn calculate_swiss_agricultural_percentage(ingredients: &[Ingredient]) -> f64 {
    agricultural_share(
        ingredients,
        |_| true,
        |i| i.computed_origins().is_some_and(|o| o.contains(&Country::CH)),
        0.0,
    )
}

/// Swiss share of the *bio* agricultural weight — the Knospe logo variant is
/// decided on the certified portion only.
fn calculate_bio_swiss_agricultural_percentage(ingredients: &[Ingredient]) -> f64 {
    agricultural_share(
        ingredients,
        |i| i.computed_bio_status().unwrap_or(false),
        |i| i.computed_origins().is_some_and(|o| o.contains(&Country::CH)),
        0.0,
    )
}

/// Knospe-certified share of the agricultural weight.
/// Empty case is 100%: with no agricultural ingredients nothing is uncertified.
/// Callers must additionally check `has_agricultural_ingredient` before turning
/// this into a claim (DEC-2).
fn calculate_knospe_certified_percentage(ingredients: &[Ingredient]) -> f64 {
    agricultural_share(ingredients, |_| true, |i| i.is_knospe_compliant(), 100.0)
}

/// Bio-CH-certified share of the agricultural weight. Same empty-case caveat as
/// `calculate_knospe_certified_percentage`.
fn calculate_bio_ch_certified_percentage(ingredients: &[Ingredient]) -> f64 {
    agricultural_share(ingredients, |_| true, |i| i.is_bio_ch_compliant(), 100.0)
}

/// Percentage (of total agricultural weight) made up of permitted non-organic
/// exceptions (Annex 3 WBF, e.g. Pektin) that are not bio-certified. The Bio-V
/// "Bio" Sachbezeichnung tolerates these only up to 5% of the agricultural weight.
fn calculate_erlaubte_ausnahme_bio_percentage(ingredients: &[Ingredient]) -> f64 {
    agricultural_share(
        ingredients,
        |_| true,
        |i| i.erlaubte_ausnahme_bio.unwrap_or(false) && !i.is_bio_ch_compliant(),
        0.0,
    )
}

/// Percentage (of total agricultural weight) made up of permitted non-organic
/// exceptions (Annex 3 WBF / Bio Suisse Part III, e.g. Pektin) that are not
/// themselves Knospe-certified. Bio Suisse tolerates these only up to 5% of the
/// agricultural weight, exactly as the Bio-V rule does (DEC-8).
fn calculate_erlaubte_ausnahme_knospe_percentage(ingredients: &[Ingredient]) -> f64 {
    agricultural_share(
        ingredients,
        |_| true,
        // Either exception flag makes an ingredient Knospe-compliant (see
        // `is_knospe_compliant`), so both count against the 5% budget — unless
        // the ingredient is bio-certified in its own right, in which case it is
        // not an exception at all.
        |i| {
            (i.erlaubte_ausnahme_bio.unwrap_or(false)
                || i.erlaubte_ausnahme_knospe.unwrap_or(false))
                && !i.is_bio.unwrap_or(false)
        },
        0.0,
    )
}

/// Whether the recipe contains at least one permitted non-organic agricultural
/// ingredient (Annex 3 WBF / Bio Suisse Part III, e.g. Pektin).
///
/// Such a recipe is NOT 100% organic-agricultural, so only the per-ingredient
/// *-marking is legally available; the blanket wordings ("Alle landwirtschaftlichen
/// Zutaten stammen aus biologischer Landwirtschaft" / the "Bio-" prefix) would be
/// untrue (DEC-4).
fn has_erlaubte_ausnahme(ingredients: &[Ingredient]) -> bool {
    ingredients
        .iter()
        .flat_map(|i| i.leaves())
        .filter(|i| i.is_agricultural())
        .any(|i| {
            (i.erlaubte_ausnahme_bio.unwrap_or(false)
                || i.erlaubte_ausnahme_knospe.unwrap_or(false))
                && !i.is_bio_ch_compliant()
        })
}

/// Whether the recipe contains any agricultural ingredient that is non-organic
/// without being declared a permitted exception (DEC-7).
fn has_undeclared_non_bio(ingredients: &[Ingredient]) -> bool {
    ingredients.iter().any(|i| i.has_undeclared_non_bio())
}

/// The processing step that marks an ingredient as wild-collected. Stored in
/// German (as all processing steps are) and used as the lookup key.
pub const WILDSAMMLUNG_STEP: &str = "aus zertifizierter Wildsammlung";

/// Wording for wild collection, which differs by regime: Bio Suisse says «aus
/// zertifizierter Wildsammlung», the Bio-Verordnung requires «aus biologisch
/// zertifizierter Wildsammlung» (Abklärung BLW, DEC-11). Both the ° legend and
/// the inline text below 10% must use the same wording.
fn wildsammlung_wording(rules: &[RuleDef]) -> String {
    if rules.contains(&RuleDef::Knospe_ShowBioSuisseLogo) {
        t!("bio_legend.aus_wildsammlung").to_string()
    } else {
        t!("bio_legend.aus_biologisch_zertifizierter_wildsammlung").to_string()
    }
}

/// Whether the recipe contains any agricultural ingredient at all.
///
/// The percentage helpers return 100% for a purely non-agricultural product
/// (salt, water) because there is nothing that could be uncertified. That is
/// the right answer for a *share*, but it must not be read as "certified": such
/// a product has nothing to certify and may make no Bio/Knospe claim (DEC-2).
fn has_agricultural_ingredient(ingredients: &[Ingredient]) -> bool {
    ingredients
        .iter()
        .flat_map(|i| i.leaves())
        .any(|i| i.is_agricultural() && i.amount > 0.0)
}

/// Determines if a product is a Monoprodukt (single agricultural ingredient)
fn is_mono_product(ingredients: &[Ingredient]) -> bool {
    ingredients.iter()
        .flat_map(|i| i.leaves())
        .filter(|i| i.is_agricultural())
        .count() == 1
}

/// Check if any leaf ingredient has aus_umstellbetrieb set
/// Umstellung anywhere in the tree — a composite parent claiming Umstellung as a
/// bought certified unit carries the flag on the parent node, which `leaves()`
/// never visits.
fn has_umstellbetrieb_in_tree(ingredients: &[Ingredient]) -> bool {
    fn node_or_descendant(i: &Ingredient) -> bool {
        i.aus_umstellbetrieb.unwrap_or(false)
            || i.children.as_ref().is_some_and(|cs| cs.iter().any(node_or_descendant))
    }
    ingredients.iter().any(node_or_descendant)
}

/// Quality claimed at a composite level is pushed DOWN onto the children's
/// rendered markers (Testing 25.06.2026). Carries the accumulated claim while
/// walking a composite subtree.
#[derive(Clone, Copy, Default)]
struct InheritedQuality {
    bio: bool,
    umstellung: bool,
}

impl InheritedQuality {
    fn from_parent(parent: &Ingredient) -> Self {
        Self {
            bio: parent.is_bio == Some(true) || parent.bio_ch == Some(true),
            umstellung: parent.aus_umstellbetrieb == Some(true),
        }
    }
}

/// Which markers the rendered ingredient list will actually contain:
/// `(simple_star, double_star)`. Mirrors the marker rules in `format` /
/// `composites_with_inherited` including parent-claim push-down, so the
/// legend lines match what is printed.
fn tree_marker_presence(ingredients: &[Ingredient]) -> (bool, bool) {
    fn walk(ing: &Ingredient, inherited: InheritedQuality, star: &mut bool, double_star: &mut bool) {
        let own_bio = ing.is_bio == Some(true) || ing.bio_ch == Some(true);
        let own_umst = ing.aus_umstellbetrieb.unwrap_or(false);
        match ing.children.as_ref().filter(|c| !c.is_empty()) {
            Some(children) => {
                let next = InheritedQuality {
                    bio: inherited.bio || own_bio,
                    umstellung: inherited.umstellung || own_umst,
                };
                for c in children {
                    walk(c, next, star, double_star);
                }
            }
            None => {
                let eff_umst = own_umst || (inherited.umstellung && ing.is_agricultural());
                let eff_bio = own_bio || (inherited.bio && ing.is_agricultural());
                if eff_umst {
                    *double_star = true;
                } else if eff_bio {
                    *star = true;
                }
            }
        }
    }
    let (mut star, mut double_star) = (false, false);
    for ing in ingredients {
        walk(ing, InheritedQuality::default(), &mut star, &mut double_star);
    }
    (star, double_star)
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
        use crate::rules::Rule;
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
        use crate::rules::Rule;
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Ingredient>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_namensgebend: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_origins", alias = "origin", skip_serializing_if = "Option::is_none")]
    pub origins: Option<Vec<Country>>,
    #[serde(default = "default_is_agricultural")]
    pub is_agricultural: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_bio: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aufzucht_ort: Option<Country>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schlachtungs_ort: Option<Country>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fangort: Option<Country>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio_ch: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub erlaubte_ausnahme_bio: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub erlaubte_ausnahme_bio_details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub erlaubte_ausnahme_knospe: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub erlaubte_ausnahme_knospe_details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_steps: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aus_umstellbetrieb: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub override_children: Option<bool>,
    /// Canonical food_db name when `name` is a curated alias term (e.g. name
    /// "Mehl" with canonical "Weizenmehl"). Drives allergen/agricultural/category
    /// lookups; `None` when `name` is itself the canonical entry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical: Option<String>,
}

fn default_is_agricultural() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub enum AmountUnit {
    #[default]
    Gram,
    Milliliter,
    /// A child's amount expressed as a percentage of its parent's total. Only
    /// valid on direct children of a composite (percentage mode); resolved to
    /// the parent's gram/ml unit before label generation (see `resolve_percentages`).
    Percent,
}

impl AmountUnit {
    pub fn translation_key(&self) -> &'static str {
        match self {
            AmountUnit::Gram => "units.g",
            AmountUnit::Milliliter => "units.ml",
            AmountUnit::Percent => "units.percent",
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
            canonical: None,
        }
    }

    pub fn is_agricultural(&self) -> bool {
        self.is_agricultural
    }

    /// Category for the Knospe origin rules: the BLV API category when the
    /// ingredient was picked from the API, else the curated food_db category
    /// (looked up via the canonical entry, e.g. "Butter" → "Kochbutter").
    /// Locally sourced ingredients otherwise have `category: None`, which made
    /// the dairy/meat origin rules silently skip them.
    pub fn effective_category(&self) -> Option<String> {
        self.category.clone().or_else(|| {
            crate::model::lookup_category(self.canonical.as_deref().unwrap_or(&self.name))
        })
    }

    /// Quality and origin aggregate **bottom-up**: when an ingredient has children
    /// (and `override_children` is not forcing leaf treatment), the children are the
    /// authoritative source for quality/origin, regardless of whether they carry
    /// weights. Weight, by contrast, is top-down (see `is_leaf`/`computed_amount`).
    fn aggregates_from_children(&self) -> bool {
        !self.override_children.unwrap_or(false)
            && self.children.as_ref().is_some_and(|c| !c.is_empty())
    }

    /// A node makes its own positive quality claim — Knospe / Bio-CH / permitted
    /// exception — which overrides bottom-up derivation from children (e.g. a bought,
    /// certified composite declared Knospe as a whole). Used only for quality, not
    /// origin/weight.
    fn claims_own_quality(&self) -> bool {
        self.is_bio == Some(true)
            || self.bio_ch == Some(true)
            || self.erlaubte_ausnahme_bio == Some(true)
            || self.erlaubte_ausnahme_knospe == Some(true)
    }

    /// Quality aggregates from children only when this node makes no own claim.
    fn aggregates_quality_from_children(&self) -> bool {
        self.aggregates_from_children() && !self.claims_own_quality()
    }

    /// Counts toward Knospe certification: Knospe-certified bio, or a permitted
    /// non-organic / non-Knospe exception (Annex 3 WBF / Bio Suisse Part III).
    /// For composites this aggregates bottom-up: compliant iff every child is.
    pub fn is_knospe_compliant(&self) -> bool {
        if self.aggregates_quality_from_children() {
            return self.children.as_ref().unwrap().iter().all(|c| c.is_knospe_compliant());
        }
        self.is_bio.unwrap_or(false)
            || self.erlaubte_ausnahme_bio.unwrap_or(false)
            || self.erlaubte_ausnahme_knospe.unwrap_or(false)
    }

    /// Counts toward Bio-CH certification: Bio-CH certified and not from a
    /// conversion farm. A permitted non-organic exception (Annex 3 WBF, e.g. Pektin)
    /// is NOT bio — it is tolerated only up to 5% of the agricultural weight, which
    /// the >= 95% Sachbezeichnung threshold enforces, so it must NOT count here.
    /// For composites this aggregates bottom-up: compliant iff every child is.
    pub fn is_bio_ch_compliant(&self) -> bool {
        if self.aggregates_quality_from_children() {
            return self.children.as_ref().unwrap().iter().all(|c| c.is_bio_ch_compliant());
        }
        self.bio_ch.unwrap_or(false) && !self.aus_umstellbetrieb.unwrap_or(false)
    }

    /// An agricultural ingredient that is neither Bio-CH certified, nor declared a
    /// permitted non-organic exception (Anhang 3 WBF), nor from a conversion farm.
    ///
    /// The 5% tolerance applies ONLY to declared exceptions, so any such ingredient
    /// rules out "Bio" regardless of its share — 4% conventional eggs do not become
    /// acceptable just by being small (DEC-7). Mirrors the bottom-up aggregation of
    /// `is_bio_ch_compliant`, so a composite making its own quality claim is judged
    /// on that claim rather than on its children.
    pub fn has_undeclared_non_bio(&self) -> bool {
        if self.aggregates_quality_from_children() {
            return self.children.as_ref().unwrap().iter().any(|c| c.has_undeclared_non_bio());
        }
        self.is_agricultural()
            && !self.bio_ch.unwrap_or(false)
            && !self.erlaubte_ausnahme_bio.unwrap_or(false)
            && !self.aus_umstellbetrieb.unwrap_or(false)
    }

    pub fn composite_name(&self) -> String {
        let mut name = String::new();
        name.push_str(&self.name);
        if let Some(children) = &self.children {
            if !children.is_empty() {
                let children = sort_children_by_weight(children);
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
        self.composites_with_rules(&[], 0.0, 0)
    }

    pub fn composites_with_rules(&self, rules: &[RuleDef], total_amount: f64, agricultural_ingredient_count: usize) -> String {
        // A quality claimed on this composite itself (bought certified unit) is
        // pushed DOWN onto the children's markers (Testing 25.06.2026) — the
        // parent name never carries `*`/`**`.
        self.composites_with_inherited(rules, total_amount, agricultural_ingredient_count, InheritedQuality::from_parent(self))
    }

    fn composites_with_inherited(&self, rules: &[RuleDef], total_amount: f64, agricultural_ingredient_count: usize, inherited: InheritedQuality) -> String {
        let mut output = String::new();
        if let Some(children) = &self.children {
            if !children.is_empty() {
                let has_bio_input_rule = rules.contains(&RuleDef::Bio_Knospe_EingabeIstBio)
                    || rules.contains(&RuleDef::Bio_PartialBioMarking);
                let suppress_asterisk = rules.contains(&RuleDef::Bio_AllAgriAreBio);

                let children = sort_children_by_weight(children);
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
                            // Bio/Umstellbetrieb markers on children — own status OR
                            // inherited from a parent-level claim (agricultural only:
                            // additives/salt never earn a bio marker).
                            if has_bio_input_rule {
                                let is_umstellbetrieb = child.aus_umstellbetrieb.unwrap_or(false)
                                    || (inherited.umstellung && child.is_agricultural());
                                let is_bio = child.computed_bio_status().unwrap_or(false)
                                    || child.computed_bio_ch_status().unwrap_or(false)
                                    || (inherited.bio && child.is_agricultural());
                                if is_umstellbetrieb {
                                    base_name = format!("{}**", base_name);
                                } else if is_bio && !suppress_asterisk {
                                    base_name = format!("{}*", base_name);
                                }
                            }
                            // Namensgebende sub-ingredients print their share of the
                            // WHOLE product (Testing 25.06.2026) — `total_amount` is the
                            // product total; percent-mode children were resolved to grams
                            // up front in `resolve_percentages`.
                            if rules.contains(&RuleDef::AP1_2_ProzentOutputNamensgebend)
                                && child.is_namensgebend == Some(true)
                                && total_amount > 0.0
                            {
                                let percentage = child.computed_amount() / total_amount * 100.0;
                                if percentage > 100.0 {
                                    // LIV Anhang 7: >100% uses the grams-per-100g format
                                    let grams_per_100g = percentage.round() as u32;
                                    base_name = format!(
                                        "{} ({})",
                                        base_name,
                                        t!("label.liv_anhang7_format", grams = grams_per_100g)
                                    );
                                } else if percentage > 0.0 {
                                    base_name = format!("{} {}", base_name, format_percentage(percentage));
                                }
                            }
                            // Recurse into children's children, extending inheritance
                            // with this child's own claim.
                            let child_inherited = InheritedQuality {
                                bio: inherited.bio || child.is_bio == Some(true) || child.bio_ch == Some(true),
                                umstellung: inherited.umstellung || child.aus_umstellbetrieb == Some(true),
                            };
                            base_name.push_str(&child.composites_with_inherited(rules, total_amount, agricultural_ingredient_count, child_inherited));
                            // Add processing steps
                            if let Some(steps) = &child.processing_steps {
                                if !steps.is_empty() {
                                    let steps_text = steps.iter().map(|s| html_escape(s)).collect::<Vec<_>>().join(", ");
                                    base_name = format!("{}, {}", base_name, steps_text);
                                }
                            }
                            // Append origin: when rules are active, respect Knospe rules;
                            // when no rules (basic composites display), always show origins.
                            if rules.is_empty() {
                                if let Some(origin_str) = format_valid_origins(&child.origins) {
                                    base_name = format!("{} {}", base_name, origin_str);
                                }
                            } else if let Some(origin_str) = format_origin_for_knospe_rules(child, rules, total_amount, agricultural_ingredient_count) {
                                base_name = format!("{} {}", base_name, origin_str);
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

    /// Recursively scale amounts by a factor. Percentage-mode children are left
    /// untouched: their share of the parent is fixed, and the derived grams scale
    /// automatically once the parent total scales.
    pub fn scale_recursive(&mut self, factor: f64) {
        self.amount *= factor;
        if let Some(children) = &mut self.children {
            for child in children {
                if child.unit == AmountUnit::Percent {
                    continue;
                }
                child.scale_recursive(factor);
            }
        }
    }

    /// Percentage mode: at least one direct child is expressed as a percentage of
    /// this node's total. In that mode the parent's own `amount` is authoritative
    /// (top-down) and children's grams are derived from it (see `resolve_percentages`).
    fn is_percentage_mode(&self) -> bool {
        self.children
            .as_ref()
            .is_some_and(|c| c.iter().any(|child| child.unit == AmountUnit::Percent))
    }

    /// Is this node a leaf (no children, override active, or children are qualitative-only)?
    /// Qualitative-only children (all amounts zero) means the parent is the authoritative
    /// source for calculations, while children are display-only (for the composites label).
    /// A percentage-mode parent is also authoritative *for weight* (its total drives the
    /// children's grams) — note this governs weight only; quality/origin still aggregate
    /// bottom-up via `aggregates_from_children`.
    fn is_leaf(&self) -> bool {
        self.override_children.unwrap_or(false)
            || self.is_percentage_mode()
            || self.children.as_ref().is_none_or(|c| {
                c.is_empty() || c.iter().all(|child| child.amount == 0.0)
            })
    }

    /// Resolve percentage-mode children into absolute (gram/ml) children, recursively.
    /// A percent child's grams become `parent_total * pct / 100` in the parent's unit.
    /// The parent stays a normal bottom-up composite afterwards, so its weight equals the
    /// sum of the resolved children (== the parent total when the percentages sum to 100%,
    /// the expected case) and quality/origin keep aggregating bottom-up untouched.
    /// Idempotent on trees that contain no percentage children.
    pub fn resolve_percentages(&self) -> Ingredient {
        let mut resolved = self.clone();
        if resolved.is_percentage_mode() {
            let parent_total = resolved.amount;
            let parent_unit = resolved.unit.clone();
            if let Some(children) = resolved.children.as_mut() {
                for child in children.iter_mut() {
                    if child.unit == AmountUnit::Percent {
                        let target = parent_total * child.amount / 100.0;
                        child.unit = parent_unit.clone();
                        // If the child is itself a weighted composite, scale its subtree so
                        // it sums to the target grams (keeps its bottom-up quality/origin
                        // intact); otherwise it's a leaf and the target is simply its amount.
                        let current = child.computed_amount();
                        let is_weighted_composite = child.children.as_ref().is_some_and(|c| {
                            !c.is_empty() && c.iter().any(|g| g.amount != 0.0)
                        });
                        if is_weighted_composite && current > 0.0 {
                            let factor = target / current;
                            if let Some(grandchildren) = child.children.as_mut() {
                                for g in grandchildren.iter_mut() {
                                    g.scale_recursive(factor);
                                }
                            }
                        }
                        child.amount = target;
                    }
                }
            }
        }
        // Recurse so nested composites (percentage or absolute) resolve too.
        if let Some(children) = resolved.children.as_mut() {
            for child in children.iter_mut() {
                *child = child.resolve_percentages();
            }
        }
        resolved
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

    /// Effective unit: own unit if leaf/override; otherwise Milliliter if any
    /// child rolls up to Milliliter, else Gram. Mixing g and ml children is
    /// rare, but a single ml child should pull the parent display to ml so
    /// the form shows e.g. "Vinaigrette 110 ml" not "110 g".
    pub fn computed_unit(&self) -> AmountUnit {
        if self.is_leaf() {
            self.unit.clone()
        } else if self.children.as_ref().unwrap()
            .iter()
            .any(|c| c.computed_unit() == AmountUnit::Milliliter)
        {
            AmountUnit::Milliliter
        } else {
            AmountUnit::Gram
        }
    }

    /// Effective bio status (bottom-up): all-children-bio when it has children,
    /// own value otherwise. Aggregates regardless of child weights, since quality
    /// is a bottom-up attribute (weight is the top-down one).
    pub fn computed_bio_status(&self) -> Option<bool> {
        if self.aggregates_quality_from_children() {
            let children = self.children.as_ref().unwrap();
            if children.iter().any(|c| c.computed_bio_status().is_some()) {
                Some(children.iter().all(|c| c.computed_bio_status().unwrap_or(false)))
            } else {
                None
            }
        } else {
            self.is_bio
        }
    }

    /// Effective bio_ch status: same bottom-up logic as bio
    pub fn computed_bio_ch_status(&self) -> Option<bool> {
        if self.aggregates_quality_from_children() {
            let children = self.children.as_ref().unwrap();
            if children.iter().any(|c| c.computed_bio_ch_status().is_some()) {
                Some(children.iter().all(|c| c.computed_bio_ch_status().unwrap_or(false)))
            } else {
                None
            }
        } else {
            self.bio_ch
        }
    }

    /// Effective origins: origin is defined on a single level. Prefer this node's
    /// own origins when set; otherwise fall back to the union of children's
    /// (bottom-up). `override_children` forces own-value treatment.
    pub fn computed_origins(&self) -> Option<Vec<Country>> {
        if let Some(own) = self.origins.as_ref().filter(|o| !o.is_empty()) {
            return Some(own.clone());
        }
        if self.aggregates_from_children() {
            // Non-agricultural children (e.g. salt, water) carry no country-of-origin
            // declaration, so their origin must not be taken over into the parent's
            // aggregated origin.
            let all: HashSet<Country> = self.children.as_ref().unwrap()
                .iter()
                .filter(|c| c.is_agricultural)
                .filter_map(|c| c.computed_origins())
                .flatten()
                .collect();
            if all.is_empty() { None } else { Some(all.into_iter().collect()) }
        } else {
            self.origins.clone()
        }
    }

    /// Any node in this subtree marked with the Import-(Umstellungs-)Knospe —
    /// Knospe quality claimed with a non-Swiss origin — that lacks a real,
    /// printable country anywhere on its branch. On an Import-Knospe label such
    /// ingredients must declare their origin (Testing 25.06.2026); non-agricultural
    /// nodes never require one.
    fn has_import_knospe_without_origin(&self) -> bool {
        let has_real_origin = self.computed_origins().is_some_and(|o| {
            o.iter().any(|c| !matches!(c, Country::Import | Country::NoOriginRequired))
        });
        let self_flagged = self.is_agricultural()
            && self.is_bio == Some(true)
            && !self.origins.as_ref().is_some_and(|o| o.contains(&Country::CH))
            && !has_real_origin;
        self_flagged
            || self
                .children
                .as_ref()
                .is_some_and(|cs| cs.iter().any(|c| c.has_import_knospe_without_origin()))
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
            canonical: None,
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
        // Composite parents delegate allergen bolding to their children
        // (lowest-level-only). The bold wrap and any origin display are
        // skipped on the parent when it has non-empty children.
        let has_children = self.ingredient.children.as_ref().is_some_and(|c| !c.is_empty());
        let mut output = match self.ingredient.is_allergen && !has_children {
            true => format!("<b>{}</b>", escaped_name),
            false => escaped_name,
        };

        // Umstellbetrieb-Stern (**) vor Bio-Stern (*) prüfen.
        // Markers live on the sub-ingredients, NEVER on a composite parent — a
        // quality claimed at the composite level is pushed down onto the children
        // in `composites_with_rules` instead (Testing 25.06.2026): "Mix (A*, B*)",
        // never "Mix* (…)".
        let is_umstellbetrieb = self.ingredient.aus_umstellbetrieb.unwrap_or(false);
        let is_bio_ingredient = self.ingredient.computed_bio_status().unwrap_or(false)
            || self.ingredient.computed_bio_ch_status().unwrap_or(false);
        let has_bio_input_rule = self.RuleDefs.contains(&RuleDef::Bio_Knospe_EingabeIstBio)
            || self.RuleDefs.contains(&RuleDef::Bio_PartialBioMarking);
        let suppress_asterisk = self.RuleDefs.contains(&RuleDef::Bio_AllAgriAreBio);

        if has_bio_input_rule && is_umstellbetrieb && !has_children {
            // Umstellbetrieb ingredients get ** instead of *
            output = format!("{}**", output);
        } else if has_bio_input_rule && is_bio_ingredient && !suppress_asterisk && !has_children {
            output = format!("{}*", output);
        }

        // Wildsammlung °-marking when ingredient >10%
        let wildsammlung_step = WILDSAMMLUNG_STEP;
        let has_wildsammlung_rule = self.RuleDefs.contains(&RuleDef::Wildsammlung_Ueber10Prozent);
        let has_wildsammlung_step = self.ingredient.processing_steps.as_ref()
            .is_some_and(|s| s.iter().any(|step| step == wildsammlung_step));
        let show_wildsammlung_marker = has_wildsammlung_rule && has_wildsammlung_step
            // Excel Zeile 12: "grösser/gleich 10 %" → inclusive boundary.
            && calculate_ingredient_percentage(self.ingredient.computed_amount(), self.total_amount) >= 10.0;

        if show_wildsammlung_marker {
            output = format!("{}°", output);
        }

        if self
            .RuleDefs.contains(&RuleDef::AP1_2_ProzentOutputNamensgebend)
        {
            if let Some(true) = self.ingredient.is_namensgebend {
                let percentage = self.ingredient.computed_amount() / self.total_amount * 100.;

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
            output = format! {"{}{}", output, self.ingredient.composites_with_rules(&self.RuleDefs, self.total_amount, self.agricultural_ingredient_count)};
        }
        // Verarbeitungsschritte ausgeben (nach Zutatname/Subkomponenten, vor Herkunft)
        // When Wildsammlung °-marker is active, exclude it from the regular processing steps
        if let Some(steps) = &self.ingredient.processing_steps {
            let filtered: Vec<_> = steps.iter()
                .filter(|s| !(show_wildsammlung_marker && s.as_str() == wildsammlung_step))
                // Below the 10% threshold the step is printed inline, so it has to
                // carry the regime's wording just like the legend does (DEC-11).
                .map(|s| {
                    if s.as_str() == wildsammlung_step {
                        html_escape(&wildsammlung_wording(&self.RuleDefs))
                    } else {
                        html_escape(s)
                    }
                })
                .collect();
            if !filtered.is_empty() {
                let steps_text = filtered.join(", ");
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

        // Composite parents normally inherit origin from their children (declared
        // at the lowest level). But origin is single-level and may equally be
        // declared top-down on the composite itself — that declaration must still
        // reach the label instead of being silently dropped (Testing 25.06.2026).
        if has_children {
            if has_declared_origin(&self.ingredient) {
                if let Some(origin_str) = format_origin_for_knospe_rules(&self.ingredient, &self.RuleDefs, self.total_amount, self.agricultural_ingredient_count) {
                    output = format!("{} {}", output, origin_str);
                }
            }
            return output;
        }

        if has_knospe_100_rule || has_knospe_90_99_rule || has_knospe_under90_rule {
            // Knospe origin rules — shared with composite children
            if let Some(origin_str) = format_origin_for_knospe_rules(&self.ingredient, &self.RuleDefs, self.total_amount, self.agricultural_ingredient_count) {
                output = format!("{} {}", output, origin_str);
            }
        } else {
            // Check for beef-specific origin display first.
            // When beef details are rendered, they replace the standard
            // herkunft display so we don't print "(Geburtsort: CH, …) (CH)".
            let mut beef_details_rendered = false;
            if self.RuleDefs.contains(&RuleDef::AP7_4_RindfleischHerkunftDetails) {
                if let Some(category) = &self.ingredient.category {
                    if is_beef_category(category) {
                        let mut beef_origin_parts = Vec::new();

                        if let Some(aufzucht_ort) = &self.ingredient.aufzucht_ort {
                            beef_origin_parts.push(t!("origin.birthplace", country = aufzucht_ort.country_code()).to_string());
                        }

                        if let Some(schlachtungs_ort) = &self.ingredient.schlachtungs_ort {
                            beef_origin_parts.push(t!("origin.slaughtered_in", country = schlachtungs_ort.country_code()).to_string());
                        }

                        if !beef_origin_parts.is_empty() {
                            output = format!("{} ({})", output, beef_origin_parts.join(", "));
                            beef_details_rendered = true;
                        }
                    }
                }
            }
            // Check for fish-specific origin display
            let mut fish_details_rendered = false;
            if self.RuleDefs.contains(&RuleDef::AP7_5_FischFangort) {
                if let Some(category) = &self.ingredient.category {
                    if is_fish_category(category) {
                        if let Some(fangort) = &self.ingredient.fangort {
                            output = format!("{} ({})", output, fangort.country_code());
                            fish_details_rendered = true;
                        }
                    }
                }
            }
            // Add country of origin display for traditional herkunft rules
            // (skipped when beef/fish details already rendered the origin).
            if !beef_details_rendered && !fish_details_rendered {
                if let Some(origin_str) = format_origin_for_knospe_rules(&self.ingredient, &self.RuleDefs, self.total_amount, self.agricultural_ingredient_count) {
                    output = format!("{} {}", output, origin_str);
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

    /// Swiss share of the agricultural weight, as the active configuration means it.
    ///
    /// In a bio context only the certified ingredients count toward "Swiss"; in
    /// the conventional one all of them do. Both Knospe branches (the origin-rule
    /// choice and the logo variant) have to answer this the same way, so the
    /// choice lives here rather than being repeated at each call site.
    fn swiss_agricultural_percentage(&self, ingredients: &[Ingredient]) -> f64 {
        if self.rule_defs.contains(&RuleDef::Bio_Knospe_EingabeIstBio) {
            calculate_bio_swiss_agricultural_percentage(ingredients)
        } else {
            calculate_swiss_agricultural_percentage(ingredients)
        }
    }

    /// Bio-V-Urteil (Sachbezeichnung «Bio», Vermarktung). Reine Funktion der
    /// Rezeptur; die «Rezeptur prüfen»-Schicht kommt in `decide_bio_check` dazu.
    fn decide_bio(&self, ingredients: &[Ingredient]) -> Option<BioVerdict> {
        if !self.rule_defs.contains(&RuleDef::Bio_ShowBioSachbezeichnung) {
            return None;
        }

        let pct = calculate_bio_ch_certified_percentage(ingredients);
        // DEC-7: the 5% tolerance covers ONLY declared permitted exceptions
        // (Anhang 3 WBF). A merely non-organic ingredient rules out «Bio»
        // no matter how small its share.
        let undeclared_non_bio = has_undeclared_non_bio(ingredients);
        // DEC-2: nothing agricultural means nothing to certify.
        let nothing_to_certify = !has_agricultural_ingredient(ingredients);
        let umstellung = has_umstellbetrieb_in_tree(ingredients);

        // Monoprodukt aus Umstellbetrieb (Excel Zeile 7): a single Bio-CH
        // agricultural ingredient from a conversion farm MAY carry «Bio», with
        // the mandatory Umstellungshinweis. is_bio_ch_compliant excludes
        // Umstellbetrieb, so the share alone would say no — this case wins.
        if umstellung && is_mono_product(ingredients) {
            let mono_is_bio_ch = ingredients
                .iter()
                .flat_map(|i| i.leaves())
                .filter(|i| i.is_agricultural())
                .all(|i| i.bio_ch == Some(true));
            if mono_is_bio_ch && !undeclared_non_bio {
                return Some(BioVerdict::Allowed { umstellung_mono: true });
            }
        }

        let qualifies = pct >= 95.0 && !undeclared_non_bio && !nothing_to_certify
            // A composite with an Umstellbetrieb ingredient may not claim «Bio».
            && !umstellung;

        if qualifies {
            return Some(BioVerdict::Allowed { umstellung_mono: false });
        }

        let mut reasons = vec![BioBlockReason::ShareBelow95];
        // The concrete reasons feed their own hint texts; guarded on non-empty
        // recipes, as an empty recipe has nothing to complain about yet.
        if undeclared_non_bio && !ingredients.is_empty() {
            reasons.push(BioBlockReason::UndeclaredNonBio);
        }
        if calculate_erlaubte_ausnahme_bio_percentage(ingredients) > 5.0 {
            reasons.push(BioBlockReason::ExceptionOver5Percent);
        }
        if umstellung && !is_mono_product(ingredients) {
            reasons.push(BioBlockReason::CompositeUmstellung);
        }
        if nothing_to_certify {
            reasons.push(BioBlockReason::NothingToCertify);
        }
        Some(BioVerdict::NotAllowed { reasons })
    }

    /// Knospe-Urteil (Logo, Variante, «Bio»-Suffix).
    fn decide_knospe(&self, ingredients: &[Ingredient]) -> Option<KnospeVerdict> {
        if !self.rule_defs.contains(&RuleDef::Knospe_ShowBioSuisseLogo) {
            return None;
        }

        if ingredients.is_empty() {
            return Some(KnospeVerdict::NoLogo {
                reasons: vec![KnospeBlockReason::NothingToCertify],
            });
        }

        let knospe_percentage = calculate_knospe_certified_percentage(ingredients);
        // Permitted non-organic exceptions count as Knospe-compliant in the
        // percentage, so it alone cannot catch e.g. 40% Pektin. Bio Suisse caps
        // them at 5% of the agricultural weight, same as Bio-V (DEC-8).
        let ausnahme_ueber_grenze =
            calculate_erlaubte_ausnahme_knospe_percentage(ingredients) > 5.0;
        let nothing_to_certify = !has_agricultural_ingredient(ingredients);

        if knospe_percentage >= 100.0 && !ausnahme_ueber_grenze && !nothing_to_certify {
            let umstellung = has_umstellbetrieb_in_tree(ingredients);
            let logo = KnospeLogo {
                // Which artwork depends on the Swiss share of the certified goods.
                swiss_cross: self.swiss_agricultural_percentage(ingredients) >= 90.0,
                umstellung,
            };
            // DEC-10: « Bio» an der Sachbezeichnung, analog Bio-V. Umstellung
            // folgt Excel Zeile 7: nur ein Monoprodukt darf «Bio» tragen.
            let bio_suffix = !umstellung || is_mono_product(ingredients);
            return Some(KnospeVerdict::Logo { logo, bio_suffix });
        }

        let mut reasons = Vec::new();
        if nothing_to_certify {
            reasons.push(KnospeBlockReason::NothingToCertify);
        }
        if knospe_percentage < 100.0 {
            reasons.push(KnospeBlockReason::NotFullyCertified);
        }
        if ausnahme_ueber_grenze {
            reasons.push(KnospeBlockReason::ExceptionOver5Percent);
        }
        Some(KnospeVerdict::NoLogo { reasons })
    }

    /// Tri-State «Rezeptur prüfen». `fulfils` ist das reine Rezeptur-Urteil des
    /// jeweiligen Regimes; diese Schicht faltet den Button-Zustand und offene
    /// Rezeptur-Fehler dazu. Einzelzutat-Modus (DEC-3): kein Urteil.
    fn decide_check(
        input: &Input,
        validation_messages: &HashMap<String, Vec<String>>,
        fulfils: bool,
    ) -> Option<CheckState> {
        if input.ignore_ingredients {
            return None;
        }
        if !input.rezeptur_vollstaendig {
            return Some(CheckState::Pending);
        }
        let has_recipe_issues = validation_messages
            .keys()
            .any(|k| k.starts_with("ingredients["));
        if fulfils && !input.ingredients.is_empty() && !has_recipe_issues {
            Some(CheckState::Ok)
        } else {
            Some(CheckState::Failed)
        }
    }

    pub fn execute(&self, input: Input) -> Output {
        // Debug logging: Show active rules
        self.log_active_rules();

        // Resolve percentage-mode composites into absolute gram/ml children up front,
        // so the entire downstream pipeline (computed_amount, QUID, sorting, validations)
        // operates on plain weights. The persisted Form keeps the percentages.
        let input = Input {
            ingredients: input.ingredients.iter().map(|i| i.resolve_percentages()).collect(),
            ..input
        };

        let mut validation_messages = HashMap::new();

        // DEC-4: the alternative marking wordings ("Alle landwirtschaftlichen Zutaten
        // stammen aus biologischer Landwirtschaft" / "Bio-" prefix) are only truthful
        // when every agricultural ingredient is organic. With a permitted non-organic
        // exception in the recipe, only the per-ingredient *-marking is available.
        let alternative_marking_allowed = !has_erlaubte_ausnahme(&input.ingredients);

        // Calculate total amount first (needed for validations)
        let mut total_amount = input.ingredients.iter().map(|x| x.computed_amount()).sum();
        if self
            .rule_defs.contains(&RuleDef::AP1_4_ManuelleEingabeTotal)
        {
            if let Some(tot) = input.total {
                total_amount = tot;
            }
        }

        // Whether the output label would carry the Import-Knospe (no Swiss cross):
        // 100% Knospe-certified but <90% Swiss share. Computed up front (pure
        // functions) because the origin validator below depends on the logo choice;
        // the logo conditionals themselves are emitted further down.
        let import_knospe_logo_would_show = !input.ingredients.is_empty() && {
            let knospe_pct = calculate_knospe_certified_percentage(&input.ingredients);
            let swiss_pct = if self.rule_defs.contains(&RuleDef::Bio_Knospe_EingabeIstBio) {
                calculate_bio_swiss_agricultural_percentage(&input.ingredients)
            } else {
                calculate_swiss_agricultural_percentage(&input.ingredients)
            };
            knospe_pct >= 100.0 && swiss_pct < 90.0
        };

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
                if let RuleDef::AP1_2_ProzentOutputNamensgebend = ruleDef {
                    self.log_rule_processing(ruleDef, "VALIDATION", Some("Checking namensgebend sub-ingredients carry an amount"));
                    validate_namensgebend_amounts(&input.ingredients, &mut validation_messages)
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
                    self.log_rule_processing(ruleDef, "VALIDATION", Some("Checking origin for Import-Knospe ingredients when the Import-Knospe shows"));
                    validate_import_knospe_origin(&input.ingredients, import_knospe_logo_would_show, &mut validation_messages)
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

        // Config-agnostic: origin must live on a single level per branch. Run
        // once (outside the per-rule loop) whenever the recipe is complete.
        if input.rezeptur_vollstaendig {
            validate_origin_single_level(&input.ingredients, &mut validation_messages);
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

        let namensgebende_zutat_input = self
            .rule_defs
            .contains(&RuleDef::AP1_3_EingabeNamensgebendeZutat);

        let mut sorted_ingredients = input.ingredients.clone();
        sorted_ingredients.sort_by(|y, x| {
            x.computed_amount()
                .partial_cmp(&y.computed_amount())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let manuelles_total_input = self
            .rule_defs
            .contains(&RuleDef::AP1_4_ManuelleEingabeTotal);

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
            let swiss_percentage = self.swiss_agricultural_percentage(&input.ingredients);

            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&format!("🇨🇭 Swiss agricultural percentage: {:.1}%", swiss_percentage).into());

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

        // Knospe: logo, variant and «Bio» suffix — decided as one typed verdict
        // (TD-1 Stufe 2), then mapped onto the conditional keys.
        let knospe_verdict = self.decide_knospe(&input.ingredients);
        let knospe_check = if self.rule_defs.contains(&RuleDef::Knospe_ShowBioSuisseLogo) {
            // The check must agree with the logo gate, otherwise logo and
            // «Rezeptur prüfen» text would contradict each other — which is
            // exactly why both read the same verdict.
            let fulfils = matches!(knospe_verdict, Some(KnospeVerdict::Logo { .. }));
            Self::decide_check(&input, &validation_messages, fulfils)
        } else {
            None
        };


        // Bio-V: «Bio» in der Sachbezeichnung — one typed verdict (TD-1 Stufe 2).
        // The percentage itself is still needed below for the legend variants.
        let has_bio_rule = self.rule_defs.contains(&RuleDef::Bio_ShowBioSachbezeichnung);
        let bio_ch_percentage = if has_bio_rule {
            calculate_bio_ch_certified_percentage(&input.ingredients)
        } else {
            0.0
        };
        let bio_verdict = self.decide_bio(&input.ingredients);
        let bio_check = if has_bio_rule {
            let fulfils = matches!(bio_verdict, Some(BioVerdict::Allowed { .. }));
            Self::decide_check(&input, &validation_messages, fulfils)
        } else {
            None
        };



        let has_meat_rule = self
            .rule_defs.contains(&RuleDef::AP7_3_HerkunftFleischUeber20Prozent);

        let mut origin_required_indices: Vec<usize> = Vec::new();
        if has_50_percent_rule || has_bio_knospe_rule || has_meat_rule {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&"🌍 Analyzing origin requirements for each ingredient:".into());

            for (index, ingredient) in input.ingredients.iter().enumerate() {
                let mut requires_herkunft = false;
                let mut reasons = Vec::new();
                // Use the aggregated weight (computed_amount), NOT the raw entered
                // amount: for a composite the weight lives in its children and the
                // parent's own `amount` is often 0, so the raw value collapses the
                // percentage toward 0 and the >50%/meat-20% flags never fire. This
                // matches the denominator `total_amount` (Σ computed_amount, ~line 1272)
                // and the validators `validate_origin` / `validate_meat_origin`.
                let percentage = calculate_ingredient_percentage(ingredient.computed_amount(), total_amount);

                // Check if >50% rule applies (non-agricultural ingredients never require origin)
                if has_50_percent_rule && percentage > 50.0 && ingredient.is_agricultural() {
                    requires_herkunft = true;
                    reasons.push(format!(">50% ({:.1}%)", percentage));
                }

                // Check if meat rule applies (meat ingredients >20%). NOTE: keys off the
                // ingredient's own `category`, which composite parents usually lack, so a
                // composite meat product won't trigger AP7_3 at the top level (category is
                // not aggregated from children — a separate facet, not fixed here).
                if has_meat_rule && percentage > 20.0 {
                    if let Some(category) = &ingredient.category {
                        if is_meat_category(category) {
                            requires_herkunft = true;
                            reasons.push(format!("meat >20% ({:.1}%)", percentage));
                        }
                    }
                }

                // Bio/Knospe rule (Testing 25.06.2026): origin is required only for
                // Import-Knospe ingredients without a country, and only when the
                // label output is the Import-Knospe.
                if has_bio_knospe_rule
                    && import_knospe_logo_would_show
                    && ingredient.has_import_knospe_without_origin()
                {
                    requires_herkunft = true;
                    reasons.push("Import-Knospe ohne Herkunft".to_string());
                }

                #[cfg(target_arch = "wasm32")]
                if requires_herkunft {
                    web_sys::console::log_1(&format!("  ✅ {} ({:.1}%): Origin required - {}", ingredient.name, percentage, reasons.join(", ")).into());
                } else {
                    web_sys::console::log_1(&format!("  ⚪ {} ({:.1}%): No origin required", ingredient.name, percentage).into());
                }

                if requires_herkunft {
                    origin_required_indices.push(index);
                }
            }
        }

        // All decisions are made — assemble the typed verdicts and derive the
        // legacy key→bool contract from them. This is the only place where
        // verdict → key happens; the exclusivity invariants follow from the
        // enum structure instead of insert/remove discipline.
        let verdicts = Verdicts {
            bio: bio_verdict,
            knospe: knospe_verdict,
            bio_check,
            knospe_check,
            alternative_marking_allowed,
            namensgebende_zutat_input,
            manuelles_total_input,
            origin_required_indices,
        };
        let verdicts_out = verdicts;


        // Prepare rule_defs for OutputFormatter, including the specific Knospe rule
        let mut output_rules = self.rule_defs.clone();
        if let Some(knospe_rule) = actual_knospe_rule {
            // Remove the generic Knospe rules and add the specific one
            output_rules.retain(|rule| !matches!(rule, RuleDef::Knospe_100_Percent_CH_NoOrigin | RuleDef::Knospe_90_99_Percent_CH_ShowOrigin));
            output_rules.push(knospe_rule);
        }

        // Inject Bio marking mode rules (only for Bio config with Bio_ShowBioSachbezeichnung)
        if self.rule_defs.contains(&RuleDef::Bio_ShowBioSachbezeichnung) {
            // Three bands (Excel "Inhaltsverzeichnis_Bio_Zusatz", Zeilen 2–4):
            //   = 100%     → "Alle landwirtschaftlichen … aus biologischer Landwirtschaft", kein *
            //   95–99.99%  → per-Zutat * + "* aus biologischer Landwirtschaft" (weder Rule injiziert;
            //                Bio_Knospe_EingabeIstBio schaltet das * frei, Legende fällt auf den
            //                aus_biologischer_landwirtschaft-Zweig durch)
            //   0–<95%     → per-Zutat * + "x% … aus biologischer Produktion"
            if bio_ch_percentage >= 100.0 {
                output_rules.push(RuleDef::Bio_AllAgriAreBio);
            } else if bio_ch_percentage > 0.0 && bio_ch_percentage < 95.0 {
                output_rules.push(RuleDef::Bio_PartialBioMarking);
            }
        }

        // Final summary logging
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::log_1(&"📈 Final Results".into());
            web_sys::console::log_1(&format!("✅ Label generation complete - {} ingredients processed", sorted_ingredients.len()).into());
            web_sys::console::log_1(&format!("📋 {} validation messages", validation_messages.len()).into());
            web_sys::console::log_1(&format!("⚖️ Total amount: {}g", total_amount).into());
        }

        // Prüfe ob Bio-Zutaten oder Umstellbetrieb vorhanden sind (für Legende).
        // Die Legende spiegelt die tatsächlich gedruckten Marker inkl.
        // Parent-Claim-Push-down: ein einfaches `*` nur für bio ohne Umstellung
        // (Umstellung kriegt stattdessen `**`), beides über den ganzen Baum.
        let has_bio_rules = output_rules.contains(&RuleDef::Bio_Knospe_EingabeIstBio)
            || output_rules.contains(&RuleDef::Bio_AllAgriAreBio)
            || output_rules.contains(&RuleDef::Bio_PartialBioMarking);
        let (tree_has_star, tree_has_double_star) = tree_marker_presence(&sorted_ingredients);
        let has_bio_ingredients = has_bio_rules && tree_has_star;
        let has_umstellbetrieb = has_bio_rules && tree_has_double_star;

        // Count agricultural ingredients for Monoprodukt detection in OutputFormatter
        let agricultural_ingredient_count = sorted_ingredients.iter()
            .flat_map(|i| i.leaves())
            .filter(|i| i.is_agricultural())
            .count();

        // Check for Wildsammlung legend (before sorted_ingredients is consumed)
        let has_wildsammlung_marker = output_rules.contains(&RuleDef::Wildsammlung_Ueber10Prozent)
            && sorted_ingredients.iter().any(|ing| {
                let pct = calculate_ingredient_percentage(ing.computed_amount(), total_amount);
                pct >= 10.0 && ing.processing_steps.as_ref()
                    .is_some_and(|s| s.iter().any(|step| step == WILDSAMMLUNG_STEP))
            });

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

        // Append Wildsammlung legend if any ingredient got the ° marker
        if has_wildsammlung_marker {
            label = format!("{}<br>° {}", label, wildsammlung_wording(&output_rules));
        }

        // Einzelzutat/Monoprodukt («Keine Zutatenliste»): the declared quality is
        // fed in as a synthetic ingredient so the Bio/Knospe rules can run, but it
        // must never be printed — the label shows no ingredient list at all (DEC-2).
        if input.ignore_ingredients {
            label = String::new();
        }

        Output {
            success: true,
            label,
            total_amount,
            validation_messages,
            verdicts: verdicts_out,
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
        let percentage = calculate_ingredient_percentage(ingredient.computed_amount(), total_amount);
        // Respect bottom-up origin: a composite satisfies the requirement when its
        // sub-ingredients carry origins, even if the parent declares none.
        let has_origin = ingredient.computed_origins().is_some_and(|v| !v.is_empty());
        if percentage > 50.0 && !has_origin && ingredient.is_agricultural() {
            validation_messages.entry(format!("ingredients[{}][origin]", i))
                .or_default()
                .push(t!("validation.origin_required_over_50_percent").to_string());
        }
    }
}

// Functions are already imported above, no need to re-export

// Use centralized category service functions

/// Format origin display string for an ingredient according to active Knospe rules.
/// Returns None if origin should not be displayed, Some("(CH)") etc. if it should.
/// Used by both OutputFormatter::format() and composites_with_rules() for consistency.
fn format_origin_for_knospe_rules(ingredient: &Ingredient, rules: &[RuleDef], total_amount: f64, agricultural_ingredient_count: usize) -> Option<String> {
    let has_knospe_100_rule = rules.contains(&RuleDef::Knospe_100_Percent_CH_NoOrigin);
    let has_knospe_90_99_rule = rules.contains(&RuleDef::Knospe_90_99_Percent_CH_ShowOrigin);
    let has_knospe_under90_rule = rules.contains(&RuleDef::Knospe_Under90_Percent_CH_IngredientRules);

    if has_knospe_100_rule {
        // Rule A: 100% Swiss agricultural ingredients — no origin display
        None
    } else if has_knospe_90_99_rule {
        // Rule B: 90-99.99% Swiss — show origin for Swiss agricultural ingredients only
        if ingredient.is_agricultural() && ingredient.computed_origins().is_some_and(|o| o.contains(&Country::CH)) {
            Some("(CH)".to_string())
        } else {
            None
        }
    } else if has_knospe_under90_rule {
        // Rule C: <90% Swiss — show origin based on specific ingredient criteria
        let percentage = calculate_ingredient_percentage(ingredient.computed_amount(), total_amount);
        let is_mono_product = agricultural_ingredient_count == 1;

        if should_show_origin_knospe_under90(ingredient, percentage, total_amount, is_mono_product) {
            format_valid_origins(&ingredient.computed_origins())
        } else {
            None
        }
    } else {
        // No Knospe rules — show origins if traditional herkunft rules apply
        let has_herkunft_rule = rules.iter().any(|x|
            *x == RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent
            || *x == RuleDef::AP7_3_HerkunftFleischUeber20Prozent
            || *x == RuleDef::Knospe_AlleZutatenHerkunft
        );
        if has_herkunft_rule {
            format_valid_origins(&ingredient.computed_origins())
        } else {
            None
        }
    }
}

/// Stable-sort a copy of the children by computed weight, descending — mirroring the
/// top-level ingredient sort. Because the sort is stable, zero-weight (qualitative)
/// children keep their manual insertion order, which encodes the legally-relevant ranking
/// when weights are missing.
fn sort_children_by_weight(children: &[Ingredient]) -> Vec<Ingredient> {
    let mut sorted = children.to_vec();
    sorted.sort_by(|a, b| {
        b.computed_amount()
            .partial_cmp(&a.computed_amount())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    sorted
}

/// Format valid origins into a parenthetical string, dropping placeholders that
/// must never reach the label: `NoOriginRequired`, and the generic `Import`
/// sentinel (imported, country unspecified — declaring "(Import)" is not valid).
fn format_valid_origins(origins: &Option<Vec<Country>>) -> Option<String> {
    origins.as_ref().and_then(|origins| {
        let valid: Vec<&str> = origins
            .iter()
            .filter(|o| !matches!(o, Country::NoOriginRequired | Country::Import))
            .map(|o| o.country_code())
            .collect();
        if valid.is_empty() {
            None
        } else {
            Some(format!("({})", valid.join(", ")))
        }
    })
}

/// Determines if an ingredient should show origin for Knospe <90% CH rules
/// Based on specific Knospe criteria for ingredient types and percentages
fn should_show_origin_knospe_under90(ingredient: &Ingredient, percentage: f64, _total_amount: f64, is_mono_product: bool) -> bool {
    // Non-agricultural ingredients (water, salt, additives like Dicarbonat) never require
    // or show an origin — this MUST win over the Monoprodukt short-circuit below, which
    // otherwise flags every origin-less ingredient once the product has a single agri leaf.
    if !ingredient.is_agricultural() {
        return false;
    }

    // For monoproducts (single ingredient products), always show origin
    if is_mono_product {
        return true;
    }

    // Name-giving ingredients (namensgebende Zutat) always show origin for Knospe
    if ingredient.is_namensgebend == Some(true) {
        return true;
    }

    // Category-based rules (only apply when ingredient has a recognized category)
    if let Some(category) = &ingredient.effective_category() {
        // Plant ingredients with more than 50% share
        if is_plant_category(category) && percentage > 50.0 {
            return true;
        }

        // Eggs/Honey/Fish/Other aquacultures with more than 10% share
        if (is_egg_category(category) ||
            is_honey_category(category) ||
            is_fish_category(category)) && percentage > 10.0 {
            return true;
        }

        // Milk/Dairy/Meat/Insects always show origin
        if is_dairy_category(category) ||
           is_meat_category(category) ||
           is_insect_category(category) {
            return true;
        }

    }

    // Swiss agricultural ingredients with >=10% share (regardless of category)
    if ingredient.is_agricultural() &&
       ingredient.computed_origins().is_some_and(|o| o.contains(&Country::CH)) &&
       percentage >= 10.0 {
        return true;
    }

    false
}

fn validate_meat_origin(
    ingredients: &[Ingredient],
    total_amount: f64,
    validation_messages: &mut HashMap<String, Vec<String>>,
) {
    for (i, ingredient) in ingredients.iter().enumerate() {
        let percentage = calculate_ingredient_percentage(ingredient.computed_amount(), total_amount);
        if percentage > 20.0 {
            // Check if this ingredient is meat-based using the category
            if let Some(category) = &ingredient.category {
                let has_origin = ingredient.computed_origins().is_some_and(|v| !v.is_empty());
                if is_meat_category(category) && !has_origin {
                    validation_messages.entry(format!("ingredients[{}][origin]", i))
                        .or_default()
                        .push(t!("validation.origin_required_meat_over_20").to_string());
                }
            }
        }
    }
}

/// Namensgebende sub-ingredients must carry an amount — without one their
/// percentage of the end product cannot appear on the label (Testing
/// 25.06.2026: missing Rosinen-%, BioVo Himbeere case). Checks all depths
/// below the top level; top-level amounts are covered by `validate_amount`.
fn validate_namensgebend_amounts(
    ingredients: &[Ingredient],
    validation_messages: &mut HashMap<String, Vec<String>>,
) {
    fn subtree_has_zero_namensgebend(ing: &Ingredient) -> bool {
        (ing.is_namensgebend == Some(true) && ing.computed_amount() <= 0.0)
            || ing
                .children
                .as_ref()
                .is_some_and(|cs| cs.iter().any(subtree_has_zero_namensgebend))
    }
    for (i, ingredient) in ingredients.iter().enumerate() {
        let flagged = ingredient
            .children
            .as_ref()
            .is_some_and(|cs| cs.iter().any(subtree_has_zero_namensgebend));
        if flagged {
            validation_messages.entry(format!("ingredients[{}][amount]", i))
                .or_default()
                .push(t!("validation.namensgebend_amount_required").to_string());
        }
    }
}

/// Reworked per Testing 25.06.2026: origin is required only for ingredients
/// carrying the Import-(Umstellungs-)Knospe without a real country, and only
/// when the label output is the Import-Knospe. Non-agricultural ingredients
/// (e.g. Dicarbonat) never require an origin.
fn validate_import_knospe_origin(
    ingredients: &[Ingredient],
    import_knospe_logo_would_show: bool,
    validation_messages: &mut HashMap<String, Vec<String>>,
) {
    if !import_knospe_logo_would_show {
        return;
    }
    for (i, ingredient) in ingredients.iter().enumerate() {
        if ingredient.has_import_knospe_without_origin() {
            validation_messages.entry(format!("ingredients[{}][origin]", i))
                .or_default()
                .push(t!("validation.origin_required_import_knospe").to_string());
        }
    }
}

/// True when this node carries an explicitly declared origin (the
/// `NoOriginRequired` sentinel does not count as a declaration).
fn has_declared_origin(ing: &Ingredient) -> bool {
    ing.origins
        .as_ref()
        .is_some_and(|o| o.iter().any(|c| !matches!(c, Country::NoOriginRequired)))
}

/// Detects whether origin is declared on more than one level within a single
/// composite branch (an ancestor and one of its descendants both carry one).
fn branch_origin_conflict(ing: &Ingredient, ancestor_has_origin: bool) -> bool {
    let self_has = has_declared_origin(ing);
    if self_has && ancestor_has_origin {
        return true;
    }
    let seen = ancestor_has_origin || self_has;
    ing.children
        .as_ref()
        .is_some_and(|children| children.iter().any(|c| branch_origin_conflict(c, seen)))
}

/// Origin must be defined on exactly one level per branch (top-down on the
/// composite OR bottom-up on its sub-ingredients, never both). Emits a warning
/// on the top-level ingredient's origin path when a branch defines it twice.
fn validate_origin_single_level(
    ingredients: &[Ingredient],
    validation_messages: &mut HashMap<String, Vec<String>>,
) {
    for (i, ingredient) in ingredients.iter().enumerate() {
        if branch_origin_conflict(ingredient, false) {
            validation_messages
                .entry(format!("ingredients[{}][origin]", i))
                .or_default()
                .push(t!("validation.origin_single_level").to_string());
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
        let percentage = calculate_ingredient_percentage(ingredient.computed_amount(), total_amount);
        let is_mono_product = agricultural_count == 1;

        // `requires_origin` keeps the Excel category/percentage thresholds as-is;
        // only the presence check aggregates bottom-up (composite origin on children).
        let requires_origin = should_show_origin_knospe_under90(ingredient, percentage, total_amount, is_mono_product);
        let has_origin = ingredient.computed_origins().is_some_and(|v| !v.is_empty());

        if requires_origin && !has_origin {
            let reason = if is_mono_product {
                t!("validation.knospe_mono_origin_required").to_string()
            } else if ingredient.is_namensgebend == Some(true) {
                t!("validation.knospe_name_giving_origin_required").to_string()
            } else if let Some(category) = &ingredient.effective_category() {
                if is_plant_category(category) && percentage > 50.0 {
                    t!("validation.knospe_plants_over_50_origin_required").to_string()
                } else if (is_egg_category(category) || is_honey_category(category) || is_fish_category(category)) && percentage > 10.0 {
                    t!("validation.knospe_egg_honey_fish_origin_required").to_string()
                } else if is_dairy_category(category) || is_meat_category(category) || is_insect_category(category) {
                    t!("validation.knospe_dairy_meat_insects_origin_required").to_string()
                } else if ingredient.is_agricultural() &&
                          ingredient.computed_origins().is_some_and(|o| o.contains(&Country::CH)) &&
                          percentage >= 10.0 {
                    t!("validation.knospe_over_10_percent_origin_required").to_string()
                } else {
                    t!("validation.knospe_general_origin_required").to_string()
                }
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
