use crate::services::UnifiedIngredient;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct IngredientSymbolsProps {
    pub ingredient: UnifiedIngredient,
    #[props(default = false)]
    pub show_unknown: bool,
}

/// Component to display visual symbols for ingredient flags
/// Note: Symbols have been removed - this component now renders nothing
#[component]
pub fn IngredientSymbols(_props: IngredientSymbolsProps) -> Element {
    rsx! {}
}

/// Compact version showing only the most important symbols
/// Note: Symbols have been removed - this component now renders nothing
#[component]
pub fn IngredientSymbolsCompact(_ingredient: UnifiedIngredient) -> Element {
    rsx! {}
}
