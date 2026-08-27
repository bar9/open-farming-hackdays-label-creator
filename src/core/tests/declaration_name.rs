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
