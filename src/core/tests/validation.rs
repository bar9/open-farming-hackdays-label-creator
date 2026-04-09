use super::*;

#[test]
fn amount_lt_zero_invalid() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::AP1_1_ZutatMengeValidierung]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new("Hafer", 0.0).build())
        .build();
    let output = calculator.execute(input);
    let validation_messages = output.validation_messages;
    assert!(validation_messages.get("ingredients[0][amount]").is_some());
    let amount_messages = validation_messages.get("ingredients[0][amount]").unwrap();
    assert!(!amount_messages.is_empty());
    assert!(amount_messages.iter().any(|m| m == "Die Menge muss grösser als 0 sein."));
}

#[test]
fn amount_gt_zero_valid() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::AP1_1_ZutatMengeValidierung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Hafer", 32.0).build())
        .build();
    let output = calculator.execute(input);
    let validation_messages = output.validation_messages;
    assert!(validation_messages.get("ingredients[0][amount]").map_or(true, |v| v.is_empty()));
}

#[test]
fn multiple_validation_errors_on_single_ingredient() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![
        RuleDef::AP1_1_ZutatMengeValidierung,
        RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent,
        RuleDef::AP7_3_HerkunftFleischUeber20Prozent,
        RuleDef::AP7_4_RindfleischHerkunftDetails,
    ]);

    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(
            IngredientBuilder::new("Rindfleisch", 600.0)
                .category("Rind")
                .build()
        )
        .ingredient(IngredientBuilder::new("Invalid Ingredient", 0.0).build())
        .total(1000.0)
        .build();

    let output = calculator.execute(input);
    let validation_messages = output.validation_messages;

    // Verify beef validation errors are present for ingredient 0
    assert!(validation_messages.contains_key("ingredients[0][origin]"));
    assert!(validation_messages.contains_key("ingredients[0][aufzucht_ort]"));
    assert!(validation_messages.contains_key("ingredients[0][schlachtungs_ort]"));

    // Verify amount validation error for ingredient 1
    assert!(validation_messages.contains_key("ingredients[1][amount]"));

    // Verify the messages are correct
    let origin_messages = validation_messages.get("ingredients[0][origin]").unwrap();
    let aufzucht_messages = validation_messages.get("ingredients[0][aufzucht_ort]").unwrap();
    let schlachtungs_messages = validation_messages.get("ingredients[0][schlachtungs_ort]").unwrap();
    let amount_messages = validation_messages.get("ingredients[1][amount]").unwrap();

    // Should contain multiple origin messages for different rules
    assert!(origin_messages.iter().any(|m| m == "Herkunftsland ist erforderlich für Zutaten über 50%."));
    assert!(origin_messages.iter().any(|m| m == "Herkunftsland ist erforderlich für Fleisch-Zutaten über 20%."));
    assert!(aufzucht_messages.iter().any(|m| m == "Aufzuchtort ist erforderlich für Rindfleisch-Zutaten."));
    assert!(schlachtungs_messages.iter().any(|m| m == "Schlachtungsort ist erforderlich für Rindfleisch-Zutaten."));
    assert!(amount_messages.iter().any(|m| m == "Die Menge muss grösser als 0 sein."));

    // Count total messages across all fields
    let total_messages: usize = validation_messages.values().map(|v| v.len()).sum();
    println!("Multiple validation errors successfully captured: {} fields with {} total messages", validation_messages.len(), total_messages);
}

#[test]
fn stacked_validation_messages_demo() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![
        RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent,
        RuleDef::AP7_3_HerkunftFleischUeber20Prozent,
    ]);

    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(
            IngredientBuilder::new("Rindfleisch", 600.0)
                .category("Rind")
                .build()
        )
        .total(1000.0)
        .build();

    let output = calculator.execute(input);
    let validation_messages = output.validation_messages;

    // Verify that BOTH validation messages are present for the same field
    let origin_messages = validation_messages.get("ingredients[0][origin]").unwrap();

    println!("Origin validation messages for beef ingredient at 60%:");
    for msg in origin_messages {
        println!("  - {}", msg);
    }

    // Both rules should have added their messages
    assert_eq!(origin_messages.len(), 2, "Should have exactly 2 validation messages for origin field");
    assert!(origin_messages.iter().any(|m| m == "Herkunftsland ist erforderlich für Zutaten über 50%."));
    assert!(origin_messages.iter().any(|m| m == "Herkunftsland ist erforderlich für Fleisch-Zutaten über 20%."));

    println!("✅ Successfully demonstrated stacked validation messages!");
}
