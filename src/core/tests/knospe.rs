use super::*;

#[test]
fn bio_knospe_alle_zutaten_herkunft_conditional() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Knospe_AlleZutatenHerkunft]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new("Milch", 300.0).build())
        .ingredient(IngredientBuilder::new("Zucker", 200.0).build())
        .total(1000.0)
        .build();
    let output = calculator.execute(input);
    let conditionals = output.conditional_elements;

    // All ingredients should require herkunft
    assert_eq!(conditionals.get("herkunft_benoetigt_0"), Some(&true));
    assert_eq!(conditionals.get("herkunft_benoetigt_1"), Some(&true));
    assert_eq!(conditionals.get("herkunft_benoetigt_ueber_50_prozent"), Some(&true));
}

#[test]
fn bio_knospe_validation_missing_origin_for_all_ingredients() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Knospe_AlleZutatenHerkunft]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new("Milch", 300.0).origin(Country::CH).build())
        .ingredient(IngredientBuilder::new("Zucker", 200.0).build())
        .total(1000.0)
        .build();
    let output = calculator.execute(input);

    // Should have validation error for the ingredient without origin
    let ingredient_1_messages = output.validation_messages.get("ingredients[1][origin]");
    assert!(ingredient_1_messages
               .map_or(false, |v| v.iter().any(|m| m == "Herkunftsland ist erforderlich für alle Zutaten (Knospe Anforderung).")));
    // Should NOT have validation error for the ingredient with origin
    assert!(output.validation_messages.get("ingredients[0][origin]").map_or(true, |v| v.is_empty()));
}

#[test]
fn bio_knospe_country_display_on_label_for_all_ingredients() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Knospe_AlleZutatenHerkunft]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Milch", 300.0).origin(Country::CH).build())
        .ingredient(IngredientBuilder::new("Zucker", 200.0).origin(Country::EU).build())
        .total(1000.0)
        .build();
    let output = calculator.execute(input);
    let label = output.label;

    // All ingredients should display country on label
    assert!(label.contains("Milch (CH)"));
    assert!(label.contains("Zucker (EU)"));
}

#[test]
fn bio_knospe_validation_all_ingredients_missing_origin() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Knospe_AlleZutatenHerkunft]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new("Milch", 300.0).build())
        .ingredient(IngredientBuilder::new("Zucker", 200.0).build())
        .total(1000.0)
        .build();
    let output = calculator.execute(input);

    // Should have validation errors for all ingredients
    let origin_messages_0 = output.validation_messages.get("ingredients[0][origin]").unwrap();
    let origin_messages_1 = output.validation_messages.get("ingredients[1][origin]").unwrap();
    assert!(origin_messages_0.iter().any(|m| m == "Herkunftsland ist erforderlich für alle Zutaten (Knospe Anforderung)."));
    assert!(origin_messages_1.iter().any(|m| m == "Herkunftsland ist erforderlich für alle Zutaten (Knospe Anforderung)."));
}

#[test]
fn bio_knospe_no_validation_errors_when_all_have_origin() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Knospe_AlleZutatenHerkunft]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Milch", 300.0).origin(Country::CH).build())
        .ingredient(IngredientBuilder::new("Zucker", 200.0).origin(Country::EU).build())
        .total(1000.0)
        .build();
    let output = calculator.execute(input);

    // Should have no validation errors
    assert!(output.validation_messages.is_empty());
}

#[test]
fn knospe_100_percent_ch_no_origin_display() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![
        RuleDef::Knospe_100_Percent_CH_NoOrigin,
        RuleDef::Knospe_90_99_Percent_CH_ShowOrigin,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).origin(Country::CH).build())
        .build();
    let output = calculator.execute(input);
    let label = output.label;

    // With 100% Swiss agricultural ingredients, no origin should be displayed
    assert!(!label.contains("(Schweiz)"));
    assert!(!label.contains("(CH)"));
    assert!(label.contains("Hafer, Weizenmehl"));
}

#[test]
fn knospe_90_99_percent_ch_show_origin_for_swiss() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![
        RuleDef::Knospe_100_Percent_CH_NoOrigin,
        RuleDef::Knospe_90_99_Percent_CH_ShowOrigin,
    ]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Hafer", 500.0).origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Olivenöl", 100.0).origin(Country::EU).build())
        .build();
    let output = calculator.execute(input);
    let label = output.label;

    // With 90% Swiss agricultural ingredients, only Swiss ingredients should show origin
    assert!(label.contains("Hafer (CH)"));
    assert!(label.contains("Weizenmehl (CH)"));
    assert!(!label.contains("Olivenöl (EU)"));
    assert!(label.contains("Olivenöl"));
}

#[test]
fn knospe_under_90_percent_ch_no_special_rules() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![
        RuleDef::Knospe_100_Percent_CH_NoOrigin,
        RuleDef::Knospe_90_99_Percent_CH_ShowOrigin,
    ]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Hafer", 400.0).origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Olivenöl", 600.0).origin(Country::EU).build())
        .build();
    let output = calculator.execute(input);
    let label = output.label;

    // With less than 90% Swiss agricultural ingredients, no special Knospe rules apply
    assert!(!label.contains("(CH)"));
    assert!(!label.contains("(EU)"));
    assert!(label.contains("Olivenöl, Hafer"));
}

