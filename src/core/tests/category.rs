use super::super::*;

#[test]
fn test_is_fish_category() {
    // Test official BLV API fish categories
    assert_eq!(is_fish_category("Fisch"), true);
    assert_eq!(is_fish_category("Meeresfische"), true);
    assert_eq!(is_fish_category("Süsswasserfische"), true);
    assert_eq!(is_fish_category("Meeresfrüchte, Krusten- und Schalentiere"), true);

    // Test generic fish terms
    assert_eq!(is_fish_category("Lachs"), true);
    assert_eq!(is_fish_category("Thunfisch"), true);
    assert_eq!(is_fish_category("Forelle"), true);

    // Test English terms
    assert_eq!(is_fish_category("fish"), true);
    assert_eq!(is_fish_category("salmon"), true);
    assert_eq!(is_fish_category("seafood"), true);

    // Test case insensitive matching
    assert_eq!(is_fish_category("FISCH"), true);
    assert_eq!(is_fish_category("meeresfische"), true);

    // Test non-fish categories
    assert_eq!(is_fish_category("Rind"), false);
    assert_eq!(is_fish_category("Getreide"), false);
    assert_eq!(is_fish_category("Milchprodukte"), false);
    assert_eq!(is_fish_category("Gemüse"), false);
}

#[test]
fn test_is_beef_category() {
    // Test beef categories
    assert_eq!(is_beef_category("Rind"), true);
    assert_eq!(is_beef_category("Rindfleisch"), true);
    assert_eq!(is_beef_category("RIND"), true);
    assert_eq!(is_beef_category("beef"), true);
    assert_eq!(is_beef_category("Kalb; Rind; Schwein"), true);

    // Test non-beef categories
    assert_eq!(is_beef_category("Schwein"), false);
    assert_eq!(is_beef_category("Geflügel"), false);
    assert_eq!(is_beef_category("Lamm, Schaf"), false);
    assert_eq!(is_beef_category("Brühwurstware"), false);
    assert_eq!(is_beef_category("Getreide"), false);
}

#[test]
fn test_is_meat_category_with_api_categories() {
    // Test official BLV API categories for meat
    assert_eq!(is_meat_category("Fleisch und Innereien"), true);
    assert_eq!(is_meat_category("Rind"), true);
    assert_eq!(is_meat_category("Schwein"), true);
    assert_eq!(is_meat_category("Kalb"), true);
    assert_eq!(is_meat_category("Geflügel"), true);
    assert_eq!(is_meat_category("Lamm, Schaf"), true);
    assert_eq!(is_meat_category("Wild"), true);

    // Test processed meat categories
    assert_eq!(is_meat_category("Brühwurstware"), true);
    assert_eq!(is_meat_category("Kochwurstware"), true);

    // Test combined categories (semicolon-separated)
    assert_eq!(is_meat_category("Kalb; Lamm, Schaf; Rind; Schwein; Wild; Geflügel"), true);
    assert_eq!(is_meat_category("Kalb; Rind; Schwein; Geflügel"), true);
    assert_eq!(is_meat_category("Kalb; Lamm, Schaf; Schwein"), true);

    // Test non-meat categories
    assert_eq!(is_meat_category("Getreide"), false);
    assert_eq!(is_meat_category("Milchprodukte"), false);
    assert_eq!(is_meat_category("Gemüse"), false);
    assert_eq!(is_meat_category("Früchte"), false);
    assert_eq!(is_meat_category("Gewürze"), false);

    // Test case insensitive matching
    assert_eq!(is_meat_category("RIND"), true);
    assert_eq!(is_meat_category("schwein"), true);
    assert_eq!(is_meat_category("Fleisch Und Innereien"), true);

    // Test fallback terms
    assert_eq!(is_meat_category("Hackfleisch"), true);
    assert_eq!(is_meat_category("Bratwurst"), true);
    assert_eq!(is_meat_category("meat"), true);
    assert_eq!(is_meat_category("beef"), true);
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
