use super::*;
pub(super) use crate::model::{lookup_agricultural, Country};

// --- Helpers ---

pub(super) fn setup_simple_calculator() -> Calculator {
    rust_i18n::set_locale("de-CH");
    let rule_defs = vec![];
    Calculator { rule_defs }
}

pub(super) fn calculator_for(config: crate::shared::Configuration) -> Calculator {
    rust_i18n::set_locale("de-CH");
    Calculator::from_registry_config(config)
}

// --- IngredientBuilder ---

pub(super) struct IngredientBuilder(Ingredient);

#[allow(dead_code)]
impl IngredientBuilder {
    pub fn new(name: &str, amount: f64) -> Self {
        Self(Ingredient {
            name: name.to_string(),
            amount,
            ..Default::default()
        })
    }

    /// Create with auto-lookup of is_agricultural field
    pub fn new_agri(name: &str, amount: f64) -> Self {
        Self(Ingredient {
            name: name.to_string(),
            amount,
            is_agricultural: lookup_agricultural(name),
            ..Default::default()
        })
    }

    pub fn allergen(mut self) -> Self { self.0.is_allergen = true; self }
    pub fn origin(mut self, country: Country) -> Self { self.0.origins = Some(vec![country]); self }
    pub fn origins(mut self, countries: Vec<Country>) -> Self { self.0.origins = Some(countries); self }
    pub fn category(mut self, cat: &str) -> Self { self.0.category = Some(cat.to_string()); self }
    pub fn namensgebend(mut self) -> Self { self.0.is_namensgebend = Some(true); self }
    pub fn aufzucht(mut self, country: Country) -> Self { self.0.aufzucht_ort = Some(country); self }
    pub fn schlachtung(mut self, country: Country) -> Self { self.0.schlachtungs_ort = Some(country); self }
    pub fn fangort(mut self, country: Country) -> Self { self.0.fangort = Some(country); self }
    pub fn agricultural(mut self, val: bool) -> Self { self.0.is_agricultural = val; self }
    pub fn bio(mut self) -> Self { self.0.is_bio = Some(true); self }
    pub fn bio_ch(mut self) -> Self { self.0.bio_ch = Some(true); self }
    pub fn umstellbetrieb(mut self) -> Self { self.0.aus_umstellbetrieb = Some(true); self }
    pub fn sub_components(mut self, subs: Vec<SubIngredient>) -> Self { self.0.sub_components = Some(subs); self }
    pub fn children(mut self, kids: Vec<Ingredient>) -> Self { self.0.children = Some(kids); self }
    pub fn processing_steps(mut self, steps: Vec<&str>) -> Self { self.0.processing_steps = Some(steps.iter().map(|s| s.to_string()).collect()); self }
    pub fn override_children(mut self) -> Self { self.0.override_children = Some(true); self }
    pub fn build(self) -> Ingredient { self.0 }
}

// --- InputBuilder ---

pub(super) struct InputBuilder(Input);

#[allow(dead_code)]
impl InputBuilder {
    pub fn new() -> Self { Self(Input::default()) }
    pub fn ingredients(mut self, ings: Vec<Ingredient>) -> Self { self.0.ingredients = ings; self }
    pub fn ingredient(mut self, ing: Ingredient) -> Self { self.0.ingredients.push(ing); self }
    pub fn total(mut self, t: f64) -> Self { self.0.total = Some(t); self }
    pub fn vollstaendig(mut self) -> Self { self.0.rezeptur_vollstaendig = true; self }
    pub fn certification_body(mut self, body: &str) -> Self { self.0.certification_body = Some(body.to_string()); self }
    pub fn build(self) -> Input { self.0 }
}

mod basic;
mod rules;
mod validation;
mod origin;
mod knospe;
mod percentage;
mod beef;
mod fish;
mod category;
mod bio;
mod golden;
mod recipes;
mod saved_ingredients;
