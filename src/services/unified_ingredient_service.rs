use crate::api::{search_food, FoodItem};
use crate::category_service::{
    is_dairy_category, is_egg_category, is_fish_category, is_honey_category,
    is_meat_category, is_plant_category
};
use crate::model::{
    food_db, ingredient_aliases, lookup_agricultural, lookup_allergen, lookup_priority,
};
use serde::{Deserialize, Serialize};

/// Unified ingredient combining data from local DB and BLV API
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnifiedIngredient {
    pub name: String,
    pub canonical: Option<String>,           // Set when `name` is an alias term; the real food_db entry
    pub priority: i32,                       // Curated ranking priority (0 = uncurated)
    pub category: Option<String>,            // From BLV API
    pub origin: Option<crate::model::Country>, // Country of origin for flag display

    // Binary flags with visual indicators
    pub is_allergen: Option<bool>,          // From local DB
    pub is_agricultural: Option<bool>,       // From local DB
    pub is_meat: Option<bool>,              // Derived from category
    pub is_fish: Option<bool>,              // Derived from category
    pub is_dairy: Option<bool>,             // Derived from category
    pub is_egg: Option<bool>,               // Derived from category
    pub is_honey: Option<bool>,             // Derived from category
    pub is_plant: Option<bool>,             // Derived from category
    pub is_bio: Option<bool>,               // From user input/saved

    pub source: IngredientSource,           // Track data origin
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum IngredientSource {
    Local,           // From food_db.csv only
    BLV,            // From BLV API only
    Merged,         // Combined from both sources
}

/// Category flags derived from BLV category
#[derive(Debug, Default)]
struct CategoryFlags {
    pub is_meat: Option<bool>,
    pub is_fish: Option<bool>,
    pub is_dairy: Option<bool>,
    pub is_egg: Option<bool>,
    pub is_honey: Option<bool>,
    pub is_plant: Option<bool>,
}

impl UnifiedIngredient {
    /// Create from local food_db entry
    pub fn from_local(name: String) -> Self {
        Self {
            priority: lookup_priority(&name),
            canonical: None,
            name: name.clone(),
            category: None,
            origin: None, // Local entries don't have origin info
            is_allergen: Some(lookup_allergen(&name)),
            is_agricultural: Some(lookup_agricultural(&name)),
            is_meat: None,
            is_fish: None,
            is_dairy: None,
            is_egg: None,
            is_honey: None,
            is_plant: None,
            is_bio: None,
            source: IngredientSource::Local,
        }
    }

    /// Create from BLV API result
    pub fn from_blv(item: FoodItem) -> Self {
        let flags = if let Some(ref category) = item.category_names {
            calculate_category_flags(category)
        } else {
            CategoryFlags::default()
        };

        Self {
            priority: lookup_priority(&item.food_name),
            canonical: None,
            name: item.food_name,
            category: item.category_names,
            origin: None, // BLV API doesn't provide origin data
            is_allergen: None,
            is_agricultural: None,
            is_meat: flags.is_meat,
            is_fish: flags.is_fish,
            is_dairy: flags.is_dairy,
            is_egg: flags.is_egg,
            is_honey: flags.is_honey,
            is_plant: flags.is_plant,
            is_bio: None,
            source: IngredientSource::BLV,
        }
    }

    /// Create merged ingredient from local data and BLV result
    pub fn merge(local_name: String, blv_item: FoodItem) -> Self {
        let flags = if let Some(ref category) = blv_item.category_names {
            calculate_category_flags(category)
        } else {
            CategoryFlags::default()
        };

        Self {
            priority: lookup_priority(&local_name),
            canonical: None,
            name: local_name.clone(),
            category: blv_item.category_names,
            origin: None, // Local/merged data typically doesn't have origin info
            // From local DB
            is_allergen: Some(lookup_allergen(&local_name)),
            is_agricultural: Some(lookup_agricultural(&local_name)),
            // From category analysis
            is_meat: flags.is_meat,
            is_fish: flags.is_fish,
            is_dairy: flags.is_dairy,
            is_egg: flags.is_egg,
            is_honey: flags.is_honey,
            is_plant: flags.is_plant,
            is_bio: None,
            source: IngredientSource::Merged,
        }
    }

    /// Create from a curated alias row. `name` is the alias term shown on the
    /// label; allergen / agricultural / category flags are derived from the
    /// canonical food_db entry. If a BLV match for the canonical is supplied,
    /// its category drives the meat/fish/dairy/etc. flags.
    pub fn from_alias(
        alias: String,
        canonical: String,
        priority: i32,
        blv_match: Option<FoodItem>,
    ) -> Self {
        let (category, flags, source) = match blv_match {
            Some(item) => {
                let flags = item
                    .category_names
                    .as_ref()
                    .map(|c| calculate_category_flags(c))
                    .unwrap_or_default();
                (item.category_names, flags, IngredientSource::Merged)
            }
            None => (None, CategoryFlags::default(), IngredientSource::Local),
        };

        Self {
            priority,
            name: alias,
            canonical: Some(canonical.clone()),
            category,
            origin: None,
            is_allergen: Some(lookup_allergen(&canonical)),
            is_agricultural: Some(lookup_agricultural(&canonical)),
            is_meat: flags.is_meat,
            is_fish: flags.is_fish,
            is_dairy: flags.is_dairy,
            is_egg: flags.is_egg,
            is_honey: flags.is_honey,
            is_plant: flags.is_plant,
            is_bio: None,
            source,
        }
    }
}

/// Search for ingredients combining local DB and BLV API results
pub async fn search_unified(query: &str, lang: &str) -> Result<Vec<UnifiedIngredient>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    // Search local database
    let local_results = search_local_db(query);

    // Search BLV API
    let blv_results = search_food(query, lang).await?;

    // Merge results
    let mut unified = Vec::new();
    let mut used_blv_indices = Vec::new();

    // Curated alias suggestions are additive: they introduce common terms
    // (e.g. "Mehl" -> "Weizenmehl") without consuming a BLV match, so the
    // canonical entry can still appear on its own. Boost rows (alias ==
    // canonical) contribute nothing here — their priority reaches the canonical
    // entry below via `lookup_priority`.
    for (alias, canonical, priority) in search_aliases(query) {
        if alias.eq_ignore_ascii_case(&canonical) {
            continue;
        }
        let blv_match = find_best_blv_match(&canonical, &blv_results).map(|(_, item)| item);
        unified.push(UnifiedIngredient::from_alias(alias, canonical, priority, blv_match));
    }

    // Try to match local ingredients with BLV results
    for local_name in local_results {
        if let Some((index, blv_match)) = find_best_blv_match(&local_name, &blv_results) {
            unified.push(UnifiedIngredient::merge(local_name, blv_match));
            used_blv_indices.push(index);
        } else {
            // Local ingredient with no BLV match
            unified.push(UnifiedIngredient::from_local(local_name));
        }
    }

    // Add unmatched BLV results
    for (index, blv_item) in blv_results.into_iter().enumerate() {
        if !used_blv_indices.contains(&index) {
            unified.push(UnifiedIngredient::from_blv(blv_item));
        }
    }

    // Defend against duplicate suggestions for the same (display name, canonical)
    // pair, keeping the highest-priority / best-source instance.
    rank_unified(&mut unified, query);
    unified.dedup_by(|a, b| a.name == b.name && a.canonical == b.canonical);

    Ok(unified)
}

