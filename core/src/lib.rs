#[derive(Clone)]
struct Input<'a> {
    ingredients: Vec<Ingredient<'a>>
}

impl Input<'_> {
    pub fn scale(&mut self, factor: f64) {
        for ingredient in self.ingredients.iter_mut() {
            ingredient.amount *=factor;
        }
    }
}

struct Output {
    success: bool,
    label: String,
    total_amount: f64
}

enum Rule {
    AllPercentages,
    MaxDetails
}

struct Lookup {

}

struct Calculator {

}

#[derive(Clone)]
struct Ingredient<'a> {
    name: & 'a str,
    is_allergen: bool,
    amount: f64
}

enum Unit {
    Percentage,
    Gramm,
    None
}

struct IngredientFormatter {
    bold: bool,
    amount_unit: Unit,
    parentheses: bool,
    show_provenance: bool
}

impl Calculator {
    fn mapAllergenes(ingredient: Ingredient) -> String {
        match ingredient.is_allergen {
            true => format!{"<b>{}</b>", ingredient.name},
            false => String::from(ingredient.name),
        }
    }
    pub fn registerRules(&self, rules: Vec<Rule>) {}
    pub fn registerLookup(&self, lookup: Lookup) {}
    pub fn execute(&self, input: Input) -> Output {
        let mut sorted_ingredients = input.ingredients.clone();
        sorted_ingredients
            .sort_by(|y, x| x.amount.partial_cmp(&y.amount).unwrap());
        Output {
            success: true,
            label: sorted_ingredients
                .into_iter()
                .map(Self::mapAllergenes)
                .collect::<Vec<_>>()
                .join(", "),
            total_amount: input.ingredients.iter().map(|x|x.amount).sum()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_simple_calculator() -> Calculator {
        let rules = vec![];
        let lookup = Lookup {};
        let calculator = Calculator{};
        calculator.registerRules(rules);
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
                Ingredient{name: "Hafer", is_allergen: false, amount: 42.}
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
                Ingredient{ name: "Hafer", is_allergen: false, amount: 42.},
                Ingredient{ name: "Zucker", is_allergen: false, amount: 42.},
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
                Ingredient{ name: "Hafer", is_allergen: false, amount: 300.},
                Ingredient{ name: "Zucker", is_allergen: false, amount: 700.}
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
                Ingredient{ name: "Weizenmehl", is_allergen: true, amount: 300.},
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
                Ingredient{ name: "Hafer", is_allergen: false, amount: 300.},
                Ingredient{ name: "Zucker", is_allergen: false, amount: 700.}
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
        // let calculator = setup_simple_calculator();
        // calculator.registerRules(vec![Rule::AllPercentages]);
        // let input = Input {
        //     ingredients: vec![
        //         Ingredient{ name: "Hafer", is_allergen: false, amount: 300.},
        //         Ingredient{ name: "Zucker", is_allergen: false, amount: 700.}
        //     ]
        // };
        // let output = calculator.execute(input);
        // let label = output.label;
        // assert!(label.contains("Hafer 30%"));
        // assert!(label.contains("Zucker 70%"));
    }

    #[test]
    fn composite_ingredients_listed_in_parentheses_on_label() {
    }

}
