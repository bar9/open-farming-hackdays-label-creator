// DEC-1 — Deklaration von glutenhaltigem Getreide.
//
// Mehl ist ein Allergen; deklarationsrechtlich muss die *Getreidesorte*
// genannt werden. Im Alltag wird Weizenmehl aber oft nur «Mehl» genannt.
// Wählt die Nutzerin den hinterlegten Vorschlag «Mehl (Weizenmehl)», muss die
// Zutatenliste darum «Weizenmehl» ausgeben.
//
// Der Normalfall bleibt: ausgegeben wird, was eingegeben wurde. Diese Datei
// prüft die Ausnahme auf Label-Ebene (was die Nutzerin tatsächlich sieht);
// die Namensauflösung selbst ist in `model.rs` unit-getestet.

use super::*;
use crate::shared::Configuration;

/// Ingredient as it is stored after picking the curated "Mehl (Weizenmehl)"
/// suggestion: the pane substitutes the declaration name and keeps the
/// canonical for the allergen/agricultural lookups.
fn selected_mehl_suggestion(amount: f64) -> Ingredient {
    let name = crate::model::declaration_name("Mehl", Some("Weizenmehl"));
    IngredientBuilder::new(&name, amount)
        .canonical("Weizenmehl")
        .allergen()
        .build()
}

// The core acceptance criterion: the label says "Weizenmehl", not "Mehl".
#[test]
fn mehl_selection_declares_weizenmehl_on_label() {
    let calculator = setup_simple_calculator();
    let output = calculator.execute(
        InputBuilder::new()
            .ingredient(selected_mehl_suggestion(500.0))
            .build(),
    );

    assert!(
        output.label.contains("Weizenmehl"),
        "label must declare the cereal species. Label: {}",
        output.label
    );
}

// Must hold in all three instances (Lebensmittelrecht, Bio-V, Knospe): the
// rule is declaration law, not a bio-scheme detail.
#[test]
fn weizenmehl_declaration_holds_in_all_configurations() {
    for config in [
        Configuration::Conventional,
        Configuration::Bio,
        Configuration::Knospe,
    ] {
        let calculator = calculator_for(config);
        let output = calculator.execute(
            InputBuilder::new()
                .ingredient(selected_mehl_suggestion(500.0))
                .build(),
        );
        assert!(
            output.label.contains("Weizenmehl"),
            "config {:?} must declare 'Weizenmehl'. Label: {}",
            config,
            output.label
        );
    }
}

// Weizenmehl is gluten-containing, so it stays flagged as an allergen (the
// label renderer bolds allergens). Substituting the name must not lose this.
#[test]
fn declared_weizenmehl_keeps_allergen_flag() {
    let ingredient = selected_mehl_suggestion(500.0);
    assert!(ingredient.is_allergen, "Weizenmehl must remain an allergen");
    assert!(
        crate::model::lookup_allergen("Weizenmehl"),
        "food_db must flag Weizenmehl as an allergen"
    );
}

// Specific flours are entered directly and must pass through untouched —
// "Dinkelmehl" must never be rewritten to "Weizenmehl".
#[test]
fn specific_flours_are_not_rewritten() {
    let calculator = setup_simple_calculator();
    for flour in ["Dinkelmehl", "Roggenmehl"] {
        let output = calculator.execute(
            InputBuilder::new()
                .ingredient(IngredientBuilder::new(flour, 500.0).allergen().build())
                .build(),
        );
        assert!(
            output.label.contains(flour),
            "'{}' must appear unchanged. Label: {}",
            flour,
            output.label
        );
        assert!(
            !output.label.contains("Weizenmehl"),
            "'{}' must not be rewritten to Weizenmehl. Label: {}",
            flour,
            output.label
        );
    }
}

// The substitution is scoped to gluten cereals. Other curated aliases keep the
// term the user typed, so "Ei" must not surface as "Hühnerei ganz".
#[test]
fn other_aliases_still_print_the_typed_term() {
    let calculator = setup_simple_calculator();
    let name = crate::model::declaration_name("Ei", Some("Hühnerei ganz"));
    let output = calculator.execute(
        InputBuilder::new()
            .ingredient(
                IngredientBuilder::new(&name, 100.0)
                    .canonical("Hühnerei ganz")
                    .allergen()
                    .build(),
            )
            .build(),
    );

    assert!(
        output.label.contains("Ei"),
        "typed term should appear. Label: {}",
        output.label
    );
    assert!(
        !output.label.contains("Hühnerei ganz"),
        "alias must not be expanded to its canonical. Label: {}",
        output.label
    );
}

