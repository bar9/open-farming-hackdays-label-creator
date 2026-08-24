use crate::conditional_keys as keys;
use super::*;
use crate::rules::RuleDef;
use crate::shared::Configuration;

// =============================================================================
// Group A — Bio-CH Sachbezeichnung (Bio_ShowBioSachbezeichnung rule)
// =============================================================================

#[test]
fn bio_ch_100_percent_sets_sachbezeichnung_suffix() {
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).bio_ch().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), Some(&true));
    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::BIO_MARKETING_NOT_ALLOWED), None);
}

#[test]
fn bio_ch_partial_sets_marketing_not_allowed() {
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), None);
    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), None);
    assert_eq!(c.get(keys::BIO_MARKETING_NOT_ALLOWED), Some(&true));
}

#[test]
fn bio_ch_erlaubte_ausnahme_within_5pct_allows_sachbezeichnung() {
    // A permitted non-organic exception (Annex 3 WBF, e.g. Pektin) is NOT bio, but up to
    // 5% of the agricultural weight is tolerated → "Bio" stays in the Sachbezeichnung.
    // At 96% (not 100%) the exception is marked per-ingredient, not via the "Alle" legend.
    let calculator = calculator_with(vec![
        RuleDef::Bio_ShowBioSachbezeichnung,
        RuleDef::Bio_Knospe_EingabeIstBio,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 960.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Pektin", 40.0).erlaubte_ausnahme_bio().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), Some(&true));
    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::BIO_MARKETING_NOT_ALLOWED), None);
    assert_eq!(c.get(keys::BIO_ERLAUBTE_AUSNAHME_UEBER_5_PROZENT), None);
    // 96% (not 100%): per-ingredient marking, Pektin unmarked, no "Alle" legend.
    assert!(output.label.contains("Hafer*"), "bio ingredient gets *. Label: {}", output.label);
    assert!(!output.label.contains("Pektin*"), "permitted non-bio exception must not be starred");
    assert!(output.label.contains("* aus biologischer Landwirtschaft"), "Label: {}", output.label);
    assert!(!output.label.contains("Alle landwirtschaftlichen"), "not 100% → no 'Alle' legend");
}

#[test]
fn bio_ch_erlaubte_ausnahme_over_5pct_blocks_sachbezeichnung() {
    // Over the 5% tolerance the permitted exception no longer counts as bio, so the
    // Bio-CH share drops below 95% and "Bio" is blocked, with a specific 5% hint.
    let calculator = calculator_with(vec![
        RuleDef::Bio_ShowBioSachbezeichnung,
        RuleDef::Bio_Knospe_EingabeIstBio,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Pektin", 400.0).erlaubte_ausnahme_bio().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), None);
    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), None);
    assert_eq!(c.get(keys::BIO_MARKETING_NOT_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::BIO_ERLAUBTE_AUSNAHME_UEBER_5_PROZENT), Some(&true));
}

#[test]
fn bio_ch_zero_percent_shows_warning() {
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), None);
    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), None);
    // B8: Warning must appear even when no ingredient is bio
    assert_eq!(c.get(keys::BIO_MARKETING_NOT_ALLOWED), Some(&true));
}

#[test]
fn bio_ch_with_non_agricultural_ignored() {
    // Non-agricultural ingredients (e.g. Salz) should be excluded from the percentage calculation
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 500.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Salz", 500.0).build()) // Salz is non-agricultural
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    // Hafer is the only agricultural ingredient and it's 100% bio_ch
    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), Some(&true));
    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), Some(&true));
}

#[test]
fn bio_ch_vs_is_bio_are_independent() {
    // is_bio (Knospe/generic bio) does NOT count toward bio_ch percentage
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 500.0).bio().build()) // is_bio only, not bio_ch
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 500.0).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    // is_bio does not count as bio_ch, so bio_ch percentage is 0%
    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), None);
    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), None);
    // B8: Warning shown because bio_ch percentage is 0%
    assert_eq!(c.get(keys::BIO_MARKETING_NOT_ALLOWED), Some(&true));
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
    let c = &output.conditionals();

    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), Some(&true));
    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), Some(&true));
}

// =============================================================================
// Group B — Bio Asterisk Marking
// =============================================================================

#[test]
fn bio_ingredients_get_asterisk() {
    let calculator = calculator_with(vec![RuleDef::Bio_Knospe_EingabeIstBio]);
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
    let calculator = calculator_with(vec![RuleDef::Bio_Knospe_EingabeIstBio]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).build())
        .build();
    let output = calculator.execute(input);

    assert!(output.label.contains("Hafer*"));
}

#[test]
fn bio_legend_appended() {
    let calculator = calculator_with(vec![RuleDef::Bio_Knospe_EingabeIstBio]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).build())
        .build();
    let output = calculator.execute(input);

    assert!(output.label.contains("* aus biologischer Landwirtschaft"));
}

