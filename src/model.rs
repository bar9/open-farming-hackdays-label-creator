use serde::{Deserialize, Serialize};
use crate::model::Unit::{Centiliter, Gram, Kilogram, Liter, Milligram, Milliliter};
use crate::model::UnitKind::{Volume, Weight};

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


#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Unit {
    Milliliter,
    Centiliter,
    Liter,
    Milligram,
    Gram,
    Kilogram,
}

pub enum UnitKind {
    Volume,
    Weight
}

impl Unit {
    pub fn scaling_factor(&self) -> (UnitKind, Unit, f64) {
        match (&self) {
            Unit::Milliliter => (Volume, Unit::Liter, 0.001),
            Unit::Centiliter => (Volume, Unit::Liter, 0.01),
            Unit::Liter => (Volume, Unit::Liter, 1.),
            Unit::Milligram => (Weight, Unit::Kilogram, 0.000001),
            Unit::Gram => (Weight, Unit::Kilogram, 0.001),
            Unit::Kilogram => (Weight, Unit::Kilogram, 1.)
        }
    }
    pub fn label(&self) -> &'static str {
        match (&self) {
            Unit::Milliliter => "ml",
            Unit::Centiliter => "cl",
            Unit::Liter => "l",
            Unit::Milligram => "mg",
            Unit::Gram => "g",
            Unit::Kilogram => "kg"
        }
    }
    pub fn all_input() -> &'static [Unit] {
        &[Milliliter, Centiliter, Liter, Milligram, Gram, Kilogram]
    }
}