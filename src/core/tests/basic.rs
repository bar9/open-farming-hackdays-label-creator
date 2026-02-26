use super::*;

#[test]
fn simple_run_of_process() {
    let calculator = setup_simple_calculator();
    let input = InputBuilder::new().build();

    let output = calculator.execute(input);
    assert!(output.success);
}

#[test]
fn single_ingredient_visible_on_label() {
    let calculator = setup_simple_calculator();
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Hafer", 42.0).build())
        .build();
    let output = calculator.execute(input);
    let label = output.label;
    assert!(label.contains("Hafer"));
}

#[test]
fn multiple_ingredients_comma_separated_on_label() {
    let calculator = setup_simple_calculator();
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Hafer", 42.0).build())
        .ingredient(IngredientBuilder::new("Zucker", 42.0).build())
        .build();
    let output = calculator.execute(input);
    let label = output.label;
    assert!(label.contains("Hafer, Zucker"));
}

#[test]
fn ingredients_ordered_by_weight_on_label() {
    let calculator = setup_simple_calculator();
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Hafer", 300.0).build())
        .ingredient(IngredientBuilder::new("Zucker", 700.0).build())
        .build();
    let output = calculator.execute(input);
    let label = output.label;
    assert!(label.contains("Zucker, Hafer"));
}

#[test]
fn allergenes_printed_bold_on_label() {
    let calculator = setup_simple_calculator();
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Weizenmehl", 300.0).allergen().build())
        .build();
    let output = calculator.execute(input);
    let label = output.label;
    assert!(label.contains("<b>Weizenmehl</b>"));
}

#[test]
fn scaled_recipe_invariant_on_label() {
    let calculator = setup_simple_calculator();
    let input1 = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Hafer", 300.0).build())
        .ingredient(IngredientBuilder::new("Zucker", 700.0).build())
        .build();
    let mut input2 = input1.clone();
    input2.scale(2.);
    let output = calculator.execute(input1);
    let scaled_output = calculator.execute(input2);

    assert_eq!(output.label, scaled_output.label);
    assert_ne!(output.total_amount, scaled_output.total_amount);
}
