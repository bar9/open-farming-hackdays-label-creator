use super::*;

#[test]
fn test_fish_functionality() {
    let calculator = calculator_for(crate::shared::Configuration::Conventional);

    // Test with fish ingredient missing fangort
    let input_missing_fangort = InputBuilder::new()
        .vollstaendig()
        .ingredient(
            IngredientBuilder::new("Lachs", 200.0)
                .category("Meeresfische")
                .build()
        )
        .build();

    let output_missing = calculator.execute(input_missing_fangort);

    // Should have validation error for fangort
    assert!(output_missing.validation_messages.contains_key("ingredients[0][fangort]"));
    let fangort_messages = output_missing.validation_messages.get("ingredients[0][fangort]").unwrap();
    assert!(fangort_messages.iter().any(|m| m == "Fangort ist erforderlich für Fisch-Zutaten."));

    // Test with fish ingredient having fangort filled
    let input_with_fangort = InputBuilder::new()
        .vollstaendig()
        .ingredient(
            IngredientBuilder::new("Lachs", 200.0)
                .category("Meeresfische")
                .fangort(Country::CH)
                .build()
        )
        .build();

    let output_with_fangort = calculator.execute(input_with_fangort);

    // Should have no validation errors
    assert!(!output_with_fangort.validation_messages.contains_key("ingredients[0][fangort]"));

    // Should display fish origin in label
    println!("Fish label output: {}", output_with_fangort.label);
    assert!(output_with_fangort.label.contains("(CH)"));

    // Test with non-fish ingredient - should not require fangort
    let input_non_fish = InputBuilder::new()
        .vollstaendig()
        .ingredient(
            IngredientBuilder::new("Weizen", 300.0)
                .category("Getreide")
                .build()
        )
        .build();

    let output_non_fish = calculator.execute(input_non_fish);

    // Should not have validation errors for fangort since it's not fish
    assert!(!output_non_fish.validation_messages.contains_key("ingredients[0][fangort]"));
}
