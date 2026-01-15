use crate::model::{lookup_allergen, lookup_agricultural, Country};
use crate::rules::{RuleDef, Rule};
use crate::category_service::{is_fish_category, is_beef_category, is_meat_category, is_egg_category, is_honey_category, is_dairy_category, is_insect_category, is_plant_category};
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::mem;

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
            ingredient.amount *= factor;
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

fn calculate_swiss_agricultural_percentage(ingredients: &Vec<Ingredient>) -> f64 {
    let total_agricultural_amount: f64 = ingredients
        .iter()
        .filter(|ingredient| ingredient.is_agricultural())
        .map(|ingredient| ingredient.amount)
        .sum();

    if total_agricultural_amount == 0.0 {
        return 0.0;
    }

    let swiss_agricultural_amount: f64 = ingredients
        .iter()
        .filter(|ingredient| ingredient.is_agricultural())
        .filter(|ingredient| matches!(ingredient.origin, Some(Country::CH)))
        .map(|ingredient| ingredient.amount)
        .sum();

    (swiss_agricultural_amount / total_agricultural_amount) * 100.0
}

fn calculate_bio_swiss_agricultural_percentage(ingredients: &Vec<Ingredient>) -> f64 {
    let total_bio_agricultural_amount: f64 = ingredients
        .iter()
        .filter(|ingredient| ingredient.is_agricultural())
        .filter(|ingredient| ingredient.is_bio.unwrap_or(false))
        .map(|ingredient| ingredient.amount)
        .sum();

    if total_bio_agricultural_amount == 0.0 {
        return 0.0;
    }

    let swiss_bio_agricultural_amount: f64 = ingredients
        .iter()
        .filter(|ingredient| ingredient.is_agricultural())
        .filter(|ingredient| ingredient.is_bio.unwrap_or(false))
        .filter(|ingredient| matches!(ingredient.origin, Some(Country::CH)))
        .map(|ingredient| ingredient.amount)
        .sum();

    (swiss_bio_agricultural_amount / total_bio_agricultural_amount) * 100.0
}

fn calculate_knospe_certified_percentage(ingredients: &Vec<Ingredient>) -> f64 {
    let total_agricultural_amount: f64 = ingredients
        .iter()
        .filter(|ingredient| ingredient.is_agricultural())
        .map(|ingredient| ingredient.amount)
        .sum();

    if total_agricultural_amount == 0.0 {
        return 100.0; // No agricultural ingredients means 100% compliance (only water/salt)
    }

    let knospe_certified_amount: f64 = ingredients
        .iter()
        .filter(|ingredient| ingredient.is_agricultural())
        .filter(|ingredient| ingredient.is_bio.unwrap_or(false))
        .map(|ingredient| ingredient.amount)
        .sum();

    (knospe_certified_amount / total_agricultural_amount) * 100.0
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
    pub sub_components: Option<Vec<SubIngredient>>,
    pub is_namensgebend: Option<bool>,
    pub origin: Option<Country>,
    #[serde(default = "default_is_agricultural")]
    pub is_agricultural: bool,
    pub is_bio: Option<bool>,
    pub category: Option<String>,
    pub aufzucht_ort: Option<Country>,
    pub schlachtungs_ort: Option<Country>,
    pub fangort: Option<Country>,
    pub aus_umstellbetrieb: Option<bool>,
    pub bio_ch: Option<bool>,
}

fn default_is_agricultural() -> bool {
    true
}

impl Ingredient {
    pub fn from_name_amount(name: String, amount: f64) -> Self {
        Self {
            name: name.clone(),
            is_allergen: lookup_allergen(&name),
            is_agricultural: lookup_agricultural(&name),
            amount,
            sub_components: None,
            is_namensgebend: None,
            origin: None,
            is_bio: None,
            category: None,
            aufzucht_ort: None,
            schlachtungs_ort: None,
            fangort: None,
            aus_umstellbetrieb: None,
            bio_ch: None,
        }
    }

    pub fn is_agricultural(&self) -> bool {
        self.is_agricultural
    }

    pub fn composite_name(&self) -> String {
        let mut name = String::new();
        name.push_str(&self.name);
        if let Some(subs) = &self.sub_components {
            if !subs.is_empty() {
                name.push_str(" (");
                name.push_str(
                    &subs
                        .iter()
                        .map(|sub| sub.name.clone())
                        .collect::<Vec<String>>()
                        .join(", "),
                );
                name.push_str(" )");
            }
        }
        name
    }

    pub fn composites(&self) -> String {
        let mut name = String::new();
        if let Some(subs) = &self.sub_components {
            if !subs.is_empty() {
                name.push_str(" (");
                name.push_str(
                    &subs
                        .iter()
                        .map(|sub| {
                            if sub.is_allergen {
                                format!("<b>{}</b>", sub.name)
                            } else {
                                sub.name.clone()
                            }
                        })
                        .collect::<Vec<String>>()
                        .join(", "),
                );
                name.push(')');
            }
        }
        name
    }
}

