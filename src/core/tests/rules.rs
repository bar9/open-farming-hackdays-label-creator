use super::*;

#[test]
fn percentage_on_label_depending_on_setting() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::AllPercentages]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Hafer", 300.0).build())
        .ingredient(IngredientBuilder::new("Zucker", 700.0).build())
        .build();
    let output = calculator.execute(input);
    let label = output.label;
    assert!(label.contains("Hafer 30%"));
    assert!(label.contains("Zucker 70%"));
}

#[test]
fn percentage_on_label_depending_on_setting_2() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::PercentagesStartsWithM]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Hafer", 300.0).build())
        .ingredient(IngredientBuilder::new("Milch", 700.0).allergen().build())
        .build();
    let output = calculator.execute(input);
    let label = output.label;
    assert!(label.contains("<b>Milch</b> 70%, Hafer"));
}

#[test]
fn ap1_2_namensgebend() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::AP1_2_ProzentOutputNamensgebend]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Hafer", 300.0).build())
        .ingredient(IngredientBuilder::new("Milch", 700.0).allergen().namensgebend().build())
        .build();
    let output = calculator.execute(input);
    let label = output.label;
    assert!(label.contains("<b>Milch</b> 70%, Hafer"));
}

#[test]
fn ap1_3_eingabe_namensgebende_zutat() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::AP1_3_EingabeNamensgebendeZutat]);
    let input = InputBuilder::new().build();
    let output = calculator.execute(input);
    let conditionals = output.conditional_elements;
    assert!(conditionals.get("namensgebende_zutat").is_some());
    assert_eq!(true, *conditionals.get("namensgebende_zutat").unwrap());
}

#[test]
fn ap1_4_manuelle_eingabe_total() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::AP1_4_ManuelleEingabeTotal]);
    let input = InputBuilder::new().build();
    let output = calculator.execute(input);
    let conditionals = output.conditional_elements;
    assert!(conditionals.get("manuelles_total").is_some());
    assert_eq!(true, *conditionals.get("manuelles_total").unwrap());
}

#[test]
fn ap1_4_manualTotalChangesPercent() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![
        RuleDef::AP1_2_ProzentOutputNamensgebend,
        RuleDef::AP1_4_ManuelleEingabeTotal,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Milch", 700.0).allergen().namensgebend().build())
        .total(350.0)
        .build();
    let output = calculator.execute(input);
    let conditionals = output.conditional_elements;
    assert!(conditionals.get("manuelles_total").is_some());
    assert_eq!(true, *conditionals.get("manuelles_total").unwrap());
}
