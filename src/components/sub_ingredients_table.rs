use crate::components::*;
use crate::core::{Ingredient, SubIngredient};
use crate::model::{food_db, lookup_allergen, lookup_agricultural};
use crate::persistence::get_saved_ingredients_list;
use crate::services::{UnifiedIngredient, IngredientSource};
// use crate::category_service::*; // Not needed since we don't derive category flags for sub-ingredients
use dioxus::prelude::*;
use rust_i18n::t;

/// Convert a SubIngredient to UnifiedIngredient for display purposes
fn subingredient_to_unified(sub_ingredient: &SubIngredient) -> UnifiedIngredient {
    // Try to get category information by searching for the ingredient name
    // Note: Since SubIngredient doesn't store category, we derive it for display

    UnifiedIngredient {
        name: sub_ingredient.name.clone(),
        category: None, // Sub-ingredients don't store category, but could be looked up if needed
        is_allergen: Some(sub_ingredient.is_allergen),
        is_agricultural: Some(lookup_agricultural(&sub_ingredient.name)), // Derive from local DB
        is_meat: None, // Could be derived from category if we had it
        is_fish: None,
        is_dairy: None,
        is_egg: None,
        is_honey: None,
        is_plant: None,
        is_bio: None,
        source: IngredientSource::Local, // Sub-ingredients are typically local
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SubIngredientsTableProps {
    ingredients: Signal<Vec<Ingredient>>,
    index: usize,
}
pub fn SubIngredientsTable(props: SubIngredientsTableProps) -> Element {
    let mut name_to_add = use_signal(String::new);

    let mut delete_callback = {
        let mut ingredients = props.ingredients;
        move |index: usize, sub_index: usize| {
            if let Some(mut ingredient) = ingredients.get_mut(index) {
                if let Some(sub_components) = &mut ingredient.sub_components {
                    sub_components.remove(sub_index);
                }
            }
        }
    };
    
    let mut toggle_allergen_callback = {
        let mut ingredients = props.ingredients;
        move |index: usize, sub_index: usize, is_allergen: bool| {
            if let Some(mut ingredient) = ingredients.get_mut(index) {
                if let Some(sub_components) = &mut ingredient.sub_components {
                    if let Some(sub_ingredient) = sub_components.get_mut(sub_index) {
                        sub_ingredient.is_allergen = is_allergen;
                    }
                }
            }
        }
    };

    let mut handle_unified_ingredient_select = {
        let mut ingredients = props.ingredients;
        move |unified_ingredient: UnifiedIngredient| {
            if let Some(mut ingredient) = ingredients.get_mut(props.index) {
                let ingredient_name = unified_ingredient.name.clone();

                // Check if this is a saved composite ingredient by checking saved ingredients
                let saved_ingredients = get_saved_ingredients_list();
                let is_saved_composite = saved_ingredients.iter().any(|i| i.name == ingredient_name);

                if is_saved_composite {
                    // If it's a saved composite ingredient, expand its sub-components
                    if let Some(saved) = saved_ingredients.iter().find(|i| i.name == ingredient_name) {
                        if let Some(saved_subs) = &saved.sub_components {
                            // Add all sub-components from the saved ingredient
                            if let Some(sub_components) = &mut ingredient.sub_components {
                                for sub in saved_subs {
                                    sub_components.push(sub.clone());
                                }
                            } else {
                                ingredient.sub_components = Some(saved_subs.clone());
                            }
                        }
                    }
                } else {
                    // Extract allergen status from unified ingredient, falling back to lookup
                    let allergen_status = unified_ingredient.is_allergen.unwrap_or_else(|| lookup_allergen(&ingredient_name));

                    if let Some(sub_components) = &mut ingredient.sub_components {
                        sub_components.push(SubIngredient {
                            name: ingredient_name,
                            is_allergen: allergen_status,
                        });
                    } else {
                        let sub_components = vec![
                            SubIngredient {
                                name: ingredient_name,
                                is_allergen: allergen_status,
                            }
                        ];
                        ingredient.sub_components = Some(sub_components);
                    }
                }
            }
            name_to_add.set(String::new());
        }
    };

    rsx! {
        div { class: "flex flex-col gap-4",
            table { class: "table border-solid",
                tr {
                    th { "{t!(\"label.zutat\")}" }
                    th { "" }
                    th { "" }
                }
                if let Some(sub_components) = props.ingredients.clone().get(props.index).and_then(|ingredient| ingredient.sub_components.clone()) {
                    for (key, ingr) in sub_components.iter().enumerate() {
                        tr { key: "{key}",
                            td {
                                class: "flex items-center gap-2",
                                div {
                                    if ingr.is_allergen {
                                        span { class: "font-bold", "{ingr.name}" }
                                    } else {
                                        "{ingr.name}"
                                    }
                                }
                                // Show ingredient symbols
                                IngredientSymbolsCompact {
                                    ingredient: subingredient_to_unified(ingr)
                                }
                            }
                            td {
                                // Show allergen status
                                {
                                    let is_custom = !food_db().iter().any(|(name, _)| name == &ingr.name);
                                    if is_custom {
                                        rsx! {
                                            // Custom ingredient - show checkbox
                                            label { class: "label cursor-pointer",
                                                input {
                                                    class: "checkbox",
                                                    r#type: "checkbox",
                                                    checked: "{ingr.is_allergen}",
                                                    oninput: move |e| {
                                                        toggle_allergen_callback(props.index, key, e.value() == "true");
                                                    },
                                                }
                                                span { class: "label-text ml-2",
                                                    {t!("label.allergen")}
                                                }
                                            }
                                        }
                                    } else if ingr.is_allergen {
                                        rsx! {
                                            // Database allergen - show text only
                                            span { class: "font-bold", "({t!(\"label.allergen\")})" }
                                        }
                                    } else {
                                        rsx! {}
                                    }
                                }
                            }
                            td {
                                button {
                                    class: "btn btn-square",
                                    dangerous_inner_html: r###"<svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>"###,
                                    onclick: move |_| {
                                        delete_callback(props.index, key);
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
        div { class: "flex flex-row gap-4 items-center",
            div { class: "flex-1",
                UnifiedIngredientInput {
                    bound_value: name_to_add,
                    on_ingredient_select: handle_unified_ingredient_select,
                    required: false,
                    placeholder: t!("placeholder.zutatName").to_string()
                }
            }
        }
    }
}
