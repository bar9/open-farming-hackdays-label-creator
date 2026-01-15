use crate::components::ingredient_detail::IngredientDetail;
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
    // Calculate category flags if we have a category
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
        // Take first origin for display (UnifiedIngredient uses single origin for flag display)
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
        source: IngredientSource::Local, // Most ingredients come from local data
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
    let delete_callback = |index, mut list: Signal<Vec<Ingredient>>| list.remove(index);
    // let name_to_add = use_signal(|| String::new());
    // let amount_to_add = use_signal(|| 0);
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
            for (key , ingr) in props.ingredients.read().iter().enumerate() {
                // ValidationDisplay {
                //     paths: vec![format!("ingredients[{}][amount]", key)],
                div { class: "grid gap-4 grid-cols-3 odd:bg-gray-100 even:bg-white items-center", key: "{key}",
                    div {
                        class: "flex items-center gap-2",
                        div {
                            class: "flex items-center gap-1",
                            // Show country flags if origins are set
                            if let Some(origins) = &ingr.origins {
                                for origin in origins.iter() {
                                    span { class: "text-lg", "{origin.flag_emoji()}" }
                                }
                            }
                            div {
                                if ingr.is_allergen {
                                    span { class: "font-bold", "{ingr.composite_name()}" }
                                } else {
                                    "{ingr.composite_name()}"
                                }
                                if ingr.is_namensgebend.unwrap_or(false) {" ({t!(\"label.namensgebend\")})"}
                            }
                        }
                        // Show ingredient symbols
                        IngredientSymbolsCompact {
                            ingredient: ingredient_to_unified(ingr)
                        }
                    }
                    div {
                        class: "text-right",
                        "{ingr.amount} " {t!(ingr.unit.translation_key())}
                    }
                    div {
                        class: "text-right",
                        div {
                            class: "join",
                            IngredientDetail {ingredients: props.ingredients, index: key, rules: props.rules}
                            button {
                                class: "btn btn-outline join-item",
                                dangerous_inner_html: r###"<svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>"###,
                                onclick: move |_| {
                                    delete_callback(key, props.ingredients);
                                },
                            }
                        }
                    }
                }
            },
            if props.ingredients.len() > 0 {
                ConditionalDisplay {
                    path: "manuelles_total".to_string(),
                    div {
                        class: "grid grid-cols-3 gap-4",
                        div {{t!("label.total")}}
                        div {
                            class: "text-right",
                            "{total_amount} " {t!("units.g")}
                        }

                        FormField {
                            label: "{t!(\"label.manuellesTotal\")}",
                            help: Some((t!("help.manuellesTotal")).into()),
                            input {
                                r#type: "number",
                                placeholder: t!("label.manuellesTotal").as_ref(),
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

                        div {

                        }
                    }
                }
                // Checkbox to mark recipe as complete - enables validation
                div { class: "form-control mt-4",
                    label { class: "label cursor-pointer justify-start gap-2",
                        input {
                            r#type: "checkbox",
                            class: "checkbox checkbox-primary",
                            checked: (props.rezeptur_vollstaendig)(),
                            onchange: move |evt| {
                                props.rezeptur_vollstaendig.set(evt.checked());
                            }
                        }
                        span { class: "label-text", "{t!(\"label.rezepturVollstaendig\")}" }
                    }
                }
            }
        }
        div { class: "grid grid-cols-3 gap-4 items-center border-top",
            // input {
            //     list: "ingredients",
            //     r#type: "flex",
            //     placeholder: t!("placeholder.zutatName").as_ref(),
            //     class: "input input-bordered bg-white input-accent w-full",
            //     oninput: move |evt| name_to_add.set(evt.data.value()),
            //     value: "{name_to_add}",
            //     datalist { id: "ingredients",
            //         for item in food_db().clone() {
            //             option { value: "{item.0}" }
            //         }
            //     }
            // }
            // div {
            //     class: "flex flex-row gap-4 items-center text-right",
            //     input {
            //         r#type: "number",
            //         placeholder: t!("placeholder.menge").as_ref(),
            //         class: "input input-bordered bg-white input-accent w-full",
            //         oninput: move |evt| {
            //             if let Ok(amount) = evt.data.value().parse::<i32>() {
            //                 amount_to_add.set(amount);
            //             }
            //         },
            //         value: "{amount_to_add}",
            //     }
            //     "g"
            // }
            IngredientDetail {
                ingredients: props.ingredients,
                index: 0,
                genesis: true,
                rules: props.rules
            } //index is ignored
            div {} // Empty cell for category column
            div {} // Empty cell for amount column
            // button {
            //     class: "btn btn-accent",
            //     onclick: move |_evt| { },
            //     "{t!(\"nav.hinzufuegen\")}"
            // }
        }

        // Show validation messages for all ingredients
        ValidationDisplay {
            paths: (0..props.ingredients.read().len()).map(|i| format!("ingredients[{}][origin]", i)).collect::<Vec<_>>(),
            div {}
        }
    }
}
