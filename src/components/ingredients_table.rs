use std::collections::HashMap;
use dioxus::prelude::*;
use crate::components::{icons, ConditionalDisplay, FormField, SubIngredientsTable};
use crate::components::ingredient_detail::IngredientDetail;
use crate::components::validation_display::ValidationDisplay;
use crate::core::{Ingredient, SubIngredient};
use crate::model::{food_db};

#[derive(Props, Clone, PartialEq)]
pub struct IngredientsTableProps {
    ingredients: Signal<Vec<Ingredient>>,
    manual_total: Signal<Option<f64>>,
    validation_messages: Memo<HashMap<String, &'static str>>
}
pub fn IngredientsTable(mut props: IngredientsTableProps) -> Element {
    let delete_callback = |index, mut list: Signal<Vec<Ingredient>>| list.remove(index);
    let mut name_to_add = use_signal(|| String::new());
    let mut amount_to_add = use_signal(|| 0);
    let total_amount = use_memo (move || {
        props.ingredients.read().iter().map(|x: &Ingredient|x.amount).sum::<f64>()
    });
    rsx! {
        div { class: "flex flex-col gap-4",
            table { class: "table border-solid",
                tr {
                    th { "Zutat (eingeben oder auswählen)" }
                    th { "Menge" }
                }
                for (key , & ref ingr) in props.ingredients.read().iter().enumerate() {
                    // ValidationDisplay {
                    //     paths: vec![format!("ingredients[{}][amount]", key)],
                        tr { key: "{key}",
                            td { "{ingr.composite_name()}" if ingr.is_namensgebend.unwrap_or(false) {" (namensgebend)"} }
                            td {
                                "{ingr.amount} g"
                            }
                            td {
                                IngredientDetail {ingredients: props.ingredients, index: key}
                            }
                            td {
                                button {
                                    class: "btn btn-square",
                                    dangerous_inner_html: r###"<svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>"###,
                                    onclick: move |_| {
                                        delete_callback(key, props.ingredients.clone());
                                    },
                                }

                            }
                        }
                    // }
                },
                if props.ingredients.len() > 0 {
                    ConditionalDisplay {
                        path: "manuelles_total",
                        tr {
                            td {
                                "Total: {total_amount} g"
                            }
                            td {
                                label {
                                    "Manuelles Total:"
                                }
                                input {
                                    r#type: "number",
                                    placeholder: "Manuelles Total",
                                    class: "input input-bordered bg-white input-accent w-full",
                                    onchange: move |evt| {
                                        if let Ok(amount) = evt.data.value().parse::<f64>() {
                                            props.manual_total.set(Some(amount));
                                        } else {
                                            props.manual_total.set(None);
                                        }
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
                placeholder: "Name",
                class: "input input-bordered bg-white input-accent w-full",
                oninput: move |evt| name_to_add.set(evt.data.value()),
                value: "{name_to_add}",
                datalist { id: "ingredients",
                    for item in food_db().clone() {
                        option { value: "{item.0}" }
                    }
                }
            }
            input {
                r#type: "number",
                placeholder: "Menge",
                class: "input input-bordered bg-white input-accent w-full",
                oninput: move |evt| {
                    if let Ok(amount) = evt.data.value().parse::<i32>() {
                        amount_to_add.set(amount);
                    }
                },
                value: "{amount_to_add}",
            }
            "g"
            button {
                class: "btn btn-accent",
                onclick: move |_evt| {
                    props
                        .ingredients
                        .write()
                        .push(
                            Ingredient::from_name_amount((&*name_to_add)(), (&*amount_to_add)() as f64),
                        );
                    name_to_add.set(String::new());
                    amount_to_add.set(0);
                },
                "Hinzufügen"
            }
        }
    }
}