/// Sort suggestions by curated priority, then query-match quality, then source,
/// then name. Sorting before dedup ensures the kept duplicate is the best one.
fn rank_unified(unified: &mut [UnifiedIngredient], query: &str) {
    unified.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| {
                let ra = relevance(&a.name, query);
                let rb = relevance(&b.name, query);
                rb.partial_cmp(&ra).unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| source_rank(&a.source).cmp(&source_rank(&b.source)))
            .then_with(|| a.name.cmp(&b.name))
    });
}

fn source_rank(source: &IngredientSource) -> u8 {
    match source {
        IngredientSource::Merged => 0,
        IngredientSource::Local => 1,
        IngredientSource::BLV => 2,
    }
}

/// Query-match quality, mirroring the BLV ranking in `api::calculate_similarity`:
/// exact 1.0, prefix 0.8, substring 0.6, otherwise character-overlap up to 0.4.
fn relevance(name: &str, query: &str) -> f32 {
    let name_lower = name.to_lowercase();
    let query_lower = query.to_lowercase();

    if name_lower == query_lower {
        return 1.0;
    }
    if name_lower.starts_with(&query_lower) || query_lower.starts_with(&name_lower) {
        return 0.8;
    }
    if name_lower.contains(&query_lower) || query_lower.contains(&name_lower) {
        return 0.6;
    }

    let name_chars: Vec<char> = name_lower.chars().collect();
    let query_chars: Vec<char> = query_lower.chars().collect();
    let common = name_chars.iter().filter(|c| query_chars.contains(c)).count();
    let max_len = name_chars.len().max(query_chars.len());
    if max_len > 0 {
        common as f32 / max_len as f32 * 0.4
    } else {
        0.0
    }
}

