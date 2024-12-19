use std::collections::HashMap;
use dioxus::prelude::*;
use crate::components::{ConditionalDisplay, FormField, SubIngredientsTable};
use crate::components::validation_display::ValidationDisplay;
use crate::core::{Ingredient, SubIngredient};
use crate::model::{food_db};

#[derive(Props, Clone, PartialEq)]
pub struct IngredientsTableProps {
    ingredients: Signal<Vec<Ingredient>>,
    manual_total: Signal<Option<f64>>,
    validation_messages: Memo<HashMap<String, &'static str>>
    // TODO: accept
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

#[derive(Props, Clone, PartialEq)]
pub struct IngredientDetailProps {
    ingredients: Signal<Vec<Ingredient>>,
    index: usize
}
pub fn IngredientDetail(mut props: IngredientDetailProps) -> Element {
    let mut is_open = use_signal(|| false);
    let mut scale_together = use_signal(|| false);
    let mut amount_to_edit = use_signal(|| 0.);
    let mut is_composite = use_signal(|| props.ingredients.get(props.index).unwrap().clone().sub_components.unwrap_or_default().len() > 0);
    let mut is_namensgebend = use_signal(|| props.ingredients.get(props.index).unwrap().is_namensgebend.unwrap_or(false));
    let ingredient = props.ingredients.get(props.index).unwrap().clone();
    let old_ingredient = props.ingredients.get(props.index).unwrap().clone();
    let old_ingredient_2 = props.ingredients.get(props.index).unwrap().clone();
    let mut update_name = move |new_name| {
        props.ingredients.write()[props.index] = Ingredient { name: new_name, ..old_ingredient.clone() };
    };
    rsx! {
        button {
            class: "btn btn-square",
            onclick: move |_| is_open.toggle(),
            dangerous_inner_html: r###"<svg class="h-6 w-6" fill="none" height="24" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" viewBox="0 0 24 24" width="24" xmlns="http://www.w3.org/2000/svg"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>"###,
        }
        if is_open() { div { class: "fixed inset-0 bg-black bg-opacity-50 backdrop-blur-md" } }
        dialog { open: "{is_open}", class: "modal",
            div { class: "modal-box bg-base-100",
                h3 { class: "font-bold text-lg", "Zutat Details" }
                FormField {
                    label: "Zutat",
                    input {
                        list: "ingredients",
                        r#type: "flex",
                        placeholder: "Name",
                        class: "input input-bordered bg-white input-accent w-full",
                        oninput: move |evt| update_name(evt.data.value()),
                        value: "{ingredient.name}",
                        datalist { id: "ingredients",
                            for item in food_db().clone() {
                                option { value: "{item.0}" }
                            }
                        }
                    }
                }
                FormField {
                    label: "Menge",
                    ValidationDisplay {
                        paths: vec![
                            format!("ingredients[{}][amount]", props.index)
                        ],
                        input {
                            r#type: "number",
                            placeholder: "Menge",
                            class: "input input-bordered bg-white input-accent w-full",
                            onchange: move |evt| {
                                if let Ok(amount) = evt.data.value().parse::<f64>() {
                                    amount_to_edit.set(amount);
                                }
                            },
                            value: "{ingredient.amount}",
                        }
                    }
                    label { class: "label cursor-pointer",
                        input {
                            class: "checkbox",
                            r#type: "checkbox",
                            checked: "{scale_together}",
                            oninput: move |e| scale_together.set(e.value() == "true"),
                        }
                        span { class: "label-text",
                            "Verhältnisse beibehalten"
                        }
                    }
                    button {
                        class: "btn btn-accent",
                        onclick: move |_evt| {
                            let old_ingredient = props.ingredients.get(props.index).unwrap().clone();
                            if *scale_together.read() {
                                let factor: f64 = (&*amount_to_edit)()
                                    / old_ingredient.amount;
                                let ingredients = props.ingredients.read().clone();
                                for (key, elem) in ingredients.iter().enumerate() {
                                    let old_ingredient = elem.clone();
                                    props.ingredients.write()[key] = Ingredient {
                                        amount: (elem.amount * factor),
                                        ..old_ingredient.clone()
                                    }
                                }
                            } else {
                                // let name = (props.ingredients.read().get(props.index).unwrap().name.clone());
                                props.ingredients.write()[props.index] = Ingredient {
                                    amount: (&*amount_to_edit)(),
                                    ..old_ingredient.clone()
                                }
                            }
                        },
                        "Anpassen"
                    }
                }

                br {}
                br {}

                FormField {
                    label: "Zusammengesetzte Zutat",
                    label { class: "label cursor-pointer",
                        input {
                            class: "checkbox",
                            r#type: "checkbox",
                            checked: "{is_composite}",
                            oninput: move |e| is_composite.set(e.value() == "true"),
                        }
                        span { class: "label-text",
                            "Zusammengesetzte Zutat"
                        }
                    }
                    if is_composite() {
                        SubIngredientsTable {
                            ingredients: props.ingredients,
                            index:  props.index
                        }
                    }
                }
                br {}
                ConditionalDisplay {
                    path: "namensgebende_zutat",
                    FormField {
                        label: "Namensgebende Zutat",
                        label { class: "label cursor-pointer",
                            input {
                                class: "checkbox",
                                r#type: "checkbox",
                                checked: "{is_namensgebend}",
                                oninput: move |e| {
                                    is_namensgebend.set(e.value() == "true");
                                    props.ingredients.write()[props.index] = Ingredient {
                                        is_namensgebend: Some(e.value() == "true"),
                                        ..old_ingredient_2.clone()
                                    }
                                },
                            }
                            span { class: "label-text",
                                "Namensgebende Zutat"
                            }
                        }
                    }
                }
                // ROW name amount
                // Name
                // Amount
                // cond. ROW change amount
                // ROW namensgebend?
                // ROW composite?
                // Maybe CompositeIngredientsTable
                div { class: "modal-action",
                    form { method: "dialog",
                        button {
                            class: "btn btn-sm",
                            onclick: move |_| is_open.toggle(),
                            onkeydown: move |evt| {
                                match evt.key() {
                                    Key::Escape => {
                                        is_open.set(false);
                                    }
                                    _ => {}
                                }
                            },
                            "× Schliessen",
                        }
                    }
                }
            }
        }
    }
}