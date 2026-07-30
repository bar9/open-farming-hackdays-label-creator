// TD-1, Stufe 0 — Invarianten des `conditional_elements`-Kontrakts.
//
// Die Conditionals verlassen `execute()` als untypisierte HashMap; fachlich
// exklusive Paare (allowed/not_allowed, ok/failed/pending, Logo-Varianten)
// werden nur durch Kontrollfluss auseinandergehalten. Bis der Kontrakt
// typisiert ist (siehe requirements/TD-1_conditionals_contract.md), prüft
// dieser Test die Ausschlüsse über eine Matrix von Rezepturen — jede neue
// Regel, die versehentlich beide Seiten eines Paars setzt, fällt hier auf.

use crate::conditional_keys as keys;
use super::*;
use crate::shared::Configuration;

/// Pairs that must never both be set in one output.
const EXCLUSIVE_PAIRS: &[(&str, &str)] = &[
    (keys::BIO_MARKETING_ALLOWED, keys::BIO_MARKETING_NOT_ALLOWED),
    (keys::KNOSPE_MARKETING_ALLOWED, keys::KNOSPE_MARKETING_NOT_ALLOWED),
    (keys::BIO_CHECK_OK, keys::BIO_CHECK_FAILED),
    (keys::BIO_CHECK_OK, keys::BIO_CHECK_PENDING),
    (keys::BIO_CHECK_FAILED, keys::BIO_CHECK_PENDING),
    (keys::KNOSPE_CHECK_OK, keys::KNOSPE_CHECK_FAILED),
    (keys::KNOSPE_CHECK_OK, keys::KNOSPE_CHECK_PENDING),
    (keys::KNOSPE_CHECK_FAILED, keys::KNOSPE_CHECK_PENDING),
    (keys::BIO_SUISSE_REGULAR, keys::BIO_SUISSE_NO_CROSS),
];

/// Implications: when `premise` is set, `consequence` must be set too.
const IMPLICATIONS: &[(&str, &str)] = &[
    // A logo is only ever shown for a marketable product.
    (keys::BIO_SUISSE_REGULAR, keys::KNOSPE_MARKETING_ALLOWED),
    (keys::BIO_SUISSE_NO_CROSS, keys::KNOSPE_MARKETING_ALLOWED),
    // The Umstellungsknospe is a variant of a shown logo, never standalone.
    (keys::KNOSPE_UMSTELLUNG_LOGO, keys::KNOSPE_MARKETING_ALLOWED),
];

fn check_invariants(ctx: &str, c: &std::collections::HashMap<String, bool>) {
    for (a, b) in EXCLUSIVE_PAIRS {
        assert!(
            !(c.get(*a) == Some(&true) && c.get(*b) == Some(&true)),
            "[{ctx}] `{a}` and `{b}` are both set — mutually exclusive by domain rule.\nConditionals: {c:?}"
        );
    }
    for (premise, consequence) in IMPLICATIONS {
        if c.get(*premise) == Some(&true) {
            assert!(
                c.get(*consequence) == Some(&true),
                "[{ctx}] `{premise}` is set but `{consequence}` is not.\nConditionals: {c:?}"
            );
        }
    }
}

/// A spread of recipes chosen to reach every decision branch: empty, mono,
/// non-agricultural-only, mixed qualities, exceptions under/over 5%,
/// Umstellung mono and composite, Swiss and import origins, composites.
fn recipe_matrix() -> Vec<(&'static str, Vec<Ingredient>)> {
    vec![
        ("empty", vec![]),
        ("plain conventional", vec![
            IngredientBuilder::new_agri("Zucker", 1000.0).origin(Country::CH).build(),
        ]),
        ("only water", vec![
            IngredientBuilder::new("Wasser", 1000.0).agricultural(false).build(),
        ]),
        ("full bio ch", vec![
            IngredientBuilder::new_agri("Hafer", 1000.0).bio_ch().origin(Country::CH).build(),
        ]),
        ("full knospe swiss", vec![
            IngredientBuilder::new_agri("Himbeeren", 1000.0).bio().bio_ch().origin(Country::CH).build(),
        ]),
        ("full knospe import", vec![
            IngredientBuilder::new_agri("Rohrzucker", 1000.0).bio().bio_ch().origin(Country::Import).build(),
        ]),
        ("mixed bio and conventional", vec![
            IngredientBuilder::new_agri("Hafer", 600.0).bio_ch().origin(Country::CH).build(),
            IngredientBuilder::new_agri("Zucker", 400.0).origin(Country::CH).build(),
        ]),
        ("exception under 5%", vec![
            IngredientBuilder::new_agri("Hafer", 960.0).bio().bio_ch().origin(Country::CH).build(),
            IngredientBuilder::new_agri("Pektin", 40.0).origin(Country::CH).erlaubte_ausnahme_bio().build(),
        ]),
        ("exception over 5%", vec![
            IngredientBuilder::new_agri("Hafer", 600.0).bio().bio_ch().origin(Country::CH).build(),
            IngredientBuilder::new_agri("Pektin", 400.0).origin(Country::CH).erlaubte_ausnahme_knospe().build(),
        ]),
        ("umstellung mono", vec![
            IngredientBuilder::new_agri("Hafer", 1000.0).bio().bio_ch().origin(Country::CH)
                .umstellbetrieb().build(),
        ]),
        ("umstellung composite", vec![
            IngredientBuilder::new_agri("Hafer", 600.0).bio().bio_ch().origin(Country::CH)
                .umstellbetrieb().build(),
            IngredientBuilder::new_agri("Zucker", 400.0).bio().bio_ch().origin(Country::CH).build(),
        ]),
        // The Umstellbetrieb ingredient is small enough that the Bio-CH share
        // stays >= 95% — exactly the case where execute() must retract the
        // already-granted "allowed" via remove(). Dropping that remove() is the
        // regression this row exists to catch.
        ("umstellung composite with high bio share", vec![
            IngredientBuilder::new_agri("Hafer", 960.0).bio_ch().origin(Country::CH).build(),
            IngredientBuilder::new_agri("Karotte", 40.0).bio_ch().origin(Country::CH)
                .umstellbetrieb().build(),
        ]),
        ("swiss share just under 90%", vec![
            IngredientBuilder::new_agri("Himbeeren", 890.0).bio().bio_ch().origin(Country::CH).build(),
            IngredientBuilder::new_agri("Rohrzucker", 110.0).bio().bio_ch().origin(Country::Import).build(),
        ]),
        ("composite with children", vec![
            IngredientBuilder::new_agri("Müesli", 0.0)
                .children(vec![
                    IngredientBuilder::new_agri("Hafer", 700.0).bio().bio_ch().origin(Country::CH).build(),
                    IngredientBuilder::new_agri("Rosinen", 300.0).origin(Country::Import).build(),
                ])
                .build(),
        ]),
    ]
}

