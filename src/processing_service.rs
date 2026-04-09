use serde::Deserialize;

/// Struktur für eine Verarbeitungsregel aus der CSV
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ProcessingRule {
    pub biosuisse_hauptgruppe: String,
    pub biosuisse_untergruppe: String,
    pub step_de: String,
    pub step_fr: String,
    pub applies_to: Option<String>,
    pub is_default: bool,
}

/// Lädt alle Verarbeitungsregeln aus der eingebetteten CSV
pub fn load_processing_rules() -> Vec<ProcessingRule> {
    let csv_data = include_str!("processing_rules.csv");
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv_data.as_bytes());

    rdr.deserialize()
        .filter_map(|r| match r {
            Ok(rule) => Some(rule),
            Err(e) => {
                tracing::warn!("Failed to parse processing rule CSV record: {}", e);
                None
            }
        })
        .collect()
}

/// Gibt verfügbare Verarbeitungsschritte für eine BioSuisse-Kategorie zurück
pub fn get_steps_for_category(hauptgruppe: &str, untergruppe: &str) -> Vec<ProcessingRule> {
    load_processing_rules()
        .into_iter()
        .filter(|r| {
            r.biosuisse_hauptgruppe == hauptgruppe
                && (r.biosuisse_untergruppe == untergruppe || r.biosuisse_untergruppe.is_empty())
        })
        .collect()
}

/// Ermittelt BioSuisse-Kategorie aus BLV-Kategorie via Mapping
/// Matches against all three language columns (de, en, fr) and handles
/// semicolon-separated category strings from the BLV API.
pub fn get_biosuisse_category_from_blv(blv_category: &str) -> Option<(String, String)> {
    let mapping_csv = include_str!("../blv_biosuisse_mapping.csv");
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(mapping_csv.as_bytes());

    let search_categories: Vec<&str> = blv_category.split(';').map(|s| s.trim()).collect();

    for result in rdr.records() {
        match result {
            Ok(record) => {
                let blv_name_de = record.get(1).unwrap_or("");
                let blv_name_en = record.get(2).unwrap_or("");
                let blv_name_fr = record.get(3).unwrap_or("");
                for search_cat in &search_categories {
                    if *search_cat == blv_name_de || *search_cat == blv_name_en || *search_cat == blv_name_fr {
                        let hauptgruppe = record.get(6).unwrap_or("").to_string();
                        let untergruppe = record.get(7).unwrap_or("").to_string();
                        if !hauptgruppe.is_empty() {
                            return Some((hauptgruppe, untergruppe));
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to parse BLV-BioSuisse mapping CSV record: {}", e);
            }
        }
    }
    None
}

/// Gibt Default-Verarbeitungsschritte für eine Kategorie zurück
pub fn get_default_steps(hauptgruppe: &str, untergruppe: &str) -> Vec<String> {
    get_steps_for_category(hauptgruppe, untergruppe)
        .into_iter()
        .filter(|r| r.is_default)
        .map(|r| r.step_de)
        .collect()
}

/// Prüft ob für eine BLV-Kategorie Verarbeitungsschritte verfügbar sind
pub fn has_processing_steps_for_blv_category(blv_category: &str) -> bool {
    if let Some((hauptgruppe, untergruppe)) = get_biosuisse_category_from_blv(blv_category) {
        !get_steps_for_category(&hauptgruppe, &untergruppe).is_empty()
    } else {
        false
    }
}

/// Gibt alle verfügbaren Verarbeitungsschritte für eine BLV-Kategorie zurück
pub fn get_steps_for_blv_category(blv_category: &str) -> Vec<ProcessingRule> {
    if let Some((hauptgruppe, untergruppe)) = get_biosuisse_category_from_blv(blv_category) {
        get_steps_for_category(&hauptgruppe, &untergruppe)
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn biosuisse_mapping_by_german_name() {
        let result = get_biosuisse_category_from_blv("Milch");
        assert!(result.is_some());
        let (hauptgruppe, _) = result.unwrap();
        assert_eq!(hauptgruppe, "Milch und Milchprodukte");
    }

    #[test]
    fn biosuisse_mapping_by_english_name() {
        let result = get_biosuisse_category_from_blv("Milk");
        assert!(result.is_some());
        let (hauptgruppe, _) = result.unwrap();
        assert_eq!(hauptgruppe, "Milch und Milchprodukte");
    }

    #[test]
    fn biosuisse_mapping_by_french_name() {
        let result = get_biosuisse_category_from_blv("Lait");
        assert!(result.is_some());
        let (hauptgruppe, _) = result.unwrap();
        assert_eq!(hauptgruppe, "Milch und Milchprodukte");
    }

    #[test]
    fn biosuisse_mapping_semicolon_separated() {
        // BLV API sometimes returns semicolon-separated categories
        let result = get_biosuisse_category_from_blv("Unknown; Milch; Other");
        assert!(result.is_some());
        let (hauptgruppe, _) = result.unwrap();
        assert_eq!(hauptgruppe, "Milch und Milchprodukte");
    }

    #[test]
    fn biosuisse_mapping_sea_fish_no_longer_maps_to_oils() {
        // Row 6671 "Meeresfische" was incorrectly mapped to "Pflanzliche Öle / Mayonnaise"
        let result = get_biosuisse_category_from_blv("Meeresfische");
        assert!(result.is_none(), "Sea fish should not map to any BioSuisse category (data error fixed)");
    }
}
