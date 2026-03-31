use super::*;
use crate::persistence::SavedIngredient;

// --- Legacy origin format compatibility (single Country → Vec<Country>) ---

#[test]
fn test_json_legacy_single_origin_string() {
    // Old format: "origin": "CH" (single string, old field name)
    let json = r#"{
        "name": "Salz",
        "is_allergen": false,
        "amount": 50.0,
        "is_agricultural": true,
        "origin": "CH"
    }"#;
    let ingredient: Ingredient = serde_json::from_str(json).expect("deserialize legacy origin");
    assert_eq!(ingredient.origins, Some(vec![Country::CH]));
}

#[test]
fn test_json_legacy_origin_null() {
    let json = r#"{
        "name": "Wasser",
        "is_allergen": false,
        "amount": 100.0,
        "is_agricultural": false,
        "origin": null
    }"#;
    let ingredient: Ingredient = serde_json::from_str(json).expect("deserialize null origin");
    assert_eq!(ingredient.origins, None);
}

#[test]
fn test_json_new_origins_array() {
    // New format: "origins": ["CH", "DE"]
    let json = r#"{
        "name": "Mischung",
        "is_allergen": false,
        "amount": 50.0,
        "is_agricultural": true,
        "origins": ["CH", "DE"]
    }"#;
    let ingredient: Ingredient = serde_json::from_str(json).expect("deserialize origins array");
    assert_eq!(ingredient.origins, Some(vec![Country::CH, Country::DE]));
}

#[test]
fn test_json_origins_missing_field() {
    // Neither origin nor origins present
    let json = r#"{
        "name": "Wasser",
        "is_allergen": false,
        "amount": 100.0,
        "is_agricultural": false
    }"#;
    let ingredient: Ingredient = serde_json::from_str(json).expect("deserialize missing origins");
    assert_eq!(ingredient.origins, None);
}

#[test]
fn test_json_origins_empty_array() {
    let json = r#"{
        "name": "Wasser",
        "is_allergen": false,
        "amount": 100.0,
        "is_agricultural": false,
        "origins": []
    }"#;
    let ingredient: Ingredient = serde_json::from_str(json).expect("deserialize empty origins");
    assert_eq!(ingredient.origins, None);
}

#[test]
fn test_json_saved_ingredient_with_legacy_origin_in_children() {
    // A saved ingredient where children use the old "origin" field
    let json = r#"[{
        "ingredient": {
            "name": "Bouillonpaste",
            "is_allergen": false,
            "amount": 9.0,
            "is_agricultural": true,
            "children": [
                {"name": "Salz", "is_allergen": false, "amount": 5.0, "is_agricultural": false, "origin": "CH"},
                {"name": "Pfeffer", "is_allergen": false, "amount": 4.0, "is_agricultural": true, "origin": "DE"}
            ]
        }
    }]"#;
    let saved: Vec<SavedIngredient> = serde_json::from_str(json).expect("deserialize");
    let children = saved[0].ingredient.children.as_ref().unwrap();
    assert_eq!(children[0].origins, Some(vec![Country::CH]));
    assert_eq!(children[1].origins, Some(vec![Country::DE]));
}

#[test]
fn test_serde_qs_legacy_single_origin() {
    // Old URL format used "origin=CH" instead of "origins[0]=CH"
    let qs = "ingredients[0][name]=Salz&ingredients[0][is_allergen]=false&ingredients[0][amount]=50&ingredients[0][origin]=CH&ingredients[0][is_agricultural]=true";

    #[derive(serde::Deserialize)]
    struct QsWrapper { ingredients: Vec<Ingredient> }

    let wrapper: QsWrapper = serde_qs::from_str(qs).expect("deserialize legacy qs origin");
    assert_eq!(wrapper.ingredients[0].origins, Some(vec![Country::CH]));
}

// --- Saved ingredient JSON roundtrip tests ---

#[test]
fn test_saved_ingredient_json_roundtrip_simple() {
    let ingredient = IngredientBuilder::new("Salz", 50.0)
        .origin(Country::CH)
        .agricultural(false)
        .build();

    let saved = vec![SavedIngredient { ingredient }];
    let json = serde_json::to_string(&saved).expect("serialize");
    let restored: Vec<SavedIngredient> = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].ingredient.name, "Salz");
    assert_eq!(restored[0].ingredient.amount, 50.0);
    assert_eq!(restored[0].ingredient.origins, Some(vec![Country::CH]));
    assert!(!restored[0].ingredient.is_agricultural);
}

