use super::*;
use crate::conditional_keys as keys;

#[test]
fn amount_lt_zero_invalid() {
    let calculator = calculator_with(vec![RuleDef::AP1_1_ZutatMengeValidierung]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new("Hafer", 0.0).build())
        .build();
    let output = calculator.execute(input);
    let validation_messages = output.validation_messages;
    assert!(validation_messages.contains_key("ingredients[0][amount]"));
    let amount_messages = validation_messages.get("ingredients[0][amount]").unwrap();
    assert!(!amount_messages.is_empty());
    assert!(amount_messages
        .iter()
        .any(|m| m == "Die Menge muss grösser als 0 sein."));
}

#[test]
fn amount_gt_zero_valid() {
    let calculator = calculator_with(vec![RuleDef::AP1_1_ZutatMengeValidierung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Hafer", 32.0).build())
        .build();
    let output = calculator.execute(input);
    let validation_messages = output.validation_messages;
    assert!(validation_messages
        .get("ingredients[0][amount]")
        .is_none_or(|v| v.is_empty()));
}

#[test]
fn multiple_validation_errors_on_single_ingredient() {
    let calculator = calculator_with(vec![
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
                .build(),
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
    let aufzucht_messages = validation_messages
        .get("ingredients[0][aufzucht_ort]")
        .unwrap();
    let schlachtungs_messages = validation_messages
        .get("ingredients[0][schlachtungs_ort]")
        .unwrap();
    let amount_messages = validation_messages.get("ingredients[1][amount]").unwrap();

    // Should contain multiple origin messages for different rules
    assert!(origin_messages
        .iter()
        .any(|m| m == "Herkunftsland ist erforderlich für Zutaten über 50%."));
    assert!(origin_messages
        .iter()
        .any(|m| m == "Herkunftsland ist erforderlich für Fleisch-Zutaten über 20%."));
    assert!(aufzucht_messages
        .iter()
        .any(|m| m == "Aufzuchtort ist erforderlich für Rindfleisch-Zutaten."));
    assert!(schlachtungs_messages
        .iter()
        .any(|m| m == "Schlachtungsort ist erforderlich für Rindfleisch-Zutaten."));
    assert!(amount_messages
        .iter()
        .any(|m| m == "Die Menge muss grösser als 0 sein."));

    // Count total messages across all fields
    let total_messages: usize = validation_messages.values().map(|v| v.len()).sum();
    println!(
        "Multiple validation errors successfully captured: {} fields with {} total messages",
        validation_messages.len(),
        total_messages
    );
}

#[test]
fn stacked_validation_messages_demo() {
    let calculator = calculator_with(vec![
        RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent,
        RuleDef::AP7_3_HerkunftFleischUeber20Prozent,
    ]);

    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(
            IngredientBuilder::new("Rindfleisch", 600.0)
                .category("Rind")
                .build(),
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
    assert_eq!(
        origin_messages.len(),
        2,
        "Should have exactly 2 validation messages for origin field"
    );
    assert!(origin_messages
        .iter()
        .any(|m| m == "Herkunftsland ist erforderlich für Zutaten über 50%."));
    assert!(origin_messages
        .iter()
        .any(|m| m == "Herkunftsland ist erforderlich für Fleisch-Zutaten über 20%."));

    println!("✅ Successfully demonstrated stacked validation messages!");
}

// --- LMR (LIV Art. 16 / AP7) boundaries --------------------------------------
// AP7.1 requires the origin of an ingredient with MORE than 50 %, AP7.3 of meat
// with MORE than 20 %. Exactly at the threshold nothing is required; this pins
// the strict comparison so it cannot silently flip to ">=" (which the Knospe
// rules use on purpose).

#[test]
fn lmr_origin_not_required_at_exactly_50_percent() {
    let calculator = calculator_with(vec![RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Hafer", 500.0).build())
        .ingredient(IngredientBuilder::new_agri("Zucker", 500.0).build())
        .build();
    let output = calculator.execute(input);
    assert!(
        output.validation_messages.is_empty(),
        "50% is not >50%: {:?}",
        output.validation_messages
    );
    assert!(!output
        .conditionals()
        .contains_key(&keys::herkunft_benoetigt(0)));
}

#[test]
fn lmr_origin_required_just_over_50_percent() {
    let calculator = calculator_with(vec![RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Hafer", 501.0).build())
        .ingredient(IngredientBuilder::new_agri("Zucker", 499.0).build())
        .build();
    let output = calculator.execute(input);
    assert!(output
        .validation_messages
        .contains_key("ingredients[0][origin]"));
    assert!(!output
        .validation_messages
        .contains_key("ingredients[1][origin]"));
    assert!(output
        .conditionals()
        .contains_key(&keys::herkunft_benoetigt(0)));
}

#[test]
fn lmr_meat_origin_not_required_at_exactly_20_percent() {
    let calculator = calculator_with(vec![RuleDef::AP7_3_HerkunftFleischUeber20Prozent]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new("Nudeln", 800.0).build())
        .ingredient(
            IngredientBuilder::new("Hackfleisch", 200.0)
                .category("Fleisch")
                .build(),
        )
        .build();
    let output = calculator.execute(input);
    assert!(
        !output
            .validation_messages
            .contains_key("ingredients[1][origin]"),
        "20% is not >20%: {:?}",
        output.validation_messages
    );
}

#[test]
fn lmr_meat_origin_required_just_over_20_percent() {
    let calculator = calculator_with(vec![RuleDef::AP7_3_HerkunftFleischUeber20Prozent]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new("Nudeln", 799.0).build())
        .ingredient(
            IngredientBuilder::new("Hackfleisch", 201.0)
                .category("Fleisch")
                .build(),
        )
        .build();
    let output = calculator.execute(input);
    assert!(output
        .validation_messages
        .contains_key("ingredients[1][origin]"));
}

// B1/K3 (11.11.2026): a composite whose entered amount exceeds the sum of its
// (partially) weighted children silently shrinks the total — warn instead.
#[test]
fn partially_weighted_composite_warns_about_uncovered_parent_amount() {
    let calculator = setup_simple_calculator();
    let output = calculator.execute(
        InputBuilder::new()
            .ingredient(
                IngredientBuilder::new("Guetzlimischung", 525.0)
                    .children(vec![
                        IngredientBuilder::new("Mailänderli", 100.0).build(),
                        IngredientBuilder::new("Kokosmakronen", 0.0).build(),
                    ])
                    .build(),
            )
            .vollstaendig()
            .build(),
    );
    let msgs = output
        .validation_messages
        .get("ingredients[0][amount]")
        .expect("a warning on the composite's amount");
    assert!(
        msgs.iter().any(|m| m.contains("525") && m.contains("100")),
        "the warning names parent and child sums: {msgs:?}"
    );
}

// No warning when all children are weighted (normal bottom-up composite) or
// none are (top-down weight from the parent, the supported pattern).
#[test]
fn fully_weighted_or_fully_weightless_composites_do_not_warn() {
    let calculator = setup_simple_calculator();
    for children in [
        vec![
            IngredientBuilder::new("A", 300.0).build(),
            IngredientBuilder::new("B", 225.0).build(),
        ],
        vec![
            IngredientBuilder::new("A", 0.0).build(),
            IngredientBuilder::new("B", 0.0).build(),
        ],
    ] {
        let output = calculator.execute(
            InputBuilder::new()
                .ingredient(
                    IngredientBuilder::new("Mix", 525.0)
                        .children(children)
                        .build(),
                )
                .vollstaendig()
                .build(),
        );
        assert!(
            !output
                .validation_messages
                .contains_key("ingredients[0][amount]"),
            "no warning expected: {:?}",
            output.validation_messages
        );
    }
}

// B3 (11.11.2026): an origin message on a composite also says that the
// origin can go on the composite OR on its sub-ingredients (single level).
#[test]
fn origin_message_on_composite_names_both_levels() {
    let calculator = calculator_for(crate::shared::Configuration::Knospe);
    let output = calculator.execute(
        InputBuilder::new()
            .ingredient(
                IngredientBuilder::new_agri("Teig", 100.0)
                    .bio()
                    .children(vec![
                        IngredientBuilder::new_agri("Mehl", 80.0).build(),
                        IngredientBuilder::new_agri("Zucker", 20.0).build(),
                    ])
                    .build(),
            )
            .vollstaendig()
            .build(),
    );
    let msgs = output
        .validation_messages
        .get("ingredients[0][origin]")
        .expect("origin required on the composite");
    assert!(
        msgs.iter()
            .any(|m| m.contains("Unterzutaten") && m.contains("Teig")),
        "level hint expected: {msgs:?}"
    );
}

// Leaves keep the plain message, no composite hint.
#[test]
fn origin_message_on_leaf_has_no_level_hint() {
    let calculator = calculator_for(crate::shared::Configuration::Knospe);
    let output = calculator.execute(
        InputBuilder::new()
            .ingredient(IngredientBuilder::new_agri("Mehl", 100.0).bio().build())
            .vollstaendig()
            .build(),
    );
    let msgs = output
        .validation_messages
        .get("ingredients[0][origin]")
        .expect("origin required");
    assert!(!msgs.iter().any(|m| m.contains("Unterzutaten")), "{msgs:?}");
}
