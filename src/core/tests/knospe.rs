use crate::conditional_keys as keys;
use super::*;

// Reworked per Testing 25.06.2026: the «alle Zutaten Herkunft» rule now flags an
// ingredient only when it carries the Import-(Umstellungs-)Knospe without a real
// country AND the label output is the Import-Knospe.
#[test]
fn bio_knospe_alle_zutaten_herkunft_conditional() {
    let calculator = calculator_with(vec![RuleDef::Knospe_AlleZutatenHerkunft]);
    let input = InputBuilder::new()
        .vollstaendig()
        // Import-Knospe without a country → origin required
        .ingredient(IngredientBuilder::new_agri("Rohrzucker", 700.0).bio().origin(Country::Import).build())
        // CH-Knospe → nothing required
        .ingredient(IngredientBuilder::new_agri("Hafer", 300.0).bio().origin(Country::CH).build())
        .total(1000.0)
        .build();
    let output = calculator.execute(input);
    let conditionals = output.conditional_elements;

    assert_eq!(conditionals.get(&keys::herkunft_benoetigt(0)), Some(&true));
    assert_eq!(conditionals.get(&keys::herkunft_benoetigt(1)), None);
    assert_eq!(conditionals.get(keys::HERKUNFT_BENOETIGT_UEBER_50_PROZENT), Some(&true));
}

#[test]
fn bio_knospe_validation_missing_origin_for_import_knospe() {
    let calculator = calculator_with(vec![RuleDef::Knospe_AlleZutatenHerkunft]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Hafer", 300.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Rohrzucker", 700.0).bio().origin(Country::Import).build())
        .total(1000.0)
        .build();
    let output = calculator.execute(input);

    // The Import-Knospe ingredient without a real country is flagged
    let ingredient_1_messages = output.validation_messages.get("ingredients[1][origin]");
    assert!(ingredient_1_messages
               .is_some_and(|v| v.iter().any(|m| m == "Herkunftsland ist erforderlich für Zutaten mit Import-Knospe, wenn die Import-Knospe auf der Etikette erscheint.")));
    // The CH-Knospe ingredient is not
    assert!(output.validation_messages.get("ingredients[0][origin]").is_none_or(|v| v.is_empty()));
}

#[test]
fn bio_knospe_no_origin_error_when_ch_knospe_logo_shows() {
    // Import-Knospe ingredient without a country, but ≥90% Swiss share → the
    // label shows the CH-Knospe, so no origin error is raised (plan assumption,
    // recorded in the Herkunft-Problemanalyse for Mirjam).
    let calculator = calculator_with(vec![RuleDef::Knospe_AlleZutatenHerkunft]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Hafer", 950.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Rohrzucker", 50.0).bio().origin(Country::Import).build())
        .total(1000.0)
        .build();
    let output = calculator.execute(input);

    assert!(output.validation_messages.is_empty(),
        "no origin error when the CH-Knospe shows, got: {:?}", output.validation_messages);
}

