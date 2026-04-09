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
fn bio_ch_zero_percent_shows_warning() {
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
    // B8: Warning must appear even when no ingredient is bio
    assert_eq!(c.get("bio_marketing_not_allowed"), Some(&true));
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
    // B8: Warning shown because bio_ch percentage is 0%
    assert_eq!(c.get("bio_marketing_not_allowed"), Some(&true));
}

#[test]
fn bio_ch_100_percent_via_full_bio_config() {
    // End-to-end test using Configuration::Bio which includes Bio_ShowBioSachbezeichnung
    let calculator = calculator_for(Configuration::Bio);
    let input = InputBuilder::new()
        .vollstaendig()
        .certification_body("CH-BIO-006 (bio.inspecta AG)")
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
        .certification_body("CH-BIO-006 (bio.inspecta AG)")
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

#[test]
fn certification_body_invalid_format() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_Knospe_ZertifizierungsstellePflicht]);
    let input = InputBuilder::new()
        .certification_body("BIO-123")
        .ingredient(IngredientBuilder::new_agri("Hafer", 1000.0).build())
        .build();
    let output = calculator.execute(input);

    // Invalid format (doesn't start with CH-BIO-) → validation error
    let messages = output.validation_messages.get("certification_body");
    assert!(messages.is_some());
    assert!(messages.unwrap().iter().any(|m| m.contains("CH-BIO-xxx")));
}

// =============================================================================
// Group F — Bio-CH 95% Threshold + Umstellbetrieb Exclusion
// =============================================================================

#[test]
fn bio_ch_95_percent_sets_sachbezeichnung_suffix() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 950.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 50.0).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    // 95% bio_ch >= 95% threshold → suffix allowed
    assert_eq!(c.get("bio_sachbezeichnung_suffix"), Some(&true));
    assert_eq!(c.get("bio_marketing_allowed"), Some(&true));
}

#[test]
fn bio_ch_94_percent_sets_marketing_not_allowed() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 940.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 60.0).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    // 94% < 95% → no suffix, marketing not allowed
    assert_eq!(c.get("bio_sachbezeichnung_suffix"), None);
    assert_eq!(c.get("bio_marketing_not_allowed"), Some(&true));
}

#[test]
fn bio_ch_umstellbetrieb_excluded_from_percentage() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).bio_ch().umstellbetrieb().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    // Umstellbetrieb ingredient excluded: only 600/1000 agricultural = 60% bio_ch → not allowed
    assert_eq!(c.get("bio_sachbezeichnung_suffix"), None);
    assert_eq!(c.get("bio_marketing_not_allowed"), Some(&true));
}

#[test]
fn bio_ch_95_with_umstellbetrieb_drops_below_threshold() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 900.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 100.0).bio_ch().umstellbetrieb().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    // 100% bio_ch but Weizenmehl is umstellbetrieb → effective 900/1000 = 90% < 95%
    assert_eq!(c.get("bio_sachbezeichnung_suffix"), None);
    assert_eq!(c.get("bio_marketing_not_allowed"), Some(&true));
}

// =============================================================================
// Group G — Bio Marking Modes (AllBio / PartialBio / NoBio)
// =============================================================================

#[test]
fn bio_all_agricultural_bio_no_asterisk() {
    // >= 95% bio_ch with Bio_ShowBioSachbezeichnung: no * on ingredients, "Alle landwirtschaftlichen" legend
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![
        RuleDef::Bio_ShowBioSachbezeichnung,
        RuleDef::Bio_Knospe_EingabeIstBio,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 960.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 40.0).build())
        .build();
    let output = calculator.execute(input);

    // No asterisk on individual ingredients
    assert!(!output.label.contains("Hafer*"), "AllBio mode should suppress individual * marking");
    // "Alle landwirtschaftlichen" legend present
    assert!(output.label.contains("Alle landwirtschaftlichen Zutaten stammen aus biologischer Landwirtschaft"));
}

