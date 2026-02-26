use super::*;
use crate::rules::RuleDef;
use crate::shared::Configuration;

// =============================================================================
// Group A — Bio-CH Sachbezeichnung (Bio_ShowBioSachbezeichnung rule)
// =============================================================================

#[test]
fn bio_ch_100_percent_sets_sachbezeichnung_suffix() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).bio_ch().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    assert_eq!(c.get("bio_sachbezeichnung_suffix"), Some(&true));
    assert_eq!(c.get("bio_marketing_allowed"), Some(&true));
    assert_eq!(c.get("bio_marketing_not_allowed"), None);
}

#[test]
fn bio_ch_partial_sets_marketing_not_allowed() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    assert_eq!(c.get("bio_sachbezeichnung_suffix"), None);
    assert_eq!(c.get("bio_marketing_allowed"), None);
    assert_eq!(c.get("bio_marketing_not_allowed"), Some(&true));
}

#[test]
fn bio_ch_zero_percent_no_conditionals() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    assert_eq!(c.get("bio_sachbezeichnung_suffix"), None);
    assert_eq!(c.get("bio_marketing_allowed"), None);
    assert_eq!(c.get("bio_marketing_not_allowed"), None);
}

#[test]
fn bio_ch_with_non_agricultural_ignored() {
    // Non-agricultural ingredients (e.g. Salz) should be excluded from the percentage calculation
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 500.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Salz", 500.0).build()) // Salz is non-agricultural
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    // Hafer is the only agricultural ingredient and it's 100% bio_ch
    assert_eq!(c.get("bio_sachbezeichnung_suffix"), Some(&true));
    assert_eq!(c.get("bio_marketing_allowed"), Some(&true));
}

#[test]
fn bio_ch_vs_is_bio_are_independent() {
    // is_bio (Knospe/generic bio) does NOT count toward bio_ch percentage
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 500.0).bio().build()) // is_bio only, not bio_ch
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 500.0).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    // is_bio does not count as bio_ch, so bio_ch percentage is 0%
    assert_eq!(c.get("bio_sachbezeichnung_suffix"), None);
    assert_eq!(c.get("bio_marketing_allowed"), None);
    assert_eq!(c.get("bio_marketing_not_allowed"), None);
}

#[test]
fn bio_ch_100_percent_via_full_bio_config() {
    // End-to-end test using Configuration::Bio which includes Bio_ShowBioSachbezeichnung
    let calculator = calculator_for(Configuration::Bio);
    let input = InputBuilder::new()
        .vollstaendig()
        .certification_body("Bio Inspecta")
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio_ch().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).bio_ch().origin(Country::CH).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    assert_eq!(c.get("bio_sachbezeichnung_suffix"), Some(&true));
    assert_eq!(c.get("bio_marketing_allowed"), Some(&true));
}

// =============================================================================
// Group B — Bio Asterisk Marking
// =============================================================================

#[test]
fn bio_ingredients_get_asterisk() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_Knospe_EingabeIstBio]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).build())
        .build();
    let output = calculator.execute(input);

    assert!(output.label.contains("Hafer*"));
    assert!(!output.label.contains("Weizenmehl*"));
}

#[test]
fn bio_ch_ingredients_get_asterisk() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_Knospe_EingabeIstBio]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).build())
        .build();
    let output = calculator.execute(input);

    assert!(output.label.contains("Hafer*"));
}

#[test]
fn bio_legend_appended() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_Knospe_EingabeIstBio]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).build())
        .build();
    let output = calculator.execute(input);

    assert!(output.label.contains("* aus biologischer Landwirtschaft"));
}

#[test]
fn no_bio_legend_without_bio_ingredients() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_Knospe_EingabeIstBio]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).build())
        .build();
    let output = calculator.execute(input);

    assert!(!output.label.contains("aus biologischer Landwirtschaft"));
}

#[test]
fn bio_asterisk_not_added_without_bio_rule() {
    // Without Bio_Knospe_EingabeIstBio rule, asterisk should not be added even if is_bio is set
    let calculator = setup_simple_calculator();
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).build())
        .build();
    let output = calculator.execute(input);

    assert!(!output.label.contains("Hafer*"));
    assert!(!output.label.contains("aus biologischer Landwirtschaft"));
}

// =============================================================================
// Group C — Knospe Bio-Branching (bio-specific Swiss percentage calculation)
// =============================================================================

