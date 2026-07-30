use super::super::*;

#[test]
fn test_is_fish_category() {
    // Test official BLV API fish categories
    assert!(is_fish_category("Fisch"));
    assert!(is_fish_category("Meeresfische"));
    assert!(is_fish_category("Süsswasserfische"));
    assert!(is_fish_category("Meeresfrüchte, Krusten- und Schalentiere"));

    // Test generic fish terms
    assert!(is_fish_category("Lachs"));
    assert!(is_fish_category("Thunfisch"));
    assert!(is_fish_category("Forelle"));

    // Test English terms
    assert!(is_fish_category("fish"));
    assert!(is_fish_category("salmon"));
    assert!(is_fish_category("seafood"));

    // Test case insensitive matching
    assert!(is_fish_category("FISCH"));
    assert!(is_fish_category("meeresfische"));

    // Test non-fish categories
    assert!(!is_fish_category("Rind"));
    assert!(!is_fish_category("Getreide"));
    assert!(!is_fish_category("Milchprodukte"));
    assert!(!is_fish_category("Gemüse"));
}

#[test]
fn test_is_beef_category() {
    // Test beef categories
    assert!(is_beef_category("Rind"));
    assert!(is_beef_category("Rindfleisch"));
    assert!(is_beef_category("RIND"));
    assert!(is_beef_category("beef"));
    assert!(is_beef_category("Kalb; Rind; Schwein"));

    // Test non-beef categories
    assert!(!is_beef_category("Schwein"));
    assert!(!is_beef_category("Geflügel"));
    assert!(!is_beef_category("Lamm, Schaf"));
    assert!(!is_beef_category("Brühwurstware"));
    assert!(!is_beef_category("Getreide"));
}

#[test]
fn test_is_meat_category_with_api_categories() {
    // Test official BLV API categories for meat
    assert!(is_meat_category("Fleisch und Innereien"));
    assert!(is_meat_category("Rind"));
    assert!(is_meat_category("Schwein"));
    assert!(is_meat_category("Kalb"));
    assert!(is_meat_category("Geflügel"));
    assert!(is_meat_category("Lamm, Schaf"));
    assert!(is_meat_category("Wild"));

    // Test processed meat categories
    assert!(is_meat_category("Brühwurstware"));
    assert!(is_meat_category("Kochwurstware"));

    // Test combined categories (semicolon-separated)
    assert!(is_meat_category("Kalb; Lamm, Schaf; Rind; Schwein; Wild; Geflügel"));
    assert!(is_meat_category("Kalb; Rind; Schwein; Geflügel"));
    assert!(is_meat_category("Kalb; Lamm, Schaf; Schwein"));

    // Test non-meat categories
    assert!(!is_meat_category("Getreide"));
    assert!(!is_meat_category("Milchprodukte"));
    assert!(!is_meat_category("Gemüse"));
    assert!(!is_meat_category("Früchte"));
    assert!(!is_meat_category("Gewürze"));

    // Test case insensitive matching
    assert!(is_meat_category("RIND"));
    assert!(is_meat_category("schwein"));
    assert!(is_meat_category("Fleisch Und Innereien"));

    // Test fallback terms
    assert!(is_meat_category("Hackfleisch"));
    assert!(is_meat_category("Bratwurst"));
    assert!(is_meat_category("meat"));
    assert!(is_meat_category("beef"));
}

#[test]
fn country_flag_emoji_test() {
    use crate::model::Country;

    // Test some key country flags
    assert_eq!(Country::CH.flag_emoji(), "🇨🇭");
    assert_eq!(Country::DE.flag_emoji(), "🇩🇪");
    assert_eq!(Country::FR.flag_emoji(), "🇫🇷");
    assert_eq!(Country::IT.flag_emoji(), "🇮🇹");
    assert_eq!(Country::EU.flag_emoji(), "🇪🇺");
    assert_eq!(Country::NoOriginRequired.flag_emoji(), "");

    println!("✅ Country flag emojis working correctly!");
    println!("🇨🇭 Switzerland, 🇩🇪 Germany, 🇫🇷 France, 🇮🇹 Italy, 🇪🇺 EU");
}