impl Default for Ingredient {
    fn default() -> Self {
        Self {
            name: String::new(),
            is_allergen: false,
            amount: 0.,
            sub_components: Some(vec![]),
            is_namensgebend: None,
            origin: None,
            is_agricultural: true,
            is_bio: None,
            category: None,
            aufzucht_ort: None,
            schlachtungs_ort: None,
            fangort: None,
            aus_umstellbetrieb: None,
            bio_ch: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SubIngredient {
    pub name: String,
    pub is_allergen: bool,
}

struct OutputFormatter {
    ingredient: Ingredient,
    RuleDefs: Vec<RuleDef>,
    total_amount: f64,
}

impl PartialEq for RuleDef {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}

impl OutputFormatter {
    pub fn from(ingredient: Ingredient, total_amount: f64, RuleDefs: Vec<RuleDef>) -> Self {
        Self {
            ingredient,
            total_amount,
            RuleDefs,
        }
    }

    pub fn format(&self) -> String {
        let mut output = match self.ingredient.is_allergen {
            true => format! {"<b>{}</b>", self.ingredient.name},
            false => self.ingredient.name.clone(),
        };
        if self.RuleDefs.contains(&RuleDef::AllPercentages) {
            let percentage = self.ingredient.amount / self.total_amount * 100.;
            output = format!(
                "{} {}",
                output,
                format_percentage(percentage)
            )
        }
        if self
            .RuleDefs.contains(&RuleDef::PercentagesStartsWithM)
            && self.ingredient.name.starts_with("M")
        {
            let percentage = self.ingredient.amount / self.total_amount * 100.;
            output = format!(
                "{} {}",
                output,
                format_percentage(percentage)
            )
        }
        // if self.RuleDefs.iter().find(|x| **x == RuleDef::MaxDetails).is_some() {
        //     output = format!{"{:?}", self.ingredient}
        // }
        if self.RuleDefs.contains(&RuleDef::AllGram) {
            output = format! {"{} {}g", self.ingredient.name, self.ingredient.amount};
        }
        if self
            .RuleDefs.contains(&RuleDef::AP1_2_ProzentOutputNamensgebend)
        {
            if let Some(true) = self.ingredient.is_namensgebend {
                let percentage = self.ingredient.amount / self.total_amount * 100.;
                output = format!(
                    "{} {}",
                    output,
                    format_percentage(percentage)
                )
            }
        }
        if self
            .RuleDefs.contains(&RuleDef::AP2_1_ZusammegesetztOutput)
            && self.ingredient.sub_components.is_some()
        {
            output = format! {"{} {}", output, self.ingredient.composites()};
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
            if let Some(Country::CH) = &self.ingredient.origin {
                output = format!("{} {}", output, t!("origin.switzerland_parentheses"));
            }
        } else if has_knospe_under90_rule {
            // Rule C: <90% Swiss agricultural ingredients - show origin based on specific ingredient criteria
            let percentage = calculate_ingredient_percentage(self.ingredient.amount, self.total_amount);
            let is_mono_product = self.total_amount == self.ingredient.amount; // Simple check for single ingredient

            if should_show_origin_knospe_under90(&self.ingredient, percentage, self.total_amount, is_mono_product) {
                if let Some(origin) = &self.ingredient.origin {
                    if !matches!(origin, Country::NoOriginRequired) {
                        let country_name = origin.display_name();
                        output = format!("{} ({})", output, country_name);
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
                            beef_origin_parts.push(t!("birthplace", country = aufzucht_ort.display_name()).to_string());
                        }

                        if let Some(schlachtungs_ort) = &self.ingredient.schlachtungs_ort {
                            beef_origin_parts.push(t!("slaughtered_in", country = schlachtungs_ort.display_name()).to_string());
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
                            output = format!("{} ({})", output, fangort.display_name());
                        }
                    }
                }
            }
            // Always display country of origin when set (CH food law requirement)
            else if let Some(origin) = &self.ingredient.origin {
                // Don't show origin for "NoOriginRequired"
                if !matches!(origin, Country::NoOriginRequired) {
                    let country_name = origin.display_name();
                    output = format!("{} ({})", output, country_name);
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

    pub fn validate_rule_dependencies(&self) -> Result<(), String> {
        use crate::rules::RuleRegistry;
        let registry = RuleRegistry::new();
        registry.validate_dependencies(&self.rule_defs)
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
        sorted_ingredients.sort_by(|y, x| x.amount.partial_cmp(&y.amount).unwrap());

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

                // Check if any ingredient needs Umstellung logo
                let has_umstellung = input.ingredients.iter().any(|ing|
                    ing.aus_umstellbetrieb.unwrap_or(false) || ing.bio_ch.unwrap_or(false)
                );

                if swiss_percentage >= 90.0 {
                    // Knospe with Swiss cross (>= 90% Swiss)
                    if has_umstellung {
                        // Use Umstellung logo instead of regular BioSuisse when Swiss percentage >= 90%
                        conditionals.insert(String::from("bio_suisse_umstellung"), true);

                        // Determine which message to show
                        let has_bio_ch = input.ingredients.iter().any(|ing|
                            ing.bio_ch.unwrap_or(false)
                        );

                        if has_bio_ch {
                            conditionals.insert(String::from("umstellung_bio_suisse_richtlinien"), true);
                        } else {
                            conditionals.insert(String::from("umstellung_biologische_landwirtschaft"), true);
                        }
                    } else {
                        // Regular BioSuisse logo with Swiss cross
                        conditionals.insert(String::from("bio_suisse_regular"), true);
                    }
                } else {
                    // Knospe without Swiss cross (< 90% Swiss, including 0% Swiss)
                    if has_umstellung {
                        // Bio Knospe Umstellung (without Swiss cross)
                        conditionals.insert(String::from("bio_suisse_no_cross_umstellung"), true);

                        // Determine which message to show
                        let has_bio_ch = input.ingredients.iter().any(|ing|
                            ing.bio_ch.unwrap_or(false)
                        );

                        if has_bio_ch {
                            conditionals.insert(String::from("umstellung_bio_suisse_richtlinien"), true);
                        } else {
                            conditionals.insert(String::from("umstellung_biologische_landwirtschaft"), true);
                        }
                    } else {
                        conditionals.insert(String::from("bio_suisse_no_cross"), true);
                    }
                }
            } else {
                #[cfg(target_arch = "wasm32")]
                web_sys::console::log_1(&format!("⚠️ Not all ingredients are Knospe-certified ({:.1}%), no logo will be shown", knospe_percentage).into());
            }
        }

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

        // Final summary logging
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::log_1(&"📈 Final Results".into());
            web_sys::console::log_1(&format!("✅ Label generation complete - {} ingredients processed", sorted_ingredients.len()).into());
            web_sys::console::log_1(&format!("📋 {} validation messages", validation_messages.len()).into());
            web_sys::console::log_1(&format!("🎛️ {} conditional elements enabled", conditionals.len()).into());
            web_sys::console::log_1(&format!("⚖️ Total amount: {}g", total_amount).into());
        }

        Output {
            success: true,
            label: sorted_ingredients
                .into_iter()
                .map(|item| OutputFormatter::from(item, total_amount, output_rules.clone()))
                .map(|fmt| fmt.format())
                .collect::<Vec<_>>()
                .join(", "),
            total_amount,
            validation_messages,
            conditional_elements: conditionals,
        }
    }
}

fn validate_amount(ingredients: &Vec<Ingredient>, validation_messages: &mut HashMap<String, Vec<String>>) {
    for (i, ingredient) in ingredients.iter().enumerate() {
        if ingredient.amount <= 0. {
            validation_messages.entry(format!("ingredients[{}][amount]", i))
                .or_default()
                .push(t!("validation.amount_greater_than_zero").to_string());
        }
    }
}

fn validate_origin(
    ingredients: &Vec<Ingredient>,
    total_amount: f64,
    validation_messages: &mut HashMap<String, Vec<String>>,
) {
    for (i, ingredient) in ingredients.iter().enumerate() {
        let percentage = calculate_ingredient_percentage(ingredient.amount, total_amount);
        if percentage > 50.0 && ingredient.origin.is_none() {
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
    if matches!(ingredient.origin, Some(Country::CH)) && percentage >= 10.0 {
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
    ingredients: &Vec<Ingredient>,
    total_amount: f64,
    validation_messages: &mut HashMap<String, Vec<String>>,
) {
    for (i, ingredient) in ingredients.iter().enumerate() {
        let percentage = calculate_ingredient_percentage(ingredient.amount, total_amount);
        if percentage > 20.0 {
            // Check if this ingredient is meat-based using the category
            if let Some(category) = &ingredient.category {
                if is_meat_category(category) && ingredient.origin.is_none() {
                    validation_messages.entry(format!("ingredients[{}][origin]", i))
                        .or_default()
                        .push(t!("validation.origin_required_meat_over_20").to_string());
                }
            }
        }
    }
}

fn validate_all_ingredients_origin(
    ingredients: &Vec<Ingredient>,
    validation_messages: &mut HashMap<String, Vec<String>>,
) {
    for (i, ingredient) in ingredients.iter().enumerate() {
        if ingredient.origin.is_none() {
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
    if certification_body.is_none() || certification_body.as_ref().map_or(true, |s| s.is_empty()) {
        validation_messages.entry("certification_body".to_string())
            .or_default()
            .push(t!("validation.certification_body_required").to_string());
    }
}

fn validate_beef_origin_details(
    ingredients: &Vec<Ingredient>,
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
    ingredients: &Vec<Ingredient>,
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
    ingredients: &Vec<Ingredient>,
    total_amount: f64,
    validation_messages: &mut HashMap<String, Vec<String>>,
) {
    for (i, ingredient) in ingredients.iter().enumerate() {
        let percentage = calculate_ingredient_percentage(ingredient.amount, total_amount);
        let is_mono_product = total_amount == ingredient.amount; // Simple check for single ingredient

        let requires_origin = should_show_origin_knospe_under90(ingredient, percentage, total_amount, is_mono_product);

        if requires_origin && ingredient.origin.is_none() {
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
mod tests {
    use super::*;
    use rust_i18n::i18n;

    // Initialize i18n for tests at module level
    i18n!();

    fn setup_simple_calculator() -> Calculator {
        // Set locale for tests
        rust_i18n::set_locale("de-CH");
        let rule_defs = vec![];
        Calculator { rule_defs }
    }

    #[test]
    fn simple_run_of_process() {
        let calculator = setup_simple_calculator();
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![],
            ..Default::default()
        };

        let output = calculator.execute(input);
        assert!(output.success);
    }

    #[test]
    fn single_ingredient_visible_on_label() {
        let calculator = setup_simple_calculator();
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![Ingredient {
                name: "Hafer".to_string(),
                is_allergen: false,
                amount: 42.,
                ..Default::default()
            }],
            ..Default::default()
        };
        let output = calculator.execute(input);
        let label = output.label;
        assert!(label.contains("Hafer"));
    }

    #[test]
    fn multiple_ingredients_comma_separated_on_label() {
        let calculator = setup_simple_calculator();
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![
                Ingredient {
                    name: "Hafer".to_string(),
                    is_allergen: false,
                    amount: 42.,
                    ..Default::default()
                },
                Ingredient {
                    name: "Zucker".to_string(),
                    is_allergen: false,
                    amount: 42.,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let output = calculator.execute(input);
        let label = output.label;
        assert!(label.contains("Hafer, Zucker"));
    }

    #[test]
    fn ingredients_ordered_by_weight_on_label() {
        let calculator = setup_simple_calculator();
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![
                Ingredient {
                    name: "Hafer".to_string(),
                    is_allergen: false,
                    amount: 300.,
                    ..Default::default()
                },
                Ingredient {
                    name: "Zucker".to_string(),
                    is_allergen: false,
                    amount: 700.,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let output = calculator.execute(input);
        let label = output.label;
        assert!(label.contains("Zucker, Hafer"));
    }

    #[test]
    fn allergenes_printed_bold_on_label() {
        let calculator = setup_simple_calculator();
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![Ingredient {
                name: "Weizenmehl".to_string(),
                is_allergen: true,
                amount: 300.,
                ..Default::default()
            }],
            ..Default::default()
        };
        let output = calculator.execute(input);
        let label = output.label;
        assert!(label.contains("<b>Weizenmehl</b>"));
    }

    #[test]
    fn scaled_recipe_invariant_on_label() {
        let calculator = setup_simple_calculator();
        let input1 = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![
                Ingredient {
                    name: "Hafer".to_string(),
                    is_allergen: false,
                    amount: 300.,
                    ..Default::default()
                },
                Ingredient {
                    name: "Zucker".to_string(),
                    is_allergen: false,
                    amount: 700.,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let mut input2 = input1.clone();
        input2.scale(2.);
        let output = calculator.execute(input1);
        let scaled_output = calculator.execute(input2);

        assert_eq!(output.label, scaled_output.label);
        assert_ne!(output.total_amount, scaled_output.total_amount);
    }

    #[test]
    fn percentage_on_label_depending_on_setting() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AllPercentages]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![
                Ingredient {
                    name: "Hafer".to_string(),
                    is_allergen: false,
                    amount: 300.,
                    ..Default::default()
                },
                Ingredient {
                    name: "Zucker".to_string(),
                    is_allergen: false,
                    amount: 700.,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let output = calculator.execute(input);
        let label = output.label;
        assert!(label.contains("Hafer 30%"));
        assert!(label.contains("Zucker 70%"));
    }

    #[test]
    fn percentage_on_label_depending_on_setting_2() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::PercentagesStartsWithM]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![
                Ingredient {
                    name: "Hafer".to_string(),
                    is_allergen: false,
                    amount: 300.,
                    ..Default::default()
                },
                Ingredient {
                    name: "Milch".to_string(),
                    is_allergen: true,
                    amount: 700.,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let output = calculator.execute(input);
        let label = output.label;
        assert!(label.contains("<b>Milch</b> 70%, Hafer"));
    }

    #[test]
    fn amount_lt_zero_invalid() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AP1_1_ZutatMengeValidierung]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![Ingredient {
                name: "Hafer".to_string(),
                is_allergen: false,
                amount: 0.0,
                ..Default::default()
            }],
            ..Default::default()
        };
        let output = calculator.execute(input);
        let validation_messages = output.validation_messages;
        assert!(validation_messages.get("ingredients[0][amount]").is_some());
        let amount_messages = validation_messages.get("ingredients[0][amount]").unwrap();
        assert!(!amount_messages.is_empty());
        assert!(amount_messages.iter().any(|m| m == "Die Menge muss grösser als 0 sein."));
    }

    #[test]
    fn amount_gt_zero_valid() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AP1_1_ZutatMengeValidierung]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![Ingredient {
                name: "Hafer".to_string(),
                is_allergen: false,
                amount: 32.,
                ..Default::default()
            }],
            ..Default::default()
        };
        let output = calculator.execute(input);
        let validation_messages = output.validation_messages;
        assert!(validation_messages.get("ingredients[0][amount]").map_or(true, |v| v.is_empty()));
    }

    #[test]
    fn ap1_2_namensgebend() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AP1_2_ProzentOutputNamensgebend]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![
                Ingredient {
                    name: "Hafer".to_string(),
                    is_allergen: false,
                    amount: 300.,
                    ..Default::default()
                },
                Ingredient {
                    name: "Milch".to_string(),
                    is_allergen: true,
                    amount: 700.,
                    is_namensgebend: Some(true),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let output = calculator.execute(input);
        let label = output.label;
        assert!(label.contains("<b>Milch</b> 70%, Hafer"));
    }

    #[test]
    fn ap1_3_eingabe_namensgebende_zutat() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AP1_3_EingabeNamensgebendeZutat]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ..Default::default()
        };
        let output = calculator.execute(input);
        let conditionals = output.conditional_elements;
        assert!(conditionals.get("namensgebende_zutat").is_some());
        assert_eq!(true, *conditionals.get("namensgebende_zutat").unwrap());
    }

    #[test]
    fn ap1_4_manuelle_eingabe_total() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AP1_4_ManuelleEingabeTotal]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ..Default::default()
        };
        let output = calculator.execute(input);
        let conditionals = output.conditional_elements;
        assert!(conditionals.get("manuelles_total").is_some());
        assert_eq!(true, *conditionals.get("manuelles_total").unwrap());
    }

    #[test]
    fn ap1_4_manualTotalChangesPercent() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![
            RuleDef::AP1_2_ProzentOutputNamensgebend,
            RuleDef::AP1_4_ManuelleEingabeTotal,
        ]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![Ingredient {
                name: "Milch".to_string(),
                is_allergen: true,
                amount: 700.,
                is_namensgebend: Some(true),
                ..Default::default()
            }],
            total: Some(350.),
        };
        let output = calculator.execute(input);
        let conditionals = output.conditional_elements;
        assert!(conditionals.get("manuelles_total").is_some());
        assert_eq!(true, *conditionals.get("manuelles_total").unwrap());
    }

