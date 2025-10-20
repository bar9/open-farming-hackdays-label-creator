use crate::model::{lookup_allergen, Country};
use crate::rules::RuleDef;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::mem;

#[derive(Clone, Default)]
pub struct Input {
    pub(crate) ingredients: Vec<Ingredient>,
    pub total: Option<f64>,
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
    pub validation_messages: HashMap<String, &'static str>,
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

impl Calculator {
    pub(crate) fn new() -> Self {
        Calculator { rule_defs: vec![] }
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
}

fn default_is_agricultural() -> bool {
    true
}

impl Ingredient {
    pub fn from_name_amount(name: String, amount: f64) -> Self {
        Self {
            name: name.clone(),
            is_allergen: lookup_allergen(&name),
            amount,
            ..Default::default()
        }
    }

    pub fn is_agricultural(&self) -> bool {
        true
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
                        .map(|sub| sub.name.clone())
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
        if self.RuleDefs.iter().any(|x| *x == RuleDef::AllPercentages) {
            output = format!(
                "{} {}%",
                output,
                (self.ingredient.amount / self.total_amount * 100.) as u8
            )
        }
        if self
            .RuleDefs
            .iter()
            .any(|x| *x == RuleDef::PercentagesStartsWithM)
            && self.ingredient.name.starts_with("M")
        {
            output = format!(
                "{} {}%",
                output,
                (self.ingredient.amount / self.total_amount * 100.) as u8
            )
        }
        // if self.RuleDefs.iter().find(|x| **x == RuleDef::MaxDetails).is_some() {
        //     output = format!{"{:?}", self.ingredient}
        // }
        if self.RuleDefs.iter().any(|x| *x == RuleDef::AllGram) {
            output = format! {"{} {}g", self.ingredient.name, self.ingredient.amount};
        }
        if self
            .RuleDefs
            .iter()
            .any(|x| *x == RuleDef::AP1_2_ProzentOutputNamensgebend)
        {
            if let Some(true) = self.ingredient.is_namensgebend {
                output = format!(
                    "{} {}%",
                    output,
                    (self.ingredient.amount / self.total_amount * 100.) as u8
                )
            }
        }
        if self
            .RuleDefs
            .iter()
            .any(|x| *x == RuleDef::AP2_1_ZusammegesetztOutput)
            && self.ingredient.sub_components.is_some()
        {
            output = format! {"{} {}", output, self.ingredient.composites()};
        }
        // Handle Knospe-specific rules first (they take precedence)
        let has_knospe_100_rule = self
            .RuleDefs
            .iter()
            .any(|x| *x == RuleDef::Knospe_100_Percent_CH_NoOrigin);
        let has_knospe_90_99_rule = self
            .RuleDefs
            .iter()
            .any(|x| *x == RuleDef::Knospe_90_99_Percent_CH_ShowOrigin);

        if has_knospe_100_rule {
            // Rule A: 100% Swiss agricultural ingredients - no origin display
            // Do nothing, origin already not displayed by default
        } else if has_knospe_90_99_rule {
            // Rule B: 90-99.99% Swiss agricultural ingredients - show origin for Swiss ingredients only
            if let Some(Country::CH) = &self.ingredient.origin {
                output = format!("{} (Schweiz)", output);
            }
        } else {
            // Add country of origin display for traditional herkunft rules (only if no Knospe rules apply)
            if self
                .RuleDefs
                .iter()
                .any(|x| *x == RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent || *x == RuleDef::AP7_2_HerkunftNamensgebendeZutat || *x == RuleDef::Bio_Knospe_AlleZutatenHerkunft)
            {
                if let Some(origin) = &self.ingredient.origin {
                    // Don't show origin for "NoOriginRequired"
                    if !matches!(origin, Country::NoOriginRequired) {
                        let country_name = origin.display_name();
                        output = format!("{} ({})", output, country_name);
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

    pub fn validate_rule_dependencies(&self) -> Result<(), String> {
        use crate::rules::RuleRegistry;
        let registry = RuleRegistry::new();
        registry.validate_dependencies(&self.rule_defs)
    }
    pub fn execute(&self, input: Input) -> Output {
        let mut validation_messages = HashMap::new();
        let mut conditionals = HashMap::new();

        // Calculate total amount first (needed for validations)
        let mut total_amount = input.ingredients.iter().map(|x| x.amount).sum();
        if self
            .rule_defs
            .iter()
            .any(|x| *x == RuleDef::AP1_4_ManuelleEingabeTotal)
        {
            if let Some(tot) = input.total {
                total_amount = tot;
            }
        }

        // validations
        for ruleDef in &self.rule_defs {
            if let RuleDef::AP1_1_ZutatMengeValidierung = ruleDef {
                validate_amount(&input.ingredients, &mut validation_messages)
            }
            if let RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent = ruleDef {
                validate_origin(&input.ingredients, total_amount, &mut validation_messages)
            }
            if let RuleDef::AP7_2_HerkunftNamensgebendeZutat = ruleDef {
                validate_namensgebende_origin(&input.ingredients, &mut validation_messages)
            }
            if let RuleDef::Bio_Knospe_AlleZutatenHerkunft = ruleDef {
                validate_all_ingredients_origin(&input.ingredients, &mut validation_messages)
            }
        }

        // conditionals
        for ruleDef in &self.rule_defs {
            if let RuleDef::AP1_3_EingabeNamensgebendeZutat = ruleDef {
                conditionals.insert(String::from("namensgebende_zutat"), true);
            }
        }

        let mut sorted_ingredients = input.ingredients.clone();
        sorted_ingredients.sort_by(|y, x| x.amount.partial_cmp(&y.amount).unwrap());

        if self
            .rule_defs
            .iter()
            .any(|x| *x == RuleDef::AP1_4_ManuelleEingabeTotal)
        {
            conditionals.insert(String::from("manuelles_total"), true);
        }

        // Determine which ingredients require country of origin display
        let has_50_percent_rule = self
            .rule_defs
            .iter()
            .any(|x| *x == RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent);
        let has_namensgebende_rule = self
            .rule_defs
            .iter()
            .any(|x| *x == RuleDef::AP7_2_HerkunftNamensgebendeZutat);
        let has_bio_knospe_rule = self
            .rule_defs
            .iter()
            .any(|x| *x == RuleDef::Bio_Knospe_AlleZutatenHerkunft);

        // Handle Knospe-specific percentage-based rules
        let has_knospe_100_rule = self
            .rule_defs
            .iter()
            .any(|x| *x == RuleDef::Knospe_100_Percent_CH_NoOrigin);
        let has_knospe_90_99_rule = self
            .rule_defs
            .iter()
            .any(|x| *x == RuleDef::Knospe_90_99_Percent_CH_ShowOrigin);

        // Calculate percentage of Swiss agricultural ingredients for Knospe rules
        let mut actual_knospe_rule: Option<RuleDef> = None;
        if has_knospe_100_rule || has_knospe_90_99_rule {
            let swiss_percentage = calculate_swiss_agricultural_percentage(&input.ingredients);
            if swiss_percentage >= 100.0 {
                actual_knospe_rule = Some(RuleDef::Knospe_100_Percent_CH_NoOrigin);
            } else if swiss_percentage >= 90.0 {
                actual_knospe_rule = Some(RuleDef::Knospe_90_99_Percent_CH_ShowOrigin);
            }
        }

        if has_50_percent_rule || has_namensgebende_rule || has_bio_knospe_rule {
            let mut has_any_herkunft_required = false;
            for (index, ingredient) in input.ingredients.iter().enumerate() {
                let mut requires_herkunft = false;

                // Check if >50% rule applies
                if has_50_percent_rule {
                    let percentage = (ingredient.amount / total_amount) * 100.0;
                    if percentage > 50.0 {
                        requires_herkunft = true;
                    }
                }

                // Check if namensgebende rule applies
                if has_namensgebende_rule && ingredient.is_namensgebend == Some(true) {
                    requires_herkunft = true;
                }

                // Check if Bio/Knospe rule applies (requires origin for ALL ingredients)
                if has_bio_knospe_rule {
                    requires_herkunft = true;
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

        // Prepare rule_defs for OutputFormatter, including the specific Knospe rule
        let mut output_rules = self.rule_defs.clone();
        if let Some(knospe_rule) = actual_knospe_rule {
            // Remove the generic Knospe rules and add the specific one
            output_rules.retain(|rule| !matches!(rule, RuleDef::Knospe_100_Percent_CH_NoOrigin | RuleDef::Knospe_90_99_Percent_CH_ShowOrigin));
            output_rules.push(knospe_rule);
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

fn validate_amount(ingredients: &Vec<Ingredient>, validation_messages: &mut HashMap<String, &str>) {
    for (i, ingredient) in ingredients.iter().enumerate() {
        if ingredient.amount <= 0. {
            validation_messages.insert(
                format!("ingredients[{}][amount]", i),
                "Die Menge muss grösser als 0 sein.",
            );
        }
    }
}

fn validate_origin(
    ingredients: &Vec<Ingredient>,
    total_amount: f64,
    validation_messages: &mut HashMap<String, &str>,
) {
    for (i, ingredient) in ingredients.iter().enumerate() {
        let percentage = (ingredient.amount / total_amount) * 100.0;
        if percentage > 50.0 && ingredient.origin.is_none() {
            validation_messages.insert(
                format!("ingredients[{}][origin]", i),
                "Herkunftsland ist erforderlich für Zutaten über 50%.",
            );
        }
    }
}

fn validate_namensgebende_origin(
    ingredients: &Vec<Ingredient>,
    validation_messages: &mut HashMap<String, &str>,
) {
    for (i, ingredient) in ingredients.iter().enumerate() {
        if ingredient.is_namensgebend == Some(true) && ingredient.origin.is_none() {
            validation_messages.insert(
                format!("ingredients[{}][origin]", i),
                "Herkunftsland ist erforderlich für namensgebende Zutaten.",
            );
        }
    }
}

fn validate_all_ingredients_origin(
    ingredients: &Vec<Ingredient>,
    validation_messages: &mut HashMap<String, &str>,
) {
    for (i, ingredient) in ingredients.iter().enumerate() {
        if ingredient.origin.is_none() {
            validation_messages.insert(
                format!("ingredients[{}][origin]", i),
                "Herkunftsland ist erforderlich für alle Zutaten (Bio/Knospe Anforderung).",
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_simple_calculator() -> Calculator {
        let rule_defs = vec![];
        Calculator { rule_defs }
    }

    #[test]
    fn simple_run_of_process() {
        let calculator = setup_simple_calculator();
        let input = Input {
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
        assert_eq!(
            "Die Menge muss grösser als 0 sein.",
            *validation_messages.get("ingredients[0][amount]").unwrap()
        )
    }

    #[test]
    fn amount_gt_zero_valid() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AP1_1_ZutatMengeValidierung]);
        let input = Input {
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
        assert!(validation_messages.get("ingredients[0][amount]").is_none());
    }

    #[test]
    fn ap1_2_namensgebend() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AP1_2_ProzentOutputNamensgebend]);
        let input = Input {
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
        assert_eq!(
            "Herkunftsland ist erforderlich für Zutaten über 50%.",
            *validation_messages.get("ingredients[0][origin]").unwrap()
        );
    }

    #[test]
    fn country_display_on_label_for_ingredients_with_origin() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent]);
        let input = Input {
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
    fn ap7_2_herkunft_namensgebende_zutat_conditional() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AP7_2_HerkunftNamensgebendeZutat]);
        let input = Input {
            ingredients: vec![Ingredient {
                name: "Milch".to_string(),
                amount: 300.,
                is_namensgebend: Some(true),
                ..Default::default()
            }],
            total: Some(1000.),
        };
        let output = calculator.execute(input);
        let conditionals = output.conditional_elements;
        assert!(conditionals.get("herkunft_benoetigt_0").is_some());
        assert_eq!(true, *conditionals.get("herkunft_benoetigt_0").unwrap());
    }

    #[test]
    fn validation_missing_origin_for_namensgebende_ingredient() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AP7_2_HerkunftNamensgebendeZutat]);
        let input = Input {
            ingredients: vec![Ingredient {
                name: "Milch".to_string(),
                amount: 300.,
                is_namensgebend: Some(true),
                origin: None, // Missing origin
                ..Default::default()
            }],
            total: Some(1000.),
        };
        let output = calculator.execute(input);
        let validation_messages = output.validation_messages;
        assert!(validation_messages.get("ingredients[0][origin]").is_some());
        assert_eq!(
            "Herkunftsland ist erforderlich für namensgebende Zutaten.",
            *validation_messages.get("ingredients[0][origin]").unwrap()
        );
    }

    #[test]
    fn country_display_on_label_for_namensgebende_ingredient() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AP7_2_HerkunftNamensgebendeZutat]);
        let input = Input {
            ingredients: vec![Ingredient {
                name: "Milch".to_string(),
                amount: 300.,
                is_namensgebend: Some(true),
                origin: Some(Country::CH),
                ..Default::default()
            }],
            total: Some(1000.),
        };
        let output = calculator.execute(input);
        let label = output.label;
        assert!(label.contains("Milch (Schweiz)"));
    }

    #[test]
    fn no_origin_required_for_non_namensgebende_ingredient() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::AP7_2_HerkunftNamensgebendeZutat]);
        let input = Input {
            ingredients: vec![Ingredient {
                name: "Zucker".to_string(),
                amount: 300.,
                is_namensgebend: Some(false), // Not namensgebend
                origin: None,
                ..Default::default()
            }],
            total: Some(1000.),
        };
        let output = calculator.execute(input);
        let validation_messages = output.validation_messages;
        let conditionals = output.conditional_elements;
        // Should not require origin validation
        assert!(validation_messages.get("ingredients[0][origin]").is_none());
        // Should not show origin field
        assert!(conditionals.get("herkunft_benoetigt_0").is_none());
    }

