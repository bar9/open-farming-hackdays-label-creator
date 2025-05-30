use dioxus::prelude::*;
use crate::components::*;
use crate::core::Ingredient;
use crate::model::food_db;
use rust_i18n::t;


// TODO: rework save/cancel (stateful modal):
// seems we already have many parts, only the writes via props.inredients.write() are to be delegated to a save() handler

#[derive(Props, Clone, PartialEq)]
pub struct IngredientDetailProps {
    ingredients: Signal<Vec<Ingredient>>,
    index: usize,
    #[props(default = false)]
    genesis: bool
}
pub fn IngredientDetail(mut props: IngredientDetailProps) -> Element {
    let index: usize;
    let mut ingredients: Signal<Vec<Ingredient>>;
    if (props.genesis) {
        ingredients = use_signal(|| vec![Ingredient::default()]);
        index = 0;
    } else {
        index = props.index;
        ingredients = props.ingredients.clone();
    }
    let mut is_open = use_signal(|| false);
    let mut scale_together = use_signal(|| false);
    let mut amount_to_edit = use_signal(|| 0.);
    let mut is_composite = use_signal(|| ingredients.get(index).unwrap().clone().sub_components.unwrap_or_default().len() > 0);
    let mut is_namensgebend = use_signal(|| ingredients.get(index).unwrap().is_namensgebend.unwrap_or(false));
    
    let ingredient = ingredients.get(index).unwrap().clone();
    let old_ingredient = ingredients.get(index).unwrap().clone();
    let old_ingredient_2 = ingredients.get(index).unwrap().clone();
    let mut update_name = move |new_name| {
        ingredients.write()[index] = Ingredient { name: new_name, ..old_ingredient.clone() };
    };
    
    let mut handle_genesis = move || {
        let mut new_ingredient = ingredients.get(index).unwrap().clone();
        new_ingredient.amount = amount_to_edit();
        props.ingredients.write().push(new_ingredient);
        ingredients = use_signal(|| vec![Ingredient::default()]);
        is_open.set(false);
    };
    
    rsx! {
        if props.genesis {
            button {
                class: "btn btn-accent",
                onclick: move |_| is_open.toggle(),
                onkeydown: move |evt: KeyboardEvent| if evt.key() == Key::Escape { is_open.set(false); },
                "{t!(\"nav.hinzufuegen\")}"
            }
        } else {
            button {
                class: "btn join-item btn-outline",
                onclick: move |_| is_open.toggle(),
                onkeydown: move |evt: KeyboardEvent| if evt.key() == Key::Escape { is_open.set(false); },
                icons::ListDetail {}
            }
        }
        if is_open() { div { class: "fixed inset-0 bg-black bg-opacity-50 backdrop-blur-md" } }
        dialog { open: "{is_open}", class: "modal",
            div { class: "modal-box bg-base-100",
                h3 { class: "font-bold text-lg", "{t!(\"label.zutatDetails\")}" }
                FormField {
                    label: t!("label.zutat"),
                    input {
                        list: "ingredients",
                        r#type: "flex",
                        placeholder: t!("placeholder.zutatName").as_ref(),
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
                    label: t!("label.menge"),
                    ValidationDisplay {
                        paths: vec![
                            format!("ingredients[{}][amount]", index)
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
                    if !props.genesis {
                        label { class: "label cursor-pointer",
                            input {
                                class: "checkbox",
                                r#type: "checkbox",
                                checked: "{scale_together}",
                                oninput: move |e| scale_together.set(e.value() == "true"),
                            }
                            span { class: "label-text",
                                "{t!(\"nav.verhaeltnisseBeibehalten\")}"
                            }
                        }
                        button {
                            class: "btn btn-accent",
                            onclick: move |_evt| {
                                let old_ingredient = ingredients.get(index).unwrap().clone();
                                if *scale_together.read() {
                                    let factor: f64 = (&*amount_to_edit)()
                                        / old_ingredient.amount;
                                    let clonedIngredients = ingredients.clone();
                                    for (key, elem) in clonedIngredients.iter().enumerate() {
                                        let old_ingredient = elem.clone();
                                        ingredients.write()[key] = Ingredient {
                                            amount: (elem.amount * factor),
                                            ..old_ingredient.clone()
                                        }
                                    }
                                } else {
                                    ingredients.write()[index] = Ingredient {
                                        amount: (&*amount_to_edit)(),
                                        ..old_ingredient.clone()
                                    }
                                }
                            },
                            "{t!(\"nav.anpassen\")}"
                        }
                    }
                }

                br {}
                br {}

                FormField {
                    label: t!("label.zusammengesetzteZutat"),
                    help: Some((t!("help.zusammengesetzteZutaten")).into()),
                    label { class: "label cursor-pointer",
                        input {
                            class: "checkbox",
                            r#type: "checkbox",
                            checked: "{is_composite}",
                            oninput: move |e| is_composite.set(e.value() == "true"),
                        }
                        span { class: "label-text",
                            "{t!(\"label.zusammengesetzteZutat\")}"
                        }
                    }
                    if is_composite() {
                        SubIngredientsTable {
                            ingredients: ingredients,
                            index:  index
                        }
                    }
                }
                br {}
                ConditionalDisplay {
                    path: "namensgebende_zutat",
                    FormField {
                        help: Some((t!("help.namensgebendeZutaten")).into()),
                        label: t!("label.namensgebendeZutat"),
                        label { class: "label cursor-pointer",
                            input {
                                class: "checkbox",
                                r#type: "checkbox",
                                checked: "{is_namensgebend}",
                                oninput: move |e| {
                                    is_namensgebend.set(e.value() == "true");
                                    ingredients.write()[index] = Ingredient {
                                        is_namensgebend: Some(e.value() == "true"),
                                        ..old_ingredient_2.clone()
                                    }
                                },
                            }
                            span { class: "label-text",
                                "{t!(\"label.namensgebendeZutat\")}"
                            }
                        }
                    }
                }
                div { class: "modal-action",
                    form { method: "dialog",
                        button {
                            class: "btn",
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
                        if props.genesis {
                            button {
                                class: "btn",
                                onclick: move |_| handle_genesis(),
                                "Speichern",
                            }
                        }
                    }
                }
            }
        }
    }
}
