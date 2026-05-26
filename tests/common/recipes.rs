// Recipe data shared between Calculator-tier (src/core/tests/) and e2e tier.
//
// The same source-of-truth recipes that live in `src/core/tests/golden.rs`
// and `src/core/tests/recipes.rs` are mirrored here so the WASM rendering
// tier can seed the form via UI clicks and assert against the rendered
// label HTML. When a recipe changes upstream (e.g. amounts adjusted in
// `requirements/Rezepte Declarino.xlsx`), update both sides.
//
// Recipes are kept deliberately minimal — only the fields relevant to the
// e2e assertions are populated. If a future test needs more (allergen
// flags, custom processing steps), extend the structs.

#![allow(dead_code)]

#[derive(Clone, Copy)]
pub enum Config {
    Lebensmittelrecht,
    Bio,
    Knospe,
}

impl Config {
    pub fn route(self) -> &'static str {
        match self {
            Config::Lebensmittelrecht => "lebensmittelrecht",
            Config::Bio => "bio",
            Config::Knospe => "knospe",
        }
    }

    /// German label as it appears in the configuration dropdown.
    pub fn label_de(self) -> &'static str {
        match self {
            Config::Lebensmittelrecht => "Lebensmittelrecht",
            Config::Bio => "Bio",
            Config::Knospe => "Knospe",
        }
    }
}

#[derive(Clone, Copy)]
pub enum BioStatus {
    /// Plain conventional ingredient.
    Conventional,
    /// `bio_ch` checkbox set (Bio config).
    BioCh,
    /// "Bio (Knospe)" radio (Knospe config) — Swiss, origin locked to CH.
    BioKnospe,
    /// "Bio (Knospe) Import" radio (Knospe config) — imported Knospe, origin kept.
    BioKnospeImport,
    /// "Nicht-landwirtschaftliche Zutat" radio (Knospe config) — Salz, Wasser, etc.
    NichtLandwirtschaftlich,
    /// "Andere" / non-bio agricultural (Knospe config).
    Andere,
}

#[derive(Clone)]
pub struct RecipeIngredient {
    pub name: &'static str,
    pub grams: f64,
    /// ISO country code as recognized by `Country` enum: "CH", "DE", "FR", "IT", "EU", or `None`.
    pub origin: Option<&'static str>,
    pub bio: BioStatus,
}

#[derive(Clone)]
pub struct Recipe {
    pub config: Config,
    pub product_name: &'static str,
    pub sachbezeichnung: &'static str,
    /// Required for Bio + Knospe; ignored for Lebensmittelrecht.
    pub certification_body: Option<&'static str>,
    pub ingredients: &'static [RecipeIngredient],
}

// =============================================================================
// MT §2.1 — Erdbeer-Fruchtaufstrich (mirrors `golden_erdbeer_fruchtaufstrich`)
// =============================================================================

pub const ERDBEER_FRUCHTAUFSTRICH: Recipe = Recipe {
    config: Config::Bio,
    product_name: "Erdbeer-Fruchtaufstrich",
    sachbezeichnung: "Konfitüre extra mit weniger Zucker",
    certification_body: Some("CH-BIO-006"),
    ingredients: &[
        // Names match `src/food_db.csv` exactly so the typeahead's exact-
        // match path is taken (see `add_full_ingredient` in tests/common).
        RecipeIngredient { name: "Erdbeere",     grams: 200.0, origin: Some("CH"), bio: BioStatus::BioCh },
        RecipeIngredient { name: "Zucker",       grams: 140.0, origin: Some("DE"), bio: BioStatus::BioCh },
        RecipeIngredient { name: "Zitronensaft", grams: 8.0,   origin: Some("IT"), bio: BioStatus::BioCh },
        RecipeIngredient { name: "Pektin",       grams: 2.0,   origin: None,        bio: BioStatus::Conventional },
    ],
};

/// Same as `ERDBEER_FRUCHTAUFSTRICH` but with all bio flags off — drives the
/// "Kein-Bio"-warning code path (MT §2.4).
pub const ERDBEER_FRUCHTAUFSTRICH_NO_BIO: Recipe = Recipe {
    config: Config::Bio,
    product_name: "Erdbeer-Fruchtaufstrich (kein Bio)",
    sachbezeichnung: "Konfitüre extra mit weniger Zucker",
    certification_body: Some("CH-BIO-006"),
    ingredients: &[
        RecipeIngredient { name: "Erdbeere",     grams: 200.0, origin: Some("CH"), bio: BioStatus::Conventional },
        RecipeIngredient { name: "Zucker",       grams: 140.0, origin: Some("DE"), bio: BioStatus::Conventional },
        RecipeIngredient { name: "Zitronensaft", grams: 8.0,   origin: Some("IT"), bio: BioStatus::Conventional },
        RecipeIngredient { name: "Pektin",       grams: 2.0,   origin: None,        bio: BioStatus::Conventional },
    ],
};

