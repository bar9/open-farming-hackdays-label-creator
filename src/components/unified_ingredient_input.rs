use crate::components::*;
use crate::persistence::get_saved_ingredients_list;
use crate::services::{search_unified, UnifiedIngredient};
use dioxus::prelude::*;
use rust_i18n::t;

#[derive(Props, Clone, PartialEq)]
pub struct UnifiedIngredientInputProps {
    pub bound_value: Signal<String>,
    pub on_ingredient_select: EventHandler<UnifiedIngredient>,
    #[props(default = false)]
    pub required: bool,
    #[props(default = String::new())]
    pub placeholder: String,
}

#[component]
pub fn UnifiedIngredientInput(mut props: UnifiedIngredientInputProps) -> Element {
    let mut is_dropdown_open = use_signal(|| false);
    let mut search_results = use_signal(|| Vec::<UnifiedIngredient>::new());
    let is_searching = use_signal(|| false);
    let mut search_request_id = use_signal(|| 0u32); // Track search requests to prevent race conditions
    let mut search_error = use_signal(|| None::<String>); // Track search errors

    // Handle input changes with search
    let mut handle_input = move |value: String| {
        props.bound_value.set(value.clone());

        if value.trim().is_empty() {
            is_dropdown_open.set(false);
            search_results.set(Vec::new());
            search_error.set(None); // Clear errors
            return;
        }

        if value.len() < 2 {
            return; // Wait for at least 2 characters
        }

        // Clear previous errors and increment request ID to track this search
        search_error.set(None);
        search_request_id.set(search_request_id() + 1);
        let current_request_id = search_request_id();

        // Trigger search
        let mut search_results_clone = search_results.clone();
        let mut is_searching_clone = is_searching.clone();
        let mut is_dropdown_open_clone = is_dropdown_open.clone();
        let search_request_id_clone = search_request_id.clone();
        let mut search_error_clone = search_error.clone();

        spawn(async move {
            is_searching_clone.set(true);

            // Get current locale for API call
            let locale = rust_i18n::locale().to_string();
            let lang = if locale.starts_with("fr") {
                "fr"
            } else if locale.starts_with("it") {
                "it"
            } else {
                "de"
            };

            match search_unified(&value, lang).await {
                Ok(results) => {
                    // Check if this response is still relevant (not superseded by newer request)
                    if current_request_id == search_request_id_clone() {
                        search_results_clone.set(results);
                        if !search_results_clone.read().is_empty() {
                            is_dropdown_open_clone.set(true);
                        }
                    }
                    // Else: ignore outdated response
                }
                Err(e) => {
                    tracing::warn!("Failed to search unified ingredients: {}", e);
                    // Only update if this is still the current request
                    if current_request_id == search_request_id_clone() {
                        search_results_clone.set(Vec::new());
                        search_error_clone.set(Some(format!("Search failed: {}", e)));
                    }
                }
            }
            // Only stop loading indicator if this is the current request
            if current_request_id == search_request_id_clone() {
                is_searching_clone.set(false);
            }
        });
    };

    let mut handle_ingredient_select = move |ingredient: UnifiedIngredient| {
        props.bound_value.set(ingredient.name.clone());
        props.on_ingredient_select.call(ingredient);
        is_dropdown_open.set(false);
    };

    let handle_blur = move |_| {
        // Delay closing to allow clicks on dropdown items
        spawn(async move {
            gloo::timers::future::TimeoutFuture::new(150).await;
            is_dropdown_open.set(false);
        });
    };

    rsx! {
        div { class: "relative w-full",
            input {
                r#type: "text",
                class: "input input-accent w-full",
                placeholder: if props.placeholder.is_empty() {
                    t!("placeholder.ingredient_name").to_string()
                } else {
                    props.placeholder.clone()
                },
                value: (props.bound_value)(),
                required: props.required,
                list: "", // Disable browser autocomplete
                autocomplete: "off",
                oninput: move |evt| handle_input(evt.value()),
                onblur: handle_blur,
                onfocus: move |_| {
                    if !search_results.read().is_empty() {
                        is_dropdown_open.set(true);
                    }
                }
            }

            // Search indicator
            if is_searching() {
                div {
                    class: "absolute right-3 top-1/2 transform -translate-y-1/2",
                    span { class: "loading loading-spinner loading-sm" }
                }
            }

            // Error indicator
            if let Some(error_msg) = search_error() {
                div {
                    class: "absolute right-3 top-1/2 transform -translate-y-1/2",
                    span {
                        class: "text-error text-sm",
                        title: "{error_msg}",
                        "⚠️"
                    }
                }
            }

            // Dropdown with results
            if is_dropdown_open() && !search_results.read().is_empty() {
                div {
                    class: "absolute z-50 w-full mt-1 bg-base-100 border border-base-300 rounded-lg shadow-lg max-h-96 overflow-y-auto",

                    // Show saved ingredients first if any match
                    {
                        let saved_ingredients = get_saved_ingredients_list();
                        let query_lower = (props.bound_value)().to_lowercase();
                        let matching_saved: Vec<_> = saved_ingredients.into_iter()
                            .filter(|saved| saved.name.to_lowercase().contains(&query_lower))
                            .collect();

                        if !matching_saved.is_empty() {
                            rsx! {
                                div { class: "px-3 py-2 text-sm font-semibold text-base-content/70 bg-base-200",
                                    "💾 " {t!("label.saved_ingredients")}
                                }
                                for (saved_idx, saved) in matching_saved.iter().enumerate() {
                                    div {
                                        key: "saved-{saved_idx}-{saved.name.clone()}",
                                        class: "px-3 py-2 hover:bg-base-200 cursor-pointer flex items-center justify-between",
                                        onclick: {
                                            let saved_clone = saved.clone();
                                            move |_| {
                                                // Convert saved ingredient to UnifiedIngredient
                                                let unified = UnifiedIngredient {
                                                    name: saved_clone.name.clone(),
                                                    category: saved_clone.category.clone(),
                                                is_allergen: Some(saved_clone.is_allergen),
                                                is_agricultural: Some(saved_clone.is_agricultural),
                                                is_meat: None,
                                                is_fish: None,
                                                is_dairy: None,
                                                is_egg: None,
                                                is_honey: None,
                                                is_plant: None,
                                                is_bio: saved_clone.is_bio,
                                                source: crate::services::IngredientSource::Local,
                                            };
                                            handle_ingredient_select(unified);
                                        }},
                                        div { class: "flex items-center gap-2",
                                            span { class: "font-medium", {saved.name.clone()} }
                                            if saved.is_allergen {
                                                span { class: "text-red-500", "🚨" }
                                            }
                                        }
                                        span { class: "text-xs text-base-content/50",
                                            {t!("label.saved_indicator")}
                                        }
                                    }
                                }
                                div { class: "border-t border-base-300" }
                            }
                        } else {
                            rsx! { }
                        }
                    }

                    // Unified search results
                    for (unified_idx, ingredient) in search_results.read().iter().enumerate() {
                        div {
                            key: "unified-{unified_idx}-{ingredient.name}",
                            class: "px-3 py-2 hover:bg-base-200 cursor-pointer",
                            onclick: {
                                let ingredient_clone = ingredient.clone();
                                move |_| handle_ingredient_select(ingredient_clone.clone())
                            },

                            div { class: "flex items-center justify-between",
                                div { class: "flex items-center gap-2 flex-1",
                                    span { class: "font-medium", {ingredient.name.clone()} }
                                    IngredientSymbolsCompact { ingredient: ingredient.clone() }
                                }
                                IngredientSourceBadge { ingredient: ingredient.clone() }
                            }

                            if let Some(category) = &ingredient.category {
                                div { class: "text-sm text-base-content/70 mt-1",
                                    "→ " {category.clone()}
                                }
                            }
                        }
                    }

                    // Legend at the bottom
                    div { class: "px-3 py-2 text-xs text-base-content/50 bg-base-100 border-t border-base-300",
                        "🚨" {t!("symbols.allergen_tooltip")} " • "
                        "🥩" {t!("symbols.meat_tooltip")} " • "
                        "🐟" {t!("symbols.fish_tooltip")} " • "
                        "🥛" {t!("symbols.dairy_tooltip")} " • "
                        "🌱" {t!("symbols.plant_tooltip")}
                    }
                }
            }
        }
    }
}