#[test]
fn bio_knospe_non_agricultural_never_requires_origin() {
    // Dicarbonat case (Testing 25.06.2026): non-agricultural ingredients must
    // never be asked for an origin.
    let calculator = calculator_with(vec![RuleDef::Knospe_AlleZutatenHerkunft]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Rohrzucker", 900.0).bio().origin(Country::Import).build())
        .ingredient(IngredientBuilder::new("Dicarbonat", 5.0).agricultural(false).build())
        .total(905.0)
        .build();
    let output = calculator.execute(input);

    assert!(output.validation_messages.contains_key("ingredients[0][origin]"));
    assert!(!output.validation_messages.contains_key("ingredients[1][origin]"),
        "non-agricultural ingredient must not require origin, got: {:?}", output.validation_messages);
}

#[test]
fn knospe_under90_non_agricultural_never_requires_origin_even_when_mono() {
    // Demo regression: with exactly ONE agricultural leaf the product is "mono", and the
    // Monoprodukt short-circuit in should_show_origin_knospe_under90 used to return true for
    // EVERY origin-less ingredient — flagging a non-agricultural additive (Dicarbonat) too,
    // even after the user correctly marked it "Nicht-landwirtschaftlich".
    let calculator = calculator_with(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Hafer", 500.0).origin(Country::EU).build())
        .ingredient(IngredientBuilder::new("Dicarbonat", 5.0).agricultural(false).build())
        .total(505.0)
        .build();
    let output = calculator.execute(input);

    // Hafer is the single agricultural leaf (mono) and carries a real origin → satisfied.
    assert!(!output.validation_messages.contains_key("ingredients[0][origin]"));
    // Dicarbonat is non-agricultural → never flagged, even though the product is mono.
    assert!(!output.validation_messages.contains_key("ingredients[1][origin]"),
        "non-agricultural ingredient must not require origin even in a mono product, got: {:?}",
        output.validation_messages);
}

#[test]
fn bio_knospe_country_display_on_label_for_all_ingredients() {
    let calculator = calculator_with(vec![RuleDef::Knospe_AlleZutatenHerkunft]);
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
fn bio_knospe_no_origin_error_for_plain_ingredients_without_knospe() {
    // Plain (non-Knospe) ingredients without origins are no longer flagged by
    // the reworked rule — only Import-Knospe ingredients are.
    let calculator = calculator_with(vec![RuleDef::Knospe_AlleZutatenHerkunft]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new("Milch", 300.0).build())
        .ingredient(IngredientBuilder::new("Zucker", 200.0).build())
        .total(1000.0)
        .build();
    let output = calculator.execute(input);

    assert!(output.validation_messages.is_empty(),
        "plain ingredients must not require origin under the reworked rule, got: {:?}",
        output.validation_messages);
}

#[test]
fn bio_knospe_no_validation_errors_when_all_have_origin() {
    let calculator = calculator_with(vec![RuleDef::Knospe_AlleZutatenHerkunft]);
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
    let calculator = calculator_with(vec![
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
    let calculator = calculator_with(vec![
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
    let calculator = calculator_with(vec![
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
    let calculator = calculator_with(vec![
        RuleDef::Knospe_Under90_Percent_CH_IngredientRules,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 400.0).origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Olivenöl", 600.0).origin(Country::EU).namensgebend().build())
        .build();
    let output = calculator.execute(input);
    let label = output.label;

    // Name-giving ingredient should show its origin
    assert!(label.contains("(EU)")); // Olivenöl should show origin (name-giving)
    // Hafer (Swiss, 40% ≥10%) shows origin under Swiss agricultural ≥10% rule
    assert!(label.contains("(CH)")); // Hafer shows origin (Swiss ≥10%)
    assert!(label.contains("Olivenöl (EU), Hafer (CH)"));
}

#[test]
fn knospe_under_90_percent_ch_namensgebende_ingredient_low_percentage_shows_origin() {
    // This test demonstrates that name-giving ingredients show origin even with low percentage
    let calculator = calculator_with(vec![
        RuleDef::Knospe_Under90_Percent_CH_IngredientRules,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 900.0).origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Vanilla", 100.0).origin(Country::EU).namensgebend().build())
        .build();
    let output = calculator.execute(input);
    let label = output.label;

    // Vanilla should show origin (name-giving ingredient) even at only 10%
    assert!(label.contains("(EU)")); // Vanilla shows origin (name-giving)
    // Hafer (Swiss, 90% ≥10%) shows origin under Swiss agricultural ≥10% rule
    assert!(label.contains("(CH)")); // Hafer shows origin (Swiss ≥10%)
    assert!(label.contains("Hafer (CH), Vanilla (EU)")); // Ordered by weight
}

#[test]
fn knospe_under_90_validation_eggs_over_10_percent() {
    let calculator = calculator_with(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
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
    let calculator = calculator_with(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
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
    let calculator = calculator_with(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
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

// Regression (Testing 25.06.2026): "Butter" picked from the local food_db resolves
// via alias to canonical "Kochbutter" and carries NO BLV category — the dairy
// always-show-origin rule silently skipped it while "Milch" (API category) worked.
// `effective_category()` must fall back to the curated food_db category.
#[test]
fn knospe_under_90_butter_without_api_category_shows_origin() {
    let calculator = calculator_with(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 950.0).origin(Country::EU).build())
        .ingredient(
            IngredientBuilder::new_agri("Butter", 50.0)
                .canonical("Kochbutter")
                .origin(Country::CH)
                .build()
        )
        .build();
    let output = calculator.execute(input);

    // Dairy always shows origin, even <10% and without a BLV API category
    assert!(output.label.contains("Butter (CH)"), "label was: {}", output.label);
}

#[test]
fn knospe_under_90_validation_butter_without_api_category_requires_origin() {
    let calculator = calculator_with(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Hafer", 950.0).origin(Country::EU).build())
        .ingredient(
            IngredientBuilder::new_agri("Butter", 50.0)
                .canonical("Kochbutter")
                .build()
        )
        .build();
    let output = calculator.execute(input);

    let butter_messages = output.validation_messages.get("ingredients[1][origin]");
    assert!(butter_messages.is_some_and(|msgs| msgs.iter().any(
        |msg| msg == "Herkunftsland ist erforderlich für Milch/Fleisch/Insekten (Knospe <90% CH Regel)."
    )));
}

#[test]
fn knospe_under_90_validation_meat_always_requires_origin() {
    let calculator = calculator_with(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
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
fn knospe_under_90_validation_fish_over_10_percent() {
    let calculator = calculator_with(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Hafer", 850.0).origin(Country::EU).build())
        .ingredient(
            IngredientBuilder::new_agri("Lachs", 150.0)
                .category("Fisch")
                .build()
        )
        .build();
    let output = calculator.execute(input);

    // Should have validation error for fish >10%
    let fish_messages = output.validation_messages.get("ingredients[1][origin]");
    assert!(fish_messages.is_some());
    let messages = fish_messages.unwrap();
    assert!(messages.iter().any(|msg| msg.contains("Eier/Honig/Fisch >10%")));
}

#[test]
fn knospe_under_90_validation_insects_always_requires_origin() {
    let calculator = calculator_with(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Hafer", 970.0).origin(Country::EU).build())
        .ingredient(
            IngredientBuilder::new_agri("Grillen", 30.0)
                .category("Insekten")
                .build()
        )
        .build();
    let output = calculator.execute(input);

    // Should have validation error for insects even at low percentage
    let insect_messages = output.validation_messages.get("ingredients[1][origin]");
    assert!(insect_messages.is_some());
    let messages = insect_messages.unwrap();
    assert!(messages.iter().any(|msg| msg == "Herkunftsland ist erforderlich für Milch/Fleisch/Insekten (Knospe <90% CH Regel)."));
}

#[test]
fn knospe_under_90_validation_plant_over_50_percent() {
    let calculator = calculator_with(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
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
    let calculator = calculator_with(vec![RuleDef::Knospe_Under90_Percent_CH_IngredientRules]);
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

#[test]
fn knospe_composite_parent_origin_only_on_lowest_level() {
    // Composite parent (CH, allergen, not bio) with two bio agricultural
    // children (CH allergen + IT). Origin and allergen bolding belong to
    // the lowest level only; the foreign IT origin must be declared under
    // the Under-90 rule (49.3% Swiss).
    let calculator = calculator_for(crate::shared::Configuration::Knospe);
    let input = InputBuilder::new()
        .vollstaendig()
        .certification_body("bio.inspecta")
        .ingredient(
            IngredientBuilder::new_agri("Mehrkornmischung", 675.0)
                .allergen()
                .origin(Country::CH)
                .children(vec![
                    IngredientBuilder::new_agri("Weizen", 333.0)
                        .allergen()
                        .origin(Country::CH)
                        .bio()
                        .build(),
                    IngredientBuilder::new_agri("Mais", 342.0)
                        .origin(Country::IT)
                        .bio()
                        .build(),
                ])
                .build(),
        )
        .build();
    let output = calculator.execute(input);
    let label = output.label;

    // Parent: no allergen bold, no (CH) origin
    assert!(!label.contains("<b>Mehrkornmischung</b>"), "parent must not be bolded, got: {}", label);
    assert!(!label.contains("Mehrkornmischung* (CH)"), "parent must not show origin, got: {}", label);
    assert!(!label.contains("Mehrkornmischung (CH)"), "parent must not show origin, got: {}", label);
    // Child Weizen: bold + (CH) — allergen, Swiss, >=10% of total
    assert!(label.contains("<b>Weizen</b>* (CH)"), "child Weizen must show bold + (CH), got: {}", label);
    // Child Mais: bio asterisk only — foreign agri without category falls below
    // the Excel "show origin" thresholds under Knospe <90% CH rule.
    assert!(label.contains("Mais*"), "child Mais must have bio asterisk, got: {}", label);
    assert!(!label.contains("Mais* (IT)"), "child Mais must NOT show (IT) per Excel rules, got: {}", label);
    // Bio legend should appear (children are bio)
    assert!(label.contains("aus biologischer Landwirtschaft"), "bio legend missing, got: {}", label);
}

// =============================================================================
// Group — Wildsammlung (Excel Zeile 12, ° marker at >= 10% of total)
// =============================================================================

#[test]
fn wildsammlung_marker_and_legend_over_10_percent() {
    let calculator = calculator_with(vec![RuleDef::Wildsammlung_Ueber10Prozent]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Bärlauch", 150.0)
            .processing_steps(vec!["aus zertifizierter Wildsammlung"]).build())
        .ingredient(IngredientBuilder::new_agri("Rapsöl", 850.0).build())
        .build();
    let output = calculator.execute(input);
    // Bärlauch = 15% >= 10% → ° marker + Wildsammlung legend.
    assert!(output.label.contains('°'), "expected ° marker; label: {}", output.label);
    assert!(output.label.contains("Wildsammlung"), "expected Wildsammlung legend; label: {}", output.label);
}

#[test]
fn wildsammlung_exactly_10_percent_shows_marker() {
    // Boundary: exactly 10% must show — Excel Zeile 12 says "grösser/gleich 10 %".
    let calculator = calculator_with(vec![RuleDef::Wildsammlung_Ueber10Prozent]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Bärlauch", 100.0)
            .processing_steps(vec!["aus zertifizierter Wildsammlung"]).build())
        .ingredient(IngredientBuilder::new_agri("Rapsöl", 900.0).build())
        .build();
    let output = calculator.execute(input);
    assert!(output.label.contains('°'), "exactly 10% must show the ° marker; label: {}", output.label);
}

#[test]
fn wildsammlung_under_10_percent_no_marker() {
    let calculator = calculator_with(vec![RuleDef::Wildsammlung_Ueber10Prozent]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Bärlauch", 50.0)
            .processing_steps(vec!["aus zertifizierter Wildsammlung"]).build())
        .ingredient(IngredientBuilder::new_agri("Rapsöl", 950.0).build())
        .build();
    let output = calculator.execute(input);
    // Bärlauch = 5% < 10% → no ° marker.
    assert!(!output.label.contains('°'), "under 10% must NOT show a ° marker; label: {}", output.label);
}

// --- DEC-8: 5% cap on permitted non-organic ingredients (Pektin) ---------
//
// Permitted exceptions count as Knospe-compliant, so the certified percentage
// stays at 100% no matter how much Pektin is in the recipe. Bio Suisse caps
// them at 5% of the agricultural weight, exactly like the Bio-V rule.

#[test]
fn knospe_erlaubte_ausnahme_over_5pct_blocks_logo() {
    let calculator = calculator_with(vec![
        RuleDef::Knospe_ShowBioSuisseLogo,
        RuleDef::Knospe_100_Percent_CH_NoOrigin,
    ]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Himbeeren", 600.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Pektin", 400.0).origin(Country::CH).erlaubte_ausnahme_knospe().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    assert_eq!(c.get(keys::KNOSPE_MARKETING_ALLOWED), None);
    assert_eq!(c.get(keys::KNOSPE_MARKETING_NOT_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::BIO_SUISSE_REGULAR), None);
    assert_eq!(c.get(keys::BIO_SUISSE_NO_CROSS), None);
    // The hint has to name the 5% limit, not a missing certification.
    assert_eq!(c.get(keys::KNOSPE_ERLAUBTE_AUSNAHME_UEBER_5_PROZENT), Some(&true));
}

#[test]
fn knospe_erlaubte_ausnahme_within_5pct_keeps_logo() {
    let calculator = calculator_with(vec![
        RuleDef::Knospe_ShowBioSuisseLogo,
        RuleDef::Knospe_100_Percent_CH_NoOrigin,
    ]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Himbeeren", 960.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Pektin", 40.0).origin(Country::CH).erlaubte_ausnahme_knospe().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    assert_eq!(c.get(keys::KNOSPE_MARKETING_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::KNOSPE_MARKETING_NOT_ALLOWED), None);
    assert_eq!(c.get(keys::KNOSPE_ERLAUBTE_AUSNAHME_UEBER_5_PROZENT), None);
}

#[test]
fn knospe_erlaubte_ausnahme_exactly_5pct_keeps_logo() {
    // Boundary: "höchstens 5%" includes exactly 5%.
    let calculator = calculator_with(vec![
        RuleDef::Knospe_ShowBioSuisseLogo,
        RuleDef::Knospe_100_Percent_CH_NoOrigin,
    ]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Himbeeren", 950.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Pektin", 50.0).origin(Country::CH).erlaubte_ausnahme_knospe().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    assert_eq!(c.get(keys::KNOSPE_MARKETING_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::KNOSPE_ERLAUBTE_AUSNAHME_UEBER_5_PROZENT), None);
}

#[test]
fn knospe_erlaubte_ausnahme_bio_flag_counts_toward_the_same_cap() {
    // Both exception flags make an ingredient Knospe-compliant, so both must
    // count against the 5% budget.
    let calculator = calculator_with(vec![
        RuleDef::Knospe_ShowBioSuisseLogo,
        RuleDef::Knospe_100_Percent_CH_NoOrigin,
    ]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Himbeeren", 600.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Nonbio", 400.0).origin(Country::CH).erlaubte_ausnahme_bio().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    assert_eq!(c.get(keys::KNOSPE_MARKETING_NOT_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::KNOSPE_ERLAUBTE_AUSNAHME_UEBER_5_PROZENT), Some(&true));
}

#[test]
fn knospe_erlaubte_ausnahme_over_5pct_fails_the_rezeptur_check() {
    // Logo gate and «Rezeptur prüfen» text must agree; otherwise the label says
    // "erfüllt die Anforderungen" while no logo is shown.
    let calculator = calculator_with(vec![
        RuleDef::Knospe_ShowBioSuisseLogo,
        RuleDef::Knospe_100_Percent_CH_NoOrigin,
    ]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Himbeeren", 600.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Pektin", 400.0).origin(Country::CH).erlaubte_ausnahme_knospe().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    assert_eq!(c.get(keys::KNOSPE_CHECK_OK), None);
    assert_eq!(c.get(keys::KNOSPE_CHECK_FAILED), Some(&true));
}

#[test]
fn knospe_uncertified_nonbio_still_blocks_without_the_5pct_hint() {
    // Existing behaviour (unchanged): a non-bio ingredient without the exception
    // checkbox blocks the logo — but the reason is the missing certification, so
    // the 5% hint must NOT appear.
    let calculator = calculator_with(vec![
        RuleDef::Knospe_ShowBioSuisseLogo,
        RuleDef::Knospe_100_Percent_CH_NoOrigin,
    ]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Himbeeren", 600.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Zucker", 400.0).origin(Country::CH).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    assert_eq!(c.get(keys::KNOSPE_MARKETING_NOT_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::KNOSPE_ERLAUBTE_AUSNAHME_UEBER_5_PROZENT), None);
}

#[test]
fn knospe_non_agricultural_exception_does_not_count() {
    // Only agricultural ingredients enter the percentage; water/salt must not
    // push a recipe over the cap.
    let calculator = calculator_with(vec![
        RuleDef::Knospe_ShowBioSuisseLogo,
        RuleDef::Knospe_100_Percent_CH_NoOrigin,
    ]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Himbeeren", 600.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new("Wasser", 400.0).agricultural(false).erlaubte_ausnahme_knospe().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    assert_eq!(c.get(keys::KNOSPE_MARKETING_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::KNOSPE_ERLAUBTE_AUSNAHME_UEBER_5_PROZENT), None);
}
// --- DEC-10: «Bio» in the Sachbezeichnung for Knospe products --------------
//
// Bio-V has always appended « Bio» when the product may be marketed as organic.
// Knospe products qualify too, so the same suffix applies — with the Bio-V rule
// for Umstellung: only a Monoprodukt may claim «Bio» (Excel Zeile 7).

#[test]
fn knospe_eligible_recipe_gets_the_bio_sachbezeichnung() {
    let calculator = calculator_for(crate::shared::Configuration::Knospe);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Himbeeren", 600.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Zucker", 400.0).bio().origin(Country::CH).build())
        .build();
    let c = calculator.execute(input).conditional_elements;

    assert_eq!(c.get(keys::KNOSPE_MARKETING_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), Some(&true));
}

#[test]
fn knospe_ineligible_recipe_gets_no_bio_sachbezeichnung() {
    let calculator = calculator_for(crate::shared::Configuration::Knospe);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Himbeeren", 600.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Zucker", 400.0).origin(Country::CH).build())
        .build();
    let c = calculator.execute(input).conditional_elements;

    assert_eq!(c.get(keys::KNOSPE_MARKETING_NOT_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), None);
}

#[test]
fn knospe_composite_umstellung_gets_no_bio_sachbezeichnung() {
    // A composite conversion product may not claim «Bio», exactly as in Bio-V.
    let calculator = calculator_for(crate::shared::Configuration::Knospe);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Himbeeren", 600.0).bio().origin(Country::CH)
            .umstellbetrieb().build())
        .ingredient(IngredientBuilder::new_agri("Zucker", 400.0).bio().origin(Country::CH).build())
        .build();
    let c = calculator.execute(input).conditional_elements;

    assert_eq!(c.get(keys::KNOSPE_MARKETING_ALLOWED), Some(&true), "the logo itself is unaffected");
    assert_eq!(c.get(keys::KNOSPE_UMSTELLUNG_LOGO), Some(&true));
    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), None);
}

#[test]
fn knospe_mono_umstellung_keeps_the_bio_sachbezeichnung() {
    // Monoprodukt aus Umstellung: «Bio» is allowed (Excel Zeile 7).
    let calculator = calculator_for(crate::shared::Configuration::Knospe);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Himbeeren", 1000.0).bio().origin(Country::CH)
            .umstellbetrieb().build())
        .build();
    let c = calculator.execute(input).conditional_elements;

    assert_eq!(c.get(keys::KNOSPE_UMSTELLUNG_LOGO), Some(&true));
    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), Some(&true));
}

#[test]
fn knospe_empty_recipe_gets_no_bio_sachbezeichnung() {
    let calculator = calculator_for(crate::shared::Configuration::Knospe);
    let c = calculator.execute(InputBuilder::new().vollstaendig().build()).conditional_elements;

    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), None);
}
