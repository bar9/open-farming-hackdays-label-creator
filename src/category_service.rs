//! Centralized service for handling food category determination and derived flags.
//! This service consolidates all category-based logic that was previously duplicated.

/// Check if a category represents fish
pub fn is_fish_category(category: &str) -> bool {
    let category_lower = category.to_lowercase();

    // Check for fish-specific categories from BLV API
    category_lower == "fisch" ||
    category_lower == "meeresfische" ||
    category_lower == "süsswasserfische" ||
    category_lower == "meeresfrüchte, krusten- und schalentiere" ||

    // Generic fish terms (fallback)
    category_lower.contains("fisch") ||
    category_lower.contains("lachs") ||
    category_lower.contains("thun") ||
    category_lower.contains("forelle") ||

    // English terms (for international compatibility)
    category_lower.contains("fish") ||
    category_lower.contains("salmon") ||
    category_lower.contains("tuna") ||
    category_lower.contains("trout") ||
    category_lower.contains("seafood")
}

/// Check if a category represents beef/cattle
pub fn is_beef_category(category: &str) -> bool {
    let category_lower = category.to_lowercase();

    // Check for beef/cattle specific categories
    category_lower == "rind"
        || category_lower == "rindfleisch"
        || category_lower.contains("rind")
        || category_lower.contains("beef")
        || category_lower.contains("cattle")
}

/// Check if a category represents meat
pub fn is_meat_category(category: &str) -> bool {
    let category_lower = category.to_lowercase();

    // Official BLV API categories for meat products
    // Direct meat category matches
    category_lower == "fleisch und innereien" ||

    // Individual animal categories from API
    category_lower == "rind" ||
    category_lower == "schwein" ||
    category_lower == "kalb" ||
    category_lower == "geflügel" ||
    category_lower == "lamm, schaf" ||
    category_lower == "wild" ||

    // Processed meat categories from API
    category_lower == "brühwurstware" ||
    category_lower == "kochwurstware" ||

    // Combined categories (semicolon-separated)
    category_lower.contains("rind") ||
    category_lower.contains("schwein") ||
    category_lower.contains("kalb") ||
    category_lower.contains("geflügel") ||
    category_lower.contains("lamm") ||
    category_lower.contains("schaf") ||
    category_lower.contains("wild") ||

    // Generic meat terms (fallback)
    category_lower.contains("fleisch") ||
    category_lower.contains("wurst") ||

    // English terms (for international compatibility)
    category_lower.contains("meat") ||
    category_lower.contains("beef") ||
    category_lower.contains("pork") ||
    category_lower.contains("veal") ||
    category_lower.contains("lamb") ||
    category_lower.contains("mutton") ||
    category_lower.contains("chicken") ||
    category_lower.contains("poultry") ||
    category_lower.contains("turkey") ||
    category_lower.contains("duck") ||
    category_lower.contains("goose") ||
    category_lower.contains("venison") ||
    category_lower.contains("rabbit") ||
    category_lower.contains("ham") ||
    category_lower.contains("bacon") ||
    category_lower.contains("sausage") ||
    category_lower.contains("salami")
}

/// Check if a category represents eggs
pub fn is_egg_category(category: &str) -> bool {
    let category_lower = category.to_lowercase();

    // Official BLV API categories and common terms for eggs
    category_lower == "eier" ||
    category_lower == "ei" ||
    category_lower.contains("eier") ||
    category_lower.contains("hühnerei") ||

    // English terms
    category_lower == "eggs" ||
    category_lower == "egg" ||
    category_lower.contains("egg")
}