#[test]
fn no_bio_legend_without_bio_ingredients() {
    let calculator = calculator_with(vec![RuleDef::Bio_Knospe_EingabeIstBio]);
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
    let calculator = calculator_with(vec![
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
    let calculator = calculator_with(vec![
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
    let calculator = calculator_with(vec![
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
    let calculator = calculator_with(vec![
        RuleDef::Knospe_ShowBioSuisseLogo,
        RuleDef::Knospe_100_Percent_CH_NoOrigin,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 900.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Olivenöl", 100.0).bio().origin(Country::EU).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::BIO_SUISSE_REGULAR), Some(&true));
    assert_eq!(c.get(keys::BIO_SUISSE_NO_CROSS), None);
}

#[test]
fn knospe_logo_no_cross_100_knospe_under_90_swiss() {
    // 100% Knospe-certified, < 90% Swiss → bio_suisse_no_cross
    let calculator = calculator_with(vec![
        RuleDef::Knospe_ShowBioSuisseLogo,
        RuleDef::Knospe_Under90_Percent_CH_IngredientRules,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 400.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Olivenöl", 600.0).bio().origin(Country::EU).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::BIO_SUISSE_REGULAR), None);
    assert_eq!(c.get(keys::BIO_SUISSE_NO_CROSS), Some(&true));
}

#[test]
fn knospe_no_logo_when_not_100_knospe() {
    // Not all agricultural ingredients are Knospe-certified → no logo at all
    let calculator = calculator_with(vec![
        RuleDef::Knospe_ShowBioSuisseLogo,
        RuleDef::Knospe_100_Percent_CH_NoOrigin,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Olivenöl", 400.0).origin(Country::EU).build()) // NOT bio
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::BIO_SUISSE_REGULAR), None);
    assert_eq!(c.get(keys::BIO_SUISSE_NO_CROSS), None);
}

// Umstellungs-Knospe on the label (Testing 25.06.2026): any ingredient aus
// Umstellung flips the logo artwork to the Umstellungsknospe variant; the
// regular/no_cross conditionals keep encoding the Suisse/Import split.
#[test]
fn knospe_umstellung_logo_with_umstellbetrieb_ingredient() {
    let calculator = calculator_with(vec![RuleDef::Knospe_ShowBioSuisseLogo]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 900.0).bio().umstellbetrieb().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 100.0).bio().origin(Country::CH).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::BIO_SUISSE_REGULAR), Some(&true));
    assert_eq!(c.get(keys::KNOSPE_UMSTELLUNG_LOGO), Some(&true));
}

#[test]
fn knospe_umstellung_logo_import_variant() {
    let calculator = calculator_with(vec![RuleDef::Knospe_ShowBioSuisseLogo]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 400.0).bio().umstellbetrieb().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Rohrzucker", 600.0).bio().origin(Country::PE).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::BIO_SUISSE_NO_CROSS), Some(&true));
    assert_eq!(c.get(keys::KNOSPE_UMSTELLUNG_LOGO), Some(&true));
}

#[test]
fn knospe_umstellung_logo_absent_without_umstellbetrieb() {
    let calculator = calculator_with(vec![RuleDef::Knospe_ShowBioSuisseLogo]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 900.0).bio().origin(Country::CH).build())
        .build();
    let output = calculator.execute(input);

    assert_eq!(output.conditionals().get(keys::KNOSPE_UMSTELLUNG_LOGO), None);
}

#[test]
fn knospe_umstellung_logo_from_composite_parent_claim() {
    // A bought certified composite declared "Umstellung" as a whole carries the
    // flag on the parent node — the whole-tree helper must still see it.
    let calculator = calculator_with(vec![RuleDef::Knospe_ShowBioSuisseLogo]);
    let input = InputBuilder::new()
        .ingredient(
            IngredientBuilder::new_agri("Müeslimischung", 900.0)
                .bio()
                .umstellbetrieb()
                .origin(Country::CH)
                .children(vec![
                    IngredientBuilder::new_agri("Hafer", 0.0).build(),
                    IngredientBuilder::new_agri("Dinkel", 0.0).build(),
                ])
                .build(),
        )
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::KNOSPE_UMSTELLUNG_LOGO), Some(&true));
}

// Tri-state «Rezeptur prüfen» result (Testing 25.06.2026): pending before the
// check button is pressed, ok/failed afterwards. The certification body is NOT
// part of this check (yellow label placeholder covers it).
#[test]
fn knospe_check_pending_before_recipe_marked_complete() {
    let calculator = calculator_with(vec![RuleDef::Knospe_ShowBioSuisseLogo]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 900.0).bio().origin(Country::CH).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::KNOSPE_CHECK_PENDING), Some(&true));
    assert_eq!(c.get(keys::KNOSPE_CHECK_OK), None);
    assert_eq!(c.get(keys::KNOSPE_CHECK_FAILED), None);
}

#[test]
fn knospe_check_hints_suppressed_for_einzelzutat() {
    // DEC-3: «Keine Zutatenliste (Einzelzutat)» has no recipe to check, so the
    // tri-state hints must stay silent in the Knospe environment too.
    for vollstaendig in [false, true] {
        let calculator = calculator_with(vec![RuleDef::Knospe_ShowBioSuisseLogo]);
        let mut builder = InputBuilder::new()
            .einzelzutat()
            .ingredient(IngredientBuilder::new_agri("Hafer", 900.0).bio().origin(Country::CH).build());
        if vollstaendig {
            builder = builder.vollstaendig();
        }
        let output = calculator.execute(builder.build());
        let c = &output.conditionals();

        assert_eq!(c.get(keys::KNOSPE_CHECK_PENDING), None, "vollstaendig={vollstaendig}");
        assert_eq!(c.get(keys::KNOSPE_CHECK_OK), None, "vollstaendig={vollstaendig}");
        assert_eq!(c.get(keys::KNOSPE_CHECK_FAILED), None, "vollstaendig={vollstaendig}");
    }
}

#[test]
fn knospe_check_ok_when_complete_and_valid() {
    let calculator = calculator_with(vec![RuleDef::Knospe_ShowBioSuisseLogo]);
    // No certification body set: must NOT block the OK state.
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Hafer", 900.0).bio().origin(Country::CH).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::KNOSPE_CHECK_PENDING), None);
    assert_eq!(c.get(keys::KNOSPE_CHECK_OK), Some(&true));
    assert_eq!(c.get(keys::KNOSPE_CHECK_FAILED), None);
}

#[test]
fn knospe_check_failed_when_recipe_has_validation_issue() {
    let calculator = calculator_with(vec![
        RuleDef::Knospe_ShowBioSuisseLogo,
        RuleDef::Knospe_AlleZutatenHerkunft,
    ]);
    let input = InputBuilder::new()
        .vollstaendig()
        // Import-Knospe without a country while the Import-Knospe logo shows
        // → recipe-scoped validation error → failed
        .ingredient(IngredientBuilder::new_agri("Hafer", 900.0).bio().origin(Country::Import).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::KNOSPE_CHECK_OK), None);
    assert_eq!(c.get(keys::KNOSPE_CHECK_FAILED), Some(&true));
}

#[test]
fn knospe_check_failed_when_not_fully_knospe() {
    let calculator = calculator_with(vec![RuleDef::Knospe_ShowBioSuisseLogo]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Olivenöl", 400.0).origin(Country::EU).build()) // NOT bio
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::KNOSPE_CHECK_OK), None);
    assert_eq!(c.get(keys::KNOSPE_CHECK_FAILED), Some(&true));
}

