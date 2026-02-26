use super::*;

#[test]
fn calculate_swiss_agricultural_percentage_100_percent() {
    let ingredients = vec![
        IngredientBuilder::new_agri("Hafer", 600.0).origin(Country::CH).build(),
        IngredientBuilder::new_agri("Weizenmehl", 400.0).origin(Country::CH).build(),
    ];

    let percentage = calculate_swiss_agricultural_percentage(&ingredients);
    assert_eq!(percentage, 100.0);
}

#[test]
fn calculate_swiss_agricultural_percentage_90_percent() {
    let ingredients = vec![
        IngredientBuilder::new_agri("Hafer", 500.0).origin(Country::CH).build(),
        IngredientBuilder::new_agri("Weizenmehl", 400.0).origin(Country::CH).build(),
        IngredientBuilder::new_agri("Olivenöl", 100.0).origin(Country::EU).build(),
    ];

    let percentage = calculate_swiss_agricultural_percentage(&ingredients);
    assert_eq!(percentage, 90.0);
}

#[test]
fn calculate_swiss_agricultural_percentage_with_non_agricultural() {
    let ingredients = vec![
        IngredientBuilder::new_agri("Hafer", 500.0).origin(Country::CH).build(),
        IngredientBuilder::new_agri("Salz", 500.0).origin(Country::EU).build(),
    ];

    let percentage = calculate_swiss_agricultural_percentage(&ingredients);
    // Only Hafer is agricultural (500g Swiss), Salz is non-agricultural (ignored in calculation)
    // Swiss agricultural: 500g, Total agricultural: 500g -> 100%
    assert_eq!(percentage, 100.0);
}

#[test]
fn test_agricultural_lookup() {
    // Test agricultural ingredients
    assert_eq!(lookup_agricultural("Hafer"), true);
    assert_eq!(lookup_agricultural("Weizenmehl"), true);
    assert_eq!(lookup_agricultural("Olivenöl"), true);
    assert_eq!(lookup_agricultural("Milch"), true);

    // Test non-agricultural ingredients
    assert_eq!(lookup_agricultural("Salz"), false);
    assert_eq!(lookup_agricultural("Wasser"), false);

    // Test unknown ingredient (should default to true)
    assert_eq!(lookup_agricultural("UnknownIngredient"), true);
}

#[test]
fn knospe_certified_percentage_no_agricultural_returns_100() {
    // When there are no agricultural ingredients, knospe certified percentage should be 100%
    let ingredients = vec![
        IngredientBuilder::new("Salz", 500.0).agricultural(false).build(),
        IngredientBuilder::new("Wasser", 500.0).agricultural(false).build(),
    ];

    let percentage = calculate_knospe_certified_percentage(&ingredients);
    assert_eq!(percentage, 100.0);
}

#[test]
fn bio_ch_certified_percentage_no_agricultural_returns_100() {
    // When there are no agricultural ingredients, bio_ch certified percentage should be 100%
    let ingredients = vec![
        IngredientBuilder::new("Salz", 500.0).agricultural(false).build(),
        IngredientBuilder::new("Wasser", 500.0).agricultural(false).build(),
    ];

    let percentage = calculate_bio_ch_certified_percentage(&ingredients);
    assert_eq!(percentage, 100.0);
}

#[test]
fn format_percentage_boundary_values() {
    assert_eq!(format_percentage(0.0), "0%");
    assert_eq!(format_percentage(0.4), "<1%");
    assert_eq!(format_percentage(0.5), "1%");
    assert_eq!(format_percentage(100.0), "100%");
}
