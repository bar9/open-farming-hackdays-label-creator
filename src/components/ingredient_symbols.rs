use crate::services::UnifiedIngredient;
use dioxus::prelude::*;
use rust_i18n::t;

#[derive(Props, Clone, PartialEq)]
pub struct IngredientSymbolsProps {
    pub ingredient: UnifiedIngredient,
    #[props(default = false)]
    pub show_unknown: bool, // Show faded symbols for unknown flags
}

/// Component to display visual symbols for ingredient flags
#[component]
pub fn IngredientSymbols(props: IngredientSymbolsProps) -> Element {
    let ingredient = &props.ingredient;

    rsx! {
        div {
            class: "flex items-center gap-1 text-lg",

            // Allergen symbol - critical info, always show if known
            if let Some(is_allergen) = ingredient.is_allergen {
                if is_allergen {
                    span {
                        class: "text-red-500",
                        title: t!("symbols.allergen_tooltip").to_string(),
                        "🚨"
                    }
                }
            } else if props.show_unknown {
                span {
                    class: "text-gray-400 opacity-50",
                    title: t!("symbols.allergen_unknown").to_string(),
                    "🚨"
                }
            }

            // Meat symbol
            if let Some(is_meat) = ingredient.is_meat {
                if is_meat {
                    span {
                        class: "text-red-800",
                        title: t!("symbols.meat_tooltip").to_string(),
                        "🥩"
                    }
                }
            } else if props.show_unknown {
                span {
                    class: "text-gray-400 opacity-50",
                    title: t!("symbols.meat_unknown").to_string(),
                    "🥩"
                }
            }

            // Fish symbol
            if let Some(is_fish) = ingredient.is_fish {
                if is_fish {
                    span {
                        class: "text-blue-600",
                        title: t!("symbols.fish_tooltip").to_string(),
                        "🐟"
                    }
                }
            } else if props.show_unknown {
                span {
                    class: "text-gray-400 opacity-50",
                    title: t!("symbols.fish_unknown").to_string(),
                    "🐟"
                }
            }

            // Dairy symbol
            if let Some(is_dairy) = ingredient.is_dairy {
                if is_dairy {
                    span {
                        class: "text-blue-500",
                        title: t!("symbols.dairy_tooltip").to_string(),
                        "🥛"
                    }
                }
            } else if props.show_unknown {
                span {
                    class: "text-gray-400 opacity-50",
                    title: t!("symbols.dairy_unknown").to_string(),
                    "🥛"
                }
            }

            // Egg symbol
            if let Some(is_egg) = ingredient.is_egg {
                if is_egg {
                    span {
                        class: "text-yellow-600",
                        title: t!("symbols.egg_tooltip").to_string(),
                        "🥚"
                    }
                }
            } else if props.show_unknown {
                span {
                    class: "text-gray-400 opacity-50",
                    title: t!("symbols.egg_unknown").to_string(),
                    "🥚"
                }
            }

            // Honey symbol
            if let Some(is_honey) = ingredient.is_honey {
                if is_honey {
                    span {
                        class: "text-amber-600",
                        title: t!("symbols.honey_tooltip").to_string(),
                        "🍯"
                    }
                }
            } else if props.show_unknown {
                span {
                    class: "text-gray-400 opacity-50",
                    title: t!("symbols.honey_unknown").to_string(),
                    "🍯"
                }
            }

            // Plant symbol
            if let Some(is_plant) = ingredient.is_plant {
                if is_plant {
                    span {
                        class: "text-green-600",
                        title: t!("symbols.plant_tooltip").to_string(),
                        "🌱"
                    }
                }
            } else if props.show_unknown {
                span {
                    class: "text-gray-400 opacity-50",
                    title: t!("symbols.plant_unknown").to_string(),
                    "🌱"
                }
            }

            // Agricultural symbol
            if let Some(is_agricultural) = ingredient.is_agricultural {
                if is_agricultural {
                    span {
                        class: "text-yellow-700",
                        title: t!("symbols.agricultural_tooltip").to_string(),
                        "🌾"
                    }
                }
            } else if props.show_unknown {
                span {
                    class: "text-gray-400 opacity-50",
                    title: t!("symbols.agricultural_unknown").to_string(),
                    "🌾"
                }
            }

            // Bio symbol
            if let Some(is_bio) = ingredient.is_bio {
                if is_bio {
                    span {
                        class: "text-green-700",
                        title: t!("symbols.bio_tooltip").to_string(),
                        "🌿"
                    }
                }
            } else if props.show_unknown {
                span {
                    class: "text-gray-400 opacity-50",
                    title: t!("symbols.bio_unknown").to_string(),
                    "🌿"
                }
            }
        }
    }
}

/// Compact version showing only the most important symbols
#[component]
pub fn IngredientSymbolsCompact(ingredient: UnifiedIngredient) -> Element {
    rsx! {
        div {
            class: "flex items-center gap-0.5 text-sm",

            // Only show confirmed positive flags in compact mode
            if ingredient.is_allergen == Some(true) {
                span {
                    class: "text-red-500",
                    title: t!("symbols.allergen_tooltip").to_string(),
                    "🚨"
                }
            }

            if ingredient.is_meat == Some(true) {
                span {
                    class: "text-red-800",
                    title: t!("symbols.meat_tooltip").to_string(),
                    "🥩"
                }
            }

            if ingredient.is_fish == Some(true) {
                span {
                    class: "text-blue-600",
                    title: t!("symbols.fish_tooltip").to_string(),
                    "🐟"
                }
            }

            if ingredient.is_dairy == Some(true) {
                span {
                    class: "text-blue-500",
                    title: t!("symbols.dairy_tooltip").to_string(),
                    "🥛"
                }
            }

            if ingredient.is_plant == Some(true) {
                span {
                    class: "text-green-600",
                    title: t!("symbols.plant_tooltip").to_string(),
                    "🌱"
                }
            }

            if ingredient.is_bio == Some(true) {
                span {
                    class: "text-green-700",
                    title: t!("symbols.bio_tooltip").to_string(),
                    "🌿"
                }
            }
        }
    }
}

