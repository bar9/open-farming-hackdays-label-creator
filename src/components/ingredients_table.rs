use crate::components::card_stack::{CardStack, GenesisModal};
use crate::components::ingredient_path::IngredientPath;
use crate::components::*;
use crate::core::Ingredient;
use crate::rules::RuleDef;
use crate::services::{UnifiedIngredient, IngredientSource};
use crate::category_service::*;
use dioxus::prelude::*;
use rust_i18n::t;
use std::collections::HashMap;

/// Convert an Ingredient to UnifiedIngredient for display purposes
fn ingredient_to_unified(ingredient: &Ingredient) -> UnifiedIngredient {
    let (is_meat, is_fish, is_dairy, is_egg, is_honey, is_plant) = if let Some(ref category) = ingredient.category {
        (
            Some(is_meat_category(category)),
            Some(is_fish_category(category)),
            Some(is_dairy_category(category)),
            Some(is_egg_category(category)),
            Some(is_honey_category(category)),
            Some(is_plant_category(category)),
        )
    } else {
        (None, None, None, None, None, None)
    };

    UnifiedIngredient {
        name: ingredient.name.clone(),
        category: ingredient.category.clone(),
        origin: ingredient.origins.as_ref().and_then(|o| o.first().cloned()),
        is_allergen: Some(ingredient.is_allergen),
        is_agricultural: Some(ingredient.is_agricultural),
        is_meat,
        is_fish,
        is_dairy,
        is_egg,
        is_honey,
        is_plant,
        is_bio: ingredient.is_bio,
        source: IngredientSource::Local,
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct IngredientsTableProps {
    ingredients: Signal<Vec<Ingredient>>,
    manual_total: Signal<Option<f64>>,
    validation_messages: Memo<HashMap<String, Vec<String>>>,
    rules: Memo<Vec<RuleDef>>,
    rezeptur_vollstaendig: Signal<bool>,
}
pub fn IngredientsTable(mut props: IngredientsTableProps) -> Element {
    let editing_path: Signal<IngredientPath> = use_signal(Vec::new);

    let total_amount = use_memo(move || {
        props
            .ingredients
            .read()
            .iter()
            .map(|x: &Ingredient| x.amount)
            .sum::<f64>()
    });

    rsx! {
        div { class: "flex flex-col gap-4",
            // Recursive tree rendering
            {render_ingredient_tree(
                &props.ingredients.read(),
                &[],
                0,
                &editing_path,
                props.ingredients,
            )}

            if props.ingredients.len() > 0 {
                ConditionalDisplay {
                    path: "manuelles_total".to_string(),
                    div {
                        class: "grid grid-cols-3 gap-4",
                        div {{t!("label.total").to_string()}}
                        div {
                            class: "text-right",
                            "{total_amount} " {t!("units.g").to_string()}
                        }

                        FormField {
                            label: "{t!(\"label.manuellesTotal\").to_string()}",
                            help: Some(t!("help.manuellesTotal").to_string()),
                            input {
                                r#type: "number",
                                placeholder: t!("label.manuellesTotal").to_string().as_ref(),
                                class: "input input-accent w-full",
                                min: "0",
                                onchange: move |evt| {
                                    if let Ok(amount) = evt.data.value().parse::<f64>() {
                                        props.manual_total.set(Some(amount));
                                    } else {
                                        props.manual_total.set(None);
                                    }
                                },
                            }
                        }

                        div {}
                    }
                }
                // Button to trigger recipe validation
                div { class: "mt-4",
                    button {
                        class: if (props.rezeptur_vollstaendig)() { "btn btn-disabled" } else { "btn btn-accent" },
                        disabled: (props.rezeptur_vollstaendig)(),
                        onclick: move |_| {
                            props.rezeptur_vollstaendig.set(true);
                        },
                        "{t!(\"label.rezepturVollstaendig\").to_string()}"
                    }
                }
            }
        }
        div { class: "grid grid-cols-3 gap-4 items-center border-top",
            GenesisModal {
                ingredients: props.ingredients,
                rules: props.rules
            }
            div {}
            div {}
        }

        // Stacking card modal for editing
        CardStack {
            ingredients: props.ingredients,
            editing_path: editing_path,
            rules: props.rules,
        }

        // Show validation messages for all ingredients
        ValidationDisplay {
            paths: (0..props.ingredients.read().len()).map(|i| format!("ingredients[{}][origin]", i)).collect::<Vec<_>>(),
            div {}
        }
    }
}

/// Recursively render the ingredient tree with indentation.
fn render_ingredient_tree(
    ingredients: &[Ingredient],
    path_prefix: &[usize],
    depth: usize,
    editing_path: &Signal<IngredientPath>,
    root_ingredients: Signal<Vec<Ingredient>>,
) -> Element {
    let elements: Vec<Element> = ingredients.iter().enumerate()
        .map(|(i, ingr)| {
            let full_path: IngredientPath = {
                let mut p = path_prefix.to_vec();
                p.push(i);
                p
            };
            let edit_path = full_path.clone();
            let mut editing_path_signal = *editing_path;
            let ingr = ingr.clone();
            let name = ingr.name.clone();
            let is_allergen = ingr.is_allergen;
            let is_namensgebend = ingr.is_namensgebend.unwrap_or(false);
            let computed_origins = ingr.computed_origins();
            let computed_amount = ingr.computed_amount();
            let unit_key = ingr.unit.translation_key().to_string();
            let unified = ingredient_to_unified(&ingr);
            let children = ingr.children.clone();
            let children_for_recurse = children.clone();
            let full_path_for_children = full_path.clone();

            rsx! {
                div {
                    class: if depth.is_multiple_of(2) { "grid gap-4 grid-cols-3 bg-gray-100 items-center" } else { "grid gap-4 grid-cols-3 bg-white items-center" },
                    style: "padding-left: {depth as f32 * 1.5}rem;",
                    key: "{i}-{name}",
                    div {
                        class: "flex items-center gap-2",
                        div {
                            class: "flex items-center gap-1",
                            if let Some(origins) = &computed_origins {
                                for origin in origins.iter() {
                                    span { class: "text-lg", "{origin.flag_emoji()}" }
                                }
                            }
                            div {
                                if is_allergen {
                                    span { class: "font-bold", "{name}" }
                                } else {
                                    "{name}"
                                }
                                if is_namensgebend { " ({t!(\"label.namensgebend\").to_string()})" }
                            }
                        }
                        IngredientSymbolsCompact {
                            ingredient: unified
                        }
                    }
                    div {
                        class: "text-right",
                        "{computed_amount} " {t!(&unit_key).to_string()}
                    }
                    div {
                        class: "text-right",
                        div {
                            class: "join",
                            button {
                                class: "btn join-item btn-outline",
                                onclick: move |_| {
                                    editing_path_signal.set(edit_path.clone());
                                },
                                icons::ListDetail {}
                            }
                            if depth == 0 {
                                button {
                                    class: "btn btn-outline join-item",
                                    dangerous_inner_html: r###"<svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>"###,
                                    onclick: {
                                        let mut root_ingredients = root_ingredients;
                                        move |_| {
                                            root_ingredients.write().remove(i);
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
                // Render children recursively (always expanded)
                if let Some(ref children) = children_for_recurse {
                    if !children.is_empty() {
                        {render_ingredient_tree(
                            children,
                            &full_path_for_children,
                            depth + 1,
                            &editing_path_signal,
                            root_ingredients,
                        )}
                    }
                }
            }
        })
        .collect();

    rsx! {
        {elements.into_iter()}
    }
}