/// Curated alias rows whose alias term matches the query (case-insensitive substring).
fn search_aliases(query: &str) -> Vec<(String, String, i32)> {
    let query_lower = query.to_lowercase();
    ingredient_aliases()
        .into_iter()
        .filter(|(alias, _, _)| alias.to_lowercase().contains(&query_lower))
        .collect()
}

/// Search local food database for matching ingredients
fn search_local_db(query: &str) -> Vec<String> {
    let query_lower = query.to_lowercase();
    food_db()
        .into_iter()
        .filter_map(|(name, _allergen)| {
            if name.to_lowercase().contains(&query_lower) {
                Some(name)
            } else {
                None
            }
        })
        .collect()
}

/// Find the best matching BLV result for a local ingredient name
fn find_best_blv_match(local_name: &str, blv_results: &[FoodItem]) -> Option<(usize, FoodItem)> {
    let local_lower = local_name.to_lowercase();

    // Look for exact or very close matches
    for (index, item) in blv_results.iter().enumerate() {
        let blv_lower = item.food_name.to_lowercase();

        // Exact match
        if blv_lower == local_lower {
            return Some((index, item.clone()));
        }

        // Check if local name is contained in BLV name or vice versa
        if blv_lower.contains(&local_lower) || local_lower.contains(&blv_lower) {
            return Some((index, item.clone()));
        }
    }

    None
}

/// Calculate category flags from BLV category string
fn calculate_category_flags(category: &str) -> CategoryFlags {
    CategoryFlags {
        is_meat: Some(is_meat_category(category)),
        is_fish: Some(is_fish_category(category)),
        is_dairy: Some(is_dairy_category(category)),
        is_egg: Some(is_egg_category(category)),
        is_honey: Some(is_honey_category(category)),
        is_plant: Some(is_plant_category(category)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, priority: i32, source: IngredientSource) -> UnifiedIngredient {
        UnifiedIngredient {
            name: name.to_string(),
            canonical: None,
            priority,
            category: None,
            origin: None,
            is_allergen: None,
            is_agricultural: None,
            is_meat: None,
            is_fish: None,
            is_dairy: None,
            is_egg: None,
            is_honey: None,
            is_plant: None,
            is_bio: None,
            source,
        }
    }

    #[test]
    fn relevance_matches_similarity_bands() {
        assert_eq!(relevance("Mehl", "mehl"), 1.0); // exact (case-insensitive)
        assert_eq!(relevance("Mehlsuppe", "mehl"), 0.8); // prefix
        assert_eq!(relevance("Weizenmehl", "mehl"), 0.6); // substring
        assert!(relevance("Apfel", "mehl") < 0.4); // character overlap only
    }

    // Query "mehl": curated alias first, boosted canonical second, uncurated
    // matches after (alphabetical among equal priority/relevance).
    #[test]
    fn ranking_puts_alias_then_boost_then_rest() {
        let mut unified = vec![
            entry("Buchweizenmehl", 0, IngredientSource::Local),
            entry("Roggenmehl", 0, IngredientSource::Local),
            entry("Weizenmehl", 50, IngredientSource::Local),
            {
                let mut a = entry("Mehl", 100, IngredientSource::Local);
                a.canonical = Some("Weizenmehl".to_string());
                a
            },
            entry("Dinkelmehl", 0, IngredientSource::Local),
        ];

        rank_unified(&mut unified, "mehl");
        let order: Vec<&str> = unified.iter().map(|u| u.name.as_str()).collect();
        assert_eq!(
            order,
            vec!["Mehl", "Weizenmehl", "Buchweizenmehl", "Dinkelmehl", "Roggenmehl"]
        );
    }
}
