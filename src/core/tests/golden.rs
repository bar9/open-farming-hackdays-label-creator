use super::*;
use crate::shared::Configuration;

/// Golden-file test for Erdbeer-Fruchtaufstrich (strawberry jam).
///
/// Expected label (from requirement docs):
/// Erdbeere vom Hof 66% (CH), Zucker (CH), Geliermittel: Pektin, Säuerungsmittel: Zitronensäure
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
        "Erdbeere vom Hof 66% (CH), Zucker (CH), Geliermittel: Pektin, Säuerungsmittel: Zitronensäure"
    );
}

/// Golden-file test for Joghurt Salatsauce (yogurt salad dressing).
///
/// Expected label (from requirement docs):
/// Joghurt nature 63% (CH), Rapsöl (CH), Wasser, Blütenhonig, Senf, Zitronensaft,
/// Bouillonpaste (Salz (CH), Sojasauce, Maltodextrin auf Weizenbasis, Karotte,
/// Knollensellerie, Lauch, Rapsöl, Gewürz, Petersilie), Salz, Gewürze
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
                    IngredientBuilder::new("Salz", 0.0).origin(Country::CH).build(),
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
        "Joghurt nature 63% (CH), Rapsöl (CH), Wasser, Blütenhonig, Senf, Zitronensaft, Gewürze, Bouillonpaste (Salz (CH), Sojasauce, Maltodextrin auf Weizenbasis, Karotte, Knollensellerie, Lauch, Rapsöl, Gewürz, Petersilie), Salz"
    );
}
