// DEC-2 — Einzelzutat/Monoprodukt: quality selection.
//
// With «Keine Zutatenliste (Einzelzutat)» there is no recipe, so no ingredient
// carries the bio flags. The declared `MonoQuality` is converted into a single
// synthetic ingredient (`Form -> Input`), which is what lets the ordinary
// Bio/Knospe rules produce the right logo and Sachbezeichnung.
//
// These tests drive the real `Form` conversion rather than hand-built inputs,
// so a regression in the wiring is caught, not just in the rules.

use crate::conditional_keys as keys;
use super::*;
use crate::pages::label_page::{Form, MonoQuality};
use crate::shared::Configuration;

/// A mono product in the given configuration with the given declared quality.
fn mono_output(config: Configuration, quality: MonoQuality) -> crate::core::Output {
    let form = Form {
        ignore_ingredients: true,
        mono_quality: quality,
        product_subtitle: "Weizenmehl".to_string(),
        // The user confirmed the (empty) recipe; DEC-3 keeps the check hints
        // silent either way, so this only rules out accidental coupling.
        rezeptur_vollstaendig: true,
        ..Form::default()
    };
    calculator_for(config).execute(form.into())
}

// --- Knospe configuration -------------------------------------------------

#[test]
fn mono_knospe_ch_shows_the_swiss_knospe() {
    let c = mono_output(Configuration::Knospe, MonoQuality::KnospeCh).conditionals();

    assert_eq!(c.get(keys::KNOSPE_MARKETING_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::BIO_SUISSE_REGULAR), Some(&true), "Swiss Knospe → logo with cross");
    assert_eq!(c.get(keys::BIO_SUISSE_NO_CROSS), None);
    assert_eq!(c.get(keys::KNOSPE_UMSTELLUNG_LOGO), None);
}

#[test]
fn mono_knospe_import_shows_the_knospe_without_cross() {
    let c = mono_output(Configuration::Knospe, MonoQuality::KnospeImport).conditionals();

    assert_eq!(c.get(keys::KNOSPE_MARKETING_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::BIO_SUISSE_NO_CROSS), Some(&true), "import → logo without cross");
    assert_eq!(c.get(keys::BIO_SUISSE_REGULAR), None);
    assert_eq!(c.get(keys::KNOSPE_UMSTELLUNG_LOGO), None);
}

#[test]
fn mono_umstellung_knospe_ch_shows_the_umstellungsknospe() {
    let c = mono_output(Configuration::Knospe, MonoQuality::UmstellungKnospeCh).conditionals();

    assert_eq!(c.get(keys::KNOSPE_MARKETING_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::BIO_SUISSE_REGULAR), Some(&true));
    // The Umstellungssatz is mandatory and rides on this flag.
    assert_eq!(c.get(keys::KNOSPE_UMSTELLUNG_LOGO), Some(&true));
}

#[test]
fn mono_umstellung_knospe_import_shows_the_imported_umstellungsknospe() {
    let c =
        mono_output(Configuration::Knospe, MonoQuality::UmstellungKnospeImport).conditionals();

    assert_eq!(c.get(keys::KNOSPE_MARKETING_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::BIO_SUISSE_NO_CROSS), Some(&true));
    assert_eq!(c.get(keys::KNOSPE_UMSTELLUNG_LOGO), Some(&true));
}

#[test]
fn mono_nicht_biologisch_shows_no_knospe() {
    let c = mono_output(Configuration::Knospe, MonoQuality::Andere).conditionals();

    assert_eq!(c.get(keys::KNOSPE_MARKETING_ALLOWED), None);
    assert_eq!(c.get(keys::KNOSPE_MARKETING_NOT_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::BIO_SUISSE_REGULAR), None);
    assert_eq!(c.get(keys::BIO_SUISSE_NO_CROSS), None);
}

#[test]
fn mono_bio_without_knospe_shows_no_knospe_logo() {
    // Bio-CH is not Knospe: in the Knospe instance such a product gets no logo.
    let c = mono_output(Configuration::Knospe, MonoQuality::Bio).conditionals();

    assert_eq!(c.get(keys::KNOSPE_MARKETING_ALLOWED), None);
    assert_eq!(c.get(keys::BIO_SUISSE_REGULAR), None);
    assert_eq!(c.get(keys::BIO_SUISSE_NO_CROSS), None);
}

// --- Bio-V configuration --------------------------------------------------

