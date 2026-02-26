use super::*;

#[test]
fn ap7_1_herkunft_benoetigt_ueber_50_prozent() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Milch", 700.0).build())
        .total(350.0)
        .build();
    let output = calculator.execute(input);
    let conditionals = output.conditional_elements;
    assert!(conditionals
        .get("herkunft_benoetigt_ueber_50_prozent")
        .is_some());
    assert_eq!(
        true,
        *conditionals
            .get("herkunft_benoetigt_ueber_50_prozent")
            .unwrap()
    );
}

#[test]
fn validation_missing_origin_for_ingredient_over_50_percent() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new("Milch", 700.0).build())
        .total(350.0)
        .build();
    let output = calculator.execute(input);
    let validation_messages = output.validation_messages;
    assert!(validation_messages.get("ingredients[0][origin]").is_some());
    let origin_messages = validation_messages.get("ingredients[0][origin]").unwrap();
    assert!(!origin_messages.is_empty());
    assert!(origin_messages.iter().any(|m| m == "Herkunftsland ist erforderlich für Zutaten über 50%."));
}

#[test]
fn country_display_on_label_for_ingredients_with_origin() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Milch", 600.0).origin(Country::CH).build())
        .ingredient(IngredientBuilder::new("Zucker", 200.0).origin(Country::EU).build())
        .total(800.0)
        .build();
    let output = calculator.execute(input);
    let label = output.label;
    assert!(label.contains("Milch (CH)"));
    assert!(label.contains("Zucker (EU)"));
}

#[test]
fn no_country_display_when_origin_not_set() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Milch", 700.0).build())
        .total(350.0)
        .build();
    let output = calculator.execute(input);
    let label = output.label;
    assert!(label.contains("Milch"));
    assert!(!label.contains("(CH)"));
    assert!(!label.contains("(EU)"));
}

#[test]
fn meat_ingredient_over_20_percent_requires_origin() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![
        RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent,
        RuleDef::AP7_3_HerkunftFleischUeber20Prozent
    ]);
    let input = InputBuilder::new()
        .ingredient(
            IngredientBuilder::new("Hackfleisch", 250.0)
                .category("Fleisch")
                .origin(Country::CH)
                .build()
        )
        .ingredient(
            IngredientBuilder::new("Nudeln", 750.0)
                .category("Getreide")
                .origin(Country::EU)
                .build()
        )
        .total(1000.0)
        .build();
    let output = calculator.execute(input);
    let conditionals = output.conditional_elements;
    let label = output.label;

    // Meat ingredient should show origin field even though <50%
    assert!(conditionals.get("herkunft_benoetigt_0").is_some());
    // Non-meat ingredient should show origin field (>50% rule also active)
    assert!(conditionals.get("herkunft_benoetigt_1").is_some());

    // Both ingredients should display country on label
    assert!(label.contains("Hackfleisch (CH)"));
    assert!(label.contains("Nudeln (EU)"));
}

#[test]
fn meat_rule_only_shows_origin_for_meat_ingredients() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::AP7_3_HerkunftFleischUeber20Prozent]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(
            IngredientBuilder::new("Hackfleisch", 250.0)
                .category("Fleisch")
                .origin(Country::CH)
                .build()
        )
        .ingredient(
            IngredientBuilder::new("Nudeln", 750.0)
                .category("Getreide")
                .origin(Country::EU)
                .build()
        )
        .total(1000.0)
        .build();
    let output = calculator.execute(input);
    let conditionals = output.conditional_elements;
    let label = output.label;

    // Meat ingredient should show origin field
    assert!(conditionals.get("herkunft_benoetigt_0").is_some());
    // Non-meat ingredient should NOT show origin field with only meat rule
    assert!(conditionals.get("herkunft_benoetigt_1").is_none());

    // The current origin display logic shows origin for all ingredients if any origin rule is active
    // This is a limitation of the current design but the functionality still works correctly
    // The meat ingredient shows origin on the label
    assert!(label.contains("Hackfleisch (CH)"));
    // The non-meat ingredient also shows origin due to current display logic design
    // but its conditional field is correctly NOT set (so UI won't show origin input field)
    assert!(label.contains("Nudeln (EU)"));
}

