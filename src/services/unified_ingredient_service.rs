use crate::api::{search_food, FoodItem};
use crate::category_service::{
    is_dairy_category, is_egg_category, is_fish_category, is_honey_category,
    is_meat_category, is_plant_category
};
use crate::model::{food_db, lookup_allergen, lookup_agricultural};
use serde::{Deserialize, Serialize};

/// Unified ingredient combining data from local DB and BLV API
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnifiedIngredient {
    pub name: String,
    pub category: Option<String>,            // From BLV API
    pub origin: Option<crate::model::Country>, // Country of origin for flag display

    // Binary flags with visual indicators
    pub is_allergen: Option<bool>,          // 🚨 From local DB
    pub is_agricultural: Option<bool>,       // 🌾 From local DB
    pub is_meat: Option<bool>,              // 🥩 Derived from category
    pub is_fish: Option<bool>,              // 🐟 Derived from category
    pub is_dairy: Option<bool>,             // 🥛 Derived from category
    pub is_egg: Option<bool>,               // 🥚 Derived from category
    pub is_honey: Option<bool>,             // 🍯 Derived from category
    pub is_plant: Option<bool>,             // 🌱 Derived from category
    pub is_bio: Option<bool>,               // 🌿 From user input/saved

    pub source: IngredientSource,           // Track data origin
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

    // Sort by source priority (Merged > Local > BLV) and then by name
    unified.sort_by(|a, b| {
        let source_priority = |source: &IngredientSource| match source {
            IngredientSource::Merged => 0,
            IngredientSource::Local => 1,
            IngredientSource::BLV => 2,
        };

        source_priority(&a.source)
            .cmp(&source_priority(&b.source))
            .then(a.name.cmp(&b.name))
    });

    Ok(unified)
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

/// Check if an ingredient name has already been used in the unified results
fn _already_merged(blv_item: &FoodItem, unified: &[UnifiedIngredient]) -> bool {
    unified.iter().any(|unified_item| {
        unified_item.name.to_lowercase() == blv_item.food_name.to_lowercase()
    })
}