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

#[test]
fn test_swiss_percentage_with_children() {
    // Composite ingredient: 100g Bouillon with 60g CH salt + 40g DE pepper
    let ingredients = vec![
        IngredientBuilder::new("Bouillon", 100.0)
            .children(vec![
                IngredientBuilder::new("Salz", 60.0).agricultural(false).origin(Country::CH).build(),
                IngredientBuilder::new("Pfeffer", 40.0).origin(Country::DE).build(),
            ])
            .build(),
    ];
    // Only Pfeffer is agricultural (40g total agricultural, 0g Swiss agricultural)
    let percentage = calculate_swiss_agricultural_percentage(&ingredients);
    assert_eq!(percentage, 0.0);
}

#[test]
fn test_knospe_percentage_with_children() {
    // Composite with mixed bio leaves
    let ingredients = vec![
        IngredientBuilder::new("Mischung", 100.0)
            .children(vec![
                IngredientBuilder::new("Hafer", 70.0).bio().build(),
                IngredientBuilder::new("Zucker", 30.0).build(), // not bio
            ])
            .build(),
    ];
    let percentage = calculate_knospe_certified_percentage(&ingredients);
    assert_eq!(percentage, 70.0);
}

#[test]
fn test_bio_ch_percentage_with_children() {
    // Composite with mixed bio_ch leaves
    let ingredients = vec![
        IngredientBuilder::new("Mischung", 100.0)
            .children(vec![
                IngredientBuilder::new("Milch", 80.0).bio_ch().build(),
                IngredientBuilder::new("Sahne", 20.0).build(), // not bio_ch
            ])
            .build(),
    ];
    let percentage = calculate_bio_ch_certified_percentage(&ingredients);
    assert_eq!(percentage, 80.0);
}

#[test]
fn test_percentage_with_override() {
    // Override node treated as single unit, not decomposed into leaves
    let ingredients = vec![
        IngredientBuilder::new("Bouillon", 100.0)
            .bio()
            .override_children()
            .children(vec![
                IngredientBuilder::new("Salz", 60.0).agricultural(false).build(),
                IngredientBuilder::new("Pfeffer", 40.0).build(), // not bio
            ])
            .build(),
    ];
    // With override, the parent node (100g, bio, agricultural=true by default) is the leaf
    let percentage = calculate_knospe_certified_percentage(&ingredients);
    assert_eq!(percentage, 100.0);
}