#[test]
fn mono_bio_gets_the_bio_sachbezeichnung() {
    let c = mono_output(Configuration::Bio, MonoQuality::Bio).conditionals();

    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), Some(&true));
    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::BIO_MARKETING_NOT_ALLOWED), None);
}

#[test]
fn mono_bio_umstellung_gets_bio_plus_the_umstellungshinweis() {
    // Excel Zeile 7: a Monoprodukt from a conversion farm may carry «Bio» in the
    // Sachbezeichnung, together with the mandatory Umstellungshinweis.
    let c = mono_output(Configuration::Bio, MonoQuality::BioUmstellung).conditionals();

    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), Some(&true));
    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), Some(&true));
    assert_eq!(c.get(keys::UMSTELLBETRIEB_HINWEIS), Some(&true));
}

#[test]
fn mono_nicht_biologisch_gets_no_bio_sachbezeichnung() {
    let c = mono_output(Configuration::Bio, MonoQuality::Andere).conditionals();

    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), None);
    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), None);
    assert_eq!(c.get(keys::BIO_MARKETING_NOT_ALLOWED), Some(&true));
}

#[test]
fn mono_nicht_landwirtschaftlich_makes_no_bio_claim() {
    // Salt/water: no agricultural ingredient at all, so there is nothing to
    // certify — and nothing may be claimed.
    let c = mono_output(Configuration::Bio, MonoQuality::NichtLandwirtschaftlich)
        .conditionals();

    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), None);
    assert_eq!(c.get(keys::BIO_MARKETING_ALLOWED), None);
}

// --- Cross-cutting --------------------------------------------------------

#[test]
fn mono_quality_does_not_leak_into_normal_recipes() {
    // With a real recipe the mono quality must be ignored entirely, otherwise a
    // stale selection would silently upgrade a conventional product.
    let form = Form {
        ignore_ingredients: false,
        mono_quality: MonoQuality::KnospeCh,
        ingredients: vec![
            IngredientBuilder::new_agri("Zucker", 1000.0).origin(Country::CH).build(),
        ],
        rezeptur_vollstaendig: true,
        ..Form::default()
    };
    let c = calculator_for(Configuration::Knospe)
        .execute(form.into())
        .conditionals();

    assert_eq!(c.get(keys::KNOSPE_MARKETING_ALLOWED), None);
    assert_eq!(c.get(keys::KNOSPE_MARKETING_NOT_ALLOWED), Some(&true));
}

#[test]
fn mono_default_quality_claims_nothing() {
    // A user who has not chosen yet must not get an unearned Bio claim.
    assert_eq!(MonoQuality::default(), MonoQuality::Andere);
    let c = mono_output(Configuration::Bio, MonoQuality::default()).conditionals();
    assert_eq!(c.get(keys::BIO_SACHBEZEICHNUNG_SUFFIX), None);
}

#[test]
fn mono_label_stays_empty_in_every_quality() {
    // «Keine Zutatenliste» means no ingredient list is rendered; the synthetic
    // ingredient exists only for the rule math and must never reach the label.
    for quality in [
        MonoQuality::Andere,
        MonoQuality::Bio,
        MonoQuality::BioUmstellung,
        MonoQuality::KnospeCh,
        MonoQuality::KnospeImport,
        MonoQuality::UmstellungKnospeCh,
        MonoQuality::UmstellungKnospeImport,
        MonoQuality::NichtLandwirtschaftlich,
    ] {
        let output = mono_output(Configuration::Knospe, quality);
        assert!(
            output.label.trim().is_empty(),
            "mono product must not render an ingredient list ({:?}): {}",
            quality,
            output.label
        );
    }
}

#[test]
fn mono_quality_survives_the_share_link_roundtrip() {
    // The selection is part of the shared URL; losing it would silently drop the
    // Knospe from a shared label.
    let form = Form {
        ignore_ingredients: true,
        mono_quality: MonoQuality::UmstellungKnospeImport,
        ..Form::default()
    };
    let qs = qs_to_string(&form).expect("serialize");
    let restored: Form = qs_from_str(&qs).expect("deserialize");

    assert_eq!(restored.mono_quality, MonoQuality::UmstellungKnospeImport);
    assert!(restored.ignore_ingredients);
}

#[test]
fn legacy_links_without_mono_quality_still_parse() {
    // URLs shared before DEC-2 carry no mono_quality; they must default to
    // «nicht-biologisch» rather than fail to load.
    let restored: Form = qs_from_str("v=2&ignore_ingredients=true").expect("legacy link parses");
    assert_eq!(restored.mono_quality, MonoQuality::Andere);
}
