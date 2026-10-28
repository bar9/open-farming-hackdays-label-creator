use super::*;
use crate::shared::Configuration;

/// Golden-file test for Erdbeer-Fruchtaufstrich (strawberry jam).
///
/// Source: `requirements/2026_01_27 Erdbeer-Fruchtaufstrich.docx`, which shows
/// «Erdbeere vom Hof 66% (CH), Zucker (CH), …». Deliberate deviation since
/// `fc654ad` (26.08.2026): a declared origin is printed only when a rule requires
/// it, so Zucker (28%, below the AP7.1 50% threshold) loses its voluntary «(CH)».
/// Voluntary origins are added by hand after copying (hint under the label).
///
/// 250g final product (manual total — water loss during cooking).
#[test]
fn golden_erdbeer_fruchtaufstrich() {
    let calculator = calculator_for(Configuration::Conventional);
    let input = InputBuilder::new()
        .ingredient(
            IngredientBuilder::new("Erdbeere vom Hof", 165.0)
                .namensgebend()
                .origin(Country::CH)
                .build(),
        )
        .ingredient(
            IngredientBuilder::new("Zucker", 70.0)
                .origin(Country::CH)
                .build(),
        )
        .ingredient(
            IngredientBuilder::new("Geliermittel: Pektin", 10.0)
                .agricultural(false)
                .build(),
        )
        .ingredient(
            IngredientBuilder::new("Säuerungsmittel: Zitronensäure", 5.0)
                .agricultural(false)
                .build(),
        )
        .total(250.0)
        .build();

    let output = calculator.execute(input);
    assert_eq!(
        output.label,
        "Erdbeere vom Hof 66% (CH), Zucker, Geliermittel: Pektin, Säuerungsmittel: Zitronensäure"
    );
}

/// Golden-file test for Joghurt Salatsauce (yogurt salad dressing).
///
/// Source: `requirements/2026_01_27 Joghurt Salatsauce Testing.docx`. Two
/// deliberate deviations from that January document:
/// - Rapsöl and the Salz inside Bouillonpaste lose their voluntary «(CH)»
///   (`fc654ad`, origin only when a rule requires it).
/// - The document lists «…, Salz, Gewürze» at the end; ingredients are now
///   sorted strictly by weight, so Gewürze (9.5 g) precede Bouillonpaste (9 g)
///   and Salz (8 g).
///
/// 450g product. Bouillonpaste is a composite ingredient with 9 children.
#[test]
fn golden_joghurt_salatsauce() {
    let calculator = calculator_for(Configuration::Conventional);
    let input = InputBuilder::new()
        .ingredient(
            IngredientBuilder::new("Joghurt nature", 283.5)
                .namensgebend()
                .origin(Country::CH)
                .build(),
        )
        .ingredient(
            IngredientBuilder::new("Rapsöl", 50.0)
                .origin(Country::CH)
                .build(),
        )
        .ingredient(IngredientBuilder::new("Wasser", 40.0).build())
        .ingredient(IngredientBuilder::new("Blütenhonig", 25.0).build())
        .ingredient(IngredientBuilder::new("Senf", 15.0).build())
        .ingredient(IngredientBuilder::new("Zitronensaft", 10.0).build())
        .ingredient(
            IngredientBuilder::new("Bouillonpaste", 9.0)
                .children(vec![
                    IngredientBuilder::new("Salz", 0.0)
                        .origin(Country::CH)
                        .build(),
                    IngredientBuilder::new("Sojasauce", 0.0).build(),
                    IngredientBuilder::new("Maltodextrin auf Weizenbasis", 0.0).build(),
                    IngredientBuilder::new("Karotte", 0.0).build(),
                    IngredientBuilder::new("Knollensellerie", 0.0).build(),
                    IngredientBuilder::new("Lauch", 0.0).build(),
                    IngredientBuilder::new("Rapsöl", 0.0).build(),
                    IngredientBuilder::new("Gewürz", 0.0).build(),
                    IngredientBuilder::new("Petersilie", 0.0).build(),
                ])
                .build(),
        )
        .ingredient(IngredientBuilder::new("Salz", 8.0).build())
        .ingredient(IngredientBuilder::new("Gewürze", 9.5).build())
        .total(450.0)
        .build();

    let output = calculator.execute(input);
    // Gewürze (9.5g) sorts before Bouillonpaste (9.0g) by weight.
    assert_eq!(
        output.label,
        "Joghurt nature 63% (CH), Rapsöl, Wasser, Blütenhonig, Senf, Zitronensaft, Gewürze, Bouillonpaste (Salz, Sojasauce, Maltodextrin auf Weizenbasis, Karotte, Knollensellerie, Lauch, Rapsöl, Gewürz, Petersilie), Salz"
    );
}
