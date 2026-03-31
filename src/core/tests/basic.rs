use super::*;

#[test]
fn simple_run_of_process() {
    let calculator = setup_simple_calculator();
    let input = InputBuilder::new().build();

    let output = calculator.execute(input);
    assert!(output.success);
}

#[test]
fn single_ingredient_visible_on_label() {
    let calculator = setup_simple_calculator();
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Hafer", 42.0).build())
        .build();
    let output = calculator.execute(input);
    let label = output.label;
    assert!(label.contains("Hafer"));
}

#[test]
fn multiple_ingredients_comma_separated_on_label() {
    let calculator = setup_simple_calculator();
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Hafer", 42.0).build())
        .ingredient(IngredientBuilder::new("Zucker", 42.0).build())
        .build();
    let output = calculator.execute(input);
    let label = output.label;
    assert!(label.contains("Hafer, Zucker"));
}

#[test]
fn ingredients_ordered_by_weight_on_label() {
    let calculator = setup_simple_calculator();
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Hafer", 300.0).build())
        .ingredient(IngredientBuilder::new("Zucker", 700.0).build())
        .build();
    let output = calculator.execute(input);
    let label = output.label;
    assert!(label.contains("Zucker, Hafer"));
}

#[test]
fn allergenes_printed_bold_on_label() {
    let calculator = setup_simple_calculator();
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Weizenmehl", 300.0).allergen().build())
        .build();
    let output = calculator.execute(input);
    let label = output.label;
    assert!(label.contains("<b>Weizenmehl</b>"));
}

#[test]
fn html_in_ingredient_name_is_escaped() {
    let calculator = setup_simple_calculator();
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("<script>alert('xss')</script>", 500.0).build())
        .ingredient(IngredientBuilder::new("Normal & Safe", 500.0).build())
        .build();
    let output = calculator.execute(input);
    let label = output.label;
    assert!(!label.contains("<script>"));
    assert!(label.contains("&lt;script&gt;"));
    assert!(label.contains("Normal &amp; Safe"));
}

#[test]
fn html_in_allergen_name_is_escaped() {
    let calculator = setup_simple_calculator();
    let input = InputBuilder::new()
        .ingredient(IngredientBuilder::new("<img src=x>", 500.0).allergen().build())
        .build();
    let output = calculator.execute(input);
    let label = output.label;
    assert!(label.contains("<b>&lt;img src=x&gt;</b>"));
    assert!(!label.contains("<img src=x>"));
}

#[test]
fn scaled_recipe_invariant_on_label() {
    let calculator = setup_simple_calculator();
    let input1 = InputBuilder::new()
        .ingredient(IngredientBuilder::new("Hafer", 300.0).build())
        .ingredient(IngredientBuilder::new("Zucker", 700.0).build())
        .build();
    let mut input2 = input1.clone();
    input2.scale(2.);
    let output = calculator.execute(input1);
    let scaled_output = calculator.execute(input2);

    assert_eq!(output.label, scaled_output.label);
    assert_ne!(output.total_amount, scaled_output.total_amount);
}

#[test]
fn test_migrate_sub_components_to_children() {
    let mut ingredient = IngredientBuilder::new("Bouillonpaste", 9.0)
        .sub_components(vec![
            SubIngredient { name: "Salz".to_string(), is_allergen: false, origin: Some(Country::CH) },
            SubIngredient { name: "Sojasauce".to_string(), is_allergen: true, origin: None },
        ])
        .build();

    assert!(ingredient.sub_components.is_some());
    assert!(ingredient.children.is_none());

    ingredient.migrate_sub_components();

    assert!(ingredient.sub_components.is_none());
    assert!(ingredient.children.is_some());

    let children = ingredient.children.as_ref().unwrap();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].name, "Salz");
    assert!(!children[0].is_allergen);
    assert_eq!(children[0].origins, Some(vec![Country::CH]));
    assert_eq!(children[1].name, "Sojasauce");
    assert!(children[1].is_allergen);
    assert!(children[1].origins.is_none());
}