#[test]
fn knospe_under_90_percent_ch_namensgebende_always_shows_origin() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![
        RuleDef::Knospe_Under90_Percent_CH_IngredientRules,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 400.0).origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Olivenöl", 600.0).origin(Country::EU).namensgebend().build())
        .build();
    let output = calculator.execute(input);
    let label = output.label;

    // With <90% Swiss agricultural ingredients and name-giving ingredient,
    // the name-giving ingredient should show its origin
    assert!(label.contains("(EU)")); // Olivenöl should show origin (name-giving)
    assert!(label.contains("(CH)")); // Hafer also shows origin (Swiss ingredient >=10%)
    assert!(label.contains("Olivenöl (EU), Hafer (CH)")); // Both should show origin
}

#[test]
fn knospe_under_90_percent_ch_namensgebende_ingredient_low_percentage_shows_origin() {
    // This test demonstrates that name-giving ingredients show origin even with low percentage
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![
        RuleDef::Knospe_Under90_Percent_CH_IngredientRules,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 900.0).origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Vanilla", 100.0).origin(Country::EU).namensgebend().build())
        .build();
    let output = calculator.execute(input);
    let label = output.label;

    // With <90% Swiss agricultural ingredients:
    // - Hafer should show origin (Swiss ingredient >=10%)
    // - Vanilla should show origin even though it's only 10% (name-giving ingredient)
    assert!(label.contains("(CH)")); // Hafer shows origin (Swiss >=10%)
    assert!(label.contains("(EU)")); // Vanilla shows origin (name-giving)
    assert!(label.contains("Hafer (CH), Vanilla (EU)")); // Ordered by weight
}

#[test]
fn knospe_under_90_validation_eggs_over_10_percent() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Hafer", 850.0).origin(Country::EU).build())
        .ingredient(
            IngredientBuilder::new_agri("Eier", 150.0)
                .category("Eier")
                .build()
        )
        .build();
    let output = calculator.execute(input);

    // Should have validation error for eggs >10%
    let egg_messages = output.validation_messages.get("ingredients[1][origin]");
    assert!(egg_messages.is_some());
    let messages = egg_messages.unwrap();
    assert!(messages.iter().any(|msg| msg.contains("Eier/Honig/Fisch >10%")));
}

#[test]
fn knospe_under_90_validation_honey_over_10_percent() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Hafer", 850.0).origin(Country::EU).build())
        .ingredient(
            IngredientBuilder::new_agri("Honig", 150.0)
                .category("Honig")
                .build()
        )
        .build();
    let output = calculator.execute(input);

    // Should have validation error for honey >10%
    let honey_messages = output.validation_messages.get("ingredients[1][origin]");
    assert!(honey_messages.is_some());
    let messages = honey_messages.unwrap();
    assert!(messages.iter().any(|msg| msg.contains("Eier/Honig/Fisch >10%")));
}

#[test]
fn knospe_under_90_validation_dairy_always_requires_origin() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Hafer", 950.0).origin(Country::EU).build())
        .ingredient(
            IngredientBuilder::new_agri("Milch", 50.0)
                .category("Milch")
                .build()
        )
        .build();
    let output = calculator.execute(input);

    // Should have validation error for dairy even at low percentage
    let milk_messages = output.validation_messages.get("ingredients[1][origin]");
    assert!(milk_messages.is_some());
    let messages = milk_messages.unwrap();
    assert!(messages.iter().any(|msg| msg == "Herkunftsland ist erforderlich für Milch/Fleisch/Insekten (Knospe <90% CH Regel)."));
}

#[test]
fn knospe_under_90_validation_meat_always_requires_origin() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Hafer", 970.0).origin(Country::EU).build())
        .ingredient(
            IngredientBuilder::new_agri("Fleisch", 30.0)
                .category("Fleisch")
                .build()
        )
        .build();
    let output = calculator.execute(input);

    // Should have validation error for meat even at low percentage
    let meat_messages = output.validation_messages.get("ingredients[1][origin]");
    assert!(meat_messages.is_some());
    let messages = meat_messages.unwrap();
    assert!(messages.iter().any(|msg| msg == "Herkunftsland ist erforderlich für Milch/Fleisch/Insekten (Knospe <90% CH Regel)."));
}

#[test]
fn knospe_under_90_validation_plant_over_50_percent() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(
            IngredientBuilder::new_agri("Weizen", 600.0)
                .category("Getreide")
                .build()
        )
        .ingredient(IngredientBuilder::new_agri("Zucker", 400.0).origin(Country::EU).build())
        .build();
    let output = calculator.execute(input);

    // Should have validation error for plant ingredient >50%
    let wheat_messages = output.validation_messages.get("ingredients[0][origin]");
    assert!(wheat_messages.is_some());
    let messages = wheat_messages.unwrap();
    assert!(messages.iter().any(|msg| msg == "Herkunftsland ist erforderlich für pflanzliche Zutaten >50% (Knospe <90% CH Regel)."));
}

#[test]
fn knospe_under_90_validation_monoproduct() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Olivenöl", 1000.0).build())
        .build();
    let output = calculator.execute(input);

    // Should have validation error for monoproduct
    let oil_messages = output.validation_messages.get("ingredients[0][origin]");
    assert!(oil_messages.is_some());
    let messages = oil_messages.unwrap();
    assert!(messages.iter().any(|msg| msg == "Herkunftsland ist erforderlich für Monoprodukte (Knospe <90% CH Regel)."));
}