// =============================================================================
// MT §3.1 — Schoggi Cookie BSK (90-99% Swiss, regular logo)
// Simplified: we keep the leaf-level ingredients only. Composite chocolate
// children are skipped — the test asserts on top-level rendering.
// =============================================================================

pub const SCHOGGI_COOKIE_BSK: Recipe = Recipe {
    config: Config::Knospe,
    product_name: "Schoggi Cookie",
    sachbezeichnung: "Schokoladenguetzli",
    certification_body: Some("CH-BIO-006"),
    // Bratbutter / Ei are food_db-exact entries; "Butter" / "Eier" would
    // collapse to near-matches via the typeahead first-result-select path.
    ingredients: &[
        RecipeIngredient { name: "Weizenmehl", grams: 33.5, origin: Some("CH"), bio: BioStatus::BioKnospe },
        RecipeIngredient { name: "Zucker",     grams: 23.0, origin: Some("CH"), bio: BioStatus::BioKnospe },
        RecipeIngredient { name: "Bratbutter", grams: 22.0, origin: Some("CH"), bio: BioStatus::BioKnospe },
        RecipeIngredient { name: "Ei",         grams: 9.0,  origin: Some("CH"), bio: BioStatus::BioKnospe },
        RecipeIngredient { name: "Salz",       grams: 0.5,  origin: Some("CH"), bio: BioStatus::NichtLandwirtschaftlich },
    ],
};

/// Same as `SCHOGGI_COOKIE_BSK` but with Bratbutter sourced from FR, drops
/// Swiss percentage below 90% → expected `bio_suisse_no_cross` logo (MT §3.2).
pub const SCHOGGI_COOKIE_BSK_FR_BUTTER: Recipe = Recipe {
    config: Config::Knospe,
    product_name: "Schoggi Cookie (FR Butter)",
    sachbezeichnung: "Schokoladenguetzli",
    certification_body: Some("CH-BIO-006"),
    ingredients: &[
        RecipeIngredient { name: "Weizenmehl", grams: 33.5, origin: Some("CH"), bio: BioStatus::BioKnospe },
        RecipeIngredient { name: "Zucker",     grams: 23.0, origin: Some("CH"), bio: BioStatus::BioKnospe },
        RecipeIngredient { name: "Bratbutter", grams: 22.0, origin: Some("FR"), bio: BioStatus::BioKnospeImport },
        RecipeIngredient { name: "Ei",         grams: 9.0,  origin: Some("CH"), bio: BioStatus::BioKnospe },
        RecipeIngredient { name: "Salz",       grams: 0.5,  origin: Some("CH"), bio: BioStatus::NichtLandwirtschaftlich },
    ],
};

// =============================================================================
// MT §1.1 — Rindshackbraten (Swiss Lebensmittelrecht, beef + allergen)
// =============================================================================

pub const RINDSHACKBRATEN: Recipe = Recipe {
    config: Config::Lebensmittelrecht,
    product_name: "Hausgemachter Rindshackbraten",
    sachbezeichnung: "Rindshackbraten",
    certification_body: None,
    ingredients: &[
        RecipeIngredient { name: "Rindfleisch", grams: 350.0, origin: Some("CH"), bio: BioStatus::Conventional },
        // "Zwiebel" (singular) is the food_db entry. Plural collapses to
        // "Silberzwiebeln" via the first-match path.
        RecipeIngredient { name: "Zwiebel",     grams: 80.0,  origin: Some("CH"), bio: BioStatus::Conventional },
        // "Brötchen" has no food_db match — expected to commit as a
        // custom ingredient via Enter (no near-match dropdown).
        RecipeIngredient { name: "Brötchen",    grams: 50.0,  origin: Some("CH"), bio: BioStatus::Conventional },
        RecipeIngredient { name: "Ei",          grams: 50.0,  origin: Some("CH"), bio: BioStatus::Conventional },
        RecipeIngredient { name: "Salz",        grams: 5.0,   origin: None,       bio: BioStatus::Conventional },
    ],
};

