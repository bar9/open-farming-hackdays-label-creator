use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub struct IngredientItem {
    pub basicInfo: BasicIngredientItem,
    pub additionalInfo: AdditionalInfo
}

impl IngredientItem {
   pub(crate) fn from_name(name: String) -> Self {
       IngredientItem {
           basicInfo: BasicIngredientItem{
               name,
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
    pub name: String,
    pub amount: i32,
}

pub fn sorted_ingredient_list(ingredients: HashMap<String, IngredientItem>) -> String {
    let mut ingredients = ingredients.values().cloned().into_iter().collect::<Vec<IngredientItem>>();
    ingredients.sort_by(|a, b| b.basicInfo.amount.cmp(&a.basicInfo.amount));

    ingredients.iter()
        .map(|ele| ele.basicInfo.name.clone())
        .collect::<Vec<_>>()
        .join(", ")
}