#[test]
fn bio_partial_bio_has_asterisks_and_percentage() {
    // 60% bio_ch: * on bio ingredients, "60% der landwirtschaftlichen..." legend
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![
        RuleDef::Bio_ShowBioSachbezeichnung,
        RuleDef::Bio_Knospe_EingabeIstBio,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).build())
        .build();
    let output = calculator.execute(input);

    // Asterisk on bio ingredient
    assert!(output.label.contains("Hafer*"), "PartialBio mode should add * on bio ingredients");
    assert!(!output.label.contains("Weizenmehl*"), "Non-bio ingredient should not have *");
    // Percentage legend
    assert!(output.label.contains("60% der landwirtschaftlichen Zutaten stammen aus biologischer Produktion"));
}

#[test]
fn bio_no_bio_no_legend() {
    // 0% bio_ch: no legend at all
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![
        RuleDef::Bio_ShowBioSachbezeichnung,
        RuleDef::Bio_Knospe_EingabeIstBio,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).build())
        .build();
    let output = calculator.execute(input);

    assert!(!output.label.contains("biologischer"), "No bio ingredients → no bio legend");
    assert!(!output.label.contains("*"), "No bio ingredients → no asterisks");
}

#[test]
fn knospe_mode_asterisk_unchanged() {
    // Bio_Knospe_EingabeIstBio without Bio_ShowBioSachbezeichnung → simple * (Knospe mode)
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_Knospe_EingabeIstBio]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).build())
        .build();
    let output = calculator.execute(input);

    assert!(output.label.contains("Hafer*"), "Knospe mode should add simple *");
    assert!(output.label.contains("* aus biologischer Landwirtschaft"), "Knospe mode should have simple legend");
    // Should NOT have the new Bio-specific legends
    assert!(!output.label.contains("Alle landwirtschaftlichen"));
    assert!(!output.label.contains("der landwirtschaftlichen Zutaten stammen"));
}

// =============================================================================
// Group H — Umstellbetrieb Full Integration
// =============================================================================

#[test]
fn umstellbetrieb_gets_double_asterisk() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_Knospe_EingabeIstBio]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).bio_ch().umstellbetrieb().build())
        .build();
    let output = calculator.execute(input);

    assert!(output.label.contains("Weizenmehl**"), "Umstellbetrieb should get **");
    assert!(output.label.contains("Hafer*"), "Regular bio should get *");
    // ** should not be followed by another * (i.e., no ***)
    assert!(!output.label.contains("***"));
}

#[test]
fn umstellbetrieb_legend_appended() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_Knospe_EingabeIstBio]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).umstellbetrieb().build())
        .build();
    let output = calculator.execute(input);

    assert!(output.label.contains("** aus Umstellung auf biologische Landwirtschaft"));
}

#[test]
fn monoprodukt_umstellbetrieb_allows_sachbezeichnung_with_note() {
    // Single agricultural ingredient + umstellbetrieb → keep suffix + hinweis
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 950.0).bio_ch().umstellbetrieb().build())
        .ingredient(IngredientBuilder::new("Salz", 50.0).agricultural(false).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    // Monoprodukt: only one agricultural leaf (Hafer), Salz is non-agricultural
    // Umstellbetrieb keeps sachbezeichnung_suffix but adds hinweis
    assert_eq!(c.get("umstellbetrieb_hinweis"), Some(&true));
}

#[test]
fn composite_umstellbetrieb_removes_sachbezeichnung() {
    // Multiple agricultural ingredients + umstellbetrieb → remove suffix
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 500.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 500.0).bio_ch().umstellbetrieb().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    // Composite with umstellbetrieb: no sachbezeichnung_suffix
    assert_eq!(c.get("bio_sachbezeichnung_suffix"), None);
    assert_eq!(c.get("bio_marketing_not_allowed"), Some(&true));
}

#[test]
fn monoprodukt_detection_single_agricultural() {
    // 1 agricultural + 1 non-agricultural → mono
    let ingredients = vec![
        IngredientBuilder::new_agri("Hafer", 900.0).build(),
        IngredientBuilder::new("Salz", 100.0).agricultural(false).build(),
    ];
    assert!(is_mono_product(&ingredients));
}

