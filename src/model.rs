use serde::{Deserialize, Serialize};

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub enum Country {
    CH, EU
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