#[test]
fn knospe_logo_shown_when_nonbio_has_erlaubte_ausnahme_bio() {
    // A non-bio ingredient that is a permitted non-organic exception (Annex 3 WBF)
    // must not block the Knospe logo — within the 5% cap (DEC-8).
    let calculator = calculator_with(vec![
        RuleDef::Knospe_ShowBioSuisseLogo,
        RuleDef::Knospe_100_Percent_CH_NoOrigin,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Himbeeren", 960.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Nonbio", 40.0).origin(Country::CH).erlaubte_ausnahme_bio().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::KNOSPE_MARKETING_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::KNOSPE_MARKETING_NOT_ALLOWED), None);
    // A logo variant must be set (exception is non-bio so 100% Swiss of the bio share → regular).
    assert!(c.get(keys::BIO_SUISSE_REGULAR) == Some(&true) || c.get(keys::BIO_SUISSE_NO_CROSS) == Some(&true));
}

#[test]
fn knospe_logo_shown_when_nonbio_has_erlaubte_ausnahme_knospe() {
    // A non-Knospe ingredient that is a permitted Bio Suisse Part III exception
    // must not block the Knospe logo — within the 5% cap (DEC-8).
    let calculator = calculator_with(vec![
        RuleDef::Knospe_ShowBioSuisseLogo,
        RuleDef::Knospe_100_Percent_CH_NoOrigin,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Himbeeren", 960.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Pektin", 40.0).origin(Country::CH).erlaubte_ausnahme_knospe().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::KNOSPE_MARKETING_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::KNOSPE_MARKETING_NOT_ALLOWED), None);
    assert!(c.get(keys::BIO_SUISSE_REGULAR) == Some(&true) || c.get(keys::BIO_SUISSE_NO_CROSS) == Some(&true));
}

#[test]
fn knospe_logo_regular_exact_90_boundary() {
    // Exactly 90% Swiss → regular logo (with cross)
    let calculator = calculator_with(vec![
        RuleDef::Knospe_ShowBioSuisseLogo,
        RuleDef::Knospe_90_99_Percent_CH_ShowOrigin,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 900.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Olivenöl", 100.0).bio().origin(Country::EU).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    // 90% Swiss → >= 90% → regular logo
    assert_eq!(c.get(keys::BIO_SUISSE_REGULAR), Some(&true));
    assert_eq!(c.get(keys::BIO_SUISSE_NO_CROSS), None);
}

#[test]
fn knospe_logo_no_cross_just_under_90_boundary() {
    // 89% Swiss → no cross logo
    let calculator = calculator_with(vec![
        RuleDef::Knospe_ShowBioSuisseLogo,
        RuleDef::Knospe_Under90_Percent_CH_IngredientRules,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 890.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Olivenöl", 110.0).bio().origin(Country::EU).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    // 89% Swiss → < 90% → no cross logo
    assert_eq!(c.get(keys::BIO_SUISSE_REGULAR), None);
    assert_eq!(c.get(keys::BIO_SUISSE_NO_CROSS), Some(&true));
}

// =============================================================================
// Group E — Certification Body validation
// =============================================================================

#[test]
fn certification_body_required() {
    let calculator = calculator_with(vec![RuleDef::Bio_Knospe_ZertifizierungsstellePflicht]);
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
    let calculator = calculator_with(vec![RuleDef::Bio_Knospe_ZertifizierungsstellePflicht]);
    let input = InputBuilder::new()
        .certification_body("CH-BIO-006 (bio.inspecta AG)")
        .ingredient(IngredientBuilder::new_agri("Hafer", 1000.0).build())
        .build();
    let output = calculator.execute(input);

    // Valid certification body → no validation error for this field
    assert!(!output.validation_messages.contains_key("certification_body"));
}

#[test]
fn certification_body_empty_string_invalid() {
    let calculator = calculator_with(vec![RuleDef::Bio_Knospe_ZertifizierungsstellePflicht]);
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
    let calculator = calculator_with(vec![RuleDef::Bio_Knospe_ZertifizierungsstellePflicht]);
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
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    // The 5% non-bio share must be a DECLARED permitted exception (Anhang 3 WBF);
    // an undeclared non-bio ingredient blocks "Bio" outright (DEC-7).
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 950.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Pektin", 50.0).erlaubte_ausnahme_bio().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    // 95% bio_ch >= 95% threshold → suffix allowed
    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), Some(&true));
    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), Some(&true));
}

// =============================================================================
// Group — DEC-7: undeclared non-organic ingredients
//
// The 5% tolerance applies ONLY to declared permitted exceptions (Anhang 3 WBF).
// Any other non-organic agricultural ingredient blocks "Bio" regardless of share:
// «Es darf zum Beispiel nicht bis zu 5 % nicht-Bio Eier verwendet werden.»
// =============================================================================

#[test]
fn bio_blocked_by_undeclared_non_bio_under_5_percent() {
    // 96% bio, 4% plain conventional egg without the exception checkbox.
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Mehl", 960.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Ei", 40.0).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), None, "4% nicht-bio Ei darf «Bio» nicht erlauben");
    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), None);
    assert_eq!(c.get(keys::BIO_MARKETING_NOT_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::BIO_NICHT_DEKLARIERTE_ZUTAT), Some(&true), "Hinweis nennt den Grund");
    assert_eq!(c.get(keys::BIO_CHECK_FAILED), Some(&true));
}