// R-19 (Testing Guetzlimischung, 29.10.2026; decision 11.11.2026): commas
// inside a BLV name would read as separate ingredients. They are printed as
// spaces: «Kakaopulver, entölt, ohne Zucker» → «Kakaopulver entölt ohne Zucker».
#[test]
fn commas_in_ingredient_names_become_spaces_on_the_label() {
    let calculator = setup_simple_calculator();
    let output = calculator.execute(
        InputBuilder::new()
            .ingredient(IngredientBuilder::new("Mailänderli, hausgemacht", 525.0).build())
            .build(),
    );
    assert!(
        output.label.contains("Mailänderli hausgemacht"),
        "top-level name comma-free. Label: {}",
        output.label
    );

    let sables = IngredientBuilder::new("Schokoladensablés", 420.0)
        .children(vec![
            IngredientBuilder::new("Weizenmehl", 200.0)
                .allergen()
                .build(),
            IngredientBuilder::new("Kakaopulver, entölt, ohne Zucker", 10.0).build(),
        ])
        .build();
    let composites = sables.composites();
    assert_eq!(
        composites,
        " (<b>Weizenmehl</b>, Kakaopulver entölt ohne Zucker)"
    );
}

#[test]
fn label_name_keeps_decimal_commas() {
    assert_eq!(label_name("Vollmilch, 3,5 % Fett"), "Vollmilch 3,5 % Fett");
    assert_eq!(
        label_name("Kakaopulver, entölt, ohne Zucker"),
        "Kakaopulver entölt ohne Zucker"
    );
    assert_eq!(
        label_name("Mehl, hell (Durchschnitt)"),
        "Mehl hell (Durchschnitt)"
    );
    assert_eq!(label_name("Zucker"), "Zucker");
}

// R-17: the FAQ recommends Manuela's Variante 1 (Testing Guetzlimischung
// 29.10.2026): each variety is a composite with its own ingredients in grams.
// Same ingredients in different varieties are not merged, so allergen sufferers
// see which biscuit contains what. Recipe from her shared link.
#[test]
fn guetzli_mix_variante_1_keeps_each_variety_with_its_own_ingredients() {
    let calculator = calculator_for(Configuration::Conventional);
    let output = calculator.execute(
        InputBuilder::new()
            .ingredient(
                IngredientBuilder::new("Mailänderli, hausgemacht", 525.0)
                    .children(vec![
                        IngredientBuilder::new("Weizenmehl", 250.0)
                            .allergen()
                            .build(),
                        IngredientBuilder::new("Butter", 120.0).allergen().build(),
                        IngredientBuilder::new("Eier", 120.0).allergen().build(),
                        IngredientBuilder::new("Milch", 20.0).allergen().build(),
                        IngredientBuilder::new("Zitrone", 10.0).build(),
                        IngredientBuilder::new("Salz", 5.0).build(),
                    ])
                    .build(),
            )
            .ingredient(
                IngredientBuilder::new("Kokosmakrone", 315.0)
                    .children(vec![
                        IngredientBuilder::new("Kokosnuss", 130.0).build(),
                        IngredientBuilder::new("Zucker", 100.0).build(),
                        IngredientBuilder::new("Eiweiss", 80.0).allergen().build(),
                        IngredientBuilder::new("Salz", 5.0).build(),
                    ])
                    .build(),
            )
            .build(),
    );
    let label = &output.label;
    assert!(
        label.contains("Mailänderli hausgemacht (<b>Weizenmehl</b>, <b>Butter</b>, <b>Eier</b>, <b>Milch</b>, Zitrone, Salz)"),
        "Label: {label}"
    );
    assert!(
        label.contains("Kokosmakrone (Kokosnuss, Zucker, <b>Eiweiss</b>, Salz)"),
        "Label: {label}"
    );
    // Salz appears in both varieties, not merged into one top-level entry.
    assert_eq!(label.matches("Salz").count(), 2, "Label: {label}");
}