    #[test]
    fn combined_rules_both_50_percent_and_namensgebende() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![
            RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent,
            RuleDef::AP7_2_HerkunftNamensgebendeZutat,
        ]);
        let input = Input {
            ingredients: vec![
                Ingredient {
                    name: "Milch".to_string(),
                    amount: 600., // >50% of 1000
                    is_namensgebend: Some(false),
                    origin: Some(Country::CH),
                    ..Default::default()
                },
                Ingredient {
                    name: "Vanille".to_string(),
                    amount: 50., // <50% but namensgebend
                    is_namensgebend: Some(true),
                    origin: Some(Country::EU),
                    ..Default::default()
                },
            ],
            total: Some(1000.),
        };
        let output = calculator.execute(input);
        let conditionals = output.conditional_elements;
        let label = output.label;

        // Both ingredients should show origin fields
        assert!(conditionals.get("herkunft_benoetigt_0").is_some());
        assert!(conditionals.get("herkunft_benoetigt_1").is_some());

        // Both should display country on label
        assert!(label.contains("Milch (Schweiz)"));
        assert!(label.contains("Vanille (EU)"));
    }

    #[test]
    fn bio_knospe_alle_zutaten_herkunft_conditional() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::Bio_Knospe_AlleZutatenHerkunft]);
        let input = Input {
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
        calculator.registerRuleDefs(vec![RuleDef::Bio_Knospe_AlleZutatenHerkunft]);
        let input = Input {
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
        assert_eq!(output.validation_messages.get("ingredients[1][origin]"),
                   Some(&"Herkunftsland ist erforderlich für alle Zutaten (Bio/Knospe Anforderung)."));
        // Should NOT have validation error for the ingredient with origin
        assert_eq!(output.validation_messages.get("ingredients[0][origin]"), None);
    }

    #[test]
    fn bio_knospe_country_display_on_label_for_all_ingredients() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::Bio_Knospe_AlleZutatenHerkunft]);
        let input = Input {
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
        calculator.registerRuleDefs(vec![RuleDef::Bio_Knospe_AlleZutatenHerkunft]);
        let input = Input {
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
        assert_eq!(output.validation_messages.get("ingredients[0][origin]"),
                   Some(&"Herkunftsland ist erforderlich für alle Zutaten (Bio/Knospe Anforderung)."));
        assert_eq!(output.validation_messages.get("ingredients[1][origin]"),
                   Some(&"Herkunftsland ist erforderlich für alle Zutaten (Bio/Knospe Anforderung)."));
    }

    #[test]
    fn bio_knospe_no_validation_errors_when_all_have_origin() {
        let mut calculator = setup_simple_calculator();
        calculator.registerRuleDefs(vec![RuleDef::Bio_Knospe_AlleZutatenHerkunft]);
        let input = Input {
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
            ingredients: vec![
                Ingredient {
                    name: "Hafer".to_string(),
                    amount: 600.,
                    origin: Some(Country::CH),
                    is_agricultural: true,
                    ..Default::default()
                },
                Ingredient {
                    name: "Weizenmehl".to_string(),
                    amount: 400.,
                    origin: Some(Country::CH),
                    is_agricultural: true,
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
            ingredients: vec![
                Ingredient {
                    name: "Hafer".to_string(),
                    amount: 500.,
                    origin: Some(Country::CH),
                    is_agricultural: true,
                    ..Default::default()
                },
                Ingredient {
                    name: "Weizenmehl".to_string(),
                    amount: 400.,
                    origin: Some(Country::CH),
                    is_agricultural: true,
                    ..Default::default()
                },
                Ingredient {
                    name: "Olivenöl".to_string(),
                    amount: 100.,
                    origin: Some(Country::EU),
                    is_agricultural: true,
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
            ingredients: vec![
                Ingredient {
                    name: "Hafer".to_string(),
                    amount: 400.,
                    origin: Some(Country::CH),
                    is_agricultural: true,
                    ..Default::default()
                },
                Ingredient {
                    name: "Olivenöl".to_string(),
                    amount: 600.,
                    origin: Some(Country::EU),
                    is_agricultural: true,
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
    fn calculate_swiss_agricultural_percentage_100_percent() {
        let ingredients = vec![
            Ingredient {
                name: "Hafer".to_string(),
                amount: 600.,
                origin: Some(Country::CH),
                is_agricultural: true,
                ..Default::default()
            },
            Ingredient {
                name: "Weizenmehl".to_string(),
                amount: 400.,
                origin: Some(Country::CH),
                is_agricultural: true,
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
                is_agricultural: true,
                ..Default::default()
            },
            Ingredient {
                name: "Weizenmehl".to_string(),
                amount: 400.,
                origin: Some(Country::CH),
                is_agricultural: true,
                ..Default::default()
            },
            Ingredient {
                name: "Olivenöl".to_string(),
                amount: 100.,
                origin: Some(Country::EU),
                is_agricultural: true,
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
                is_agricultural: true,
                ..Default::default()
            },
            Ingredient {
                name: "Salz".to_string(),
                amount: 500.,
                origin: Some(Country::EU),
                is_agricultural: false,  // This field is ignored since getter always returns true
                ..Default::default()
            },
        ];

        let percentage = calculate_swiss_agricultural_percentage(&ingredients);
        // Since getter always returns true, all ingredients are considered agricultural
        // Swiss: 500g, Total: 1000g -> 50%
        assert_eq!(percentage, 50.0);
    }
}
