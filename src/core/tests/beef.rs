use super::*;

#[test]
fn beef_origin_display_shows_geburtsort() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::AP7_4_RindfleischHerkunftDetails]);
    let input = InputBuilder::new()
        .ingredient(
            IngredientBuilder::new("Rindfleisch", 500.0)
                .category("Rind")
                .aufzucht(Country::FR)
                .schlachtung(Country::DE)
                .build()
        )
        .build();
    let output = calculator.execute(input);
    let label = output.label;

    // In test environment, i18n returns key names instead of translations
    // This is expected behavior - the important thing is that the right keys are being used
    assert!(label.contains("birthplace"));
    assert!(label.contains("slaughtered_in"));
    assert!(!label.contains("Aufgezogen in"));
}

#[test]
fn test_beef_with_swiss_conventional_rules() {
    let calculator = calculator_for(crate::shared::Configuration::Conventional);

    // Test with beef ingredient having both fields filled (simulating real usage)
    let input = InputBuilder::new()
        .ingredient(
            IngredientBuilder::new("Rindfleisch", 300.0)
                .category("Rind")
                .aufzucht(Country::FR)
                .schlachtung(Country::DE)
                .build()
        )
        .build();

    let output = calculator.execute(input);

    // Should have no validation errors
    assert!(!output.validation_messages.contains_key("ingredients[0][aufzucht_ort]"));
    assert!(!output.validation_messages.contains_key("ingredients[0][schlachtungs_ort]"));

    // Should display beef-specific origin format in label (not traditional origin)
    // In test environment, i18n returns key names
    assert!(output.label.contains("birthplace"));
    assert!(output.label.contains("slaughtered_in"));
    assert!(output.label.contains("(birthplace, slaughtered_in)"));

    // Should NOT contain traditional origin format since beef rule takes precedence
    assert!(!output.label.contains("(Frankreich)"));
    assert!(!output.label.contains("(Deutschland)"));
}

#[test]
fn test_beef_origin_validation_and_display() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![
        RuleDef::AP7_4_RindfleischHerkunftDetails,
    ]);

    // Test with beef ingredient missing both aufzucht_ort and schlachtungs_ort
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(
            IngredientBuilder::new("Rindfleisch", 300.0)
                .category("Rind")
                .build()
        )
        .build();

    let output = calculator.execute(input);

    // Should have validation errors for both fields
    assert!(output.validation_messages.contains_key("ingredients[0][aufzucht_ort]"));
    assert!(output.validation_messages.contains_key("ingredients[0][schlachtungs_ort]"));
    let aufzucht_messages = output.validation_messages.get("ingredients[0][aufzucht_ort]").unwrap();
    let schlachtungs_messages = output.validation_messages.get("ingredients[0][schlachtungs_ort]").unwrap();
    assert!(aufzucht_messages.iter().any(|m| m == "Aufzuchtort ist erforderlich für Rindfleisch-Zutaten."));
    assert!(schlachtungs_messages.iter().any(|m| m == "Schlachtungsort ist erforderlich für Rindfleisch-Zutaten."));

    // Test with beef ingredient having both fields filled
    let input_with_beef_origins = InputBuilder::new()
        .vollstaendig()
        .ingredient(
            IngredientBuilder::new("Rindfleisch", 300.0)
                .category("Rind")
                .aufzucht(Country::FR)
                .schlachtung(Country::DE)
                .build()
        )
        .build();

    let output_with_origins = calculator.execute(input_with_beef_origins);

    // Should have no validation errors
    assert!(!output_with_origins.validation_messages.contains_key("ingredients[0][aufzucht_ort]"));
    assert!(!output_with_origins.validation_messages.contains_key("ingredients[0][schlachtungs_ort]"));

    // Should display beef-specific origin format in label (using translation keys in test env)
    assert!(output_with_origins.label.contains("birthplace"));
    assert!(output_with_origins.label.contains("slaughtered_in"));
    assert!(output_with_origins.label.contains("Rindfleisch (birthplace, slaughtered_in)"));

    // Test with non-beef ingredient - should not require beef fields
    let input_non_beef = InputBuilder::new()
        .vollstaendig()
        .ingredient(
            IngredientBuilder::new("Schweinefleisch", 300.0)
                .category("Schwein")
                .build()
        )
        .build();

    let output_non_beef = calculator.execute(input_non_beef);

    // Should not have validation errors for beef fields since it's not beef
    assert!(!output_non_beef.validation_messages.contains_key("ingredients[0][aufzucht_ort]"));
    assert!(!output_non_beef.validation_messages.contains_key("ingredients[0][schlachtungs_ort]"));
}
