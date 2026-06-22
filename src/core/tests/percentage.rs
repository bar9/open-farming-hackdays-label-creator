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

// --- Percentage-mode composites (children entered as % of the parent total) ---

#[test]
fn resolve_percentages_basic() {
    // Parent 200g with children 60% / 40% -> 120g / 80g.
    let parent = IngredientBuilder::new("Sauce", 200.0)
        .children(vec![
            IngredientBuilder::new("Tomate", 60.0).unit(AmountUnit::Percent).build(),
            IngredientBuilder::new("Wasser", 40.0).unit(AmountUnit::Percent).build(),
        ])
        .build();

    let resolved = parent.resolve_percentages();
    let kids = resolved.children.as_ref().unwrap();
    assert_eq!(kids[0].amount, 120.0);
    assert_eq!(kids[0].unit, AmountUnit::Gram);
    assert_eq!(kids[1].amount, 80.0);
    // Percentages sum to 100% -> parent weight equals the entered total.
    assert_eq!(resolved.computed_amount(), 200.0);
}

#[test]
fn resolve_percentages_non_100_sum() {
    // 60% / 30% (sum 90%) -> 120g / 60g; parent weighs the declared parts (180g).
    let parent = IngredientBuilder::new("Sauce", 200.0)
        .children(vec![
            IngredientBuilder::new("Tomate", 60.0).unit(AmountUnit::Percent).build(),
            IngredientBuilder::new("Wasser", 30.0).unit(AmountUnit::Percent).build(),
        ])
        .build();

    let resolved = parent.resolve_percentages();
    let kids = resolved.children.as_ref().unwrap();
    assert_eq!(kids[0].amount, 120.0);
    assert_eq!(kids[1].amount, 60.0);
    assert_eq!(resolved.computed_amount(), 180.0);
}

#[test]
fn percentage_mode_parent_is_authoritative_before_resolution() {
    // On the unresolved tree, a percentage-mode parent's weight is its own total,
    // not the meaningless sum of the percentage values.
    let parent = IngredientBuilder::new("Sauce", 200.0)
        .children(vec![
            IngredientBuilder::new("Tomate", 60.0).unit(AmountUnit::Percent).build(),
            IngredientBuilder::new("Wasser", 40.0).unit(AmountUnit::Percent).build(),
        ])
        .build();
    assert_eq!(parent.computed_amount(), 200.0);
}

#[test]
fn scale_recursive_leaves_percentage_children() {
    // Scaling a percentage-mode composite doubles the parent total and leaves the
    // child percentages untouched (their derived grams double automatically).
    let mut parent = IngredientBuilder::new("Sauce", 200.0)
        .children(vec![
            IngredientBuilder::new("Tomate", 60.0).unit(AmountUnit::Percent).build(),
            IngredientBuilder::new("Wasser", 40.0).unit(AmountUnit::Percent).build(),
        ])
        .build();

    parent.scale_recursive(2.0);
    assert_eq!(parent.amount, 400.0);
    let kids = parent.children.as_ref().unwrap();
    assert_eq!(kids[0].amount, 60.0); // still 60%
    assert_eq!(kids[1].amount, 40.0);
    // Resolved grams reflect the new total.
    let resolved = parent.resolve_percentages();
    assert_eq!(resolved.children.as_ref().unwrap()[0].amount, 240.0);
}

#[test]
fn resolve_percentages_idempotent_on_absolute_tree() {
    let parent = IngredientBuilder::new("Sauce", 0.0)
        .children(vec![
            IngredientBuilder::new("Tomate", 120.0).build(),
            IngredientBuilder::new("Wasser", 80.0).build(),
        ])
        .build();
    let resolved = parent.resolve_percentages();
    assert_eq!(resolved.children.as_ref().unwrap()[0].amount, 120.0);
    assert_eq!(resolved.computed_amount(), 200.0);
}