#[test]
fn bio_allowed_when_same_ingredient_is_declared_exception() {
    // Identical recipe, but the 4% is declared a permitted exception → Bio allowed.
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Mehl", 960.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Pektin", 40.0).erlaubte_ausnahme_bio().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), Some(&true));
    assert_eq!(c.get(keys::BIO_NICHT_DEKLARIERTE_ZUTAT), None);
    assert_eq!(c.get(keys::BIO_CHECK_OK), Some(&true));
}

#[test]
fn bio_still_blocked_when_declared_exception_over_5_percent() {
    // The existing 5% ceiling keeps working, and this is NOT the undeclared case.
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Mehl", 900.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Pektin", 100.0).erlaubte_ausnahme_bio().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::BIO_MARKETING_NOT_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::BIO_ERLAUBTE_AUSNAHME_UEBER_5_PROZENT), Some(&true));
    assert_eq!(c.get(keys::BIO_NICHT_DEKLARIERTE_ZUTAT), None, "deklariert — anderer Grund");
}

#[test]
fn bio_non_agricultural_ingredient_does_not_block() {
    // Salz/Wasser are not agricultural, so they are outside the bio calculus.
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Mehl", 960.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Salz", 40.0).agricultural(false).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::BIO_NICHT_DEKLARIERTE_ZUTAT), None);
    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), Some(&true));
}

#[test]
fn bio_mono_umstellbetrieb_still_allowed() {
    // Umstellbetrieb ingredients are bio-certified and handled by the conversion
    // logic; the new check must not catch them (ticket acceptance criterion).
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Hafer", 1000.0).bio_ch().umstellbetrieb().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::BIO_NICHT_DEKLARIERTE_ZUTAT), None);
    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), Some(&true));
    assert_eq!(c.get(keys::UMSTELLBETRIEB_HINWEIS), Some(&true));
}

#[test]
fn bio_blocked_by_undeclared_non_bio_inside_composite() {
    // The offending ingredient hides inside a composite that makes no own claim.
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let fuellung = IngredientBuilder::new_agri("Füllung", 400.0)
        .children(vec![
            IngredientBuilder::new_agri("Aprikosen", 380.0).bio_ch().build(),
            IngredientBuilder::new_agri("Ei", 20.0).build(),
        ])
        .build();
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Mehl", 600.0).bio_ch().build())
        .ingredient(fuellung)
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::BIO_NICHT_DEKLARIERTE_ZUTAT), Some(&true));
    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), None);
}

#[test]
fn bio_composite_claiming_own_bio_quality_is_not_blocked() {
    // A bought, certified composite carries the claim on the parent node; its
    // children are then not second-guessed (mirrors is_bio_ch_compliant).
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let fertigmischung = IngredientBuilder::new_agri("Fertigmischung", 400.0)
        .bio_ch()
        .children(vec![
            IngredientBuilder::new_agri("Aprikosen", 380.0).build(),
            IngredientBuilder::new_agri("Ei", 20.0).build(),
        ])
        .build();
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Mehl", 600.0).bio_ch().build())
        .ingredient(fertigmischung)
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    // Scope of this test is the DEC-7 check only: the parent's claim shields its
    // children from being flagged as undeclared non-bio.
    assert_eq!(c.get(keys::BIO_NICHT_DEKLARIERTE_ZUTAT), None);
    // NOTE: `bio_marketing_allowed` is NOT asserted here. calculate_bio_ch_certified_percentage
    // walks `leaves()` and therefore ignores a parent-level claim, while
    // `is_bio_ch_compliant` honours it — a pre-existing divergence that predates
    // DEC-7 and would need its own ticket.
}

#[test]
fn bio_ch_94_percent_sets_marketing_not_allowed() {
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 940.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 60.0).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    // 94% < 95% → no suffix, marketing not allowed
    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), None);
    assert_eq!(c.get(keys::BIO_MARKETING_NOT_ALLOWED), Some(&true));
}

#[test]
fn bio_ch_umstellbetrieb_excluded_from_percentage() {
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).bio_ch().umstellbetrieb().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    // Umstellbetrieb ingredient excluded: only 600/1000 agricultural = 60% bio_ch → not allowed
    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), None);
    assert_eq!(c.get(keys::BIO_MARKETING_NOT_ALLOWED), Some(&true));
}

#[test]
fn bio_ch_95_with_umstellbetrieb_drops_below_threshold() {
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 900.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 100.0).bio_ch().umstellbetrieb().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    // 100% bio_ch but Weizenmehl is umstellbetrieb → effective 900/1000 = 90% < 95%
    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), None);
    assert_eq!(c.get(keys::BIO_MARKETING_NOT_ALLOWED), Some(&true));
}

// =============================================================================
// Group G — Bio Marking Modes (AllBio / PartialBio / NoBio)
// =============================================================================

#[test]
fn bio_100pct_all_bio_no_asterisk_alle_legend() {
    // Exactly 100% bio_ch: no * on ingredients, "Alle landwirtschaftlichen" legend.
    let calculator = calculator_with(vec![
        RuleDef::Bio_ShowBioSachbezeichnung,
        RuleDef::Bio_Knospe_EingabeIstBio,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 960.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 40.0).bio_ch().build())
        .build();
    let output = calculator.execute(input);

    // No asterisk on individual ingredients
    assert!(!output.label.contains("Hafer*"), "100% mode should suppress individual * marking");
    // "Alle landwirtschaftlichen" legend present
    assert!(output.label.contains("Alle landwirtschaftlichen Zutaten stammen aus biologischer Landwirtschaft"));
}