/// Is this Sachbezeichnung a plain pack of eggs (DEC-13)?
///
/// Deliberately much stricter than `is_egg_category`: that one answers "does
/// this ingredient contain egg" for the origin rules, where a substring match
/// is what you want. Here a false positive would silently relabel the
/// Grundpreis field of an unrelated product, so only the whole Sachbezeichnung
/// counts — «Eier», «Ei» and the French/Italian equivalents, optionally with a
/// qualifier like «Eier aus Freilandhaltung». Compounds such as «Eierlikör»,
/// «Eiernudeln» or «Eiweiss» are sold by weight and must not match.
pub fn is_egg_sachbezeichnung(sachbezeichnung: &str) -> bool {
    // Only the first word decides; a trailing qualifier ("Eier aus
    // Freilandhaltung", "Uova bio") is still a pack of eggs.
    let first = sachbezeichnung
        .trim()
        .split([' ', ',', '-', '/'])
        .find(|w| !w.is_empty())
        .unwrap_or("")
        .to_lowercase();

    matches!(
        first.as_str(),
        // de: Ei/Eier, plus the common "Frischei(er)" spelling.
        "ei" | "eier" | "frischei" | "frischeier"
        // fr: œuf/œufs, also written oeuf/oeufs.
        | "oeuf" | "oeufs" | "œuf" | "œufs"
        // it: uovo/uova.
        | "uovo" | "uova"
        // en, for completeness with the other detectors here.
        | "egg" | "eggs"
    )
}

/// Is this ingredient, by its name, an egg or egg part for the Knospe origin
/// rule «Eier und Honig ab 10 %»? Mirjam (02.10.2026) asked for exactly these
/// names: Ei, Eier, Hühnerei, Hühnerei ganz, Eigelb, Eiweiss. The whole name
/// (or its first word for «Hühnerei ganz») must match, so «Eierlikör» or
/// «Eiernudeln» do not count. FR/IT/EN equivalents are included for the other
/// UI languages.
pub fn is_egg_ingredient_name(name: &str) -> bool {
    let n = name.trim().to_lowercase();
    let first = n
        .split([' ', ',', '(', '-', '/'])
        .find(|w| !w.is_empty())
        .unwrap_or("");
    let egg_word = matches!(
        first,
        "ei" | "eier"
            | "hühnerei"
            | "hühnereier"
            | "eigelb"
            | "eiweiss"
            | "eiweiß"
            | "vollei"
            | "frischei"
            | "frischeier"
            | "oeuf"
            | "oeufs"
            | "œuf"
            | "œufs"
            | "uovo"
            | "uova"
            | "tuorlo"
            | "albume"
            | "egg"
            | "eggs"
    );
    // «jaune d'œuf», «blanc d'œuf»: egg parts only together with «œuf/oeuf».
    let fr_egg_part =
        matches!(first, "jaune" | "blanc") && (n.contains("œuf") || n.contains("oeuf"));
    egg_word || fr_egg_part
}

/// Is this ingredient, by its name, honey (Knospe origin rule «ab 10 %»)?
pub fn is_honey_ingredient_name(name: &str) -> bool {
    let n = name.trim().to_lowercase();
    let first = n
        .split([' ', ',', '(', '-', '/'])
        .find(|w| !w.is_empty())
        .unwrap_or("");
    matches!(
        first,
        "honig" | "blütenhonig" | "waldhonig" | "miel" | "miele" | "honey"
    )
}

/// Check if a category represents honey
pub fn is_honey_category(category: &str) -> bool {
    let category_lower = category.to_lowercase();

    // Official BLV API categories and common terms for honey
    category_lower == "honig" ||
    category_lower.contains("honig") ||

    // English terms
    category_lower == "honey" ||
    category_lower.contains("honey")
}

