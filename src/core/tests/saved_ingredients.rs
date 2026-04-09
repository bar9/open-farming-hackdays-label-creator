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

// --- Form-level legacy URL migration ---
//
// These tests cover what happens when the user opens a shared link that was
// produced before the data structure refactoring (commit 85411ee). Such URLs
// use the old `sub_components` field instead of `children`, and the oldest of
// them have no `v` (version) field at all. The runtime migration lives in
// `parse_form_from_saved_params` in src/pages/label_page.rs, which is wasm-only
// because it calls js_sys; here we exercise the same `serde_qs` + migration
// codepath against a minimal stand-in for the Form struct.

#[derive(serde::Deserialize, Debug)]
struct LegacyFormStub {
    #[serde(default = "default_legacy_version")]
    v: u8,
    #[serde(default)]
    ingredients: Vec<Ingredient>,
}

/// Mirrors `default_version()` in src/pages/label_page.rs. Must stay at 1 so
/// URLs missing `v` are treated as legacy and pulled through migration.
fn default_legacy_version() -> u8 { 1 }

/// Mirrors the migration in label_page.rs::parse_form_from_saved_params.
fn migrate_legacy_form(form: &mut LegacyFormStub) {
    if form.v < 2 {
        for ing in &mut form.ingredients {
            ing.migrate_sub_components();
        }
        form.v = 2;
    }
}

#[test]
fn legacy_url_with_v1_and_sub_components_migrates_to_children() {
    // A shared link from when v=1 was the format: explicit v=1, sub_components
    // populated, no `children` field. Migration should turn sub_components
    // into children.
    let qs = "v=1\
        &ingredients[0][name]=Bouillonpaste\
        &ingredients[0][is_allergen]=false\
        &ingredients[0][amount]=9\
        &ingredients[0][is_agricultural]=true\
        &ingredients[0][sub_components][0][name]=Salz\
        &ingredients[0][sub_components][0][is_allergen]=false\
        &ingredients[0][sub_components][0][origin]=CH\
        &ingredients[0][sub_components][1][name]=Sojasauce\
        &ingredients[0][sub_components][1][is_allergen]=true";

    let mut form: LegacyFormStub = serde_qs::from_str(qs).expect("deserialize legacy v=1 url");
    assert_eq!(form.v, 1);
    migrate_legacy_form(&mut form);

    assert_eq!(form.v, 2);
    let parent = &form.ingredients[0];
    assert!(parent.sub_components.is_none(), "sub_components should be drained");
    let children = parent.children.as_ref().expect("children should be populated");
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].name, "Salz");
    assert_eq!(children[0].origins, Some(vec![Country::CH]));
    assert!(children[1].is_allergen);
    assert_eq!(children[1].name, "Sojasauce");
}

#[test]
fn legacy_url_without_v_field_still_migrates_sub_components() {
    // The pre-85411ee Form had no `v` field at all. With default_version()
    // returning 1, a missing `v` deserializes as v=1 and migration runs.
    let qs = "ingredients[0][name]=Bouillonpaste\
        &ingredients[0][is_allergen]=false\
        &ingredients[0][amount]=9\
        &ingredients[0][is_agricultural]=true\
        &ingredients[0][sub_components][0][name]=Salz\
        &ingredients[0][sub_components][0][is_allergen]=false\
        &ingredients[0][sub_components][0][origin]=CH";

    let mut form: LegacyFormStub = serde_qs::from_str(qs).expect("deserialize legacy v-less url");
    assert_eq!(form.v, 1, "missing v must default to 1 so legacy URLs migrate");

    migrate_legacy_form(&mut form);

    assert_eq!(form.v, 2);
    let parent = &form.ingredients[0];
    assert!(parent.sub_components.is_none(), "sub_components should be drained after migration");
    let children = parent.children.as_ref().expect("children should be populated after migration");
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].name, "Salz");
    assert_eq!(children[0].origins, Some(vec![Country::CH]));
}

#[test]
fn current_url_with_explicit_v2_does_not_double_migrate() {
    // Sanity check: current-format URLs (v=2, children populated, no
    // sub_components) must pass through migration unchanged.
    let qs = "v=2\
        &ingredients[0][name]=Bouillonpaste\
        &ingredients[0][is_allergen]=false\
        &ingredients[0][amount]=9\
        &ingredients[0][is_agricultural]=true\
        &ingredients[0][children][0][name]=Salz\
        &ingredients[0][children][0][is_allergen]=false\
        &ingredients[0][children][0][amount]=5\
        &ingredients[0][children][0][origins][0]=CH";

    let mut form: LegacyFormStub = serde_qs::from_str(qs).expect("deserialize v=2 url");
    assert_eq!(form.v, 2);

    let pre_children = form.ingredients[0].children.clone();
    migrate_legacy_form(&mut form);

    let parent = &form.ingredients[0];
    assert!(parent.sub_components.is_none());
    assert_eq!(parent.children, pre_children, "v=2 children must not be touched");
    assert_eq!(parent.children.as_ref().unwrap()[0].origins, Some(vec![Country::CH]));
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