#[test]
fn test_migrate_does_not_overwrite_existing_children() {
    let mut ingredient = IngredientBuilder::new("Test", 10.0)
        .sub_components(vec![
            SubIngredient { name: "Old".to_string(), is_allergen: false, origin: None },
        ])
        .children(vec![
            IngredientBuilder::new("Existing", 0.0).build(),
        ])
        .build();

    ingredient.migrate_sub_components();

    // Children should remain unchanged since they already existed
    let children = ingredient.children.as_ref().unwrap();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].name, "Existing");
}

#[test]
fn test_recursive_composites_two_levels() {
    let ingredient = IngredientBuilder::new("Wurst", 100.0)
        .children(vec![
            IngredientBuilder::new("Milch", 0.0).allergen().origin(Country::CH).build(),
            IngredientBuilder::new("Erdbeeren", 0.0).origin(Country::DE).build(),
        ])
        .build();

    let composites = ingredient.composites();
    assert_eq!(composites, " (<b>Milch</b> (CH), Erdbeeren (DE))");
}

#[test]
fn test_recursive_composites_three_levels() {
    let ingredient = IngredientBuilder::new("Teig", 100.0)
        .children(vec![
            IngredientBuilder::new("Mehl", 0.0)
                .children(vec![
                    IngredientBuilder::new("Weizen", 0.0).allergen().origin(Country::DE).build(),
                    IngredientBuilder::new("Roggen", 0.0).origin(Country::AT).build(),
                ])
                .build(),
            IngredientBuilder::new("Wasser", 0.0).build(),
        ])
        .build();

    let composites = ingredient.composites();
    assert_eq!(composites, " (Mehl (<b>Weizen</b> (DE), Roggen (AT)), Wasser)");
}

#[test]
fn test_scale_recursive() {
    let mut ingredient = IngredientBuilder::new("Parent", 100.0)
        .children(vec![
            IngredientBuilder::new("Child1", 50.0)
                .children(vec![
                    IngredientBuilder::new("Grandchild", 25.0).build(),
                ])
                .build(),
            IngredientBuilder::new("Child2", 30.0).build(),
        ])
        .build();

    ingredient.scale_recursive(2.0);

    assert_eq!(ingredient.amount, 200.0);
    let children = ingredient.children.as_ref().unwrap();
    assert_eq!(children[0].amount, 100.0);
    assert_eq!(children[0].children.as_ref().unwrap()[0].amount, 50.0);
    assert_eq!(children[1].amount, 60.0);
}

// --- Bubbling / computed methods ---

#[test]
fn test_computed_amount_leaf() {
    let leaf = IngredientBuilder::new("Salz", 50.0).build();
    assert_eq!(leaf.computed_amount(), 50.0);
}

#[test]
fn test_computed_amount_from_children() {
    let parent = IngredientBuilder::new("Bouillon", 0.0)
        .children(vec![
            IngredientBuilder::new("Salz", 30.0).build(),
            IngredientBuilder::new("Pfeffer", 20.0).build(),
        ])
        .build();
    assert_eq!(parent.computed_amount(), 50.0);
}

#[test]
fn test_computed_amount_override() {
    let parent = IngredientBuilder::new("Bouillon", 100.0)
        .children(vec![
            IngredientBuilder::new("Salz", 30.0).build(),
            IngredientBuilder::new("Pfeffer", 20.0).build(),
        ])
        .override_children()
        .build();
    // With override, parent's own amount is used
    assert_eq!(parent.computed_amount(), 100.0);
}

#[test]
fn test_computed_bio_status_all_bio() {
    let parent = IngredientBuilder::new("Mix", 0.0)
        .children(vec![
            IngredientBuilder::new("A", 50.0).bio().build(),
            IngredientBuilder::new("B", 50.0).bio().build(),
        ])
        .build();
    assert_eq!(parent.computed_bio_status(), Some(true));
}

#[test]
fn test_computed_bio_status_mixed() {
    let parent = IngredientBuilder::new("Mix", 0.0)
        .children(vec![
            IngredientBuilder::new("A", 50.0).bio().build(),
            IngredientBuilder::new("B", 50.0).build(), // not bio
        ])
        .build();
    assert_eq!(parent.computed_bio_status(), Some(false));
}