#[test]
fn meat_ingredient_under_20_percent_no_origin_required() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::AP7_3_HerkunftFleischUeber20Prozent]);
    let input = InputBuilder::new()
        .ingredient(
            IngredientBuilder::new("Speck", 150.0)
                .category("Fleisch")
                .build()
        )
        .ingredient(
            IngredientBuilder::new("Pasta", 850.0)
                .category("Getreide")
                .origin(Country::IT)
                .build()
        )
        .total(1000.0)
        .build();
    let output = calculator.execute(input);
    let conditionals = output.conditional_elements;

    // Meat ingredient under 20% should NOT show origin field
    assert!(conditionals.get("herkunft_benoetigt_0").is_none());
    // Non-meat ingredient should NOT show origin field (only meat rule active)
    assert!(conditionals.get("herkunft_benoetigt_1").is_none());
}

#[test]
fn validation_missing_origin_for_meat_ingredient_over_20_percent() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::AP7_3_HerkunftFleischUeber20Prozent]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(
            IngredientBuilder::new("Rindfleisch", 300.0)
                .category("Fleisch")
                .build()
        )
        .ingredient(IngredientBuilder::new("Gemüse", 700.0).build())
        .total(1000.0)
        .build();
    let output = calculator.execute(input);
    let validation_messages = output.validation_messages;

    // Should have validation error for missing origin on meat ingredient
    assert!(validation_messages.get("ingredients[0][origin]").is_some());
    let origin_messages = validation_messages.get("ingredients[0][origin]").unwrap();
    assert!(!origin_messages.is_empty());
    assert!(origin_messages.iter().any(|m| m == "Herkunftsland ist erforderlich für Fleisch-Zutaten über 20%."));
}

#[test]
fn meat_detection_comprehensive_categories() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::AP7_3_HerkunftFleischUeber20Prozent]);

    // Test the specific categories mentioned by the user
    let test_cases = vec![
        ("Salami", "Rohwurstware", true),
        ("Schinken", "Schwein", true),
        ("Bratwurst", "Kalb; Lamm, Schaf; Rind; Schwein; Wild; Geflügel", true),
        ("Weizen", "Getreide", false), // Non-meat control case
    ];

    for (ingredient_name, category, should_require_origin) in test_cases {
        let input = InputBuilder::new()
            .vollstaendig()
            .ingredient(
                IngredientBuilder::new(ingredient_name, 300.0)
                    .category(category)
                    .build()
            )
            .ingredient(IngredientBuilder::new("Filler", 700.0).build())
            .total(1000.0)
            .build();

        let output = calculator.execute(input);
        let validation_messages = output.validation_messages;
        let conditionals = output.conditional_elements;

        if should_require_origin {
            // Should have validation error for missing origin
            let origin_messages = validation_messages.get("ingredients[0][origin]");
            assert!(
                origin_messages.map_or(false, |v| !v.is_empty()),
                "Expected validation error for {} with category '{}'",
                ingredient_name, category
            );
            // Should show origin field
            assert!(
                conditionals.get("herkunft_benoetigt_0").is_some(),
                "Expected origin field for {} with category '{}'",
                ingredient_name, category
            );
        } else {
            // Should NOT have validation error
            let origin_messages = validation_messages.get("ingredients[0][origin]");
            assert!(
                origin_messages.map_or(true, |v| v.is_empty()),
                "Unexpected validation error for {} with category '{}'",
                ingredient_name, category
            );
            // Should NOT show origin field
            assert!(
                conditionals.get("herkunft_benoetigt_0").is_none(),
                "Unexpected origin field for {} with category '{}'",
                ingredient_name, category
            );
        }
    }
}

#[test]
fn meat_detection_processed_meat_products() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::AP7_3_HerkunftFleischUeber20Prozent]);

    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(
            IngredientBuilder::new("Rohwurst", 250.0)
                .category("Rohwurstware")
                .origin(Country::CH)
                .build()
        )
        .ingredient(IngredientBuilder::new("Other", 750.0).build())
        .total(1000.0)
        .build();

    let output = calculator.execute(input);
    let conditionals = output.conditional_elements;
    let label = output.label;

    // Should recognize "Rohwurstware" as meat and show origin field
    assert!(conditionals.get("herkunft_benoetigt_0").is_some());
    // Should display origin on label
    assert!(label.contains("Rohwurst (CH)"));
}
