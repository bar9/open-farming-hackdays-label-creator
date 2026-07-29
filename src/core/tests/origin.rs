use crate::conditional_keys as keys;
use super::*;

#[test]
fn ap7_1_herkunft_benoetigt_ueber_50_prozent() {
    let calculator = calculator_with(vec![RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Milch", 700.0).build())
        .total(350.0)
        .build();
    let output = calculator.execute(input);
    let conditionals = output.conditionals();
    assert!(conditionals.contains_key(keys::HERKUNFT_BENOETIGT_UEBER_50_PROZENT));
    assert!(
        *conditionals
            .get(keys::HERKUNFT_BENOETIGT_UEBER_50_PROZENT)
            .unwrap()
    );
}

#[test]
fn herkunft_benoetigt_composite_children_over_50_percent() {
    // A composite whose weight lives in its children (parent amount 0) and which
    // aggregates to >50% of the product must trigger the "Herkunft benötigt" flag.
    // Before the computed_amount fix, line 1558 read the raw parent amount (0), so
    // the percentage collapsed to 0% and the flag never fired for composites.
    let calculator = calculator_with(vec![RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent]);
    let input = InputBuilder::new()
        .ingredient(
            IngredientBuilder::new("Fruchtmischung", 0.0)
                .children(vec![
                    IngredientBuilder::new("Himbeere", 400.0).build(),
                    IngredientBuilder::new("Erdbeere", 200.0).build(),
                ])
                .build(),
        )
        .ingredient(IngredientBuilder::new("Zucker", 200.0).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();
    // Composite = 600 / 800 = 75% > 50% → its origin is required (top-level index 0).
    assert_eq!(c.get(&keys::herkunft_benoetigt(0)), Some(&true));
    assert_eq!(c.get(keys::HERKUNFT_BENOETIGT_UEBER_50_PROZENT), Some(&true));
}

#[test]
fn herkunft_benoetigt_composite_under_50_percent_not_required() {
    // Guard against a raw-amount regression: a composite aggregating to <50% must NOT
    // be flagged, while a genuine >50% sibling still is.
    let calculator = calculator_with(vec![RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent]);
    let input = InputBuilder::new()
        .ingredient(
            IngredientBuilder::new("Fruchtmischung", 0.0)
                .children(vec![
                    IngredientBuilder::new("Himbeere", 200.0).build(),
                    IngredientBuilder::new("Erdbeere", 100.0).build(),
                ])
                .build(),
        )
        .ingredient(IngredientBuilder::new("Zucker", 500.0).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();
    // Composite = 300 / 800 = 37.5% < 50% → NOT required.
    assert_eq!(c.get(&keys::herkunft_benoetigt(0)), None);
    // Zucker = 500 / 800 = 62.5% > 50% → still required (rule fires for real >50%).
    assert_eq!(c.get(&keys::herkunft_benoetigt(1)), Some(&true));
}

#[test]
fn validation_missing_origin_for_ingredient_over_50_percent() {
    let calculator = calculator_with(vec![RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new("Milch", 700.0).build())
        .total(350.0)
        .build();
    let output = calculator.execute(input);
    let validation_messages = output.validation_messages;
    assert!(validation_messages.contains_key("ingredients[0][origin]"));
    let origin_messages = validation_messages.get("ingredients[0][origin]").unwrap();
    assert!(!origin_messages.is_empty());
    assert!(origin_messages.iter().any(|m| m == "Herkunftsland ist erforderlich für Zutaten über 50%."));
}

#[test]
fn origin_over_50_percent_skips_non_agricultural() {
    // A non-agricultural ingredient above 50% weight must NOT require an origin (validate_origin
    // previously ignored is_agricultural entirely).
    let calculator = calculator_with(vec![RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new("Dicarbonat", 700.0).agricultural(false).build())
        .ingredient(IngredientBuilder::new_agri("Hafer", 300.0).origin(Country::CH).build())
        .build();
    let output = calculator.execute(input);
    // Dicarbonat = 700/1000 = 70% > 50% but non-agricultural → no origin required.
    assert!(!output.validation_messages.contains_key("ingredients[0][origin]"),
        "non-agricultural >50% must not require origin, got: {:?}", output.validation_messages);
}

#[test]
fn dicarbonat_from_db_is_non_agricultural() {
    // Dicarbonat is now in food_db as non-agricultural → auto-detected via name lookup, so it
    // never requires an origin even without the user explicitly marking it.
    let calculator = calculator_with(vec![RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Dicarbonat", 700.0).build())
        .ingredient(IngredientBuilder::new_agri("Hafer", 300.0).origin(Country::CH).build())
        .build();
    let output = calculator.execute(input);
    // new_agri("Dicarbonat") looks up food_db → is_agricultural=false → no origin required at 70%.
    assert!(!output.validation_messages.contains_key("ingredients[0][origin]"),
        "Dicarbonat (food_db non-agricultural) must not require origin, got: {:?}", output.validation_messages);
}

#[test]
fn import_placeholder_never_printed_verbatim() {
    let calculator = calculator_with(vec![RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent]);
    let input = InputBuilder::new()
        // CH + generic Import placeholder: CH must print, Import must be dropped.
        .ingredient(IngredientBuilder::new("Milch", 600.0).origins(vec![Country::CH, Country::Import]).build())
        // Import-only: no origin should be printed at all (not "(Import)").
        .ingredient(IngredientBuilder::new("Zucker", 200.0).origin(Country::Import).build())
        .total(800.0)
        .build();
    let output = calculator.execute(input);
    let label = output.label;
    assert!(!label.contains("Import"), "literal 'Import' must never appear on the label. Label: {}", label);
    assert!(label.contains("Milch (CH)"), "CH origin should still print alongside a dropped Import. Label: {}", label);
    assert!(!label.contains("Zucker ("), "Import-only ingredient should show no origin. Label: {}", label);
}

#[test]
fn country_display_on_label_for_ingredients_with_origin() {
    let calculator = calculator_with(vec![RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent]);
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

// Regression (Testing 25.06.2026, Bio Verordnung): an origin declared top-down on
// the composite parent was silently dropped by the composite early-return in
// OutputFormatter::format — selected Herkunft never reached the label.
#[test]
fn composite_parent_declared_origin_shows_on_label() {
    let calculator = calculator_with(vec![
        RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent,
        RuleDef::AP2_1_ZusammegesetztOutput,
    ]);
    let input = InputBuilder::new()
        .ingredient(
            IngredientBuilder::new("Himbeerstreusel", 600.0)
                .origin(Country::CH)
                .children(vec![
                    IngredientBuilder::new("Himbeere", 0.0).build(),
                    IngredientBuilder::new("Zucker", 0.0).build(),
                ])
                .build(),
        )
        .ingredient(IngredientBuilder::new("Haferflocken", 400.0).origin(Country::AT).build())
        .total(1000.0)
        .build();
    let output = calculator.execute(input);
    let label = output.label;
    assert!(
        label.contains("Himbeerstreusel (Himbeere, Zucker) (CH)"),
        "parent-declared origin must print after the composite list. Label: {}",
        label
    );
    assert!(label.contains("Haferflocken (AT)"));
}

// Composite without a parent-declared origin keeps the lowest-level-only display.
#[test]
fn composite_parent_without_declared_origin_shows_none() {
    let calculator = calculator_with(vec![
        RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent,
        RuleDef::AP2_1_ZusammegesetztOutput,
    ]);
    let input = InputBuilder::new()
        .ingredient(
            IngredientBuilder::new("Himbeerstreusel", 600.0)
                .children(vec![
                    IngredientBuilder::new("Himbeere", 0.0).origin(Country::CH).build(),
                    IngredientBuilder::new("Zucker", 0.0).build(),
                ])
                .build(),
        )
        .ingredient(IngredientBuilder::new("Haferflocken", 400.0).build())
        .total(1000.0)
        .build();
    let output = calculator.execute(input);
    let label = output.label;
    assert!(
        label.contains("Himbeere (CH)"),
        "child origin still prints at the lowest level. Label: {}",
        label
    );
    assert!(
        !label.contains("Zucker) (CH)") && !label.contains("Himbeerstreusel (CH)"),
        "parent must not show an origin it never declared. Label: {}",
        label
    );
}

#[test]
fn no_country_display_when_origin_not_set() {
    let calculator = calculator_with(vec![RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent]);
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
    let calculator = calculator_with(vec![
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
    let conditionals = output.conditionals();
    let label = output.label;

    // Meat ingredient should show origin field even though <50%
    assert!(conditionals.contains_key(&keys::herkunft_benoetigt(0)));
    // Non-meat ingredient should show origin field (>50% rule also active)
    assert!(conditionals.contains_key(&keys::herkunft_benoetigt(1)));

    // Both ingredients should display country on label
    assert!(label.contains("Hackfleisch (CH)"));
    assert!(label.contains("Nudeln (EU)"));
}

#[test]
fn meat_rule_only_shows_origin_for_meat_ingredients() {
    let calculator = calculator_with(vec![RuleDef::AP7_3_HerkunftFleischUeber20Prozent]);
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
    let conditionals = output.conditionals();
    let label = output.label;

    // Meat ingredient should show origin field
    assert!(conditionals.contains_key(&keys::herkunft_benoetigt(0)));
    // Non-meat ingredient should NOT show origin field with only meat rule
    assert!(!conditionals.contains_key(&keys::herkunft_benoetigt(1)));

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
    let calculator = calculator_with(vec![RuleDef::AP7_3_HerkunftFleischUeber20Prozent]);
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
    let conditionals = output.conditionals();

    // Meat ingredient under 20% should NOT show origin field
    assert!(!conditionals.contains_key(&keys::herkunft_benoetigt(0)));
    // Non-meat ingredient should NOT show origin field (only meat rule active)
    assert!(!conditionals.contains_key(&keys::herkunft_benoetigt(1)));
}

#[test]
fn validation_missing_origin_for_meat_ingredient_over_20_percent() {
    let calculator = calculator_with(vec![RuleDef::AP7_3_HerkunftFleischUeber20Prozent]);
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
    assert!(validation_messages.contains_key("ingredients[0][origin]"));
    let origin_messages = validation_messages.get("ingredients[0][origin]").unwrap();
    assert!(!origin_messages.is_empty());
    assert!(origin_messages.iter().any(|m| m == "Herkunftsland ist erforderlich für Fleisch-Zutaten über 20%."));
}

#[test]
fn meat_detection_comprehensive_categories() {
    let calculator = calculator_with(vec![RuleDef::AP7_3_HerkunftFleischUeber20Prozent]);

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
        let conditionals = output.conditionals();
        let validation_messages = output.validation_messages;

        if should_require_origin {
            // Should have validation error for missing origin
            let origin_messages = validation_messages.get("ingredients[0][origin]");
            assert!(
                origin_messages.is_some_and(|v| !v.is_empty()),
                "Expected validation error for {} with category '{}'",
                ingredient_name, category
            );
            // Should show origin field
            assert!(
                conditionals.contains_key(&keys::herkunft_benoetigt(0)),
                "Expected origin field for {} with category '{}'",
                ingredient_name, category
            );
        } else {
            // Should NOT have validation error
            let origin_messages = validation_messages.get("ingredients[0][origin]");
            assert!(
                origin_messages.is_none_or(|v| v.is_empty()),
                "Unexpected validation error for {} with category '{}'",
                ingredient_name, category
            );
            // Should NOT show origin field
            assert!(
                !conditionals.contains_key(&keys::herkunft_benoetigt(0)),
                "Unexpected origin field for {} with category '{}'",
                ingredient_name, category
            );
        }
    }
}

#[test]
fn meat_detection_processed_meat_products() {
    let calculator = calculator_with(vec![RuleDef::AP7_3_HerkunftFleischUeber20Prozent]);

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
    let conditionals = output.conditionals();
    let label = output.label;

    // Should recognize "Rohwurstware" as meat and show origin field
    assert!(conditionals.contains_key(&keys::herkunft_benoetigt(0)));
    // Should display origin on label
    assert!(label.contains("Rohwurst (CH)"));
}

#[test]
fn single_non_ch_non_eu_country_displays_on_label() {
    // L3: Individual countries like AT, FR, AL should display in the ingredients list
    let calculator = calculator_with(vec![RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent]);

    let test_cases = vec![
        (Country::AT, "AT"),
        (Country::FR, "FR"),
        (Country::AL, "AL"),
        (Country::DO, "DO"),
    ];

    for (country, expected_code) in test_cases {
        let input = InputBuilder::new()
            .ingredient(IngredientBuilder::new("Mehl", 800.0).origin(country).build())
            .ingredient(IngredientBuilder::new("Salz", 200.0).build())
            .total(1000.0)
            .build();
        let output = calculator.execute(input);
        assert!(
            output.label.contains(&format!("Mehl ({})", expected_code)),
            "Expected 'Mehl ({})' in label, got: {}",
            expected_code, output.label
        );
    }
}