#[test]
fn conditionals_are_consistent_across_the_recipe_matrix() {
    for config in [Configuration::Bio, Configuration::Knospe, Configuration::Conventional] {
        for (name, ingredients) in recipe_matrix() {
            // Each recipe in every check state: untouched, and confirmed.
            for vollstaendig in [false, true] {
                let mut builder = InputBuilder::new().ingredients(ingredients.clone());
                if vollstaendig {
                    builder = builder.vollstaendig();
                }
                let output = calculator_for(config).execute(builder.build());
                let ctx = format!("{config:?} / {name} / vollstaendig={vollstaendig}");
                check_invariants(&ctx, &output.conditionals());
            }
        }
    }
}

#[test]
fn every_produced_conditional_is_consumed_somewhere() {
    // Guards against dead outputs like the former `is_bio_eingabe`, which was
    // inserted for years without a single reader. Producing a conditional that
    // no UI or test consumes is a silent contract break, so the full key set is
    // pinned here; extend the list when a new conditional gains a consumer.
    const KNOWN_CONSUMED: &[&str] = &[
        keys::ALTERNATIVE_MARKING_ALLOWED,
        keys::NAMENSGEBENDE_ZUTAT,
        keys::MANUELLES_TOTAL,
        keys::BIO_SUISSE_REGULAR,
        keys::BIO_SUISSE_NO_CROSS,
        keys::KNOSPE_UMSTELLUNG_LOGO,
        keys::KNOSPE_MARKETING_ALLOWED,
        keys::KNOSPE_MARKETING_NOT_ALLOWED,
        keys::KNOSPE_ERLAUBTE_AUSNAHME_UEBER_5_PROZENT,
        keys::KNOSPE_CHECK_PENDING,
        keys::KNOSPE_CHECK_OK,
        keys::KNOSPE_CHECK_FAILED,
        keys::BIO_SACHBEZEICHNUNG_SUFFIX,
        keys::BIO_MARKETING_ALLOWED,
        keys::BIO_MARKETING_NOT_ALLOWED,
        keys::BIO_NICHT_DEKLARIERTE_ZUTAT,
        keys::BIO_ERLAUBTE_AUSNAHME_UEBER_5_PROZENT,
        keys::UMSTELLBETRIEB_HINWEIS,
        keys::BIO_CHECK_PENDING,
        keys::BIO_CHECK_OK,
        keys::BIO_CHECK_FAILED,
        keys::HERKUNFT_BENOETIGT_UEBER_50_PROZENT,
    ];

    for config in [Configuration::Bio, Configuration::Knospe, Configuration::Conventional] {
        for (name, ingredients) in recipe_matrix() {
            for vollstaendig in [false, true] {
                let mut builder = InputBuilder::new().ingredients(ingredients.clone());
                if vollstaendig {
                    builder = builder.vollstaendig();
                }
                let output = calculator_for(config).execute(builder.build());
                for key in output.conditionals().keys() {
                    // Per-ingredient origin flags are indexed dynamically.
                    if key.starts_with(keys::HERKUNFT_BENOETIGT_PREFIX) {
                        continue;
                    }
                    assert!(
                        KNOWN_CONSUMED.contains(&key.as_str()),
                        "[{config:?} / {name}] execute() produced `{key}`, which no known \
                         consumer reads. Either wire it to the UI or remove the insert; \
                         if it is genuinely new and consumed, add it to KNOWN_CONSUMED."
                    );
                }
            }
        }
    }
}