#[test]
fn monoprodukt_detection_multiple_agricultural() {
    // 2 agricultural → not mono
    let ingredients = vec![
        IngredientBuilder::new_agri("Hafer", 500.0).build(),
        IngredientBuilder::new_agri("Weizenmehl", 500.0).build(),
    ];
    assert!(!is_mono_product(&ingredients));
}

// =============================================================================
// Group I — Composite Ingredients: Bio & Umstellbetrieb Marking
// =============================================================================

#[test]
fn composite_child_umstellbetrieb_gets_double_asterisk_on_label() {
    // A composite ingredient with an umstellbetrieb child should show ** on that child
    let calculator = calculator_for(Configuration::Knospe);
    let input = InputBuilder::new()
        .vollstaendig()
        .certification_body("CH-BIO-006 (bio.inspecta AG)")
        .ingredient(
            IngredientBuilder::new_agri("Müeslimischung", 100.0)
                .bio()
                .origin(Country::CH)
                .children(vec![
                    IngredientBuilder::new_agri("Hafer", 60.0)
                        .bio()
                        .origin(Country::CH)
                        .build(),
                    IngredientBuilder::new_agri("Dinkel", 40.0)
                        .bio()
                        .umstellbetrieb()
                        .origin(Country::CH)
                        .build(),
                ])
                .build(),
        )
        .build();

    let output = calculator.execute(input);

    // Parent gets * (computed bio from children)
    assert!(output.label.contains("Müeslimischung*"), "Parent should get bio asterisk. Label: {}", output.label);
    // Hafer child gets * (bio)
    assert!(output.label.contains("Hafer*"), "Bio child should get *. Label: {}", output.label);
    // Dinkel child gets ** (umstellbetrieb)
    assert!(output.label.contains("Dinkel**"), "Umstellbetrieb child should get **. Label: {}", output.label);
    // No *** anywhere
    assert!(!output.label.contains("***"), "Should not have ***. Label: {}", output.label);
}

#[test]
fn composite_child_umstellbetrieb_triggers_legend() {
    // Umstellbetrieb on a child inside a composite should trigger the ** legend
    let calculator = calculator_for(Configuration::Knospe);
    let input = InputBuilder::new()
        .vollstaendig()
        .certification_body("CH-BIO-006 (bio.inspecta AG)")
        .ingredient(
            IngredientBuilder::new_agri("Getreidemischung", 80.0)
                .bio()
                .origin(Country::CH)
                .children(vec![
                    IngredientBuilder::new_agri("Weizen", 50.0)
                        .bio()
                        .origin(Country::CH)
                        .build(),
                    IngredientBuilder::new_agri("Roggen", 30.0)
                        .bio()
                        .umstellbetrieb()
                        .origin(Country::CH)
                        .build(),
                ])
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Zucker", 20.0)
                .bio()
                .origin(Country::CH)
                .build(),
        )
        .build();

    let output = calculator.execute(input);

    assert!(output.label.contains("** aus Umstellung auf biologische Landwirtschaft"),
        "Umstellbetrieb legend should appear for composite child. Label: {}", output.label);
}

#[test]
fn composite_children_bio_markers_in_knospe_context() {
    // All bio children inside a composite should get * in a Knospe context
    let calculator = calculator_for(Configuration::Knospe);
    let input = InputBuilder::new()
        .vollstaendig()
        .certification_body("CH-BIO-006 (bio.inspecta AG)")
        .ingredient(
            IngredientBuilder::new_agri("Schokolade", 50.0)
                .bio()
                .origin(Country::EU)
                .children(vec![
                    IngredientBuilder::new_agri("Zucker", 25.0)
                        .bio()
                        .origin(Country::EU)
                        .build(),
                    IngredientBuilder::new_agri("Kakaobutter", 25.0)
                        .bio()
                        .origin(Country::EU)
                        .build(),
                ])
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Butter", 50.0)
                .bio()
                .origin(Country::CH)
                .category("Butter")
                .build(),
        )
        .build();

    let output = calculator.execute(input);

    // Children inside composite get bio asterisk
    assert!(output.label.contains("Zucker*"), "Bio child Zucker should get *. Label: {}", output.label);
    assert!(output.label.contains("Kakaobutter*"), "Bio child Kakaobutter should get *. Label: {}", output.label);
    // Parent composite has explicit is_bio=true, so it also gets *
    assert!(output.label.contains("Schokolade*"), "Bio parent with explicit is_bio should get *. Label: {}", output.label);
}