/// Is this ingredient, by its name, milk or a milk product? Mirjam
/// (12.11.2026): «alles aus Milch» must always carry its origin on a BIO
/// KNOSPE label, with or without a BLV category match. The category check
/// (`is_dairy_category`) stays authoritative when a category exists; this
/// name fallback catches manual entries and food_db/BLV spelling variants
/// («Butter», «Greyerzer, vollfett»). Plant-based lookalikes (Kokosmilch,
/// Kakaobutter, Erdnussbutter) and Leberkäse must never match.
pub fn is_dairy_ingredient_name(name: &str) -> bool {
    let n = name.trim().to_lowercase();
    let first = n
        .split([' ', ',', '(', '-', '/'])
        .find(|w| !w.is_empty())
        .unwrap_or("");

    // Non-dairy compounds that end in a dairy word.
    let not_dairy = matches!(
        first,
        "kokosmilch"
            | "kokosnussmilch"
            | "mandelmilch"
            | "hafermilch"
            | "sojamilch"
            | "reismilch"
            | "dinkelmilch"
            | "kakaobutter"
            | "erdnussbutter"
            | "sheabutter"
            | "mandelbutter"
            | "nussbutter"
            | "leberkäse"
            | "fleischkäse"
    );
    if not_dairy {
        return false;
    }
    // fr/it plant "milks" and cocoa butter: «lait de coco», «burro di cacao» …
    if matches!(first, "lait" | "latte")
        && ["coco", "amande", "avoine", "soja", "riz", "mandorla", "avena", "soia", "riso"]
            .iter()
            .any(|p| n.contains(p))
    {
        return false;
    }
    if matches!(first, "beurre" | "burro") && (n.contains("cacao") || n.contains("cacahuète")) {
        return false;
    }

    // Exact dairy words on the first word (de/fr/it/en + common cheeses).
    let exact = matches!(
        first,
        "milch"
            | "butter"
            | "rahm"
            | "sahne"
            | "joghurt"
            | "jogurt"
            | "quark"
            | "käse"
            | "molke"
            | "kefir"
            | "crème"
            | "mozzarella"
            | "mascarpone"
            | "ricotta"
            | "parmesan"
            | "parmigiano"
            | "sbrinz"
            | "sprinz"
            | "emmentaler"
            | "greyerzer"
            | "gruyère"
            | "appenzeller"
            | "tilsiter"
            | "raclette"
            | "feta"
            | "camembert"
            | "brie"
            | "gorgonzola"
            | "mutschli"
            | "vacherin"
            | "schabziger"
            | "lait"
            | "beurre"
            | "fromage"
            | "yogourt"
            | "yaourt"
            | "séré"
            | "sérac"
            | "latte"
            | "burro"
            | "panna"
            | "formaggio"
            | "yogurt"
            | "milk"
            | "cream"
            | "cheese"
            | "yoghurt"
            | "curd"
            | "whey"
    );
    if exact {
        return true;
    }

    // German compounds: Vollmilch, Buttermilch, Magerquark, Hartkäse,
    // Sauerrahm, Vollmilchpulver … (exclusions already handled above).
    [
        "milch",
        "milchpulver",
        "butter",
        "rahm",
        "quark",
        "käse",
        "joghurt",
        "jogurt",
    ]
    .iter()
    .any(|s| first.ends_with(s))
}

/// Check if a category represents milk or dairy products
pub fn is_dairy_category(category: &str) -> bool {
    let category_lower = category.to_lowercase();

    // Official BLV API categories for dairy products
    category_lower == "milch und milchprodukte" ||
    category_lower == "milch" ||
    category_lower == "milchprodukte" ||
    category_lower == "käse" ||
    category_lower == "joghurt" ||
    category_lower == "quark" ||
    category_lower == "butter" ||
    category_lower == "rahm" ||
    category_lower == "sahne" ||
    category_lower == "frischkäse" ||
    // BLV subcategories like «Hartkäse», «Weichkäse», «Halbhartkäse»
    // (Mirjam 13.11.2026: Greyerzer arrives as «Hartkäse»).
    category_lower.ends_with("käse") ||
    category_lower.contains("milchprodukt") ||

    // English terms
    category_lower == "milk" ||
    category_lower == "dairy" ||
    category_lower.contains("milk") ||
    category_lower.contains("dairy") ||
    category_lower.contains("cheese") ||
    category_lower.contains("yogurt") ||
    category_lower.contains("yoghurt") ||
    category_lower.contains("butter") ||
    category_lower.contains("cream")
}

/// Check if a category represents insects or insect products
pub fn is_insect_category(category: &str) -> bool {
    let category_lower = category.to_lowercase();

    // Categories for insect products
    category_lower == "insekten" ||
    category_lower == "insektenprodukte" ||
    category_lower.contains("insekt") ||
    category_lower.contains("grille") ||
    category_lower.contains("heuschrecke") ||
    category_lower.contains("mehlwurm") ||

    // English terms
    category_lower == "insects" ||
    category_lower.contains("insect") ||
    category_lower.contains("cricket") ||
    category_lower.contains("grasshopper") ||
    category_lower.contains("mealworm")
}

