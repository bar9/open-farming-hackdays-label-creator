use serde::{Deserialize, Serialize};

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
pub fn sorted_ingredient_list(mut ingredients:Vec<IngredientItem>) -> String {
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
