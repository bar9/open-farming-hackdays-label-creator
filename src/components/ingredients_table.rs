use std::collections::HashMap;
use dioxus::prelude::*;
use crate::components::*;
use crate::components::ingredient_detail::IngredientDetail;
use crate::core::Ingredient;
use crate::model::{food_db};
use rust_i18n::t;

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
            div { class: "grid gap-4 grid-cols-3 border-bottom items-center",
                span { class: "font-bold", "{t!(\"label.zutatEingeben\")}" }
                span { class: "font-bold text-right", "{t!(\"Menge\")}" }
                span {}
            }
            for (key , & ref ingr) in props.ingredients.read().iter().enumerate() {
                // ValidationDisplay {
                //     paths: vec![format!("ingredients[{}][amount]", key)],
                div { class: "grid gap-4 grid-cols-3 odd:bg-gray-100 even:bg-white items-center", key: "{key}",
                    div { "{ingr.composite_name()}" if ingr.is_namensgebend.unwrap_or(false) {" ({t!(\"label.namensgebend\")}"} }
                    div {
                        class: "text-right",
                        "{ingr.amount} g"
                    }
                    div {
                        class: "text-right",
                        div {
                            class: "join",
                            IngredientDetail {ingredients: props.ingredients, index: key}
                            button {
                                class: "btn btn-outline join-item",
                                dangerous_inner_html: r###"<svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>"###,
                                onclick: move |_| {
                                    delete_callback(key, props.ingredients.clone());
                                },
                            }
                        }
                    }
                }
            },
            if props.ingredients.len() > 0 {
                ConditionalDisplay {
                    path: "manuelles_total",
                    div {
                        class: "grid grid-cols-3 gap-4",
                        div {"Total"}
                        div {
                            class: "text-right",
                            "{total_amount} g"
                        }

                        FormField {
                            label: "{t!(\"label.manuellesTotal\")}",
                            help: Some((t!("help.manuellesTotal")).into()),
                            input {
                                r#type: "number",
                                placeholder: t!("label.manuellesTotal").as_ref(),
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

                        div {

                        }
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
                genesis: true
            } //index is ignored
            // button {
            //     class: "btn btn-accent",
            //     onclick: move |_evt| { },
            //     "{t!(\"nav.hinzufuegen\")}"
            // }
        }
    }
}