#[test]
fn knospe_bio_branching_uses_bio_swiss_percentage() {
    // With Bio_Knospe_EingabeIstBio, only bio ingredients are counted for Swiss %
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![
        RuleDef::Knospe_100_Percent_CH_NoOrigin,
        RuleDef::Knospe_90_99_Percent_CH_ShowOrigin,
        RuleDef::Bio_Knospe_EingabeIstBio,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 500.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Olivenöl", 500.0).origin(Country::EU).build()) // not bio → ignored
        .build();
    let output = calculator.execute(input);

    // Only bio ingredients counted: Hafer (500g CH) / Hafer (500g total bio) = 100% Swiss
    // So Knospe 100% rule should apply (no origin display)
    assert!(!output.label.contains("(Schweiz)"));
    assert!(!output.label.contains("(CH)"));
}

#[test]
fn knospe_bio_branching_ignores_non_bio() {
    // Non-bio ingredients excluded → can push to 100% tier
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![
        RuleDef::Knospe_100_Percent_CH_NoOrigin,
        RuleDef::Knospe_90_99_Percent_CH_ShowOrigin,
        RuleDef::Bio_Knospe_EingabeIstBio,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 300.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 200.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Olivenöl", 500.0).origin(Country::EU).build()) // not bio
        .build();
    let output = calculator.execute(input);

    // Bio ingredients: Hafer 300 CH + Weizenmehl 200 CH = 500 all Swiss → 100%
    // Knospe 100% rule → no origin display
    assert!(!output.label.contains("(Schweiz)"));
}

#[test]
fn knospe_without_bio_rule_uses_all_ingredients() {
    // Without Bio_Knospe_EingabeIstBio, all agricultural ingredients are counted
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![
        RuleDef::Knospe_100_Percent_CH_NoOrigin,
        RuleDef::Knospe_90_99_Percent_CH_ShowOrigin,
    ]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Hafer", 500.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Olivenöl", 500.0).origin(Country::EU).build())
        .build();
    let output = calculator.execute(input);

    // Without bio rule: all agricultural counted → 500 CH / 1000 total = 50% → <90% → no special rule
    // Neither 100% nor 90-99% rule applies
    assert!(!output.label.contains("(Schweiz)"));
    assert!(!output.label.contains("(CH)"));
}

// =============================================================================
// Group D — Knospe Logo Variants (Knospe_ShowBioSuisseLogo)
// =============================================================================

#[test]
fn knospe_logo_regular_100_knospe_90_plus_swiss() {
    // 100% Knospe-certified, >= 90% Swiss → bio_suisse_regular
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![
        RuleDef::Knospe_ShowBioSuisseLogo,
        RuleDef::Knospe_100_Percent_CH_NoOrigin,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 900.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Olivenöl", 100.0).bio().origin(Country::EU).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    assert_eq!(c.get("bio_suisse_regular"), Some(&true));
    assert_eq!(c.get("bio_suisse_no_cross"), None);
}

#[test]
fn knospe_logo_no_cross_100_knospe_under_90_swiss() {
    // 100% Knospe-certified, < 90% Swiss → bio_suisse_no_cross
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![
        RuleDef::Knospe_ShowBioSuisseLogo,
        RuleDef::Knospe_Under90_Percent_CH_IngredientRules,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 400.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Olivenöl", 600.0).bio().origin(Country::EU).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    assert_eq!(c.get("bio_suisse_regular"), None);
    assert_eq!(c.get("bio_suisse_no_cross"), Some(&true));
}

#[test]
fn knospe_no_logo_when_not_100_knospe() {
    // Not all agricultural ingredients are Knospe-certified → no logo at all
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![
        RuleDef::Knospe_ShowBioSuisseLogo,
        RuleDef::Knospe_100_Percent_CH_NoOrigin,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Olivenöl", 400.0).origin(Country::EU).build()) // NOT bio
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    assert_eq!(c.get("bio_suisse_regular"), None);
    assert_eq!(c.get("bio_suisse_no_cross"), None);
}

#[test]
fn knospe_logo_regular_exact_90_boundary() {
    // Exactly 90% Swiss → regular logo (with cross)
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![
        RuleDef::Knospe_ShowBioSuisseLogo,
        RuleDef::Knospe_90_99_Percent_CH_ShowOrigin,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 900.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Olivenöl", 100.0).bio().origin(Country::EU).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    // 90% Swiss → >= 90% → regular logo
    assert_eq!(c.get("bio_suisse_regular"), Some(&true));
    assert_eq!(c.get("bio_suisse_no_cross"), None);
}

#[test]
fn knospe_logo_no_cross_just_under_90_boundary() {
    // 89% Swiss → no cross logo
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![
        RuleDef::Knospe_ShowBioSuisseLogo,
        RuleDef::Knospe_Under90_Percent_CH_IngredientRules,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 890.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Olivenöl", 110.0).bio().origin(Country::EU).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    // 89% Swiss → < 90% → no cross logo
    assert_eq!(c.get("bio_suisse_regular"), None);
    assert_eq!(c.get("bio_suisse_no_cross"), Some(&true));
}

// =============================================================================
// Group E — Certification Body validation
// =============================================================================

#[test]
fn certification_body_required() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_Knospe_ZertifizierungsstellePflicht]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 1000.0).build())
        .build();
    let output = calculator.execute(input);

    // Missing certification body should produce a validation error
    let messages = output.validation_messages.get("certification_body");
    assert!(messages.is_some());
    assert!(!messages.unwrap().is_empty());
}

#[test]
fn certification_body_valid() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_Knospe_ZertifizierungsstellePflicht]);
    let input = InputBuilder::new()
        .certification_body("Bio Inspecta")
        .ingredient(IngredientBuilder::new_agri("Hafer", 1000.0).build())
        .build();
    let output = calculator.execute(input);

    // Valid certification body → no validation error for this field
    assert!(output.validation_messages.get("certification_body").is_none());
}

#[test]
fn certification_body_empty_string_invalid() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_Knospe_ZertifizierungsstellePflicht]);
    let input = InputBuilder::new()
        .certification_body("")
        .ingredient(IngredientBuilder::new_agri("Hafer", 1000.0).build())
        .build();
    let output = calculator.execute(input);

    // Empty string should be treated as missing
    let messages = output.validation_messages.get("certification_body");
    assert!(messages.is_some());
    assert!(!messages.unwrap().is_empty());
}
