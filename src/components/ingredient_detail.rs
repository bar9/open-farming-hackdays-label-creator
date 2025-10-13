use crate::components::*;
use crate::core::Ingredient;
use crate::model::{food_db, lookup_allergen, Country};
use crate::shared::Validations;
use dioxus::prelude::*;
use rust_i18n::t;

// TODO: rework save/cancel (stateful modal):
// seems we already have many parts, only the writes via props.inredients.write() are to be delegated to a save() handler

#[derive(Props, Clone, PartialEq)]
pub struct IngredientDetailProps {
    ingredients: Signal<Vec<Ingredient>>,
    index: usize,
    #[props(default = false)]
    genesis: bool,
}
pub fn IngredientDetail(mut props: IngredientDetailProps) -> Element {
    let index: usize;
    let mut ingredients: Signal<Vec<Ingredient>>;
    if props.genesis {
        ingredients = use_signal(|| vec![Ingredient::default()]);
        index = 0;
    } else {
        index = props.index;
        ingredients = props.ingredients;
    }
    let mut is_open = use_signal(|| false);
    let mut scale_together = use_signal(|| false);
    let mut amount_to_edit = use_signal(|| 0.);
    let mut is_composite = use_signal(|| {
        !ingredients
            .get(index)
            .unwrap()
            .clone()
            .sub_components
            .unwrap_or_default()
            .is_empty()
    });
    let mut is_namensgebend = use_signal(|| {
        ingredients
            .get(index)
            .unwrap()
            .is_namensgebend
            .unwrap_or(false)
    });
    let mut selected_origin = use_signal(|| {
        ingredients
            .get(index)
            .unwrap()
            .origin
            .clone()
    });

    let ingredient = ingredients.get(index).unwrap().clone();
    let old_ingredient = ingredients.get(index).unwrap().clone();
    let old_ingredient_2 = ingredients.get(index).unwrap().clone();
    let old_ingredient_3 = ingredients.get(index).unwrap().clone();
    let mut update_name = move |new_name: String| {
        ingredients.write()[index] = Ingredient {
            name: new_name.clone(),
            is_allergen: lookup_allergen(&new_name),
            ..old_ingredient.clone()
        };
    };

    let mut handle_genesis = move || {
        let mut new_ingredient = ingredients.get(index).unwrap().clone();
        new_ingredient.amount = amount_to_edit();
        new_ingredient.is_allergen = lookup_allergen(&new_ingredient.name);
        
        // Check if ingredient with same name and properties already exists (only for non-composite)
        if new_ingredient.sub_components.is_none() || new_ingredient.sub_components.as_ref().unwrap().is_empty() {
            let mut existing_ingredients = props.ingredients.write();
            if let Some(existing_index) = existing_ingredients.iter().position(|ing| {
                ing.name == new_ingredient.name 
                && ing.is_allergen == new_ingredient.is_allergen
                && ing.is_namensgebend == new_ingredient.is_namensgebend
                && (ing.sub_components.is_none() || ing.sub_components.as_ref().unwrap().is_empty())
            }) {
                // Merge amounts instead of adding duplicate
                existing_ingredients[existing_index].amount += new_ingredient.amount;
            } else {
                existing_ingredients.push(new_ingredient);
            }
        } else {
            props.ingredients.write().push(new_ingredient);
        }
        
        ingredients = use_signal(|| vec![Ingredient::default()]);
        is_open.set(false);
    };

    let herkunft_path = format!("herkunft_benoetigt_{}", index);

    // Check for validation errors for this ingredient
    let validations_context = use_context::<Validations>();
    let has_validation_error = use_memo(move || {
        let validation_entries = (*validations_context.0.read()).clone();
        validation_entries.contains_key(&format!("ingredients[{}][origin]", index)) ||
        validation_entries.contains_key(&format!("ingredients[{}][amount]", index))
    });

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
                class: if *has_validation_error.read() {
                    "btn join-item btn-outline btn-error relative"
                } else {
                    "btn join-item btn-outline"
                },
                onclick: move |_| is_open.toggle(),
                onkeydown: move |evt: KeyboardEvent| if evt.key() == Key::Escape { is_open.set(false); },
                icons::ListDetail {}
                if *has_validation_error.read() {
                    span {
                        class: "absolute -top-2 -right-2 bg-error text-error-content rounded-full w-4 h-4 text-xs flex items-center justify-center",
                        "!"
                    }
                }
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
                        class: "input input-accent w-full",
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
                            class: "input input-accent w-full",
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
                                    let factor: f64 = (*amount_to_edit)()
                                        / old_ingredient.amount;
                                    let clonedIngredients = ingredients;
                                    for (key, elem) in clonedIngredients.iter().enumerate() {
                                        let old_ingredient = elem.clone();
                                        ingredients.write()[key] = Ingredient {
                                            amount: (elem.amount * factor),
                                            ..old_ingredient.clone()
                                        }
                                    }
                                } else {
                                    ingredients.write()[index] = Ingredient {
                                        amount: (*amount_to_edit)(),
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
                    path: "namensgebende_zutat".to_string(),
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
                ConditionalDisplay {
                    path: herkunft_path,
                    FormField {
                        label: "Herkunft",
                        ValidationDisplay {
                            paths: vec![
                                format!("ingredients[{}][origin]", index)
                            ],
                            select {
                                class: "select select-bordered w-full",
                                value: match selected_origin.read().as_ref() {
                                    Some(Country::CH) => "CH",
                                    Some(Country::EU) => "EU",
                                    None => "",
                                },
                                onchange: move |e| {
                                    let country = match e.value().as_str() {
                                        "EU" => Country::EU,
                                        "CH" => Country::CH,
                                        _ => return, // Don't update for empty selection
                                    };
                                    selected_origin.set(Some(country.clone()));
                                    ingredients.write()[index] = Ingredient {
                                        origin: Some(country),
                                        ..old_ingredient_3.clone()
                                    }
                                },
                                option { value: "", selected: selected_origin.read().is_none(), "Bitte wählen..." }
                                option { value: "CH", selected: matches!(selected_origin.read().as_ref(), Some(Country::CH)), "Schweiz" }
                                option { value: "EU", selected: matches!(selected_origin.read().as_ref(), Some(Country::EU)), "EU" }
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
                                if evt.key() == Key::Escape {
                                    is_open.set(false);
                                }
                            },
                            "× " {t!("nav.schliessen")},
                        }
                        if props.genesis {
                            button {
                                class: "btn",
                                onclick: move |_| handle_genesis(),
                                {t!("nav.speichern")},
                            }
                        }
                    }
                }
            }
        }
    }
}