#[test]
fn bio_95_99_band_uses_per_ingredient_asterisk() {
    // 96% bio_ch (Bio in Sachbezeichnung, but not all bio): per-ingredient * +
    // "* aus biologischer Landwirtschaft", NOT the "Alle" legend (Excel Zeilen 2–4 vs. 3).
    let calculator = calculator_with(vec![
        RuleDef::Bio_ShowBioSachbezeichnung,
        RuleDef::Bio_Knospe_EingabeIstBio,
    ]);
    // The non-bio remainder must be a declared permitted exception, otherwise "Bio"
    // is blocked entirely (DEC-7) — so the 95–99.99% band IS the exception case.
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 960.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Pektin", 40.0).erlaubte_ausnahme_bio().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), Some(&true), "96% >= 95% → Bio in Sachbezeichnung");
    assert!(output.label.contains("Hafer*"), "bio ingredient gets *. Label: {}", output.label);
    assert!(!output.label.contains("Pektin*"), "non-bio ingredient not starred");
    assert!(output.label.contains("* aus biologischer Landwirtschaft"), "Label: {}", output.label);
    assert!(!output.label.contains("Alle landwirtschaftlichen"), "not 100% → no 'Alle' legend");
}

#[test]
fn bio_partial_bio_has_asterisks_and_percentage() {
    // 60% bio_ch: * on bio ingredients, "60% der landwirtschaftlichen..." legend
    let calculator = calculator_with(vec![
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
    let calculator = calculator_with(vec![
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
    let calculator = calculator_with(vec![RuleDef::Bio_Knospe_EingabeIstBio]);
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
    let calculator = calculator_with(vec![RuleDef::Bio_Knospe_EingabeIstBio]);
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
    let calculator = calculator_with(vec![RuleDef::Bio_Knospe_EingabeIstBio]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 400.0).umstellbetrieb().build())
        .build();
    let output = calculator.execute(input);

    assert!(output.label.contains("** aus Umstellung auf biologische Landwirtschaft"));
}

#[test]
fn monoprodukt_umstellbetrieb_allows_sachbezeichnung_with_note() {
    // Excel Zeile 7: a single Bio-CH agricultural ingredient from a conversion farm MAY
    // carry "Bio" + the mandatory Umstellungshinweis. Register the full Bio config rule set
    // (incl. Bio_Knospe_EingabeIstBio) so the ** marker + legend render on the label.
    let calculator = calculator_with(vec![
        RuleDef::Bio_ShowBioSachbezeichnung,
        RuleDef::Bio_Knospe_EingabeIstBio,
    ]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 950.0).bio_ch().umstellbetrieb().build())
        .ingredient(IngredientBuilder::new("Salz", 50.0).agricultural(false).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    // "Bio" IS allowed for the mono-Umstellbetrieb case (was wrongly blocked before this fix).
    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), Some(&true));
    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::BIO_MARKETING_NOT_ALLOWED), None);
    assert_eq!(c.get(keys::UMSTELLBETRIEB_HINWEIS), Some(&true));
    // The mandatory Umstellung declaration is printed on the label via the ** marker + legend.
    assert!(output.label.contains("Hafer**"), "expected ** marker on Hafer; label: {}", output.label);
    assert!(output.label.contains("** aus Umstellung auf biologische Landwirtschaft"),
        "expected Umstellung legend; label: {}", output.label);
}

#[test]
fn composite_umstellbetrieb_removes_sachbezeichnung() {
    // Multiple agricultural ingredients + umstellbetrieb → remove suffix
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 500.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Weizenmehl", 500.0).bio_ch().umstellbetrieb().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    // Composite with umstellbetrieb: no sachbezeichnung_suffix
    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), None);
    assert_eq!(c.get(keys::BIO_MARKETING_NOT_ALLOWED), Some(&true));
}

// =============================================================================
// Group — BioV tri-state «Rezeptur prüfen» (bio_check_pending/ok/failed)
// =============================================================================

#[test]
fn bio_check_pending_before_rezeptur_vollstaendig() {
    // Not yet checked → pending, and neither ok nor failed is asserted.
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 1000.0).bio_ch().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();
    assert_eq!(c.get(keys::BIO_CHECK_PENDING), Some(&true));
    assert_eq!(c.get(keys::BIO_CHECK_OK), None);
    assert_eq!(c.get(keys::BIO_CHECK_FAILED), None);
}

// =============================================================================
// Group — DEC-4: alternative_marking_allowed
//
// The blanket wordings («Alle landwirtschaftlichen Zutaten stammen aus
// biologischer Landwirtschaft» / «Bio-» prefix) are only truthful when EVERY
// agricultural ingredient is organic. A permitted non-organic exception
// (Anhang 3 WBF, e.g. Pektin) leaves only the per-ingredient *-marking.
// =============================================================================

#[test]
fn alternative_marking_allowed_when_all_agricultural_are_bio() {
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Himbeeren", 500.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Zucker", 500.0).bio_ch().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::BIO_CHECK_OK), Some(&true));
    assert_eq!(c.get(keys::ALTERNATIVE_MARKING_ALLOWED), Some(&true));
}

// DEC-16: the blanket wording talks about «alle landwirtschaftlichen Zutaten»,
// so it must not appear when the recipe has none — a jar of wild garlic from
// certified wild collection is gathered, not farmed.
#[test]
fn alternative_marking_suppressed_when_there_are_no_agricultural_ingredients() {
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(
            IngredientBuilder::new("B\u{e4}rlauch", 1000.0)
                .agricultural(false)
                .bio_ch()
                .processing_steps(vec!["aus zertifizierter Wildsammlung"])
                .build(),
        )
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(
        c.get(keys::ALTERNATIVE_MARKING_ALLOWED),
        None,
        "no agricultural ingredients \u{2192} the blanket wording says nothing; conditionals: {:?}",
        c
    );
}

