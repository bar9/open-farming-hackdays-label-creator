use super::*;

#[test]
fn ap1_2_namensgebend() {
    let calculator = calculator_with(vec![RuleDef::AP1_2_ProzentOutputNamensgebend]);
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
    let calculator = calculator_with(vec![RuleDef::AP1_3_EingabeNamensgebendeZutat]);
    let input = InputBuilder::new().build();
    let output = calculator.execute(input);
    let conditionals = output.conditional_elements;
    assert!(conditionals.contains_key("namensgebende_zutat"));
    assert!(*conditionals.get("namensgebende_zutat").unwrap());
}

#[test]
fn ap1_4_manuelle_eingabe_total() {
    let calculator = calculator_with(vec![RuleDef::AP1_4_ManuelleEingabeTotal]);
    let input = InputBuilder::new().build();
    let output = calculator.execute(input);
    let conditionals = output.conditional_elements;
    assert!(conditionals.contains_key("manuelles_total"));
    assert!(*conditionals.get("manuelles_total").unwrap());
}

#[test]
fn ap1_4_manualTotalChangesPercent() {
    let calculator = calculator_with(vec![
        RuleDef::AP1_2_ProzentOutputNamensgebend,
        RuleDef::AP1_4_ManuelleEingabeTotal,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Milch", 700.0).allergen().namensgebend().build())
        .total(350.0)
        .build();
    let output = calculator.execute(input);
    let conditionals = output.conditional_elements;
    assert!(conditionals.contains_key("manuelles_total"));
    assert!(*conditionals.get("manuelles_total").unwrap());
}
