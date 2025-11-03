use crate::core::Ingredient;
use serde::{Deserialize, Serialize};
use web_sys::Storage;
use rust_i18n::t;

const SAVED_INGREDIENTS_KEY: &str = "saved_composite_ingredients";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedIngredient {
    pub ingredient: Ingredient,
    pub timestamp: f64,
}

/// Get localStorage instance
fn get_storage() -> Option<Storage> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
}

/// Save a composite ingredient to localStorage
pub fn save_composite_ingredient(ingredient: &Ingredient) -> Result<(), String> {
    let storage = get_storage().ok_or_else(|| t!("errors.localstorage_unavailable").to_string())?;
    
    // Get existing saved ingredients
    let mut saved = get_saved_ingredients();
    
    // Check if an ingredient with the same name already exists
    if let Some(index) = saved.iter().position(|s| s.ingredient.name == ingredient.name) {
        // Update existing ingredient
        saved[index] = SavedIngredient {
            ingredient: ingredient.clone(),
            timestamp: 0.0,  // Simple timestamp placeholder
        };
    } else {
        // Add new ingredient
        saved.push(SavedIngredient {
            ingredient: ingredient.clone(),
            timestamp: 0.0,  // Simple timestamp placeholder
        });
    }
    
    // Serialize and save using serde_json
    let json = serde_json::to_string(&saved).map_err(|e| e.to_string())?;
    storage
        .set_item(SAVED_INGREDIENTS_KEY, &json)
        .map_err(|_| t!("errors.localstorage_save_failed").to_string())?;
    
    Ok(())
}

/// Get all saved composite ingredients from localStorage
pub fn get_saved_ingredients() -> Vec<SavedIngredient> {
    let storage = match get_storage() {
        Some(s) => s,
        None => return vec![],
    };
    
    let json = match storage.get_item(SAVED_INGREDIENTS_KEY) {
        Ok(Some(data)) => data,
        _ => return vec![],
    };
    
    match serde_json::from_str(&json) {
        Ok(ingredients) => ingredients,
        Err(e) => {
            tracing::warn!("{}", t!("errors.parse_saved_ingredients_failed", error = e));
            vec![]
        }
    }
}

/// Get just the ingredient list (without metadata)
pub fn get_saved_ingredients_list() -> Vec<Ingredient> {
    get_saved_ingredients()
        .into_iter()
        .map(|s| s.ingredient)
        .collect()
}

/// Delete a saved ingredient by name
pub fn delete_saved_ingredient(name: &str) -> Result<(), String> {
    let storage = get_storage().ok_or_else(|| t!("errors.localstorage_unavailable").to_string())?;
    
    // Get existing saved ingredients
    let mut saved = get_saved_ingredients();
    
    // Remove the ingredient with matching name
    saved.retain(|s| s.ingredient.name != name);
    
    // Serialize and save
    let json = serde_json::to_string(&saved).map_err(|e| e.to_string())?;
    storage
        .set_item(SAVED_INGREDIENTS_KEY, &json)
        .map_err(|_| t!("errors.localstorage_save_failed").to_string())?;
    
    Ok(())
}

/// Clear all saved ingredients
pub fn clear_saved_ingredients() -> Result<(), String> {
    let storage = get_storage().ok_or_else(|| t!("errors.localstorage_unavailable").to_string())?;
    storage
        .remove_item(SAVED_INGREDIENTS_KEY)
        .map_err(|_| t!("errors.localstorage_clear_failed").to_string())?;
    Ok(())
}