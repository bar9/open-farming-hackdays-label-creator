use super::*;
use crate::shared::Configuration;

// =============================================================================
// Helpers: Composite chocolate sub-ingredients
// =============================================================================

/// Dark chocolate sub-ingredients (proportional to parent weight).
/// Composition: Zucker 47.8%, Kakaomasse 38.3%, Kakaobutter 13.7%, Vanilleextrakt 0.2%
fn dark_chocolate_children(parent_grams: f64) -> Vec<Ingredient> {
    vec![
        IngredientBuilder::new_agri("Zucker", parent_grams * 0.478)
            .bio()
            .origin(Country::EU)
            .build(),
        IngredientBuilder::new_agri("Kakaomasse", parent_grams * 0.383)
            .bio()
            .origin(Country::EU)
            .build(),
        IngredientBuilder::new_agri("Kakaobutter", parent_grams * 0.137)
            .bio()
            .origin(Country::EU)
            .build(),
        IngredientBuilder::new_agri("Vanilleextrakt", parent_grams * 0.002)
            .bio()
            .origin(Country::EU)
            .build(),
    ]
}

/// Milk chocolate sub-ingredients (proportional to parent weight).
/// Composition: Zucker 38%, Vollmilchpulver 27% (CH, BSK), Kakaobutter 26%, Kakaomasse 8.8%, Vanille 0.2%
fn milk_chocolate_children(parent_grams: f64) -> Vec<Ingredient> {
    vec![
        IngredientBuilder::new_agri("Zucker", parent_grams * 0.38)
            .bio()
            .origin(Country::EU)
            .build(),
        IngredientBuilder::new_agri("Vollmilchpulver", parent_grams * 0.27)
            .bio()
            .origin(Country::CH)
            .build(),
        IngredientBuilder::new_agri("Kakaobutter", parent_grams * 0.26)
            .bio()
            .origin(Country::EU)
            .build(),
        IngredientBuilder::new_agri("Kakaomasse", parent_grams * 0.088)
            .bio()
            .origin(Country::EU)
            .build(),
        IngredientBuilder::new_agri("Vanille", parent_grams * 0.002)
            .bio()
            .origin(Country::EU)
            .build(),
    ]
}

// =============================================================================
// Test 1: Schoggi Cookie BK — Under90% Swiss → bio_suisse_no_cross
// =============================================================================

/// Recipe: Schokoladenguetzli with dark chocolate, BK quality.
/// ~63% Swiss bio agricultural ingredients → Under90 rule applies.
/// Expected logo: bio_suisse_no_cross (BIO without Swiss cross)
#[test]
fn recipe_schoggi_cookie_bk() {
    let calculator = calculator_for(Configuration::Knospe);
    let input = InputBuilder::new()
        .vollstaendig()
        .certification_body("CH-BIO-006 (bio.inspecta AG)")
        .ingredient(
            IngredientBuilder::new_agri("Weizenmehl", 33.5)
                .bio()
                .origin(Country::CH)
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Butter", 20.0)
                .bio()
                .origin(Country::CH)
                .category("Butter")
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Schokoladewürfel", 17.0)
                .bio()
                .origin(Country::EU)
                .children(dark_chocolate_children(17.0))
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Zucker", 16.0)
                .bio()
                .origin(Country::PE)
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Eier", 11.0)
                .bio()
                .origin(Country::CH)
                .category("Eier")
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Orangenschale", 2.0)
                .bio()
                .origin(Country::EU)
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Salz", 0.5)
                .origin(Country::CH)
                .build(),
        )
        .build();

    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    // Logo: bio_suisse_no_cross (Under90% Swiss)
    assert_eq!(c.get("bio_suisse_no_cross"), Some(&true), "Expected bio_suisse_no_cross logo");
    assert_eq!(c.get("bio_suisse_regular"), None, "Should NOT have bio_suisse_regular logo");

    // Origin display per Excel "Herkunft muss angegeben werden":
    // Butter: Ja → show (CH)
    assert!(output.label.contains("Butter* (CH)"), "Butter should show origin (CH). Label: {}", output.label);
    // Eier: Ja → show (CH)
    assert!(output.label.contains("Eier* (CH)"), "Eier should show origin (CH). Label: {}", output.label);
    // Zucker: Nein
    assert!(!output.label.contains("Zucker* (PE)"), "Zucker should NOT show origin. Label: {}", output.label);
    assert!(output.label.contains("Zucker*"), "Zucker should have bio asterisk. Label: {}", output.label);
    // Weizenmehl: Nein
    assert!(!output.label.contains("Weizenmehl* (CH)"), "Weizenmehl should NOT show origin. Label: {}", output.label);
    assert!(output.label.contains("Weizenmehl*"), "Weizenmehl should have bio asterisk. Label: {}", output.label);
    // Schokoladewürfel: Nein
    assert!(!output.label.contains("Schokoladewürfel* (EU)"), "Schokoladewürfel should NOT show origin. Label: {}", output.label);
    // Orangenschale: Nein
    assert!(!output.label.contains("Orangenschale* (EU)"), "Orangenschale should NOT show origin. Label: {}", output.label);
    // Salz: Nein (non-agricultural, no bio asterisk)
    assert!(!output.label.contains("Salz (CH)"), "Salz should NOT show origin. Label: {}", output.label);
    assert!(!output.label.contains("Salz*"), "Salz should NOT have bio asterisk. Label: {}", output.label);

    // Bio asterisk legend
    assert!(output.label.contains("aus biologischer Landwirtschaft"), "Should have bio legend. Label: {}", output.label);

    // No validation errors
    assert!(output.validation_messages.is_empty(), "Expected no validation errors, got: {:?}", output.validation_messages);
}

