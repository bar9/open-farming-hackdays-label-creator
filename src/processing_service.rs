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
        .filter_map(|r| r.ok())
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
pub fn get_biosuisse_category_from_blv(blv_category: &str) -> Option<(String, String)> {
    let mapping_csv = include_str!("../blv_biosuisse_mapping.csv");
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(mapping_csv.as_bytes());

    for result in rdr.records() {
        if let Ok(record) = result {
            let blv_name = record.get(1).unwrap_or("");
            if blv_name == blv_category {
                let hauptgruppe = record.get(6).unwrap_or("").to_string();
                let untergruppe = record.get(7).unwrap_or("").to_string();
                if !hauptgruppe.is_empty() {
                    return Some((hauptgruppe, untergruppe));
                }
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