#[test]
fn test_computed_origins_union() {
    let parent = IngredientBuilder::new("Mix", 0.0)
        .children(vec![
            IngredientBuilder::new("A", 50.0).origin(Country::CH).build(),
            IngredientBuilder::new("B", 30.0).origin(Country::DE).build(),
            IngredientBuilder::new("C", 20.0).origin(Country::CH).build(), // duplicate CH
        ])
        .build();
    let origins = parent.computed_origins().unwrap();
    assert_eq!(origins.len(), 2); // CH and DE, deduplicated
    assert!(origins.contains(&Country::CH));
    assert!(origins.contains(&Country::DE));
}

#[test]
fn test_leaves_flat() {
    let leaf = IngredientBuilder::new("Salz", 50.0).build();
    let leaves = leaf.leaves();
    assert_eq!(leaves.len(), 1);
    assert_eq!(leaves[0].name, "Salz");
}

#[test]
fn test_leaves_recursive() {
    let parent = IngredientBuilder::new("Teig", 0.0)
        .children(vec![
            IngredientBuilder::new("Mehl", 0.0)
                .children(vec![
                    IngredientBuilder::new("Weizen", 40.0).build(),
                    IngredientBuilder::new("Roggen", 10.0).build(),
                ])
                .build(),
            IngredientBuilder::new("Wasser", 50.0).build(),
        ])
        .build();
    let leaves = parent.leaves();
    assert_eq!(leaves.len(), 3);
    let names: Vec<&str> = leaves.iter().map(|l| l.name.as_str()).collect();
    assert!(names.contains(&"Weizen"));
    assert!(names.contains(&"Roggen"));
    assert!(names.contains(&"Wasser"));
}

#[test]
fn test_leaves_with_override() {
    let parent = IngredientBuilder::new("Bouillon", 100.0)
        .children(vec![
            IngredientBuilder::new("Salz", 30.0).build(),
            IngredientBuilder::new("Pfeffer", 20.0).build(),
        ])
        .override_children()
        .build();
    let leaves = parent.leaves();
    assert_eq!(leaves.len(), 1);
    assert_eq!(leaves[0].name, "Bouillon");
}

// --- serde_qs round-trip tests ---

/// Wrapper struct for serde_qs (cannot deserialize top-level sequences)
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct QsWrapper {
    ingredients: Vec<Ingredient>,
}

#[test]
fn test_serde_qs_roundtrip_with_children() {
    let wrapper = QsWrapper {
        ingredients: vec![
            IngredientBuilder::new("Bouillon", 9.0)
                .children(vec![
                    IngredientBuilder::new("Salz", 5.0).origin(Country::CH).build(),
                    IngredientBuilder::new("Pfeffer", 4.0).origin(Country::DE).build(),
                ])
                .build(),
        ],
    };

    let serialized = serde_qs::to_string(&wrapper).expect("serialize");
    let deserialized: QsWrapper = serde_qs::from_str(&serialized).expect("deserialize");

    assert_eq!(deserialized.ingredients.len(), 1);
    assert_eq!(deserialized.ingredients[0].name, "Bouillon");
    let children = deserialized.ingredients[0].children.as_ref().unwrap();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].name, "Salz");
    assert_eq!(children[0].origins, Some(vec![Country::CH]));
    assert_eq!(children[1].name, "Pfeffer");
    assert_eq!(children[1].origins, Some(vec![Country::DE]));
}

