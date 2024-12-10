use std::cmp::PartialEq;
use std::mem;
use crate::rules::RuleDef;

#[derive(Clone)]
pub struct Input {
    pub(crate) ingredients: Vec<Ingredient>
}

impl Input {
    pub fn scale(&mut self, factor: f64) {
        for ingredient in self.ingredients.iter_mut() {
            ingredient.amount *=factor;
        }
    }
}

pub struct Output {
    pub success: bool,
    pub label: String,
    pub total_amount: f64
}

pub struct Lookup {

}

pub struct Calculator {
    pub(crate) RuleDefs: Vec<RuleDef>
}

impl Calculator {
    pub(crate) fn new() -> Self {
        Calculator {
            RuleDefs: vec![]
        }
    }
}

#[derive(Clone, Debug)]
pub struct Ingredient {
    pub name: String,
    pub is_allergen: bool,
    pub amount: f64
}

#[derive(Clone)]
pub struct CombinedIngredient {
    pub name: String,
    pub amount: f64,
    pub ingredients: Vec<Ingredient>
}

pub enum Unit {
    Percentage,
    Gramm,
    None
}

struct OutputFormatter {
    ingredient: Ingredient,
    RuleDefs: Vec<RuleDef>,
    total_amount: f64
    // bold: FnOnce(),
    // amount_unit: Unit,
    // parentheses: bool,
    // show_provenance: bool
}

impl PartialEq for RuleDef {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}

impl OutputFormatter {
    pub fn from(ingredient: Ingredient, total_amount: f64, RuleDefs: Vec<RuleDef>) -> Self {
        Self {
            ingredient, total_amount, RuleDefs
        }
    }

    pub fn format(&self) -> String {
        let mut output = "".to_string();
        output = match self.ingredient.is_allergen {
            true => format!{"<b>{}</b>", self.ingredient.name},
            false => String::from(self.ingredient.name.clone()),
        };
        if (self.RuleDefs.iter().find(|x| **x == RuleDef::AllPercentages)).is_some() {
            output = format!("{} {}%", output, (self.ingredient.amount / self.total_amount * 100.) as u8)
        }
        if (self.RuleDefs.iter().find(|x| **x == RuleDef::PercentagesStartsWithM)).is_some() {
            if (self.ingredient.name.starts_with("M")) {
                output = format!("{} {}%", output, (self.ingredient.amount / self.total_amount * 100.) as u8)
            }
        }
        if (self.RuleDefs.iter().find(|x| **x == RuleDef::MaxDetails)).is_some() {
            output = format!{"{:?}", self.ingredient}
        }
        if (self.RuleDefs.iter().find(|x| **x == RuleDef::AllGram)).is_some() {
            output = format!{"{} {}g", self.ingredient.name, self.ingredient.amount}
        }
        if (self.RuleDefs.iter().find(|x| **x == RuleDef::Composite)).is_some() {
            if self.ingredient.name == "Brot" {
                output = format!{"{} (<b>Weizenmehl</b>, Wasser, Hefe, Salz)", output}
            }
        }
        output
    }
}

impl Calculator {
    pub fn registerRuleDefs(&mut self, RuleDefs: Vec<RuleDef>) {
        self.RuleDefs = RuleDefs;
    }
    pub fn registerLookup(&self, lookup: Lookup) {}
    pub fn execute(&self, input: Input) -> Output {
        let mut sorted_ingredients = input.ingredients.clone();
        sorted_ingredients
            .sort_by(|y, x| x.amount.partial_cmp(&y.amount).unwrap());

        for ruleDef in &self.RuleDefs {
            match ruleDef {
                _ => {}
            }
        }

        let total_amount = input.ingredients.iter().map(|x|x.amount).sum();

        Output {
            success: true,
            label: sorted_ingredients
                .into_iter()
                .map(|item| OutputFormatter::from(item, total_amount, self.RuleDefs.clone()))
                .map(|fmt| fmt.format())
                .collect::<Vec<_>>()
                .join(", "),
            total_amount
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_simple_calculator() -> Calculator {
        let RuleDefs = vec![];
        let lookup = Lookup {};
        let mut calculator = Calculator{ RuleDefs };
        calculator.registerLookup(lookup);
        calculator
    }

    #[test]
    fn simple_run_of_process() {
        let calculator = setup_simple_calculator();
        let input = Input{ ingredients: vec![] };

        let output = calculator.execute(input);
        assert!(output.success);
    }

    #[test]
    fn single_ingredient_visible_on_label() {
        let calculator = setup_simple_calculator();
        let input = Input {
            ingredients: vec![
                Ingredient{name: "Hafer".to_string(), is_allergen: false, amount: 42.}
            ]
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
                Ingredient{ name: "Hafer".to_string(), is_allergen: false, amount: 42.},
                Ingredient{ name: "Zucker".to_string(), is_allergen: false, amount: 42.},
            ]
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
                Ingredient{ name: "Hafer".to_string(), is_allergen: false, amount: 300.},
                Ingredient{ name: "Zucker".to_string(), is_allergen: false, amount: 700.}
            ]
        };
        let output = calculator.execute(input);
        let label = output.label;
        assert!(label.contains("Zucker, Hafer"));
    }

    #[test]
    fn allergenes_printed_bold_on_label() {
        let calculator = setup_simple_calculator();
        let input = Input {
            ingredients: vec![
                Ingredient{ name: "Weizenmehl".to_string(), is_allergen: true, amount: 300.},
            ]
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
                Ingredient{ name: "Hafer".to_string(), is_allergen: false, amount: 300.},
                Ingredient{ name: "Zucker".to_string(), is_allergen: false, amount: 700.}
            ]
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
                Ingredient{ name: "Hafer".to_string(), is_allergen: false, amount: 300.},
                Ingredient{ name: "Zucker".to_string(), is_allergen: false, amount: 700.}
            ]
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
                Ingredient{ name: "Hafer".to_string(), is_allergen: false, amount: 300.},
                Ingredient{ name: "Milch".to_string(), is_allergen: true, amount: 700.}
            ]
        };
        let output = calculator.execute(input);
        let label = output.label;
        assert!(label.contains("<b>Milch</b> 70%, Hafer"));
    }

    #[test]
    fn composite_ingredients_listed_in_parentheses_on_label() {
    }

}