// Reported after the first DEC-16 fix: the case that actually occurs in the UI.
// Ticking «Wildsammlung» does NOT clear is_agricultural (that flag only follows
// the «Nicht-landwirtschaftlich» quality), so a recipe made purely of
// wild-collected ingredients still counted as agricultural and printed the
// blanket sentence.
// The reported bug renders on the label itself, not just as a hint: a purely
// wild-collected recipe hits 100% bio-CH and printed the AllBio legend.
#[test]
fn wild_only_recipe_omits_the_alle_landwirtschaftlichen_legend() {
    let calculator = calculator_for(crate::shared::Configuration::Bio);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(
            IngredientBuilder::new_agri("B\u{e4}rlauch", 342.0)
                .bio_ch()
                .processing_steps(vec!["aus zertifizierter Wildsammlung"])
                .build(),
        )
        .build();
    let label = calculator.execute(input).label;

    assert!(
        !label.contains("Alle landwirtschaftlichen Zutaten stammen aus biologischer Landwirtschaft"),
        "wild-only recipe must not print the AllBio legend; label: {}",
        label
    );
}

#[test]
fn alternative_marking_suppressed_when_every_ingredient_is_wild_collected() {
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(
            IngredientBuilder::new_agri("B\u{e4}rlauch", 1000.0)
                .bio_ch()
                .processing_steps(vec!["aus zertifizierter Wildsammlung"])
                .build(),
        )
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(
        c.get(keys::ALTERNATIVE_MARKING_ALLOWED),
        None,
        "everything wild-collected \u{2192} nothing farmed to talk about; conditionals: {:?}",
        c
    );
}

// Guard the other half: one farmed ingredient alongside wild collection is
// enough for the wording to be meaningful again.
#[test]
fn alternative_marking_allowed_when_at_least_one_ingredient_is_agricultural() {
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(
            IngredientBuilder::new("B\u{e4}rlauch", 500.0)
                .agricultural(false)
                .bio_ch()
                .processing_steps(vec!["aus zertifizierter Wildsammlung"])
                .build(),
        )
        .ingredient(IngredientBuilder::new_agri("Raps\u{f6}l", 500.0).bio_ch().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::ALTERNATIVE_MARKING_ALLOWED), Some(&true));
}

#[test]
fn alternative_marking_suppressed_with_erlaubte_ausnahme_bio() {
    // Ticket example: Konfitüre with 5 g Pektin as a permitted non-organic
    // agricultural ingredient. Recipe still qualifies for Bio, but only the
    // *-marking per ingredient is allowed.
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Himbeeren", 500.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Zucker", 500.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Pektin", 5.0).erlaubte_ausnahme_bio().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    // The positive verdict must remain — only the alternative-wording hint goes.
    assert_eq!(c.get(keys::BIO_CHECK_OK), Some(&true), "Rezeptur erfüllt die Bio-Anforderungen weiterhin");
    assert_eq!(c.get(keys::ALTERNATIVE_MARKING_ALLOWED), None);
}

#[test]
fn alternative_marking_suppressed_with_erlaubte_ausnahme_knospe() {
    // Same rule in the Knospe environment (ticket: Bio-V *and* Knospe).
    let calculator = calculator_with(vec![
        RuleDef::Knospe_ShowBioSuisseLogo,
        RuleDef::Knospe_100_Percent_CH_NoOrigin,
    ]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Himbeeren", 995.0).bio().origin(Country::CH).build())
        .ingredient(IngredientBuilder::new_agri("Pektin", 5.0).origin(Country::CH).erlaubte_ausnahme_knospe().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::KNOSPE_CHECK_OK), Some(&true), "Rezeptur erfüllt die Knospe-Anforderungen weiterhin");
    assert_eq!(c.get(keys::ALTERNATIVE_MARKING_ALLOWED), None);
}

#[test]
fn alternative_marking_allowed_when_exception_ingredient_is_also_bio() {
    // Flag set but the ingredient IS bio-certified: it is not actually a
    // non-organic exception, so the blanket wording stays truthful.
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Himbeeren", 995.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Pektin", 5.0).bio_ch().erlaubte_ausnahme_bio().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::ALTERNATIVE_MARKING_ALLOWED), Some(&true));
}

#[test]
fn alternative_marking_suppressed_for_nested_erlaubte_ausnahme() {
    // The exception sits inside a composite — leaves() must find it.
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let fuellung = IngredientBuilder::new_agri("Füllung", 500.0)
        .children(vec![
            IngredientBuilder::new_agri("Aprikosen", 495.0).bio_ch().build(),
            IngredientBuilder::new_agri("Pektin", 5.0).erlaubte_ausnahme_bio().build(),
        ])
        .build();
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Zucker", 500.0).bio_ch().build())
        .ingredient(fuellung)
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();

    assert_eq!(c.get(keys::ALTERNATIVE_MARKING_ALLOWED), None);
}

#[test]
fn bio_check_hints_suppressed_for_einzelzutat() {
    // DEC-3: «Keine Zutatenliste (Einzelzutat)» has no recipe, so none of the
    // tri-state «Rezeptur prüfen» hints may appear — neither before nor after
    // the check button would have been pressed.
    for vollstaendig in [false, true] {
        let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
        let mut builder = InputBuilder::new()
            .einzelzutat()
            .ingredient(IngredientBuilder::new_agri("Hafer", 1000.0).bio_ch().build());
        if vollstaendig {
            builder = builder.vollstaendig();
        }
        let output = calculator.execute(builder.build());
        let c = &output.conditionals();

        assert_eq!(c.get(keys::BIO_CHECK_PENDING), None, "vollstaendig={vollstaendig}");
        assert_eq!(c.get(keys::BIO_CHECK_OK), None, "vollstaendig={vollstaendig}");
        assert_eq!(c.get(keys::BIO_CHECK_FAILED), None, "vollstaendig={vollstaendig}");
    }
}