/// Check if a category represents plant-based ingredients
pub fn is_plant_category(category: &str) -> bool {
    let category_lower = category.to_lowercase();

    // First exclude animal products
    if is_meat_category(category)
        || is_fish_category(category)
        || is_egg_category(category)
        || is_honey_category(category)
        || is_dairy_category(category)
        || is_insect_category(category)
    {
        return false;
    }

    // Check for plant-based category keywords
    category_lower.contains("gemüse") ||
    category_lower.contains("obst") ||
    category_lower.contains("getreide") ||
    category_lower.contains("nüsse") ||
    category_lower.contains("samen") ||
    category_lower.contains("früchte") ||
    category_lower.contains("hülsenfrüchte") ||
    category_lower.contains("kräuter") ||
    category_lower.contains("sprossen") ||
    category_lower.contains("kartoffel") ||
    category_lower.contains("brot") ||
    category_lower.contains("teigwaren") ||
    category_lower.contains("zucker") ||
    category_lower.contains("kaffee") ||
    category_lower.contains("kakao") ||
    category_lower.contains("schokolade") ||
    category_lower.contains("wein") ||
    category_lower.contains("bier") ||
    category_lower.contains("öl") ||
    category_lower.contains("pflanzlich") ||
    category_lower.contains("hefe") ||
    category_lower.contains("konfitüre") ||
    // English keywords
    category_lower.contains("vegetable") ||
    category_lower.contains("fruit") ||
    category_lower.contains("grain") ||
    category_lower.contains("nut") ||
    category_lower.contains("seed") ||
    category_lower.contains("legume") ||
    category_lower.contains("pulse") ||
    category_lower.contains("herb") ||
    category_lower.contains("potato") ||
    category_lower.contains("bread") ||
    category_lower.contains("pasta") ||
    category_lower.contains("plant")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dairy_ingredient_name_detection() {
        // Mirjam's recipe (12.11.2026): these must count as dairy by name.
        assert!(is_dairy_ingredient_name("Butter"));
        assert!(is_dairy_ingredient_name("Greyerzer, vollfett"));
        assert!(is_dairy_ingredient_name("Greyerzer vollfett"));
        assert!(is_dairy_ingredient_name("Milch"));
        assert!(is_dairy_ingredient_name("Joghurt nature"));
        assert!(is_dairy_ingredient_name("Magerquark"));
        assert!(is_dairy_ingredient_name("Quark"));
        // Compounds
        assert!(is_dairy_ingredient_name("Vollmilchpulver"));
        assert!(is_dairy_ingredient_name("Buttermilch"));
        assert!(is_dairy_ingredient_name("Sauerrahm"));
        assert!(is_dairy_ingredient_name("Hartkäse vollfett"));
        assert!(is_dairy_ingredient_name("Kochbutter"));
        // fr/it
        assert!(is_dairy_ingredient_name("Beurre"));
        assert!(is_dairy_ingredient_name("Lait entier"));
        assert!(is_dairy_ingredient_name("Burro"));
        assert!(is_dairy_ingredient_name("Formaggio"));

        // Plant lookalikes and traps must NOT match.
        assert!(!is_dairy_ingredient_name("Kakaobutter"));
        assert!(!is_dairy_ingredient_name("Erdnussbutter"));
        assert!(!is_dairy_ingredient_name("Kokosnussmilch"));
        assert!(!is_dairy_ingredient_name("Kokosmilch"));
        assert!(!is_dairy_ingredient_name("Hafermilch"));
        assert!(!is_dairy_ingredient_name("Lait de coco"));
        assert!(!is_dairy_ingredient_name("Burro di cacao"));
        assert!(!is_dairy_ingredient_name("Leberkäse"));
        assert!(!is_dairy_ingredient_name("Weizenmehl"));
        assert!(!is_dairy_ingredient_name("Zucker"));
    }

    #[test]
    fn test_meat_category_detection() {
        assert!(is_meat_category("Fleisch und Innereien"));
        assert!(is_meat_category("Rind"));
        assert!(is_meat_category("Schwein"));
        assert!(is_meat_category(
            "Kalb; Lamm, Schaf; Rind; Schwein; Wild; Geflügel"
        ));
        assert!(is_meat_category("beef"));
        assert!(!is_meat_category("Getreide"));
        assert!(!is_meat_category("Milch"));
    }

    #[test]
    fn test_fish_category_detection() {
        assert!(is_fish_category("Fisch"));
        assert!(is_fish_category("Meeresfische"));
        assert!(is_fish_category("Lachs"));
        assert!(is_fish_category("fish"));
        assert!(!is_fish_category("Rind"));
        assert!(!is_fish_category("Getreide"));
    }

    #[test]
    fn test_beef_category_detection() {
        assert!(is_beef_category("Rind"));
        assert!(is_beef_category("Rindfleisch"));
        assert!(is_beef_category("beef"));
        assert!(!is_beef_category("Schwein"));
        assert!(!is_beef_category("Geflügel"));
    }

    #[test]
    fn test_plant_category_detection() {
        assert!(is_plant_category("Getreide"));
        assert!(is_plant_category("Gemüse"));
        assert!(is_plant_category("Obst"));
        assert!(!is_plant_category("Fleisch"));
        assert!(!is_plant_category("Milch"));
    }

    // DEC-13: a pack of eggs is declared by count, so the Sachbezeichnung
    // switches the Grundpreis field to «Anzahl Eier».
    #[test]
    fn egg_sachbezeichnung_matches_plain_egg_packs() {
        assert!(is_egg_sachbezeichnung("Eier"));
        assert!(is_egg_sachbezeichnung("Ei"));
        assert!(is_egg_sachbezeichnung("eier"));
        assert!(is_egg_sachbezeichnung("  Eier  "));
        // Qualifiers keep it a pack of eggs.
        assert!(is_egg_sachbezeichnung("Eier aus Freilandhaltung"));
        assert!(is_egg_sachbezeichnung("Eier, Bio"));
        assert!(is_egg_sachbezeichnung("Frischeier"));
        // fr / it
        assert!(is_egg_sachbezeichnung("\u{152}ufs"));
        assert!(is_egg_sachbezeichnung(
            "Oeufs de poules \u{e9}lev\u{e9}es en plein air"
        ));
        assert!(is_egg_sachbezeichnung("Uova"));
    }

    #[test]
    fn egg_sachbezeichnung_ignores_products_merely_containing_egg() {
        // Sold by weight \u{2014} must keep the normal Grundpreis/Abtropfgewicht.
        assert!(!is_egg_sachbezeichnung("Eierlik\u{f6}r"));
        assert!(!is_egg_sachbezeichnung("Eiernudeln"));
        assert!(!is_egg_sachbezeichnung("Eiweiss"));
        assert!(!is_egg_sachbezeichnung("Eiersalat"));
        assert!(!is_egg_sachbezeichnung("Teigwaren mit Ei"));
        assert!(!is_egg_sachbezeichnung(""));
        assert!(!is_egg_sachbezeichnung("Konfit\u{fc}re"));
    }
}

#[cfg(test)]
mod origin_name_tests {
    use super::*;

    #[test]
    fn egg_names_from_mirjams_list_match() {
        for n in [
            "Ei",
            "Eier",
            "Hühnerei",
            "Hühnerei ganz",
            "Eigelb",
            "Eiweiss",
            "Œufs",
            "jaune d'œuf",
            "Uova",
            "Tuorlo",
        ] {
            assert!(is_egg_ingredient_name(n), "{n}");
        }
    }

    #[test]
    fn egg_compounds_and_lookalikes_do_not_match() {
        for n in [
            "Eierlikör",
            "Eiernudeln",
            "Eisbergsalat",
            "Weizenmehl",
            "blanc de poulet",
            "Eis",
        ] {
            assert!(!is_egg_ingredient_name(n), "{n}");
        }
    }

    #[test]
    fn honey_names_match() {
        for n in ["Honig", "Blütenhonig", "Miel", "Miele"] {
            assert!(is_honey_ingredient_name(n), "{n}");
        }
        assert!(!is_honey_ingredient_name("Honigmelone"));
    }
}