// =============================================================================
// Test 2: Schoggi Cookie BK mit Milch — Under90% Swiss → bio_suisse_no_cross
// =============================================================================

/// Recipe: Schokoladenguetzli with milk chocolate, BK quality.
/// ~87% Swiss bio agricultural ingredients → Under90 rule applies.
/// Expected logo: bio_suisse_no_cross (BIO without Swiss cross)
#[test]
fn recipe_schoggi_cookie_bk_mit_milch() {
    let calculator = calculator_for(Configuration::Knospe);
    let input = InputBuilder::new()
        .vollstaendig()
        .certification_body("CH-BIO-006 (bio.inspecta AG)")
        .ingredient(
            IngredientBuilder::new_agri("Weizenmehl", 33.5)
                .bio()
                .origin(Country::CH)
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Butter", 22.0)
                .bio()
                .origin(Country::CH)
                .category("Butter")
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Zucker", 18.0)
                .bio()
                .origin(Country::CH)
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Milchschokoladewürfel", 17.0)
                .bio()
                .origin(Country::EU)
                .children(milk_chocolate_children(17.0))
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Eier", 9.0)
                .bio()
                .origin(Country::CH)
                .category("Eier")
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Salz", 0.5)
                .origin(Country::CH)
                .build(),
        )
        .build();

    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    // Logo: bio_suisse_no_cross (Under90% Swiss)
    assert_eq!(c.get("bio_suisse_no_cross"), Some(&true), "Expected bio_suisse_no_cross logo");
    assert_eq!(c.get("bio_suisse_regular"), None, "Should NOT have bio_suisse_regular logo");

    // Origin display per Excel:
    // Butter: Ja → show (CH)
    assert!(output.label.contains("Butter* (CH)"), "Butter should show origin (CH). Label: {}", output.label);
    // Sub-ingredient Vollmilchpulver: Ja (shown inside composite parentheses)
    assert!(output.label.contains("Vollmilchpulver (CH)"), "Vollmilchpulver should show origin (CH) inside composite. Label: {}", output.label);
    // Zucker: Nein
    assert!(!output.label.contains("Zucker* (CH)"), "Zucker should NOT show origin. Label: {}", output.label);
    // Weizenmehl: Nein
    assert!(!output.label.contains("Weizenmehl* (CH)"), "Weizenmehl should NOT show origin. Label: {}", output.label);
    // Eier: Nein (9% < 10%, eggs only trigger at >10%)
    assert!(!output.label.contains("Eier* (CH)"), "Eier should NOT show origin (9% < 10%). Label: {}", output.label);
    // Schokoladewürfel: Nein
    assert!(!output.label.contains("Milchschokoladewürfel* (EU)"), "Milchschokoladewürfel should NOT show origin. Label: {}", output.label);
    // Salz: Nein
    assert!(!output.label.contains("Salz (CH)"), "Salz should NOT show origin. Label: {}", output.label);

    // Bio asterisk legend
    assert!(output.label.contains("aus biologischer Landwirtschaft"), "Should have bio legend. Label: {}", output.label);

    // No validation errors
    assert!(output.validation_messages.is_empty(), "Expected no validation errors, got: {:?}", output.validation_messages);
}

// =============================================================================
// Test 3: Schoggi Cookie BSK mit Milch — 90-99% Swiss → bio_suisse_regular
// =============================================================================

