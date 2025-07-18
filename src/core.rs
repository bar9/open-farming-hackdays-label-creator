use crate::model::lookup_allergen;
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
        output
    }
}

impl Calculator {
    pub fn registerRuleDefs(&mut self, rule_defs: Vec<RuleDef>) {
        self.rule_defs = rule_defs;
    }
    pub fn execute(&self, input: Input) -> Output {
        let mut validation_messages = HashMap::new();
        let mut conditionals = HashMap::new();

        // validations
        for ruleDef in &self.rule_defs {
            if let RuleDef::AP1_1_ZutatMengeValidierung = ruleDef {
                validate_amount(&input.ingredients, &mut validation_messages)
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

        let mut total_amount = input.ingredients.iter().map(|x| x.amount).sum();
        if self
            .rule_defs
            .iter()
            .any(|x| *x == RuleDef::AP1_4_ManuelleEingabeTotal)
        {
            conditionals.insert(String::from("manuelles_total"), true);
            if let Some(tot) = input.total {
                total_amount = tot;
            }
        }

        Output {
            success: true,
            label: sorted_ingredients
                .into_iter()
                .map(|item| OutputFormatter::from(item, total_amount, self.rule_defs.clone()))
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
}