#[test]
fn bio_check_ok_when_vollstaendig_and_qualifies() {
    // Checked + >= 95% Bio-CH + no recipe issues → ok.
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Hafer", 1000.0).bio_ch().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();
    assert_eq!(c.get(keys::BIO_CHECK_OK), Some(&true));
    assert_eq!(c.get(keys::BIO_CHECK_PENDING), None);
    assert_eq!(c.get(keys::BIO_CHECK_FAILED), None);
}

#[test]
fn bio_check_failed_when_vollstaendig_but_under_95() {
    // Checked but only 60% Bio-CH → does not qualify → failed.
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Hafer", 600.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Weizen", 400.0).build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();
    assert_eq!(c.get(keys::BIO_CHECK_FAILED), Some(&true));
    assert_eq!(c.get(keys::BIO_CHECK_OK), None);
}

#[test]
fn bio_check_failed_when_recipe_issue_despite_qualifying() {
    // Qualifies on percentage (100% Bio-CH) but a per-ingredient validation error is
    // open (>50% ingredient without origin) → the check must fail, not pass.
    let calculator = calculator_with(vec![
        RuleDef::Bio_ShowBioSachbezeichnung,
        RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent,
    ]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Hafer", 1000.0).bio_ch().build())
        .build();
    let output = calculator.execute(input);
    let c = &output.conditionals();
    assert!(output.validation_messages.contains_key("ingredients[0][origin]"),
        "expected an open origin error; messages: {:?}", output.validation_messages);
    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), Some(&true), "still qualifies on percentage");
    assert_eq!(c.get(keys::BIO_CHECK_FAILED), Some(&true));
    assert_eq!(c.get(keys::BIO_CHECK_OK), None);
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

    // Star on ONE level only: the children carry the markers, so the composite
    // parent must NOT also get a star (testing round 2026-06-17).
    assert!(!output.label.contains("Müeslimischung*"), "Composite parent must not duplicate the children's star. Label: {}", output.label);
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
    // Star on ONE level only: the children show the markers, so the composite
    // parent must NOT also get a star — even with explicit is_bio (2026-06-17).
    assert!(!output.label.contains("Schokolade*"), "Composite parent must not duplicate the children's star. Label: {}", output.label);
}

// Counterpart to the "one level only" rule: when a composite is declared Knospe as a
// whole (parent-claim override, Phase 9) but its children carry NO bio markers, the
// star belongs on the parent — there is no child star to defer to (2026-06-17).
#[test]
fn composite_parent_claim_pushes_star_down_to_children() {
    // Testing 25.06.2026: the * must ALWAYS sit on the sub-ingredients, never on
    // the composite name — a parent-level quality claim is inherited by the
    // (agricultural) children instead.
    let calculator = calculator_for(Configuration::Knospe);
    let input = InputBuilder::new()
        .vollstaendig()
        .certification_body("CH-BIO-006 (bio.inspecta AG)")
        .ingredient(
            IngredientBuilder::new_agri("Fertigmischung", 100.0)
                .bio() // parent claims Knospe as a whole
                .origin(Country::CH)
                .children(vec![
                    IngredientBuilder::new_agri("Komponente A", 60.0).build(),
                    IngredientBuilder::new_agri("Komponente B", 40.0).build(),
                ])
                .build(),
        )
        .build();

    let output = calculator.execute(input);

    assert!(!output.label.contains("Fertigmischung*"), "Composite parent must never carry the star. Label: {}", output.label);
    assert!(output.label.contains("Komponente A*"), "Children inherit the parent-level claim. Label: {}", output.label);
    assert!(output.label.contains("Komponente B*"), "Children inherit the parent-level claim. Label: {}", output.label);
    // Inherited stars also drive the legend.
    assert!(output.label.contains("aus biologischer Landwirtschaft"), "Legend must appear for inherited stars. Label: {}", output.label);
}

#[test]
fn composite_parent_umstellung_claim_pushes_double_star_down() {
    let calculator = calculator_for(Configuration::Knospe);
    let input = InputBuilder::new()
        .ingredient(
            IngredientBuilder::new_agri("Fertigmischung", 100.0)
                .bio()
                .umstellbetrieb() // whole composite from Umstellung
                .origin(Country::CH)
                .children(vec![
                    IngredientBuilder::new_agri("Komponente A", 60.0).build(),
                    // Non-agricultural additive: NO inherited marker
                    IngredientBuilder::new("Zusatzstoff", 40.0).agricultural(false).build(),
                ])
                .build(),
        )
        .build();

    let output = calculator.execute(input);

    assert!(!output.label.contains("Fertigmischung*"), "Parent carries no marker. Label: {}", output.label);
    assert!(output.label.contains("Komponente A**"), "Agricultural child inherits **. Label: {}", output.label);
    assert!(!output.label.contains("Zusatzstoff*"), "Non-agricultural child must not inherit a marker. Label: {}", output.label);
    assert!(output.label.contains("aus Umstellung"), "** legend must appear. Label: {}", output.label);
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
    let calculator = calculator_with(vec![RuleDef::Bio_Knospe_EingabeIstBio, RuleDef::AP2_1_ZusammegesetztOutput]);
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
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
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
    let c = &output.conditionals();

    // 600/1000 = 60% bio_ch (umstellbetrieb excluded) → below 95% threshold
    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), None,
        "Umstellbetrieb child should be excluded from bio_ch %. Conditionals: {:?}", c);
}

// --- DEC-6: the green Bio badge follows the recipe, not the check button ----
//
// The badge is the Bio-V counterpart of the Knospe logo, which has always been
// driven by the recipe math. `bio_marketing_allowed` is what the preview reads,
// so these tests pin its behaviour before «Rezeptur prüfen» is pressed.

#[test]
fn bio_marketing_allowed_without_pressing_rezeptur_pruefen() {
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    // Note: no .vollstaendig() — the user has not pressed the button.
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 1000.0).bio_ch().build())
        .build();
    let c = calculator.execute(input).conditionals();

    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), Some(&true));
    // The hint texts stay coupled to the button.
    assert_eq!(c.get(keys::BIO_CHECK_PENDING), Some(&true));
    assert_eq!(c.get(keys::BIO_CHECK_OK), None);
}