/// Recipe: Schokoladenguetzli with milk chocolate, BSK quality (mostly Swiss).
/// ~91% Swiss bio agricultural ingredients → 90-99% rule applies.
/// Expected logo: bio_suisse_regular (BIO SUISSE with Swiss cross)
#[test]
fn recipe_schoggi_cookie_bsk_mit_milch() {
    let calculator = calculator_for(Configuration::Knospe);
    let input = InputBuilder::new()
        .vollstaendig()
        .certification_body("CH-BIO-006 (bio.inspecta AG)")
        .ingredient(
            IngredientBuilder::new_agri("Weizenmehl", 33.5)
                .bio()
                .origin(Country::CH)
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Zucker", 23.0)
                .bio()
                .origin(Country::CH)
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Butter", 22.0)
                .bio()
                .origin(Country::CH)
                .category("Butter")
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Milchschokoladewürfel", 12.0)
                .bio()
                .origin(Country::EU)
                .children(milk_chocolate_children(12.0))
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Eier", 9.0)
                .bio()
                .origin(Country::CH)
                .category("Eier")
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Salz", 0.5)
                .origin(Country::CH)
                .build(),
        )
        .build();

    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    // Logo: bio_suisse_regular (90-99% Swiss)
    assert_eq!(c.get("bio_suisse_regular"), Some(&true), "Expected bio_suisse_regular logo");
    assert_eq!(c.get("bio_suisse_no_cross"), None, "Should NOT have bio_suisse_no_cross logo");

    // Origin display per Excel (90-99% rule: show origin for Swiss agricultural ingredients):
    // Weizenmehl: Ja* → show (CH)
    assert!(output.label.contains("Weizenmehl* (CH)"), "Weizenmehl should show origin (CH). Label: {}", output.label);
    // Zucker: Ja* → show (CH)
    assert!(output.label.contains("Zucker* (CH)"), "Zucker should show origin (CH). Label: {}", output.label);
    // Butter: Ja* → show (CH)
    assert!(output.label.contains("Butter* (CH)"), "Butter should show origin (CH). Label: {}", output.label);
    // Eier: Ja* → show (CH)
    assert!(output.label.contains("Eier* (CH)"), "Eier should show origin (CH). Label: {}", output.label);
    // Schokoladewürfel: Nein (EU, not Swiss)
    assert!(!output.label.contains("Milchschokoladewürfel* (EU)"), "Milchschokoladewürfel should NOT show origin. Label: {}", output.label);
    // Sub-ingredient Vollmilchpulver: Ja* (shown inside composite)
    assert!(output.label.contains("Vollmilchpulver (CH)"), "Vollmilchpulver should show origin inside composite. Label: {}", output.label);
    // Salz: Nein (non-agricultural, should NOT show origin even though CH)
    assert!(!output.label.contains("Salz (CH)"), "Salz should NOT show origin (non-agricultural). Label: {}", output.label);

    // Bio asterisk legend
    assert!(output.label.contains("aus biologischer Landwirtschaft"), "Should have bio legend. Label: {}", output.label);

    // No validation errors
    assert!(output.validation_messages.is_empty(), "Expected no validation errors, got: {:?}", output.validation_messages);
}

// =============================================================================
// Test 4: Bärlauch Pesto BK — Under90% Swiss → bio_suisse_no_cross
// =============================================================================

