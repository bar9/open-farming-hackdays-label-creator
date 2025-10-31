use gloo::net::http::Request;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FoodItem {
    pub id: i64,
    #[serde(rename = "foodName")]
    pub food_name: String,
    pub generic: Option<bool>,
    #[serde(rename = "categoryNames")]
    pub category_names: Option<String>,
    pub amount: Option<f64>,
    pub foodid: Option<i64>,
    #[serde(rename = "valueTypeCode")]
    pub value_type_code: Option<String>,
}

fn calculate_similarity(s1: &str, s2: &str) -> f32 {
    let s1_lower = s1.to_lowercase();
    let s2_lower = s2.to_lowercase();
    
    // Exact match gets highest score
    if s1_lower == s2_lower {
        return 1.0;
    }
    
    // Check if one string starts with the other
    if s1_lower.starts_with(&s2_lower) || s2_lower.starts_with(&s1_lower) {
        return 0.8;
    }
    
    // Check if one string contains the other
    if s1_lower.contains(&s2_lower) || s2_lower.contains(&s1_lower) {
        return 0.6;
    }
    
    // Calculate character overlap ratio
    let s1_chars: Vec<char> = s1_lower.chars().collect();
    let s2_chars: Vec<char> = s2_lower.chars().collect();
    let common_chars = s1_chars.iter().filter(|c| s2_chars.contains(c)).count();
    let max_len = s1_chars.len().max(s2_chars.len());
    
    if max_len > 0 {
        common_chars as f32 / max_len as f32 * 0.4
    } else {
        0.0
    }
}

pub async fn search_food(name: &str, lang: &str) -> Result<Vec<FoodItem>, String> {
    let url = format!(
        "https://api.webapp.prod.blv.foodcase-services.com/BLV_WebApp_WS/webresources/BLV-api/foods?search={}&lang={}&limit=100",
        urlencoding::encode(name),
        lang
    );

    tracing::info!("Fetching food suggestions for '{}' from API", name);

    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.ok() {
        tracing::warn!("API returned non-OK status for '{}'", name);
        return Ok(Vec::new());
    }

    // Parse as array directly since the API returns an array
    let foods: Vec<FoodItem> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    tracing::info!("Found {} food items for '{}'", foods.len(), name);

    // Sort by similarity score, but return all items
    let mut foods_with_scores: Vec<(FoodItem, f32)> = foods
        .into_iter()
        .map(|food| {
            let similarity = calculate_similarity(&food.food_name, name);
            tracing::debug!("Food '{}' has similarity score {:.2} with '{}'", food.food_name, similarity, name);
            (food, similarity)
        })
        .collect();

    // Sort by similarity score (highest first)
    foods_with_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Return sorted food items
    let sorted_foods: Vec<FoodItem> = foods_with_scores.into_iter().map(|(food, _)| food).collect();

    tracing::info!("Returning {} food suggestions for '{}'", sorted_foods.len(), name);
    Ok(sorted_foods)
}