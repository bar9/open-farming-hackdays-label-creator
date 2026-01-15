/// Centralized service for handling food category determination and derived flags
/// This service consolidates all category-based logic that was previously duplicated

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
    category_lower == "rind" ||
    category_lower == "rindfleisch" ||
    category_lower.contains("rind") ||
    category_lower.contains("beef") ||
    category_lower.contains("cattle")
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
    if is_meat_category(category) ||
       is_fish_category(category) ||
       is_egg_category(category) ||
       is_honey_category(category) ||
       is_dairy_category(category) ||
       is_insect_category(category) {
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

/// Get a summary of all category flags for a given category string
/// This is useful for debugging and understanding category classification
pub fn get_category_flags(category: &str) -> CategoryFlags {
    CategoryFlags {
        is_meat: is_meat_category(category),
        is_fish: is_fish_category(category),
        is_beef: is_beef_category(category),
        is_egg: is_egg_category(category),
        is_honey: is_honey_category(category),
        is_dairy: is_dairy_category(category),
        is_insect: is_insect_category(category),
        is_plant: is_plant_category(category),
    }
}

/// Structure representing all category flags for an ingredient
#[derive(Debug, Clone, PartialEq)]
pub struct CategoryFlags {
    pub is_meat: bool,
    pub is_fish: bool,
    pub is_beef: bool,
    pub is_egg: bool,
    pub is_honey: bool,
    pub is_dairy: bool,
    pub is_insect: bool,
    pub is_plant: bool,
}

impl CategoryFlags {
    /// Get a human-readable summary of which categories apply
    pub fn summary(&self) -> Vec<&'static str> {
        let mut summary = Vec::new();
        if self.is_meat { summary.push("Fleisch"); }
        if self.is_fish { summary.push("Fisch"); }
        if self.is_beef { summary.push("Rindfleisch"); }
        if self.is_egg { summary.push("Ei"); }
        if self.is_honey { summary.push("Honig"); }
        if self.is_dairy { summary.push("Milchprodukt"); }
        if self.is_insect { summary.push("Insekt"); }
        if self.is_plant { summary.push("Pflanzlich"); }
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meat_category_detection() {
        assert!(is_meat_category("Fleisch und Innereien"));
        assert!(is_meat_category("Rind"));
        assert!(is_meat_category("Schwein"));
        assert!(is_meat_category("Kalb; Lamm, Schaf; Rind; Schwein; Wild; Geflügel"));
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

    #[test]
    fn test_category_flags() {
        let flags = get_category_flags("Rind");
        assert!(flags.is_meat);
        assert!(flags.is_beef);
        assert!(!flags.is_fish);
        assert!(!flags.is_plant);

        let summary = flags.summary();
        assert!(summary.contains(&"Fleisch"));
        assert!(summary.contains(&"Rindfleisch"));
    }
}