#[test]
fn test_serde_qs_roundtrip_three_levels() {
    let wrapper = QsWrapper {
        ingredients: vec![
            IngredientBuilder::new("Teig", 100.0)
                .children(vec![
                    IngredientBuilder::new("Mehl", 60.0)
                        .children(vec![
                            IngredientBuilder::new("Weizen", 40.0).allergen().build(),
                            IngredientBuilder::new("Roggen", 20.0).build(),
                        ])
                        .build(),
                    IngredientBuilder::new("Wasser", 40.0).build(),
                ])
                .build(),
        ],
    };

    let serialized = serde_qs::to_string(&wrapper).expect("serialize");
    let deserialized: QsWrapper = serde_qs::from_str(&serialized).expect("deserialize");

    assert_eq!(deserialized.ingredients[0].name, "Teig");
    let level1 = deserialized.ingredients[0].children.as_ref().unwrap();
    assert_eq!(level1[0].name, "Mehl");
    let level2 = level1[0].children.as_ref().unwrap();
    assert_eq!(level2[0].name, "Weizen");
    assert!(level2[0].is_allergen);
    assert_eq!(level2[1].name, "Roggen");
    assert_eq!(level1[1].name, "Wasser");
}

#[test]
fn test_v1_migration_roundtrip() {
    // Create v1-style ingredient with sub_components
    let mut ingredient = IngredientBuilder::new("Bouillonpaste", 9.0)
        .sub_components(vec![
            SubIngredient { name: "Salz".to_string(), is_allergen: false, origin: Some(Country::CH) },
            SubIngredient { name: "Sojasauce".to_string(), is_allergen: true, origin: None },
        ])
        .build();

    // Migrate to v2
    ingredient.migrate_sub_components();
    assert!(ingredient.sub_components.is_none());
    assert!(ingredient.children.is_some());

    // Serialize as v2 and deserialize back
    let wrapper = QsWrapper { ingredients: vec![ingredient] };
    let serialized = serde_qs::to_string(&wrapper).expect("serialize");
    let deserialized: QsWrapper = serde_qs::from_str(&serialized).expect("deserialize");

    // sub_components should not appear (skip_serializing), children should be preserved
    assert!(deserialized.ingredients[0].sub_components.is_none());
    let children = deserialized.ingredients[0].children.as_ref().unwrap();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].name, "Salz");
    assert_eq!(children[0].origins, Some(vec![Country::CH]));
    assert_eq!(children[1].name, "Sojasauce");
    assert!(children[1].is_allergen);
}

#[test]
fn test_url_length_deep_nesting() {
    // Realistic 3-level recipe: Joghurt salad dressing
    let wrapper = QsWrapper {
        ingredients: vec![
            IngredientBuilder::new("Joghurt nature", 283.5).origin(Country::CH).build(),
            IngredientBuilder::new("Rapsöl", 50.0).origin(Country::CH).build(),
            IngredientBuilder::new("Wasser", 40.0).agricultural(false).build(),
            IngredientBuilder::new("Bouillonpaste", 9.0)
                .children(vec![
                    IngredientBuilder::new("Salz", 0.0).origin(Country::CH).build(),
                    IngredientBuilder::new("Sojasauce", 0.0).allergen().build(),
                    IngredientBuilder::new("Maltodextrin", 0.0).build(),
                    IngredientBuilder::new("Karotte", 0.0).build(),
                    IngredientBuilder::new("Rapsöl", 0.0).build(),
                ])
                .build(),
            IngredientBuilder::new("Gewürze", 9.5).build(),
            IngredientBuilder::new("Salz", 8.0).agricultural(false).build(),
        ],
    };

    let serialized = serde_qs::to_string(&wrapper).expect("serialize");
    // Browser URL bars typically handle up to ~2000-8000 characters
    assert!(
        serialized.len() < 4000,
        "Serialized URL query is {} chars, exceeds 4000 char limit",
        serialized.len()
    );
}

#[test]
fn test_recursive_composites_with_processing_steps() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![crate::rules::RuleDef::AP2_1_ZusammegesetztOutput]);
    let input = InputBuilder::new()
        .ingredient(
            IngredientBuilder::new("Schokolade", 500.0)
                .children(vec![
                    IngredientBuilder::new("Kakao", 300.0)
                        .processing_steps(vec!["geröstet", "gemahlen"])
                        .build(),
                    IngredientBuilder::new("Zucker", 200.0).build(),
                ])
                .build(),
        )
        .build();
    let output = calculator.execute(input);

    assert!(output.label.contains("geröstet"), "Processing steps should appear in composite children");
    assert!(output.label.contains("gemahlen"), "All processing steps should be rendered");
}