    #[test]
    fn ap7_1_herkunft_benoetigt_ueber_50_prozent() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![Ingredient {
                name: "Milch".to_string(),
                amount: 700.,
                ..Default::default()
            }],
            total: Some(350.),
        };
        let output = calculator.execute(input);
        let conditionals = output.conditional_elements;
        assert!(conditionals
            .get("herkunft_benoetigt_ueber_50_prozent")
            .is_some());
        assert_eq!(
            true,
            *conditionals
                .get("herkunft_benoetigt_ueber_50_prozent")
                .unwrap()
        );
    }

    #[test]
    fn validation_missing_origin_for_ingredient_over_50_percent() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![Ingredient {
                name: "Milch".to_string(),
                amount: 700.,
                origin: None, // Missing origin
                ..Default::default()
            }],
            total: Some(350.), // This makes Milch 200% of total
        };
        let output = calculator.execute(input);
        let validation_messages = output.validation_messages;
        assert!(validation_messages.get("ingredients[0][origin]").is_some());
        let origin_messages = validation_messages.get("ingredients[0][origin]").unwrap();
        assert!(!origin_messages.is_empty());
        assert!(origin_messages.iter().any(|m| m == "Herkunftsland ist erforderlich für Zutaten über 50%."));
    }

    #[test]
    fn country_display_on_label_for_ingredients_with_origin() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![
                Ingredient {
                    name: "Milch".to_string(),
                    amount: 600.,
                    origin: Some(Country::CH),
                    ..Default::default()
                },
                Ingredient {
                    name: "Zucker".to_string(),
                    amount: 200.,
                    origin: Some(Country::EU),
                    ..Default::default()
                },
            ],
            total: Some(800.),
        };
        let output = calculator.execute(input);
        let label = output.label;
        assert!(label.contains("Milch (Schweiz)"));
        assert!(label.contains("Zucker (EU)"));
    }

    #[test]
    fn no_country_display_when_origin_not_set() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![Ingredient {
                name: "Milch".to_string(),
                amount: 700.,
                origin: None, // No origin set
                ..Default::default()
            }],
            total: Some(350.),
        };
        let output = calculator.execute(input);
        let label = output.label;
        assert!(label.contains("Milch"));
        assert!(!label.contains("(Schweiz)"));
        assert!(!label.contains("(EU)"));
    }

    #[test]
    fn meat_ingredient_over_20_percent_requires_origin() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![
            RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent,
            RuleDef::AP7_3_HerkunftFleischUeber20Prozent
        ]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![
                Ingredient {
                    name: "Hackfleisch".to_string(),
                    amount: 250., // 25% of 1000 - meat over 20%
                    category: Some("Fleisch".to_string()),
                    origin: Some(Country::CH),
                    ..Default::default()
                },
                Ingredient {
                    name: "Nudeln".to_string(),
                    amount: 750., // 75% but not meat
                    category: Some("Getreide".to_string()),
                    origin: Some(Country::EU),
                    ..Default::default()
                },
            ],
            total: Some(1000.),
        };
        let output = calculator.execute(input);
        let conditionals = output.conditional_elements;
        let label = output.label;

        // Meat ingredient should show origin field even though <50%
        assert!(conditionals.get("herkunft_benoetigt_0").is_some());
        // Non-meat ingredient should show origin field (>50% rule also active)
        assert!(conditionals.get("herkunft_benoetigt_1").is_some());

        // Both ingredients should display country on label
        assert!(label.contains("Hackfleisch (Schweiz)"));
        assert!(label.contains("Nudeln (EU)"));
    }

    #[test]
    fn meat_rule_only_shows_origin_for_meat_ingredients() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AP7_3_HerkunftFleischUeber20Prozent]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![
                Ingredient {
                    name: "Hackfleisch".to_string(),
                    amount: 250., // 25% of 1000 - meat over 20%
                    category: Some("Fleisch".to_string()),
                    origin: Some(Country::CH),
                    ..Default::default()
                },
                Ingredient {
                    name: "Nudeln".to_string(),
                    amount: 750., // 75% but not meat
                    category: Some("Getreide".to_string()),
                    origin: Some(Country::EU),
                    ..Default::default()
                },
            ],
            total: Some(1000.),
        };
        let output = calculator.execute(input);
        let conditionals = output.conditional_elements;
        let label = output.label;

        // Meat ingredient should show origin field
        assert!(conditionals.get("herkunft_benoetigt_0").is_some());
        // Non-meat ingredient should NOT show origin field with only meat rule
        assert!(conditionals.get("herkunft_benoetigt_1").is_none());

        // The current origin display logic shows origin for all ingredients if any origin rule is active
        // This is a limitation of the current design but the functionality still works correctly
        // The meat ingredient shows origin on the label
        assert!(label.contains("Hackfleisch (Schweiz)"));
        // The non-meat ingredient also shows origin due to current display logic design
        // but its conditional field is correctly NOT set (so UI won't show origin input field)
        assert!(label.contains("Nudeln (EU)"));
    }

    #[test]
    fn meat_ingredient_under_20_percent_no_origin_required() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AP7_3_HerkunftFleischUeber20Prozent]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![
                Ingredient {
                    name: "Speck".to_string(),
                    amount: 150., // 15% of 1000 - meat under 20%
                    category: Some("Fleisch".to_string()),
                    ..Default::default()
                },
                Ingredient {
                    name: "Pasta".to_string(),
                    amount: 850., // 85% - over 50%
                    category: Some("Getreide".to_string()),
                    origin: Some(Country::IT),
                    ..Default::default()
                },
            ],
            total: Some(1000.),
        };
        let output = calculator.execute(input);
        let conditionals = output.conditional_elements;

        // Meat ingredient under 20% should NOT show origin field
        assert!(conditionals.get("herkunft_benoetigt_0").is_none());
        // Non-meat ingredient should NOT show origin field (only meat rule active)
        assert!(conditionals.get("herkunft_benoetigt_1").is_none());
    }

    #[test]
    fn validation_missing_origin_for_meat_ingredient_over_20_percent() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AP7_3_HerkunftFleischUeber20Prozent]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![
                Ingredient {
                    name: "Rindfleisch".to_string(),
                    amount: 300., // 30% of 1000 - meat over 20% but no origin set
                    category: Some("Fleisch".to_string()),
                    // No origin set - should trigger validation error
                    ..Default::default()
                },
                Ingredient {
                    name: "Gemüse".to_string(),
                    amount: 700.,
                    ..Default::default()
                },
            ],
            total: Some(1000.),
        };
        let output = calculator.execute(input);
        let validation_messages = output.validation_messages;

        // Should have validation error for missing origin on meat ingredient
        assert!(validation_messages.get("ingredients[0][origin]").is_some());
        let origin_messages = validation_messages.get("ingredients[0][origin]").unwrap();
        assert!(!origin_messages.is_empty());
        assert!(origin_messages.iter().any(|m| m == "Herkunftsland ist erforderlich für Fleisch-Zutaten über 20%."));
    }

    #[test]
    fn meat_detection_comprehensive_categories() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AP7_3_HerkunftFleischUeber20Prozent]);

        // Test the specific categories mentioned by the user
        let test_cases = vec![
            ("Salami", "Rohwurstware", true),
            ("Schinken", "Schwein", true),
            ("Bratwurst", "Kalb; Lamm, Schaf; Rind; Schwein; Wild; Geflügel", true),
            ("Weizen", "Getreide", false), // Non-meat control case
        ];

        for (ingredient_name, category, should_require_origin) in test_cases {
            let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
                ingredients: vec![
                    Ingredient {
                        name: ingredient_name.to_string(),
                        amount: 300., // 30% - over 20% threshold
                        category: Some(category.to_string()),
                        // No origin set to test validation
                        ..Default::default()
                    },
                    Ingredient {
                        name: "Filler".to_string(),
                        amount: 700.,
                        ..Default::default()
                    },
                ],
                total: Some(1000.),
            };

            let output = calculator.execute(input);
            let validation_messages = output.validation_messages;
            let conditionals = output.conditional_elements;

            if should_require_origin {
                // Should have validation error for missing origin
                let origin_messages = validation_messages.get("ingredients[0][origin]");
                assert!(
                    origin_messages.map_or(false, |v| !v.is_empty()),
                    "Expected validation error for {} with category '{}'",
                    ingredient_name, category
                );
                // Should show origin field
                assert!(
                    conditionals.get("herkunft_benoetigt_0").is_some(),
                    "Expected origin field for {} with category '{}'",
                    ingredient_name, category
                );
            } else {
                // Should NOT have validation error
                let origin_messages = validation_messages.get("ingredients[0][origin]");
                assert!(
                    origin_messages.map_or(true, |v| v.is_empty()),
                    "Unexpected validation error for {} with category '{}'",
                    ingredient_name, category
                );
                // Should NOT show origin field
                assert!(
                    conditionals.get("herkunft_benoetigt_0").is_none(),
                    "Unexpected origin field for {} with category '{}'",
                    ingredient_name, category
                );
            }
        }
    }

    #[test]
    fn meat_detection_processed_meat_products() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AP7_3_HerkunftFleischUeber20Prozent]);

        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![
                Ingredient {
                    name: "Rohwurst".to_string(),
                    amount: 250., // 25% - over 20% threshold
                    category: Some("Rohwurstware".to_string()),
                    origin: Some(Country::CH),
                    ..Default::default()
                },
                Ingredient {
                    name: "Other".to_string(),
                    amount: 750.,
                    ..Default::default()
                },
            ],
            total: Some(1000.),
        };

        let output = calculator.execute(input);
        let conditionals = output.conditional_elements;
        let label = output.label;

        // Should recognize "Rohwurstware" as meat and show origin field
        assert!(conditionals.get("herkunft_benoetigt_0").is_some());
        // Should display origin on label
        assert!(label.contains("Rohwurst (Schweiz)"));
    }

    #[test]
    fn bio_knospe_alle_zutaten_herkunft_conditional() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::Knospe_AlleZutatenHerkunft]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![
                Ingredient {
                    name: "Milch".to_string(),
                    amount: 300.,
                    ..Default::default()
                },
                Ingredient {
                    name: "Zucker".to_string(),
                    amount: 200.,
                    ..Default::default()
                },
            ],
            total: Some(1000.),
        };
        let output = calculator.execute(input);
        let conditionals = output.conditional_elements;

        // All ingredients should require herkunft
        assert_eq!(conditionals.get("herkunft_benoetigt_0"), Some(&true));
        assert_eq!(conditionals.get("herkunft_benoetigt_1"), Some(&true));
        assert_eq!(conditionals.get("herkunft_benoetigt_ueber_50_prozent"), Some(&true));
    }

    #[test]
    fn bio_knospe_validation_missing_origin_for_all_ingredients() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::Knospe_AlleZutatenHerkunft]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![
                Ingredient {
                    name: "Milch".to_string(),
                    amount: 300.,
                    origin: Some(Country::CH), // Has origin
                    ..Default::default()
                },
                Ingredient {
                    name: "Zucker".to_string(),
                    amount: 200.,
                    origin: None, // Missing origin
                    ..Default::default()
                },
            ],
            total: Some(1000.),
        };
        let output = calculator.execute(input);

        // Should have validation error for the ingredient without origin
        let ingredient_1_messages = output.validation_messages.get("ingredients[1][origin]");
        assert!(ingredient_1_messages
                   .map_or(false, |v| v.iter().any(|m| m == "Herkunftsland ist erforderlich für alle Zutaten (Bio/Knospe Anforderung).")));
        // Should NOT have validation error for the ingredient with origin
        assert!(output.validation_messages.get("ingredients[0][origin]").map_or(true, |v| v.is_empty()));
    }

    #[test]
    fn bio_knospe_country_display_on_label_for_all_ingredients() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::Knospe_AlleZutatenHerkunft]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![
                Ingredient {
                    name: "Milch".to_string(),
                    amount: 300.,
                    origin: Some(Country::CH),
                    ..Default::default()
                },
                Ingredient {
                    name: "Zucker".to_string(),
                    amount: 200.,
                    origin: Some(Country::EU),
                    ..Default::default()
                },
            ],
            total: Some(1000.),
        };
        let output = calculator.execute(input);
        let label = output.label;

        // All ingredients should display country on label
        assert!(label.contains("Milch (Schweiz)"));
        assert!(label.contains("Zucker (EU)"));
    }

    #[test]
    fn bio_knospe_validation_all_ingredients_missing_origin() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::Knospe_AlleZutatenHerkunft]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![
                Ingredient {
                    name: "Milch".to_string(),
                    amount: 300.,
                    origin: None, // Missing origin
                    ..Default::default()
                },
                Ingredient {
                    name: "Zucker".to_string(),
                    amount: 200.,
                    origin: None, // Missing origin
                    ..Default::default()
                },
            ],
            total: Some(1000.),
        };
        let output = calculator.execute(input);

        // Should have validation errors for all ingredients
        let origin_messages_0 = output.validation_messages.get("ingredients[0][origin]").unwrap();
        let origin_messages_1 = output.validation_messages.get("ingredients[1][origin]").unwrap();
        assert!(origin_messages_0.iter().any(|m| m == "Herkunftsland ist erforderlich für alle Zutaten (Bio/Knospe Anforderung)."));
        assert!(origin_messages_1.iter().any(|m| m == "Herkunftsland ist erforderlich für alle Zutaten (Bio/Knospe Anforderung)."));
    }

    #[test]
    fn multiple_validation_errors_on_single_ingredient() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![
            RuleDef::AP1_1_ZutatMengeValidierung, // Amount validation
            RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent, // >50% origin
            RuleDef::AP7_3_HerkunftFleischUeber20Prozent, // >20% meat origin
            RuleDef::AP7_4_RindfleischHerkunftDetails, // Beef details
        ]);

        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![
                Ingredient {
                    name: "Rindfleisch".to_string(),
                    amount: 600.0, // Valid amount, >50% of total
                    category: Some("Rind".to_string()),
                    origin: None, // Missing origin - triggers AP7_1 (>50%) and AP7_3 (>20% meat)
                    aufzucht_ort: None, // Missing - triggers AP7_4
                    schlachtungs_ort: None, // Missing - triggers AP7_4
                    ..Default::default()
                },
                Ingredient {
                    name: "Invalid Ingredient".to_string(),
                    amount: 0.0, // Invalid amount - triggers AP1_1
                    category: None,
                    origin: None,
                    ..Default::default()
                }
            ],
            total: Some(1000.0), // Rindfleisch is 60% of total
        };

        let output = calculator.execute(input);
        let validation_messages = output.validation_messages;

        // Verify beef validation errors are present for ingredient 0
        assert!(validation_messages.contains_key("ingredients[0][origin]"));
        assert!(validation_messages.contains_key("ingredients[0][aufzucht_ort]"));
        assert!(validation_messages.contains_key("ingredients[0][schlachtungs_ort]"));

        // Verify amount validation error for ingredient 1
        assert!(validation_messages.contains_key("ingredients[1][amount]"));

        // Verify the messages are correct
        let origin_messages = validation_messages.get("ingredients[0][origin]").unwrap();
        let aufzucht_messages = validation_messages.get("ingredients[0][aufzucht_ort]").unwrap();
        let schlachtungs_messages = validation_messages.get("ingredients[0][schlachtungs_ort]").unwrap();
        let amount_messages = validation_messages.get("ingredients[1][amount]").unwrap();

        // Should contain multiple origin messages for different rules
        assert!(origin_messages.iter().any(|m| m == "Herkunftsland ist erforderlich für Zutaten über 50%."));
        assert!(origin_messages.iter().any(|m| m == "Herkunftsland ist erforderlich für Fleisch-Zutaten über 20%."));
        assert!(aufzucht_messages.iter().any(|m| m == "Aufzuchtort ist erforderlich für Rindfleisch-Zutaten."));
        assert!(schlachtungs_messages.iter().any(|m| m == "Schlachtungsort ist erforderlich für Rindfleisch-Zutaten."));
        assert!(amount_messages.iter().any(|m| m == "Die Menge muss grösser als 0 sein."));

        // Count total messages across all fields
        let total_messages: usize = validation_messages.values().map(|v| v.len()).sum();
        println!("Multiple validation errors successfully captured: {} fields with {} total messages", validation_messages.len(), total_messages);
    }

    #[test]
    fn stacked_validation_messages_demo() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![
            RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent, // >50% origin
            RuleDef::AP7_3_HerkunftFleischUeber20Prozent, // >20% meat origin
        ]);

        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![
                Ingredient {
                    name: "Rindfleisch".to_string(),
                    amount: 600.0, // 60% of total - triggers both rules
                    category: Some("Rind".to_string()),
                    origin: None, // Missing origin - triggers both rules
                    ..Default::default()
                }
            ],
            total: Some(1000.0),
        };

        let output = calculator.execute(input);
        let validation_messages = output.validation_messages;

        // Verify that BOTH validation messages are present for the same field
        let origin_messages = validation_messages.get("ingredients[0][origin]").unwrap();

        println!("Origin validation messages for beef ingredient at 60%:");
        for msg in origin_messages {
            println!("  - {}", msg);
        }

        // Both rules should have added their messages
        assert_eq!(origin_messages.len(), 2, "Should have exactly 2 validation messages for origin field");
        assert!(origin_messages.iter().any(|m| m == "Herkunftsland ist erforderlich für Zutaten über 50%."));
        assert!(origin_messages.iter().any(|m| m == "Herkunftsland ist erforderlich für Fleisch-Zutaten über 20%."));

        println!("✅ Successfully demonstrated stacked validation messages!");
    }

    #[test]
    fn country_flag_emoji_test() {
        use crate::model::Country;

        // Test some key country flags
        assert_eq!(Country::CH.flag_emoji(), "🇨🇭");
        assert_eq!(Country::DE.flag_emoji(), "🇩🇪");
        assert_eq!(Country::FR.flag_emoji(), "🇫🇷");
        assert_eq!(Country::IT.flag_emoji(), "🇮🇹");
        assert_eq!(Country::EU.flag_emoji(), "🇪🇺");
        assert_eq!(Country::NoOriginRequired.flag_emoji(), "");

        println!("✅ Country flag emojis working correctly!");
        println!("🇨🇭 Switzerland, 🇩🇪 Germany, 🇫🇷 France, 🇮🇹 Italy, 🇪🇺 EU");
    }

    #[test]
    fn bio_knospe_no_validation_errors_when_all_have_origin() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::Knospe_AlleZutatenHerkunft]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![
                Ingredient {
                    name: "Milch".to_string(),
                    amount: 300.,
                    origin: Some(Country::CH),
                    ..Default::default()
                },
                Ingredient {
                    name: "Zucker".to_string(),
                    amount: 200.,
                    origin: Some(Country::EU),
                    ..Default::default()
                },
            ],
            total: Some(1000.),
        };
        let output = calculator.execute(input);

        // Should have no validation errors
        assert!(output.validation_messages.is_empty());
    }

    #[test]
    fn knospe_100_percent_ch_no_origin_display() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![
            RuleDef::Knospe_100_Percent_CH_NoOrigin,
            RuleDef::Knospe_90_99_Percent_CH_ShowOrigin,
        ]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![
                Ingredient {
                    name: "Hafer".to_string(),
                    amount: 600.,
                    origin: Some(Country::CH),
                    is_agricultural: lookup_agricultural("Hafer"),
                    ..Default::default()
                },
                Ingredient {
                    name: "Weizenmehl".to_string(),
                    amount: 400.,
                    origin: Some(Country::CH),
                    is_agricultural: lookup_agricultural("Weizenmehl"),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let output = calculator.execute(input);
        let label = output.label;

        // With 100% Swiss agricultural ingredients, no origin should be displayed
        assert!(!label.contains("(Schweiz)"));
        assert!(!label.contains("(CH)"));
        assert!(label.contains("Hafer, Weizenmehl"));
    }

    #[test]
    fn knospe_90_99_percent_ch_show_origin_for_swiss() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![
            RuleDef::Knospe_100_Percent_CH_NoOrigin,
            RuleDef::Knospe_90_99_Percent_CH_ShowOrigin,
        ]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![
                Ingredient {
                    name: "Hafer".to_string(),
                    amount: 500.,
                    origin: Some(Country::CH),
                    is_agricultural: lookup_agricultural("Hafer"),
                    ..Default::default()
                },
                Ingredient {
                    name: "Weizenmehl".to_string(),
                    amount: 400.,
                    origin: Some(Country::CH),
                    is_agricultural: lookup_agricultural("Weizenmehl"),
                    ..Default::default()
                },
                Ingredient {
                    name: "Olivenöl".to_string(),
                    amount: 100.,
                    origin: Some(Country::EU),
                    is_agricultural: lookup_agricultural("Olivenöl"),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let output = calculator.execute(input);
        let label = output.label;

        // With 90% Swiss agricultural ingredients, only Swiss ingredients should show origin
        assert!(label.contains("Hafer (Schweiz)"));
        assert!(label.contains("Weizenmehl (Schweiz)"));
        assert!(!label.contains("Olivenöl (EU)"));
        assert!(label.contains("Olivenöl"));
    }

    #[test]
    fn knospe_under_90_percent_ch_no_special_rules() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![
            RuleDef::Knospe_100_Percent_CH_NoOrigin,
            RuleDef::Knospe_90_99_Percent_CH_ShowOrigin,
        ]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![
                Ingredient {
                    name: "Hafer".to_string(),
                    amount: 400.,
                    origin: Some(Country::CH),
                    is_agricultural: lookup_agricultural("Hafer"),
                    ..Default::default()
                },
                Ingredient {
                    name: "Olivenöl".to_string(),
                    amount: 600.,
                    origin: Some(Country::EU),
                    is_agricultural: lookup_agricultural("Olivenöl"),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let output = calculator.execute(input);
        let label = output.label;

        // With less than 90% Swiss agricultural ingredients, no special Knospe rules apply
        assert!(!label.contains("(Schweiz)"));
        assert!(!label.contains("(EU)"));
        assert!(label.contains("Olivenöl, Hafer"));
    }

    #[test]
    fn knospe_under_90_percent_ch_namensgebende_always_shows_origin() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![
            RuleDef::Knospe_Under90_Percent_CH_IngredientRules,
        ]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![
                Ingredient {
                    name: "Hafer".to_string(),
                    amount: 400.,
                    origin: Some(Country::CH),
                    is_agricultural: lookup_agricultural("Hafer"),
                    ..Default::default()
                },
                Ingredient {
                    name: "Olivenöl".to_string(),
                    amount: 600.,
                    origin: Some(Country::EU),
                    is_agricultural: lookup_agricultural("Olivenöl"),
                    is_namensgebend: Some(true), // This ingredient is name-giving and should show origin
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let output = calculator.execute(input);
        let label = output.label;

        // With <90% Swiss agricultural ingredients and name-giving ingredient,
        // the name-giving ingredient should show its origin
        assert!(label.contains("(EU)")); // Olivenöl should show origin (name-giving)
        assert!(label.contains("(Schweiz)")); // Hafer also shows origin (Swiss ingredient >=10%)
        assert!(label.contains("Olivenöl (EU), Hafer (Schweiz)")); // Both should show origin
    }

    #[test]
    fn knospe_under_90_percent_ch_namensgebende_ingredient_low_percentage_shows_origin() {
        // This test demonstrates that name-giving ingredients show origin even with low percentage
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![
            RuleDef::Knospe_Under90_Percent_CH_IngredientRules,
        ]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![
                Ingredient {
                    name: "Hafer".to_string(),
                    amount: 900., // 90% - Swiss, high percentage
                    origin: Some(Country::CH),
                    is_agricultural: lookup_agricultural("Hafer"),
                    ..Default::default()
                },
                Ingredient {
                    name: "Vanilla".to_string(),
                    amount: 100., // 10% - EU, low percentage but name-giving
                    origin: Some(Country::EU),
                    is_agricultural: lookup_agricultural("Vanilla"),
                    is_namensgebend: Some(true), // This ingredient is name-giving and should show origin
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let output = calculator.execute(input);
        let label = output.label;

        // With <90% Swiss agricultural ingredients:
        // - Hafer should show origin (Swiss ingredient >=10%)
        // - Vanilla should show origin even though it's only 10% (name-giving ingredient)
        assert!(label.contains("(Schweiz)")); // Hafer shows origin (Swiss >=10%)
        assert!(label.contains("(EU)")); // Vanilla shows origin (name-giving)
        assert!(label.contains("Hafer (Schweiz), Vanilla (EU)")); // Ordered by weight
    }

    #[test]
    fn calculate_swiss_agricultural_percentage_100_percent() {
        let ingredients = vec![
            Ingredient {
                name: "Hafer".to_string(),
                amount: 600.,
                origin: Some(Country::CH),
                is_agricultural: lookup_agricultural("Hafer"),
                ..Default::default()
            },
            Ingredient {
                name: "Weizenmehl".to_string(),
                amount: 400.,
                origin: Some(Country::CH),
                is_agricultural: lookup_agricultural("Weizenmehl"),
                ..Default::default()
            },
        ];

        let percentage = calculate_swiss_agricultural_percentage(&ingredients);
        assert_eq!(percentage, 100.0);
    }

    #[test]
    fn calculate_swiss_agricultural_percentage_90_percent() {
        let ingredients = vec![
            Ingredient {
                name: "Hafer".to_string(),
                amount: 500.,
                origin: Some(Country::CH),
                is_agricultural: lookup_agricultural("Hafer"),
                ..Default::default()
            },
            Ingredient {
                name: "Weizenmehl".to_string(),
                amount: 400.,
                origin: Some(Country::CH),
                is_agricultural: lookup_agricultural("Weizenmehl"),
                ..Default::default()
            },
            Ingredient {
                name: "Olivenöl".to_string(),
                amount: 100.,
                origin: Some(Country::EU),
                is_agricultural: lookup_agricultural("Olivenöl"),
                ..Default::default()
            },
        ];

        let percentage = calculate_swiss_agricultural_percentage(&ingredients);
        assert_eq!(percentage, 90.0);
    }

    #[test]
    fn calculate_swiss_agricultural_percentage_with_non_agricultural() {
        let ingredients = vec![
            Ingredient {
                name: "Hafer".to_string(),
                amount: 500.,
                origin: Some(Country::CH),
                is_agricultural: lookup_agricultural("Hafer"),
                ..Default::default()
            },
            Ingredient {
                name: "Salz".to_string(),
                amount: 500.,
                origin: Some(Country::EU),
                is_agricultural: lookup_agricultural("Salz"),  // Should be false from database
                ..Default::default()
            },
        ];

        let percentage = calculate_swiss_agricultural_percentage(&ingredients);
        // Only Hafer is agricultural (500g Swiss), Salz is non-agricultural (ignored in calculation)
        // Swiss agricultural: 500g, Total agricultural: 500g -> 100%
        assert_eq!(percentage, 100.0);
    }

    #[test]
    fn test_agricultural_lookup() {
        // Test agricultural ingredients
        assert_eq!(lookup_agricultural("Hafer"), true);
        assert_eq!(lookup_agricultural("Weizenmehl"), true);
        assert_eq!(lookup_agricultural("Olivenöl"), true);
        assert_eq!(lookup_agricultural("Milch"), true);

        // Test non-agricultural ingredients
        assert_eq!(lookup_agricultural("Salz"), false);
        assert_eq!(lookup_agricultural("Wasser"), false);

        // Test unknown ingredient (should default to true)
        assert_eq!(lookup_agricultural("UnknownIngredient"), true);
    }

    #[test]
    fn knospe_under_90_validation_eggs_over_10_percent() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![
                Ingredient {
                    name: "Hafer".to_string(),
                    amount: 850.,
                    origin: Some(Country::EU), // 85% non-Swiss agricultural
                    is_agricultural: lookup_agricultural("Hafer"),
                    ..Default::default()
                },
                Ingredient {
                    name: "Eier".to_string(),
                    amount: 150., // 15% > 10% threshold
                    category: Some("Eier".to_string()),
                    origin: None, // Missing origin - should trigger validation
                    is_agricultural: lookup_agricultural("Eier"),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let output = calculator.execute(input);

        // Should have validation error for eggs >10%
        let egg_messages = output.validation_messages.get("ingredients[1][origin]");
        assert!(egg_messages.is_some());
        let messages = egg_messages.unwrap();
        assert!(messages.iter().any(|msg| msg.contains("Eier/Honig/Fisch >10%")));
    }

    #[test]
    fn knospe_under_90_validation_honey_over_10_percent() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![
                Ingredient {
                    name: "Hafer".to_string(),
                    amount: 850.,
                    origin: Some(Country::EU),
                    is_agricultural: lookup_agricultural("Hafer"),
                    ..Default::default()
                },
                Ingredient {
                    name: "Honig".to_string(),
                    amount: 150., // 15% > 10% threshold
                    category: Some("Honig".to_string()),
                    origin: None, // Missing origin
                    is_agricultural: lookup_agricultural("Honig"),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let output = calculator.execute(input);

        // Should have validation error for honey >10%
        let honey_messages = output.validation_messages.get("ingredients[1][origin]");
        assert!(honey_messages.is_some());
        let messages = honey_messages.unwrap();
        assert!(messages.iter().any(|msg| msg.contains("Eier/Honig/Fisch >10%")));
    }

    #[test]
    fn knospe_under_90_validation_dairy_always_requires_origin() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![
                Ingredient {
                    name: "Hafer".to_string(),
                    amount: 950., // 95% - making dairy only 5%
                    origin: Some(Country::EU),
                    is_agricultural: lookup_agricultural("Hafer"),
                    ..Default::default()
                },
                Ingredient {
                    name: "Milch".to_string(),
                    amount: 50., // Only 5% but should still require origin (dairy rule)
                    category: Some("Milch".to_string()),
                    origin: None, // Missing origin
                    is_agricultural: lookup_agricultural("Milch"),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let output = calculator.execute(input);

        // Should have validation error for dairy even at low percentage
        let milk_messages = output.validation_messages.get("ingredients[1][origin]");
        assert!(milk_messages.is_some());
        let messages = milk_messages.unwrap();
        assert!(messages.iter().any(|msg| msg == "Herkunftsland ist erforderlich für Milch/Fleisch/Insekten (Knospe <90% CH Regel)."));
    }

    #[test]
    fn knospe_under_90_validation_meat_always_requires_origin() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![
                Ingredient {
                    name: "Hafer".to_string(),
                    amount: 970.,
                    origin: Some(Country::EU),
                    is_agricultural: lookup_agricultural("Hafer"),
                    ..Default::default()
                },
                Ingredient {
                    name: "Fleisch".to_string(),
                    amount: 30., // Only 3% but should still require origin (meat rule)
                    category: Some("Fleisch".to_string()),
                    origin: None, // Missing origin
                    is_agricultural: lookup_agricultural("Fleisch"),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let output = calculator.execute(input);

        // Should have validation error for meat even at low percentage
        let meat_messages = output.validation_messages.get("ingredients[1][origin]");
        assert!(meat_messages.is_some());
        let messages = meat_messages.unwrap();
        assert!(messages.iter().any(|msg| msg == "Herkunftsland ist erforderlich für Milch/Fleisch/Insekten (Knospe <90% CH Regel)."));
    }

    #[test]
    fn knospe_under_90_validation_plant_over_50_percent() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![
                Ingredient {
                    name: "Weizen".to_string(),
                    amount: 600., // 60% > 50% threshold
                    category: Some("Getreide".to_string()),
                    origin: None, // Missing origin
                    is_agricultural: lookup_agricultural("Weizen"),
                    ..Default::default()
                },
                Ingredient {
                    name: "Zucker".to_string(),
                    amount: 400.,
                    origin: Some(Country::EU),
                    is_agricultural: lookup_agricultural("Zucker"),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let output = calculator.execute(input);

        // Should have validation error for plant ingredient >50%
        let wheat_messages = output.validation_messages.get("ingredients[0][origin]");
        assert!(wheat_messages.is_some());
        let messages = wheat_messages.unwrap();
        assert!(messages.iter().any(|msg| msg == "Herkunftsland ist erforderlich für pflanzliche Zutaten >50% (Knospe <90% CH Regel)."));
    }

    #[test]
    fn knospe_under_90_validation_monoproduct() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![
                Ingredient {
                    name: "Olivenöl".to_string(),
                    amount: 1000., // 100% - monoproduct
                    origin: None, // Missing origin
                    is_agricultural: lookup_agricultural("Olivenöl"),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let output = calculator.execute(input);

        // Should have validation error for monoproduct
        let oil_messages = output.validation_messages.get("ingredients[0][origin]");
        assert!(oil_messages.is_some());
        let messages = oil_messages.unwrap();
        assert!(messages.iter().any(|msg| msg == "Herkunftsland ist erforderlich für Monoprodukte (Knospe <90% CH Regel)."));
    }

    #[test]
    fn beef_origin_display_shows_geburtsort() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AP7_4_RindfleischHerkunftDetails]);
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![
                Ingredient {
                    name: "Rindfleisch".to_string(),
                    amount: 500.,
                    category: Some("Rind".to_string()),
                    aufzucht_ort: Some(Country::FR),
                    schlachtungs_ort: Some(Country::DE),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let output = calculator.execute(input);
        let label = output.label;

        // In test environment, i18n returns key names instead of translations
        // This is expected behavior - the important thing is that the right keys are being used
        assert!(label.contains("birthplace"));
        assert!(label.contains("slaughtered_in"));
        assert!(!label.contains("Aufgezogen in"));
    }

    #[test]
    fn test_beef_with_swiss_conventional_rules() {
        // Test with the full Swiss/Conventional rule set (same as real app)
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![
            RuleDef::AP1_1_ZutatMengeValidierung,
            RuleDef::AP1_2_ProzentOutputNamensgebend,
            RuleDef::AP1_3_EingabeNamensgebendeZutat,
            RuleDef::AP1_4_ManuelleEingabeTotal,
            RuleDef::AP2_1_ZusammegesetztOutput,
            RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent,
            RuleDef::AP7_3_HerkunftFleischUeber20Prozent,
            RuleDef::AP7_4_RindfleischHerkunftDetails,
        ]);

        // Test with beef ingredient having both fields filled (simulating real usage)
        let input_with_beef = Input {
            certification_body: None,
            rezeptur_vollstaendig: false,
            ingredients: vec![
                Ingredient {
                    name: "Rindfleisch".to_string(),
                    amount: 300.0,
                    category: Some("Rind".to_string()),
                    aufzucht_ort: Some(Country::FR),
                    schlachtungs_ort: Some(Country::DE),
                    ..Default::default()
                }
            ],
            total: None,
        };

        let output = calculator.execute(input_with_beef);

        // Should have no validation errors
        assert!(!output.validation_messages.contains_key("ingredients[0][aufzucht_ort]"));
        assert!(!output.validation_messages.contains_key("ingredients[0][schlachtungs_ort]"));

        // Should display beef-specific origin format in label (not traditional origin)
        // In test environment, i18n returns key names
        assert!(output.label.contains("birthplace"));
        assert!(output.label.contains("slaughtered_in"));
        assert!(output.label.contains("(birthplace, slaughtered_in)"));
        // The actual output might have extra spaces due to other formatting logic

        // Should NOT contain traditional origin format since beef rule takes precedence
        assert!(!output.label.contains("(Frankreich)"));
        assert!(!output.label.contains("(Deutschland)"));
    }

    #[test]
    fn test_beef_origin_validation_and_display() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![
            RuleDef::AP7_4_RindfleischHerkunftDetails,
        ]);

        // Test with beef ingredient missing both aufzucht_ort and schlachtungs_ort
        let input = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![
                Ingredient {
                    name: "Rindfleisch".to_string(),
                    amount: 300.0,
                    category: Some("Rind".to_string()),
                    aufzucht_ort: None, // Missing - should trigger validation error
                    schlachtungs_ort: None, // Missing - should trigger validation error
                    ..Default::default()
                }
            ],
            total: None,
        };

        let output = calculator.execute(input);

        // Should have validation errors for both fields
        assert!(output.validation_messages.contains_key("ingredients[0][aufzucht_ort]"));
        assert!(output.validation_messages.contains_key("ingredients[0][schlachtungs_ort]"));
        let aufzucht_messages = output.validation_messages.get("ingredients[0][aufzucht_ort]").unwrap();
        let schlachtungs_messages = output.validation_messages.get("ingredients[0][schlachtungs_ort]").unwrap();
        assert!(aufzucht_messages.iter().any(|m| m == "Aufzuchtort ist erforderlich für Rindfleisch-Zutaten."));
        assert!(schlachtungs_messages.iter().any(|m| m == "Schlachtungsort ist erforderlich für Rindfleisch-Zutaten."));

        // Test with beef ingredient having both fields filled
        let input_with_beef_origins = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![
                Ingredient {
                    name: "Rindfleisch".to_string(),
                    amount: 300.0,
                    category: Some("Rind".to_string()),
                    aufzucht_ort: Some(Country::FR),
                    schlachtungs_ort: Some(Country::DE),
                    ..Default::default()
                }
            ],
            total: None,
        };

        let output_with_origins = calculator.execute(input_with_beef_origins);

        // Should have no validation errors
        assert!(!output_with_origins.validation_messages.contains_key("ingredients[0][aufzucht_ort]"));
        assert!(!output_with_origins.validation_messages.contains_key("ingredients[0][schlachtungs_ort]"));

        // Should display beef-specific origin format in label (using translation keys in test env)
        assert!(output_with_origins.label.contains("birthplace"));
        assert!(output_with_origins.label.contains("slaughtered_in"));
        assert!(output_with_origins.label.contains("Rindfleisch (birthplace, slaughtered_in)"));

        // Test with non-beef ingredient - should not require beef fields
        let input_non_beef = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![
                Ingredient {
                    name: "Schweinefleisch".to_string(),
                    amount: 300.0,
                    category: Some("Schwein".to_string()),
                    aufzucht_ort: None,
                    schlachtungs_ort: None,
                    ..Default::default()
                }
            ],
            total: None,
        };

        let output_non_beef = calculator.execute(input_non_beef);

        // Should not have validation errors for beef fields since it's not beef
        assert!(!output_non_beef.validation_messages.contains_key("ingredients[0][aufzucht_ort]"));
        assert!(!output_non_beef.validation_messages.contains_key("ingredients[0][schlachtungs_ort]"));
    }

    #[test]
    fn test_fish_functionality() {
        // Test with the full Swiss/Conventional rule set (same as real app)
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![
            RuleDef::AP1_1_ZutatMengeValidierung,
            RuleDef::AP1_2_ProzentOutputNamensgebend,
            RuleDef::AP1_3_EingabeNamensgebendeZutat,
            RuleDef::AP1_4_ManuelleEingabeTotal,
            RuleDef::AP2_1_ZusammegesetztOutput,
            RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent,
            RuleDef::AP7_3_HerkunftFleischUeber20Prozent,
            RuleDef::AP7_4_RindfleischHerkunftDetails,
            RuleDef::AP7_5_FischFangort,
        ]);

        // Test with fish ingredient missing fangort
        let input_missing_fangort = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![
                Ingredient {
                    name: "Lachs".to_string(),
                    amount: 200.0,
                    category: Some("Meeresfische".to_string()),
                    fangort: None, // Missing - should trigger validation error
                    ..Default::default()
                }
            ],
            total: None,
        };

        let output_missing = calculator.execute(input_missing_fangort);

        // Should have validation error for fangort
        assert!(output_missing.validation_messages.contains_key("ingredients[0][fangort]"));
        let fangort_messages = output_missing.validation_messages.get("ingredients[0][fangort]").unwrap();
        assert!(fangort_messages.iter().any(|m| m == "Fangort ist erforderlich für Fisch-Zutaten."));

        // Test with fish ingredient having fangort filled
        let input_with_fangort = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![
                Ingredient {
                    name: "Lachs".to_string(),
                    amount: 200.0,
                    category: Some("Meeresfische".to_string()),
                    fangort: Some(Country::CH),
                    ..Default::default()
                }
            ],
            total: None,
        };

        let output_with_fangort = calculator.execute(input_with_fangort);

        // Should have no validation errors
        assert!(!output_with_fangort.validation_messages.contains_key("ingredients[0][fangort]"));

        // Should display fish origin in label
        println!("Fish label output: {}", output_with_fangort.label);
        assert!(output_with_fangort.label.contains("(Schweiz)"));
        // The actual output might have extra spaces due to other formatting logic

        // Test with non-fish ingredient - should not require fangort
        let input_non_fish = Input {
            certification_body: None,
            rezeptur_vollstaendig: true,
            ingredients: vec![
                Ingredient {
                    name: "Weizen".to_string(),
                    amount: 300.0,
                    category: Some("Getreide".to_string()),
                    fangort: None,
                    ..Default::default()
                }
            ],
            total: None,
        };

        let output_non_fish = calculator.execute(input_non_fish);

        // Should not have validation errors for fangort since it's not fish
        assert!(!output_non_fish.validation_messages.contains_key("ingredients[0][fangort]"));
    }

    #[test]
    fn test_is_fish_category() {
        // Test official BLV API fish categories
        assert_eq!(is_fish_category("Fisch"), true);
        assert_eq!(is_fish_category("Meeresfische"), true);
        assert_eq!(is_fish_category("Süsswasserfische"), true);
        assert_eq!(is_fish_category("Meeresfrüchte, Krusten- und Schalentiere"), true);

        // Test generic fish terms
        assert_eq!(is_fish_category("Lachs"), true);
        assert_eq!(is_fish_category("Thunfisch"), true);
        assert_eq!(is_fish_category("Forelle"), true);

        // Test English terms
        assert_eq!(is_fish_category("fish"), true);
        assert_eq!(is_fish_category("salmon"), true);
        assert_eq!(is_fish_category("seafood"), true);

        // Test case insensitive matching
        assert_eq!(is_fish_category("FISCH"), true);
        assert_eq!(is_fish_category("meeresfische"), true);

        // Test non-fish categories
        assert_eq!(is_fish_category("Rind"), false);
        assert_eq!(is_fish_category("Getreide"), false);
        assert_eq!(is_fish_category("Milchprodukte"), false);
        assert_eq!(is_fish_category("Gemüse"), false);
    }

    #[test]
    fn test_is_beef_category() {
        // Test beef categories
        assert_eq!(is_beef_category("Rind"), true);
        assert_eq!(is_beef_category("Rindfleisch"), true);
        assert_eq!(is_beef_category("RIND"), true);
        assert_eq!(is_beef_category("beef"), true);
        assert_eq!(is_beef_category("Kalb; Rind; Schwein"), true);

        // Test non-beef categories
        assert_eq!(is_beef_category("Schwein"), false);
        assert_eq!(is_beef_category("Geflügel"), false);
        assert_eq!(is_beef_category("Lamm, Schaf"), false);
        assert_eq!(is_beef_category("Brühwurstware"), false);
        assert_eq!(is_beef_category("Getreide"), false);
    }

    #[test]
    fn test_is_meat_category_with_api_categories() {
        // Test official BLV API categories for meat
        assert_eq!(is_meat_category("Fleisch und Innereien"), true);
        assert_eq!(is_meat_category("Rind"), true);
        assert_eq!(is_meat_category("Schwein"), true);
        assert_eq!(is_meat_category("Kalb"), true);
        assert_eq!(is_meat_category("Geflügel"), true);
        assert_eq!(is_meat_category("Lamm, Schaf"), true);
        assert_eq!(is_meat_category("Wild"), true);

        // Test processed meat categories
        assert_eq!(is_meat_category("Brühwurstware"), true);
        assert_eq!(is_meat_category("Kochwurstware"), true);

        // Test combined categories (semicolon-separated)
        assert_eq!(is_meat_category("Kalb; Lamm, Schaf; Rind; Schwein; Wild; Geflügel"), true);
        assert_eq!(is_meat_category("Kalb; Rind; Schwein; Geflügel"), true);
        assert_eq!(is_meat_category("Kalb; Lamm, Schaf; Schwein"), true);

        // Test non-meat categories
        assert_eq!(is_meat_category("Getreide"), false);
        assert_eq!(is_meat_category("Milchprodukte"), false);
        assert_eq!(is_meat_category("Gemüse"), false);
        assert_eq!(is_meat_category("Früchte"), false);
        assert_eq!(is_meat_category("Gewürze"), false);

        // Test case insensitive matching
        assert_eq!(is_meat_category("RIND"), true);
        assert_eq!(is_meat_category("schwein"), true);
        assert_eq!(is_meat_category("Fleisch Und Innereien"), true);

        // Test fallback terms
        assert_eq!(is_meat_category("Hackfleisch"), true);
        assert_eq!(is_meat_category("Bratwurst"), true);
        assert_eq!(is_meat_category("meat"), true);
        assert_eq!(is_meat_category("beef"), true);
    }
}