#[test]
fn test_saved_ingredient_json_roundtrip_nested_children() {
    let ingredient = IngredientBuilder::new("Bouillonpaste", 9.0)
        .children(vec![
            IngredientBuilder::new("Salz", 5.0).origin(Country::CH).build(),
            IngredientBuilder::new("Sojasauce", 3.0).allergen().origin(Country::DE).build(),
            IngredientBuilder::new("Maltodextrin", 1.0)
                .processing_steps(vec!["getrocknet"])
                .build(),
        ])
        .build();

    let saved = vec![SavedIngredient { ingredient }];
    let json = serde_json::to_string(&saved).expect("serialize");
    let restored: Vec<SavedIngredient> = serde_json::from_str(&json).expect("deserialize");

    let children = restored[0].ingredient.children.as_ref().unwrap();
    assert_eq!(children.len(), 3);
    assert_eq!(children[0].name, "Salz");
    assert_eq!(children[0].origins, Some(vec![Country::CH]));
    assert_eq!(children[1].name, "Sojasauce");
    assert!(children[1].is_allergen);
    assert_eq!(children[2].processing_steps, Some(vec!["getrocknet".to_string()]));
}

#[test]
fn test_saved_ingredient_json_roundtrip_three_levels() {
    let ingredient = IngredientBuilder::new("Kuchen", 1000.0)
        .children(vec![
            IngredientBuilder::new("Schokolade", 500.0)
                .children(vec![
                    IngredientBuilder::new("Kakao", 300.0).bio().origin(Country::EU).build(),
                    IngredientBuilder::new("Zucker", 200.0).origin(Country::CH).build(),
                ])
                .build(),
            IngredientBuilder::new("Mehl", 500.0).allergen().build(),
        ])
        .build();

    let saved = vec![SavedIngredient { ingredient }];
    let json = serde_json::to_string(&saved).expect("serialize");
    let restored: Vec<SavedIngredient> = serde_json::from_str(&json).expect("deserialize");

    let level1 = restored[0].ingredient.children.as_ref().unwrap();
    assert_eq!(level1[0].name, "Schokolade");
    let level2 = level1[0].children.as_ref().unwrap();
    assert_eq!(level2[0].name, "Kakao");
    assert_eq!(level2[0].is_bio, Some(true));
    assert_eq!(level2[0].origins, Some(vec![Country::EU]));
    assert_eq!(level2[1].name, "Zucker");
    assert!(level1[1].is_allergen);
}

#[test]
fn test_saved_ingredient_legacy_sub_components_migration() {
    // Raw JSON with old sub_components format (skip_serializing means we can't produce this via serde)
    let legacy_json = r#"[{
        "ingredient": {
            "name": "Bouillonpaste",
            "is_allergen": false,
            "amount": 9.0,
            "sub_components": [
                {"name": "Salz", "is_allergen": false, "origin": "CH"},
                {"name": "Sojasauce", "is_allergen": true, "origin": null}
            ],
            "is_agricultural": true
        }
    }]"#;

    let mut saved: Vec<SavedIngredient> = serde_json::from_str(legacy_json).expect("deserialize legacy");
    assert_eq!(saved.len(), 1);

    // Migrate, mirroring what get_saved_ingredients() does
    saved[0].ingredient.migrate_sub_components();

    assert!(saved[0].ingredient.sub_components.is_none());
    let children = saved[0].ingredient.children.as_ref().unwrap();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].name, "Salz");
    assert_eq!(children[0].origins, Some(vec![Country::CH]));
    assert!(!children[0].is_allergen);
    assert_eq!(children[1].name, "Sojasauce");
    assert!(children[1].is_allergen);
    assert_eq!(children[1].origins, None);
}