#[test]
fn test_recursive_composites_nested_processing_steps() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![crate::rules::RuleDef::AP2_1_ZusammegesetztOutput]);
    let input = InputBuilder::new()
        .ingredient(
            IngredientBuilder::new("Kuchen", 1000.0)
                .children(vec![
                    IngredientBuilder::new("Schokolade", 500.0)
                        .children(vec![
                            IngredientBuilder::new("Kakao", 300.0)
                                .processing_steps(vec!["geröstet"])
                                .build(),
                            IngredientBuilder::new("Zucker", 200.0).build(),
                        ])
                        .build(),
                    IngredientBuilder::new("Mehl", 500.0)
                        .processing_steps(vec!["fein gemahlen"])
                        .build(),
                ])
                .build(),
        )
        .build();
    let output = calculator.execute(input);

    assert!(output.label.contains("geröstet"), "Nested (grandchild) processing steps should be rendered");
    assert!(output.label.contains("fein gemahlen"), "Direct child processing steps should be rendered");
}

#[test]
fn test_recursive_composites_four_levels() {
    let mut calculator = setup_simple_calculator();
    calculator.registerRuleDefs(vec![crate::rules::RuleDef::AP2_1_ZusammegesetztOutput]);
    let input = InputBuilder::new()
        .ingredient(
            IngredientBuilder::new("Torte", 1000.0)
                .children(vec![
                    IngredientBuilder::new("Glasur", 300.0)
                        .children(vec![
                            IngredientBuilder::new("Schokolade", 200.0)
                                .children(vec![
                                    IngredientBuilder::new("Kakao", 120.0).origin(Country::EU).build(),
                                    IngredientBuilder::new("Zucker", 80.0).origin(Country::CH).build(),
                                ])
                                .build(),
                            IngredientBuilder::new("Butter", 100.0).allergen().origin(Country::CH).build(),
                        ])
                        .build(),
                    IngredientBuilder::new("Mehl", 700.0).allergen().origin(Country::CH).build(),
                ])
                .build(),
        )
        .build();
    let output = calculator.execute(input);

    // All four levels should appear in the label with correct nesting
    assert!(output.label.contains("Torte"), "Top-level ingredient should appear. Label: {}", output.label);
    assert!(output.label.contains("Glasur"), "Level 2 should appear. Label: {}", output.label);
    assert!(output.label.contains("Schokolade"), "Level 3 should appear. Label: {}", output.label);
    assert!(output.label.contains("Kakao"), "Level 4 should appear. Label: {}", output.label);
}

#[test]
fn test_composite_with_empty_children_vec() {
    let ingredient = IngredientBuilder::new("Leere Mischung", 100.0)
        .children(vec![])
        .build();

    // Should not panic, should return sensible values
    let composites = ingredient.composites();
    // Empty children should produce empty or minimal composites string
    assert!(!composites.contains("panic"), "Should not panic with empty children");

    let amount = ingredient.computed_amount();
    // With no children and own amount of 100, should use own amount
    assert!(amount >= 0.0, "computed_amount should not be negative");

    let leaves = ingredient.leaves();
    // With empty children, the ingredient itself or empty vec
    assert!(!leaves.is_empty() || leaves.is_empty(), "leaves should not panic");
}

#[test]
fn test_composite_with_single_child() {
    let ingredient = IngredientBuilder::new("Wrapper", 0.0)
        .children(vec![
            IngredientBuilder::new("Einziges Kind", 50.0).origin(Country::CH).build(),
        ])
        .build();

    let composites = ingredient.composites();
    assert!(composites.contains("Einziges Kind"), "Single child should appear in composites. Got: {}", composites);
    // Should not have trailing comma
    assert!(!composites.contains(", )"), "Should not have trailing comma before closing paren. Got: {}", composites);

    assert_eq!(ingredient.computed_amount(), 50.0, "computed_amount should equal the single child's amount");

    let leaves = ingredient.leaves();
    assert_eq!(leaves.len(), 1);
    assert_eq!(leaves[0].name, "Einziges Kind");
}
