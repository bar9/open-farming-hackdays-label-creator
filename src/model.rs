use std::collections::HashMap;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub struct IngredientItem {
    pub basicInfo: BasicIngredientItem,
    pub additionalInfo: AdditionalInfo
}

impl IngredientItem {
   pub(crate) fn from_name(name: String) -> Self {
       let is_allergen = lookup_allergen(&name);
       IngredientItem {
           basicInfo: BasicIngredientItem {
               standard_ingredient: StandardIngredient {name, is_allergen},
               amount: 0,
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
pub fn sorted_ingredient_list(ingredients: HashMap<String, IngredientItem>) -> String {
    let mut ingredients = ingredients.values().cloned().into_iter().collect::<Vec<IngredientItem>>();
    ingredients.sort_by(|a, b| b.basicInfo.amount.cmp(&a.basicInfo.amount));

    let ingredients_string =
    ingredients.iter()
        .map(|ele| {
            if ele.basicInfo.standard_ingredient.is_allergen {
                format! {"<b>{} ({} g)</b>", ele.basicInfo.standard_ingredient.name.clone(), ele.basicInfo.amount}
            } else {
                format! {"{} ({} g)", ele.basicInfo.standard_ingredient.name.clone(), ele.basicInfo.amount}
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("<span>{}</span>", ingredients_string)
}

pub fn lookup_allergen(name: &str) -> bool {
    match name {
        "Erdnüsse" => true,
        "Haselnüsse" => true,
        _ => false
    }
}

pub fn food_db() -> Vec<(String, bool)> {
    // let mut db = Vec::new();
    // let db_csv = include_bytes!("food_db.csv");
    // let mut reader = csv::Reader::from_reader(db_csv);
    // for record in reader.records() {
    //     let record = record.unwrap();
    //     db.push((record.get(0).unwrap().to_string(), {
    //         if record.get(1).unwrap() == "1" {
    //             true
    //         } else {
    //             false
    //         }
    //     }));
    // }
    // db
    vec![
        (String::from("Hafer"), false),
        (String::from("Honig"), false),
        (String::from("Erdnüsse"), true),
        (String::from("Haselnüsse"), true),
        (String::from("Honig"), false),
        (String::from("Mandelmus"), false),
    ]
}