#[test]
fn test_saved_ingredient_dedup_by_name() {
    // Simulate the upsert logic from save_composite_ingredient
    let mut saved = vec![
        SavedIngredient {
            ingredient: IngredientBuilder::new("Bouillonpaste", 9.0)
                .children(vec![
                    IngredientBuilder::new("Salz", 5.0).build(),
                ])
                .build(),
        },
        SavedIngredient {
            ingredient: IngredientBuilder::new("Schokolade", 100.0).build(),
        },
    ];

    // "Save" an updated Bouillonpaste (same name, different children)
    let updated = IngredientBuilder::new("Bouillonpaste", 18.0)
        .children(vec![
            IngredientBuilder::new("Salz", 10.0).build(),
            IngredientBuilder::new("Pfeffer", 8.0).build(),
        ])
        .build();

    // Apply same upsert logic as persistence.rs
    if let Some(index) = saved.iter().position(|s| s.ingredient.name == updated.name) {
        saved[index] = SavedIngredient { ingredient: updated };
    } else {
        saved.push(SavedIngredient { ingredient: updated });
    }

    assert_eq!(saved.len(), 2, "Should update existing, not add duplicate");
    assert_eq!(saved[0].ingredient.amount, 18.0, "Bouillonpaste should be updated");
    let children = saved[0].ingredient.children.as_ref().unwrap();
    assert_eq!(children.len(), 2, "Updated children should have 2 entries");
    assert_eq!(saved[1].ingredient.name, "Schokolade", "Other entries unchanged");
}

#[test]
fn test_saved_ingredient_json_preserves_all_optional_fields() {
    let ingredient = Ingredient {
        name: "Vollständig".to_string(),
        is_allergen: true,
        amount: 42.0,
        unit: AmountUnit::Milliliter,
        sub_components: None,
        children: Some(vec![
            IngredientBuilder::new("Kind", 20.0).build(),
        ]),
        is_namensgebend: Some(true),
        origins: Some(vec![Country::CH, Country::DE]),
        is_agricultural: true,
        is_bio: Some(true),
        category: Some("Fleisch".to_string()),
        aufzucht_ort: Some(Country::CH),
        schlachtungs_ort: Some(Country::DE),
        fangort: Some(Country::AT),
        bio_ch: Some(true),
        erlaubte_ausnahme_bio: Some(true),
        erlaubte_ausnahme_bio_details: Some("Grund".to_string()),
        erlaubte_ausnahme_knospe: Some(false),
        erlaubte_ausnahme_knospe_details: Some("Kein Grund".to_string()),
        processing_steps: Some(vec!["geröstet".to_string(), "gemahlen".to_string()]),
        aus_umstellbetrieb: Some(true),
        override_children: Some(true),
    };

    let saved = vec![SavedIngredient { ingredient }];
    let json = serde_json::to_string(&saved).expect("serialize");
    let restored: Vec<SavedIngredient> = serde_json::from_str(&json).expect("deserialize");

    let i = &restored[0].ingredient;
    assert_eq!(i.name, "Vollständig");
    assert!(i.is_allergen);
    assert_eq!(i.amount, 42.0);
    assert_eq!(i.unit, AmountUnit::Milliliter);
    assert_eq!(i.children.as_ref().unwrap().len(), 1);
    assert_eq!(i.is_namensgebend, Some(true));
    assert_eq!(i.origins, Some(vec![Country::CH, Country::DE]));
    assert_eq!(i.is_bio, Some(true));
    assert_eq!(i.category, Some("Fleisch".to_string()));
    assert_eq!(i.aufzucht_ort, Some(Country::CH));
    assert_eq!(i.schlachtungs_ort, Some(Country::DE));
    assert_eq!(i.fangort, Some(Country::AT));
    assert_eq!(i.bio_ch, Some(true));
    assert_eq!(i.erlaubte_ausnahme_bio, Some(true));
    assert_eq!(i.erlaubte_ausnahme_bio_details, Some("Grund".to_string()));
    assert_eq!(i.erlaubte_ausnahme_knospe, Some(false));
    assert_eq!(i.erlaubte_ausnahme_knospe_details, Some("Kein Grund".to_string()));
    assert_eq!(i.processing_steps, Some(vec!["geröstet".to_string(), "gemahlen".to_string()]));
    assert_eq!(i.aus_umstellbetrieb, Some(true));
    assert_eq!(i.override_children, Some(true));
}
