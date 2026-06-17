//! Per-attribute aggregation directions for (composite) ingredients:
//! weight top-down, quality + origin bottom-up (origin single-level).

use super::*;

/// Use case 1: a composite carries the total weight; its sub-ingredients are
/// weightless (qualitative) but each carry a quality + origin. Weight is read
/// top-down from the parent, quality/origin aggregate bottom-up from children.
#[test]
fn topdown_weight_with_bottomup_quality_and_origin() {
    let composite = IngredientBuilder::new("Konfitüre", 250.0)
        .children(vec![
            IngredientBuilder::new("Erdbeeren", 0.0).bio().origin(Country::CH).build(),
            IngredientBuilder::new("Zucker", 0.0).bio().origin(Country::CH).build(),
        ])
        .build();

    // Weight: top-down — parent total, not the (zero) sum of children.
    assert_eq!(composite.computed_amount(), 250.0);

    // Quality: bottom-up — all children bio ⇒ composite bio / Knospe-compliant.
    assert_eq!(composite.computed_bio_status(), Some(true));
    assert!(composite.is_knospe_compliant());

    // Origin: bottom-up — union of children's origins.
    assert!(composite.computed_origins().unwrap().contains(&Country::CH));
}

/// A single non-compliant child makes the whole composite non-compliant (bottom-up).
#[test]
fn mixed_quality_composite_is_not_knospe_compliant() {
    let composite = IngredientBuilder::new("Mischung", 100.0)
        .children(vec![
            IngredientBuilder::new("Apfel", 0.0).bio().origin(Country::CH).build(),
            IngredientBuilder::new("Zusatz", 0.0).origin(Country::CH).build(), // not bio
        ])
        .build();

    assert_eq!(composite.computed_bio_status(), Some(false));
    assert!(!composite.is_knospe_compliant());
}

/// Non-agricultural children (salt, water, …) carry no country-of-origin
/// declaration, so their origin must NOT be taken over into the parent's
/// aggregated origin. Regression: it was wrongly carried over for such quality.
#[test]
fn non_agricultural_child_origin_not_aggregated() {
    let composite = IngredientBuilder::new("Mischung", 100.0)
        .children(vec![
            IngredientBuilder::new("Kräuter", 0.0).origin(Country::FR).build(), // agricultural (default)
            IngredientBuilder::new("Salz", 0.0).agricultural(false).origin(Country::CH).build(), // non-agri
        ])
        .build();

    let origins = composite.computed_origins().expect("agricultural child still provides an origin");
    assert!(origins.contains(&Country::FR), "agricultural child origin should aggregate. Got: {:?}", origins);
    assert!(!origins.contains(&Country::CH), "non-agricultural child origin must NOT aggregate. Got: {:?}", origins);
}

/// Quality "parent claim overrides": a composite declared Knospe as a whole (e.g. a
/// bought, certified composite ingredient) counts as Knospe even if a child isn't
/// individually marked bio. Without a parent claim it derives bottom-up.
#[test]
fn parent_quality_claim_overrides_children() {
    let claimed = IngredientBuilder::new("Knospe-Schokolade", 100.0)
        .bio()
        .children(vec![
            IngredientBuilder::new("Kakao", 0.0).bio().build(),
            IngredientBuilder::new("Zucker", 0.0).build(), // no own quality claim
        ])
        .build();
    assert!(claimed.is_knospe_compliant(), "parent Knospe claim should override a non-bio child");
    assert_eq!(claimed.computed_bio_status(), Some(true));

    let derived = IngredientBuilder::new("Mischung", 100.0)
        .children(vec![
            IngredientBuilder::new("Kakao", 0.0).bio().build(),
            IngredientBuilder::new("Zucker", 0.0).build(),
        ])
        .build();
    assert!(!derived.is_knospe_compliant(), "no parent claim → derive bottom-up (not all bio)");
}

/// Origin is defined on a single level: an explicit parent origin wins over
/// whatever the children carry (no silent union across levels).
#[test]
fn parent_origin_takes_precedence_over_children() {
    let composite = IngredientBuilder::new("Saft", 100.0)
        .origin(Country::EU)
        .children(vec![
            IngredientBuilder::new("Apfel", 0.0).origin(Country::CH).build(),
        ])
        .build();

    assert_eq!(composite.computed_origins(), Some(vec![Country::EU]));
}

/// override_children forces leaf treatment: the node's own flags win.
#[test]
fn override_children_uses_own_quality() {
    let composite = IngredientBuilder::new("Block", 100.0)
        .bio()
        .origin(Country::CH)
        .override_children()
        .children(vec![
            IngredientBuilder::new("X", 0.0).build(), // not bio, no origin
        ])
        .build();

    assert_eq!(composite.computed_bio_status(), Some(true));
    assert_eq!(composite.computed_origins(), Some(vec![Country::CH]));
}

/// Weighted children still aggregate by descending into leaves.
#[test]
fn weighted_children_still_sum_and_descend() {
    let composite = IngredientBuilder::new("Vinaigrette", 0.0)
        .children(vec![
            IngredientBuilder::new("Öl", 70.0).bio().origin(Country::CH).build(),
            IngredientBuilder::new("Essig", 30.0).bio().origin(Country::CH).build(),
        ])
        .build();

    assert_eq!(composite.computed_amount(), 100.0);
    assert_eq!(composite.computed_bio_status(), Some(true));
}

