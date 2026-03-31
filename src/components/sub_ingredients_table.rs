use crate::components::*;
use crate::core::Ingredient;
use crate::model::{lookup_allergen, lookup_agricultural};
use crate::persistence::get_saved_ingredients_list;
use crate::services::{UnifiedIngredient, IngredientSource};
use dioxus::prelude::*;
use rust_i18n::t;

#[derive(Props, Clone, PartialEq)]
pub struct SubIngredientsTableProps {
    ingredients: Signal<Vec<Ingredient>>,
    index: usize,
    on_edit_child: EventHandler<usize>,
}
pub fn SubIngredientsTable(props: SubIngredientsTableProps) -> Element {
    let name_to_add = use_signal(String::new);
    // Store the selected ingredient from dropdown (if any) to preserve metadata
    let selected_ingredient: Signal<Option<UnifiedIngredient>> = use_signal(|| None);

    let mut delete_callback = {
        let mut ingredients = props.ingredients;
        move |index: usize, child_index: usize| {
            if let Some(mut ingredient) = ingredients.get_mut(index) {
                if let Some(children) = &mut ingredient.children {
                    children.remove(child_index);
                }
            }
        }
    };

    // When user selects from dropdown, just populate the input field (don't add yet)
    let handle_dropdown_select = {
        let mut name_to_add = name_to_add;
        let mut selected_ingredient = selected_ingredient;
        move |unified_ingredient: UnifiedIngredient| {
            name_to_add.set(unified_ingredient.name.clone());
            selected_ingredient.set(Some(unified_ingredient));
        }
    };

    // Actually add the ingredient (called by button click)
    let mut add_ingredient = {
        let mut ingredients = props.ingredients;
        let mut name_to_add = name_to_add;
        let mut selected_ingredient = selected_ingredient;
        move || {
            let value = name_to_add();
            if value.trim().is_empty() {
                return;
            }

            // Use the selected ingredient if available, otherwise create custom
            let unified_ingredient = selected_ingredient().unwrap_or_else(|| UnifiedIngredient {
                name: value.clone(),
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
                source: IngredientSource::Local,
            });

            if let Some(mut ingredient) = ingredients.get_mut(props.index) {
                let ingredient_name = unified_ingredient.name.clone();

                // Check if this is a saved composite ingredient by checking saved ingredients
                let saved_ingredients = get_saved_ingredients_list();
                let is_saved_composite = saved_ingredients.iter().any(|i| i.name == ingredient_name);

                if is_saved_composite {
                    // If it's a saved composite ingredient, expand its children
                    if let Some(saved) = saved_ingredients.iter().find(|i| i.name == ingredient_name) {
                        if let Some(saved_children) = &saved.children {
                            // Add all children from the saved ingredient
                            if let Some(children) = &mut ingredient.children {
                                for child in saved_children {
                                    children.push(child.clone());
                                }
                            } else {
                                ingredient.children = Some(saved_children.clone());
                            }
                        }
                    }
                } else {
                    // Extract allergen status from unified ingredient, falling back to lookup
                    let allergen_status = unified_ingredient.is_allergen.unwrap_or_else(|| lookup_allergen(&ingredient_name));

                    let new_child = Ingredient {
                        name: ingredient_name.clone(),
                        is_allergen: allergen_status,
                        is_agricultural: lookup_agricultural(&ingredient_name),
                        ..Default::default()
                    };

                    if let Some(children) = &mut ingredient.children {
                        children.push(new_child);
                    } else {
                        ingredient.children = Some(vec![new_child]);
                    }
                }
            }
            name_to_add.set(String::new());
            selected_ingredient.set(None);
        }
    };

    rsx! {
        div { class: "flex flex-col gap-4",
            table { class: "table border-solid",
                tr {
                    th { "{t!(\"label.zutat\").to_string()}" }
                    th { "" }
                    th { "" }
                }
                if let Some(children) = props.ingredients.clone().get(props.index).and_then(|ingredient| ingredient.children.clone()) {
                    for (key, child) in children.iter().enumerate() {
                        tr { key: "{key}-{child.name}",
                            td {
                                class: "flex items-center gap-2",
                                div {
                                    if child.is_allergen {
                                        span { class: "font-bold", "{child.name}" }
                                    } else {
                                        "{child.name}"
                                    }
                                }
                            }
                            td {
                                button {
                                    class: "btn btn-square btn-sm btn-ghost",
                                    title: t!("nav.bearbeiten").to_string(),
                                    onclick: {
                                        let on_edit = props.on_edit_child;
                                        move |_| on_edit.call(key)
                                    },
                                    icons::ListDetail {}
                                }
                            }
                            td {
                                button {
                                    class: "btn btn-square btn-sm",
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
        div { class: "flex flex-row gap-2 items-center",
            div { class: "flex-1",
                UnifiedIngredientInput {
                    bound_value: name_to_add,
                    on_ingredient_select: handle_dropdown_select,
                    required: false,
                    placeholder: t!("placeholder.zutatName").to_string()
                }
            }
            button {
                class: "btn btn-primary",
                r#type: "button",
                disabled: name_to_add().trim().is_empty(),
                onclick: move |_| {
                    add_ingredient();
                },
                {t!("nav.hinzufuegen").to_string()}
            }
        }
    }
}
