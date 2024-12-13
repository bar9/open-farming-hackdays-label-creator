use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::core::{Calculator, Ingredient, Input, Lookup};
use crate::rules::RuleDef;

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub struct IngredientItem {
    pub basicInfo: BasicIngredientItem,
    pub additionalInfo: AdditionalInfo
}

impl IngredientItem {
    pub(crate) fn from_name_amount(name: String, amount: i32) -> Self {
        let is_allergen = lookup_allergen(&name);
        IngredientItem {
            basicInfo: BasicIngredientItem {
                standard_ingredient: StandardIngredient {name, is_allergen},
                amount,
            },
            additionalInfo: AdditionalInfo::None,
        }
    }
}

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub enum AdditionalInfo {
    Meat {
        origin: Country
    },
    Milk {
        pasteurized: bool
    },
    None
}

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub enum Country {
    CH, EU
}

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub struct BasicIngredientItem {
    pub standard_ingredient: StandardIngredient,
    pub amount: i32,
}

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub struct StandardIngredient {
    pub name: String,
    pub is_allergen: bool
}
pub fn processed_ingredient_list(ingredients: Vec<IngredientItem>, rules: Vec<RuleDef>) -> String {

    let lookup = Lookup {};
    let mut calculator = Calculator::new();
    calculator.registerRuleDefs(rules);
    calculator.registerLookup(lookup);
    let input1 = Input {
        ingredients: ingredients.iter().map(|ing_item| {
            Ingredient{
                name: ing_item.clone().basicInfo.standard_ingredient.name,
                is_allergen: ing_item.basicInfo.standard_ingredient.is_allergen,
                amount: ing_item.basicInfo.amount as f64
            }
        }).collect()

    };

    let output = calculator.execute(input1);

    format!("<span>{}</span>", output.label)
}

pub fn validations(ingredients: Vec<IngredientItem>, rules: Vec<RuleDef>) -> HashMap<String, &'static str> {

    let lookup = Lookup {};
    let mut calculator = Calculator::new();
    calculator.registerRuleDefs(rules);
    calculator.registerLookup(lookup);
    let input1 = Input {
        ingredients: ingredients.iter().map(|ing_item| {
            Ingredient{
                name: ing_item.clone().basicInfo.standard_ingredient.name,
                is_allergen: ing_item.basicInfo.standard_ingredient.is_allergen,
                amount: ing_item.basicInfo.amount as f64
            }
        }).collect()

    };

    let output = calculator.execute(input1);
    output.validation_messages

}

pub fn lookup_allergen(name: &str) -> bool {
    let mut is_allergen = false;
    for entry in food_db() {
        if entry.0.as_str() == name && entry.1 == true {
            is_allergen = true;
        }
    }
    is_allergen
}

pub fn food_db() -> Vec<(String, bool)> {
    let mut db: Vec<(String, bool)> = Vec::new();
    let db_csv = include_str!("food_db.csv");
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(db_csv.as_bytes());

    for record in rdr.records() {
        let record = record.unwrap();
        db.push((
            record.get(0).unwrap().to_string(), {
                record.get(1).unwrap()
                    .eq("1")
            })
        );
    }
    db
}