/// A top-down composite of CH-Knospe children counts toward both the Swiss and
/// the Knospe percentages at the parent's full weight.
#[test]
fn topdown_composite_counts_toward_percentages() {
    let composite = IngredientBuilder::new("Konfitüre", 200.0)
        .children(vec![
            IngredientBuilder::new("Erdbeeren", 0.0).bio().origin(Country::CH).build(),
            IngredientBuilder::new("Zucker", 0.0).bio().origin(Country::CH).build(),
        ])
        .build();
    let ingredients = vec![composite];

    assert_eq!(calculate_swiss_agricultural_percentage(&ingredients), 100.0);
    assert_eq!(calculate_knospe_certified_percentage(&ingredients), 100.0);
    assert_eq!(calculate_bio_swiss_agricultural_percentage(&ingredients), 100.0);
}

/// Origin defined on both a composite and one of its children (same branch)
/// is flagged; origin on a single level is not.
#[test]
fn origin_single_level_branch_conflict() {
    use std::collections::HashMap;

    // Conflict: parent AND child both declare an origin.
    let conflict = IngredientBuilder::new("Saft", 100.0)
        .origin(Country::CH)
        .children(vec![
            IngredientBuilder::new("Apfel", 0.0).origin(Country::CH).build(),
        ])
        .build();
    let mut msgs = HashMap::new();
    validate_origin_single_level(&[conflict], &mut msgs);
    assert!(msgs.contains_key("ingredients[0][origin]"));

    // Single level (parent only): no conflict.
    let parent_only = IngredientBuilder::new("Saft", 100.0)
        .origin(Country::CH)
        .children(vec![IngredientBuilder::new("Apfel", 0.0).build()])
        .build();
    let mut msgs = HashMap::new();
    validate_origin_single_level(&[parent_only], &mut msgs);
    assert!(msgs.is_empty());

    // Single level (children only): no conflict.
    let children_only = IngredientBuilder::new("Saft", 0.0)
        .children(vec![
            IngredientBuilder::new("Apfel", 60.0).origin(Country::CH).build(),
            IngredientBuilder::new("Birne", 40.0).origin(Country::Import).build(),
        ])
        .build();
    let mut msgs = HashMap::new();
    validate_origin_single_level(&[children_only], &mut msgs);
    assert!(msgs.is_empty());
}

/// Regression: a Lebensmittelrecht composite (weighted children, parent + child
/// origins) must still render its name in the label via execute().
#[test]
fn lebensmittelrecht_composite_renders_in_label() {
    let calculator = calculator_for(crate::shared::Configuration::Conventional);
    let input = InputBuilder::new()
        .vollstaendig()
        .ingredient(
            IngredientBuilder::new("Salzbouillon", 9.0)
                .origins(vec![Country::CH, Country::DE])
                .children(vec![
                    IngredientBuilder::new("Salz", 5.0).agricultural(false).origin(Country::CH).build(),
                    IngredientBuilder::new("Pfeffer", 4.0).origin(Country::DE).build(),
                ])
                .build(),
        )
        .total(100.0)
        .build();
    let output = calculator.execute(input);
    assert!(output.label.contains("Salzbouillon"), "label missing composite name:\n{}", output.label);
}

/// Origin presence validations must respect aggregated (bottom-up) origin: a
/// composite >50% of total with origin only on its children satisfies the
/// >50%-origin-required rule; a composite with no origin anywhere is flagged.
#[test]
fn over_50_origin_satisfied_by_bottom_up_composite() {
    use std::collections::HashMap;

    let composite = IngredientBuilder::new("Saft", 80.0)
        .children(vec![
            IngredientBuilder::new("Apfel", 50.0).origin(Country::CH).build(),
            IngredientBuilder::new("Birne", 30.0).origin(Country::CH).build(),
        ])
        .build();
    let mut msgs = HashMap::new();
    validate_origin(&[composite], 100.0, &mut msgs);
    assert!(
        !msgs.contains_key("ingredients[0][origin]"),
        "bottom-up composite origin should satisfy the >50% rule, got: {:?}",
        msgs
    );

    let composite_no_origin = IngredientBuilder::new("Saft", 80.0)
        .children(vec![
            IngredientBuilder::new("Apfel", 50.0).build(),
            IngredientBuilder::new("Birne", 30.0).build(),
        ])
        .build();
    let mut msgs2 = HashMap::new();
    validate_origin(&[composite_no_origin], 100.0, &mut msgs2);
    assert!(
        msgs2.contains_key("ingredients[0][origin]"),
        "composite with no origin on any level should still be flagged"
    );
}

/// The new `Import` origin renders without a flag and round-trips through
/// the serde_qs URL encoding (backwards/forwards compatibility).
#[test]
fn import_origin_no_flag_and_roundtrips() {
    assert_eq!(Country::Import.flag_emoji(), "");
    assert_eq!(Country::Import.country_code(), "Import");

    let ing = IngredientBuilder::new("Rohrzucker", 50.0).origin(Country::Import).build();
    let encoded = qs_to_string(&ing).unwrap();
    let decoded: Ingredient = qs_from_str(&encoded).unwrap();
    assert_eq!(decoded.origins, Some(vec![Country::Import]));
}