#[test]
fn bio_marketing_not_allowed_for_an_empty_recipe() {
    // Guard against the badge appearing on an untouched form: an empty recipe
    // has a vacuous 100% Bio share.
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let c = calculator.execute(InputBuilder::new().build()).conditionals();

    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), None);
    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), None);
}

#[test]
fn bio_marketing_not_allowed_when_the_recipe_does_not_qualify() {
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Hafer", 500.0).bio_ch().build())
        .ingredient(IngredientBuilder::new_agri("Zucker", 500.0).build())
        .build();
    let c = calculator.execute(input).conditionals();

    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), None);
    assert_eq!(c.get(keys::BIO_MARKETING_NOT_ALLOWED), Some(&true));
}

#[test]
fn bio_check_texts_are_unchanged_by_the_badge_decoupling() {
    // After pressing «Rezeptur prüfen» a qualifying recipe still reports ok, so
    // the hint texts keep their old semantics.
    let calculator = calculator_with(vec![RuleDef::Bio_ShowBioSachbezeichnung]);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(IngredientBuilder::new_agri("Hafer", 1000.0).bio_ch().build())
        .build();
    let c = calculator.execute(input).conditionals();

    assert_eq!(c.get(keys::BIO_CHECK_OK), Some(&true));
    assert_eq!(c.get(keys::BIO_CHECK_PENDING), None);
    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), Some(&true));
}

// --- DEC-11: wild collection under the Bio-Verordnung ----------------------
//
// The step is stored identically in both regimes; only the printed wording
// differs. Bio Suisse says «aus zertifizierter Wildsammlung», the Bio-V
// requires «aus biologisch zertifizierter Wildsammlung» (Abklärung BLW).

// DEC-16 narrowed this: the 10% °-marking is a Bio-Suisse rule, so the Bio-V
// prints wild collection inline next to the ingredient at any share — with its
// own wording (DEC-11), but never as a ° legend.
#[test]
fn biov_wildsammlung_prints_inline_without_a_degree_marker() {
    let calculator = calculator_for(crate::shared::Configuration::Bio);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Bärlauch", 150.0).bio_ch()
            .processing_steps(vec!["aus zertifizierter Wildsammlung"]).build())
        .ingredient(IngredientBuilder::new_agri("Rapsöl", 850.0).bio_ch().build())
        .build();
    let label = calculator.execute(input).label;

    assert!(
        !label.contains('°'),
        "the 10% ° rule is Knospe-only (DEC-16); label: {}",
        label
    );
    assert!(
        label.contains("aus biologisch zertifizierter Wildsammlung"),
        "Bio-V must still use the «biologisch» wording; label: {}",
        label
    );
}

// The counterpart of the DEC-16 narrowing: Knospe keeps the ° marking.
#[test]
fn knospe_wildsammlung_still_marks_above_10_percent() {
    let calculator = calculator_for(crate::shared::Configuration::Knospe);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Bärlauch", 150.0).bio().origin(Country::CH)
            .processing_steps(vec!["aus zertifizierter Wildsammlung"]).build())
        .ingredient(IngredientBuilder::new_agri("Rapsöl", 850.0).bio().origin(Country::CH).build())
        .build();
    let label = calculator.execute(input).label;

    assert!(
        label.contains('°'),
        "Knospe keeps the 10% ° marking (DEC-16); label: {}",
        label
    );
}

#[test]
fn knospe_wildsammlung_legend_wording_is_unchanged() {
    let calculator = calculator_for(crate::shared::Configuration::Knospe);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Bärlauch", 150.0).bio().origin(Country::CH)
            .processing_steps(vec!["aus zertifizierter Wildsammlung"]).build())
        .ingredient(IngredientBuilder::new_agri("Rapsöl", 850.0).bio().origin(Country::CH).build())
        .build();
    let label = calculator.execute(input).label;

    assert!(label.contains('°'), "label: {}", label);
    assert!(
        label.contains("aus zertifizierter Wildsammlung"),
        "Knospe wording must stay as-is; label: {}",
        label
    );
    assert!(
        !label.contains("biologisch zertifizierter Wildsammlung"),
        "Knospe must not adopt the Bio-V wording; label: {}",
        label
    );
}

#[test]
fn biov_wildsammlung_under_10_percent_prints_the_bio_wording_inline() {
    // Below the threshold there is no ° marker; the step is printed next to the
    // ingredient instead — and must carry the same Bio-V wording.
    let calculator = calculator_for(crate::shared::Configuration::Bio);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Bärlauch", 50.0).bio_ch()
            .processing_steps(vec!["aus zertifizierter Wildsammlung"]).build())
        .ingredient(IngredientBuilder::new_agri("Rapsöl", 950.0).bio_ch().build())
        .build();
    let label = calculator.execute(input).label;

    assert!(!label.contains('°'), "5% < 10% → no ° marker; label: {}", label);
    assert!(
        label.contains("aus biologisch zertifizierter Wildsammlung"),
        "inline step must use the Bio-V wording; label: {}",
        label
    );
}

#[test]
fn knospe_wildsammlung_under_10_percent_keeps_its_wording_inline() {
    let calculator = calculator_for(crate::shared::Configuration::Knospe);
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new_agri("Bärlauch", 50.0).bio().origin(Country::CH)
            .processing_steps(vec!["aus zertifizierter Wildsammlung"]).build())
        .ingredient(IngredientBuilder::new_agri("Rapsöl", 950.0).bio().origin(Country::CH).build())
        .build();
    let label = calculator.execute(input).label;

    assert!(!label.contains('°'), "label: {}", label);
    assert!(label.contains("aus zertifizierter Wildsammlung"), "label: {}", label);
    assert!(
        !label.contains("biologisch zertifizierter Wildsammlung"),
        "label: {}",
        label
    );
}
