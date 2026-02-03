use crate::services::UnifiedIngredient;
use dioxus::prelude::*;

/// Component to display visual symbols for ingredient flags
/// Note: Symbols have been removed - this component now renders nothing
#[component]
pub fn IngredientSymbols(ingredient: UnifiedIngredient, #[props(default = false)] show_unknown: bool) -> Element {
    let _ = (ingredient, show_unknown);
    rsx! {}
}

/// Compact version showing only the most important symbols
/// Note: Symbols have been removed - this component now renders nothing
#[component]
pub fn IngredientSymbolsCompact(ingredient: UnifiedIngredient) -> Element {
    let _ = ingredient;
    rsx! {}
}