/// Recipe: Bärlauchpesto with imported nuts, BK quality.
/// ~71% Swiss bio agricultural ingredients → Under90 rule applies.
/// Expected logo: bio_suisse_no_cross (BIO without Swiss cross)
#[test]
fn recipe_baerlauch_pesto_bk() {
    let calculator = calculator_for(Configuration::Knospe);
    let input = InputBuilder::new()
        .vollstaendig()
        .certification_body("CH-BIO-006 (bio.inspecta AG)")
        .ingredient(
            IngredientBuilder::new_agri("Rapsöl", 150.0)
                .bio()
                .origin(Country::CH)
                .category("Öle")
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Bärlauch", 100.0)
                .bio()
                .origin(Country::CH)
                .namensgebend()
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Mandeln", 50.0)
                .bio()
                .origin(Country::TR)
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Parmesan", 50.0)
                .bio()
                .origin(Country::IT)
                .category("Käse")
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Salz", 5.0)
                .origin(Country::EU)
                .build(),
        )
        .build();

    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    // Logo: bio_suisse_no_cross (Under90% Swiss)
    assert_eq!(c.get("bio_suisse_no_cross"), Some(&true), "Expected bio_suisse_no_cross logo");
    assert_eq!(c.get("bio_suisse_regular"), None, "Should NOT have bio_suisse_regular logo");

    // Origin display per Excel:
    // Bärlauch: Ja → show (CH) — namensgebende Zutat (name-giving ingredient of "Bärlauch Pesto")
    // Note: namensgebend also triggers percentage display
    assert!(output.label.contains("Bärlauch*") && output.label.contains("(CH)"),
        "Bärlauch should show origin (CH). Label: {}", output.label);
    // Rapsöl: Ja → show (CH) — Swiss agricultural with category, ≥10%
    assert!(output.label.contains("Rapsöl* (CH)"), "Rapsöl should show origin (CH). Label: {}", output.label);
    // Parmesan: Ja → show (IT) — dairy category, always shows
    assert!(output.label.contains("Parmesan* (IT)"), "Parmesan should show origin (IT) as dairy. Label: {}", output.label);
    // Mandeln: Nein — not Swiss, no special category
    assert!(!output.label.contains("Mandeln* (TR)"), "Mandeln should NOT show origin. Label: {}", output.label);
    assert!(output.label.contains("Mandeln*"), "Mandeln should have bio asterisk. Label: {}", output.label);
    // Salz: Nein — non-agricultural
    assert!(!output.label.contains("Salz (EU)"), "Salz should NOT show origin. Label: {}", output.label);

    // Bio asterisk legend
    assert!(output.label.contains("aus biologischer Landwirtschaft"), "Should have bio legend. Label: {}", output.label);

    // No validation errors
    assert!(output.validation_messages.is_empty(), "Expected no validation errors, got: {:?}", output.validation_messages);
}

// =============================================================================
// Test 5: Bärlauch Pesto BSK — 90-99% Swiss → bio_suisse_regular
// =============================================================================

/// Recipe: Bärlauchpesto with Swiss nuts, BSK quality (mostly Swiss).
/// ~90.5% Swiss bio agricultural ingredients → 90-99% rule applies.
/// Expected logo: bio_suisse_regular (BIO SUISSE with Swiss cross)
#[test]
fn recipe_baerlauch_pesto_bsk() {
    let calculator = calculator_for(Configuration::Knospe);
    let input = InputBuilder::new()
        .vollstaendig()
        .certification_body("CH-BIO-006 (bio.inspecta AG)")
        .ingredient(
            IngredientBuilder::new_agri("Rapsöl", 175.0)
                .bio()
                .origin(Country::CH)
                .category("Öle")
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Bärlauch", 150.0)
                .bio()
                .origin(Country::CH)
                .namensgebend()
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Baumnüsse", 150.0)
                .bio()
                .origin(Country::CH)
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Parmesan", 50.0)
                .bio()
                .origin(Country::IT)
                .category("Käse")
                .build(),
        )
        .ingredient(
            IngredientBuilder::new_agri("Salz", 5.0)
                .origin(Country::EU)
                .build(),
        )
        .build();

    let output = calculator.execute(input);
    let c = &output.conditional_elements;

    // Logo: bio_suisse_regular (90-99% Swiss)
    assert_eq!(c.get("bio_suisse_regular"), Some(&true), "Expected bio_suisse_regular logo");
    assert_eq!(c.get("bio_suisse_no_cross"), None, "Should NOT have bio_suisse_no_cross logo");

    // Origin display per Excel (90-99% rule: show origin for Swiss agricultural ingredients only):
    // Rapsöl: Ja* → show (CH)
    assert!(output.label.contains("Rapsöl* (CH)"), "Rapsöl should show origin (CH). Label: {}", output.label);
    // Bärlauch: Ja* → show (CH) — also namensgebend so includes percentage
    assert!(output.label.contains("Bärlauch*") && output.label.contains("(CH)"),
        "Bärlauch should show origin (CH). Label: {}", output.label);
    // Baumnüsse: Ja* → show (CH)
    assert!(output.label.contains("Baumnüsse* (CH)"), "Baumnüsse should show origin (CH). Label: {}", output.label);
    // Parmesan: Nein — not Swiss, 90-99% rule only shows Swiss origins
    assert!(!output.label.contains("Parmesan* (IT)"), "Parmesan should NOT show origin (not Swiss, 90-99% rule). Label: {}", output.label);
    assert!(output.label.contains("Parmesan*"), "Parmesan should have bio asterisk. Label: {}", output.label);
    // Salz: Nein — non-agricultural
    assert!(!output.label.contains("Salz (EU)"), "Salz should NOT show origin. Label: {}", output.label);

    // Bio asterisk legend
    assert!(output.label.contains("aus biologischer Landwirtschaft"), "Should have bio legend. Label: {}", output.label);

    // No validation errors
    assert!(output.validation_messages.is_empty(), "Expected no validation errors, got: {:?}", output.validation_messages);
}