#[test]
fn composite_parent_no_asterisk_when_bio_inherited_from_children() {
    // When parent has no explicit is_bio/bio_ch, bio status is computed from children.
    // Lowest-level-only rule: parent should NOT get *, only children.
    let calculator = calculator_for(Configuration::Knospe);
    let input = InputBuilder::new()
        .vollstaendig()
        .certification_body("CH-BIO-006 (bio.inspecta AG)")
        .ingredient(
            IngredientBuilder::new_agri("Schokolade", 50.0)
                .origin(Country::EU)
                .children(vec![
                    IngredientBuilder::new_agri("Zucker", 25.0)
                        .bio()
                        .origin(Country::EU)
                        .build(),
                    IngredientBuilder::new_agri("Kakaobutter", 25.0)
                        .bio()
                        .origin(Country::EU)
                        .build(),
                ])
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Butter", 50.0)
                .bio()
                .origin(Country::CH)
                .category("Butter")
                .build(),
        )
        .build();

    let output = calculator.execute(input);

    // Children should get bio asterisk
    assert!(output.label.contains("Zucker*"), "Bio child Zucker should get *. Label: {}", output.label);
    assert!(output.label.contains("Kakaobutter*"), "Bio child Kakaobutter should get *. Label: {}", output.label);
    // Parent should NOT get * because bio was inherited from children (lowest-level-only)
    assert!(!output.label.contains("Schokolade*"), "Parent with inherited bio should NOT get *. Label: {}", output.label);
    assert!(output.label.contains("Schokolade"), "Parent name should still appear. Label: {}", output.label);
}

#[test]
fn composite_mixed_bio_and_nonbio_children() {
    // Composite with mix of bio and non-bio children: only bio children get *
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_Knospe_EingabeIstBio, RuleDef::AP2_1_ZusammegesetztOutput]);
    let input = InputBuilder::new()
        .ingredient(
            IngredientBuilder::new_agri("Gewürzmischung", 100.0)
                .children(vec![
                    IngredientBuilder::new_agri("Pfeffer", 60.0)
                        .bio()
                        .build(),
                    IngredientBuilder::new("Salz", 40.0)
                        .agricultural(false)
                        .build(),
                ])
                .build(),
        )
        .build();

    let output = calculator.execute(input);

    // Pfeffer is bio → gets *
    assert!(output.label.contains("Pfeffer*"), "Bio child should get *. Label: {}", output.label);
    // Salz is not bio → no *
    assert!(!output.label.contains("Salz*"), "Non-bio child should not get *. Label: {}", output.label);
}

#[test]
fn composite_umstellbetrieb_child_excluded_from_bio_ch_percentage() {
    // Umstellbetrieb children inside composites should be excluded from bio_ch % calculation
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(
            IngredientBuilder::new_agri("Müesli", 1000.0)
                .children(vec![
                    IngredientBuilder::new_agri("Hafer", 600.0)
                        .bio_ch()
                        .origin(Country::CH)
                        .build(),
                    IngredientBuilder::new_agri("Dinkel", 400.0)
                        .bio_ch()
                        .umstellbetrieb()
                        .origin(Country::CH)
                        .build(),
                ])
                .build(),
        )
        .build();

    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    // 600/1000 = 60% bio_ch (umstellbetrieb excluded) → below 95% threshold
    assert_eq!(c.get("bio_sachbezeichnung_suffix"), None,
        "Umstellbetrieb child should be excluded from bio_ch %. Conditionals: {:?}", c);
}