// =============================================================================
// MT §3.3 — Wildkräuter / Wildsammlung scenarios
// =============================================================================

/// 25g Wildkräuter of ~99g total → > 10% threshold, ° marker + legend expected.
pub const WILDKRAEUTER_25G: Recipe = Recipe {
    config: Config::Knospe,
    product_name: "Wildkräuter-Mix",
    sachbezeichnung: "Wildkräuter-Mix",
    certification_body: Some("CH-BIO-006"),
    ingredients: &[
        RecipeIngredient { name: "Wildkräuter", grams: 25.0, origin: Some("CH"), bio: BioStatus::BioKnospe },
        RecipeIngredient { name: "Salz",        grams: 1.0,  origin: Some("CH"), bio: BioStatus::NichtLandwirtschaftlich },
        RecipeIngredient { name: "Olivenöl",    grams: 73.0, origin: Some("CH"), bio: BioStatus::BioKnospe },
    ],
};

/// 5g Wildkräuter of ~99g total → < 10% → no ° marker, no legend.
pub const WILDKRAEUTER_5G: Recipe = Recipe {
    config: Config::Knospe,
    product_name: "Wildkräuter-Mix klein",
    sachbezeichnung: "Wildkräuter-Mix",
    certification_body: Some("CH-BIO-006"),
    ingredients: &[
        RecipeIngredient { name: "Wildkräuter", grams: 5.0,  origin: Some("CH"), bio: BioStatus::BioKnospe },
        RecipeIngredient { name: "Salz",        grams: 1.0,  origin: Some("CH"), bio: BioStatus::NichtLandwirtschaftlich },
        RecipeIngredient { name: "Olivenöl",    grams: 93.0, origin: Some("CH"), bio: BioStatus::BioKnospe },
    ],
};

// =============================================================================
// `Rezepte Declarino.xlsx` — Bärlauch Pesto variants
// Mirrors `src/core/tests/recipes.rs::recipe_baerlauch_pesto_{bk,bsk}`.
// Both are flat recipes (no composites) so seedable via `seed_recipe_via_ui`.
// =============================================================================

/// Bärlauch Pesto BK — ~71% Swiss agri (250g CH / 350g agri) → Under90%
/// → expected `bio_suisse_no_cross` logo.
pub const BAERLAUCH_PESTO_BK: Recipe = Recipe {
    config: Config::Knospe,
    product_name: "Bärlauch Pesto",
    sachbezeichnung: "Bärlauchpesto",
    certification_body: Some("CH-BIO-006"),
    ingredients: &[
        RecipeIngredient { name: "Rapsöl",   grams: 150.0, origin: Some("CH"), bio: BioStatus::BioKnospe },
        RecipeIngredient { name: "Bärlauch", grams: 100.0, origin: Some("CH"), bio: BioStatus::BioKnospe },
        RecipeIngredient { name: "Mandeln",  grams: 50.0,  origin: Some("TR"), bio: BioStatus::BioKnospeImport },
        RecipeIngredient { name: "Parmesan", grams: 50.0,  origin: Some("IT"), bio: BioStatus::BioKnospeImport },
        RecipeIngredient { name: "Salz",     grams: 5.0,   origin: Some("EU"), bio: BioStatus::NichtLandwirtschaftlich },
    ],
};

/// Bärlauch Pesto BSK — ~90.5% Swiss agri (475g CH / 525g agri) → 90-99%
/// → expected `bio_suisse_regular` logo. Tier B forces all CH bio agri to
/// show origin.
pub const BAERLAUCH_PESTO_BSK: Recipe = Recipe {
    config: Config::Knospe,
    product_name: "Bärlauch Pesto BSK",
    sachbezeichnung: "Bärlauchpesto",
    certification_body: Some("CH-BIO-006"),
    ingredients: &[
        RecipeIngredient { name: "Rapsöl",    grams: 175.0, origin: Some("CH"), bio: BioStatus::BioKnospe },
        RecipeIngredient { name: "Bärlauch",  grams: 150.0, origin: Some("CH"), bio: BioStatus::BioKnospe },
        RecipeIngredient { name: "Baumnüsse", grams: 150.0, origin: Some("CH"), bio: BioStatus::BioKnospe },
        RecipeIngredient { name: "Parmesan",  grams: 50.0,  origin: Some("IT"), bio: BioStatus::BioKnospeImport },
        RecipeIngredient { name: "Salz",      grams: 5.0,   origin: Some("EU"), bio: BioStatus::NichtLandwirtschaftlich },
    ],
};
