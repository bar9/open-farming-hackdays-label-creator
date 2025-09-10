use crate::core::{Ingredient, SubIngredient};
use crate::model::{food_db, lookup_allergen};
use crate::persistence::get_saved_ingredients_list;
use dioxus::prelude::*;
use rust_i18n::t;

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

    let add_callback = {
        let mut ingredients = props.ingredients;
        let mut name_to_add = name_to_add;
        move |_evt| {
            if let Some(mut ingredient) = ingredients.get_mut(props.index) {
                let ingredient_name = name_to_add();
                
                // Check if this is a saved composite ingredient
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
                    // Check if ingredient is in database and get allergen status
                    let allergen_status = lookup_allergen(&ingredient_name);
                    
                    if let Some(sub_components) = &mut ingredient.sub_components {
                        sub_components.push(SubIngredient {
                            name: ingredient_name.clone(),
                            is_allergen: allergen_status,
                        });
                    } else {
                        let sub_components = vec![
                            SubIngredient {
                                name: ingredient_name.clone(),
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
                    th { "{t!(\"label.zutatEingeben\")}" }
                    th { "" }
                    th { "" }
                }
                if let Some(sub_components) = props.ingredients.clone().get(props.index).and_then(|ingredient| ingredient.sub_components.clone()) {
                    for (key, ingr) in sub_components.iter().enumerate() {
                        tr { key: "{key}",
                            td { 
                                if ingr.is_allergen {
                                    span { class: "font-bold", "{ingr.name}" }
                                } else {
                                    "{ingr.name}"
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
        div { class: "flex flex-row gap-4",
            input {
                list: "ingredients",
                r#type: "flex",
                placeholder: t!("placeholder.zutatName").as_ref(),
                class: "input input-accent w-full",
                oninput: move |evt| {
                    name_to_add.set(evt.data.value());
                },
                value: "{name_to_add}",
                datalist { id: "ingredients",
                    // First, add saved composite ingredients
                    for saved_ing in get_saved_ingredients_list() {
                        option { 
                            value: "{saved_ing.name}",
                            label: "(Gespeichert)"
                        }
                    }
                    // Then add database ingredients
                    for item in food_db().clone() {
                        option { 
                            value: "{item.0}",
                            // Show allergen marker in label without duplicating the name
                            label: if item.1 { "(Allergen)" } else { "" }
                        }
                    }
                }
            }
            button {
                class: "btn btn-accent",
                onclick: add_callback,
                "{t!(\"nav.hinzufuegen\")}"
            }
        }
    }
}
