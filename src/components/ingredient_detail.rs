use crate::components::*;
use crate::core::{Ingredient, SubIngredient};
use crate::model::{food_db, lookup_allergen};
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
    // let mut scale_together = use_signal(|| false); // Recipe calculator - commented out for now
    
    // Local state for editing - won't be saved until user clicks save
    let original_ingredient = ingredients.get(index).unwrap().clone();
    let mut edit_name = use_signal(|| original_ingredient.name.clone());
    let mut edit_amount = use_signal(|| original_ingredient.amount);
    let mut edit_is_composite = use_signal(|| {
        !original_ingredient
            .sub_components
            .clone()
            .unwrap_or_default()
            .is_empty()
    });
    let mut edit_is_namensgebend = use_signal(|| {
        original_ingredient
            .is_namensgebend
            .unwrap_or(false)
    });
    let mut edit_sub_components = use_signal(|| original_ingredient.sub_components.clone());
    
    // Check if the current name is in the food database
    let mut is_custom_ingredient = use_signal(|| {
        !food_db().iter().any(|(name, _)| name == &edit_name())
    });
    
    // Track allergen status separately for custom ingredients
    let mut is_allergen_custom = use_signal(|| original_ingredient.is_allergen);
    
    // Create a wrapper ingredients signal for SubIngredientsTable
    // Initialize it once and keep it stable
    let mut wrapper_ingredients = use_signal(|| {
        vec![Ingredient {
            name: original_ingredient.name.clone(),
            amount: original_ingredient.amount,
            is_allergen: original_ingredient.is_allergen,
            is_namensgebend: original_ingredient.is_namensgebend,
            sub_components: original_ingredient.sub_components.clone(),
        }]
    });
    
    // When composite mode changes, sync the wrapper
    use_effect(move || {
        let _ = edit_is_composite(); // Track this dependency
        if edit_is_composite() {
            // Initialize wrapper with current edit state
            wrapper_ingredients.write()[0] = Ingredient {
                name: edit_name(),
                amount: edit_amount(),
                is_allergen: is_allergen_custom(),
                is_namensgebend: Some(edit_is_namensgebend()),
                sub_components: edit_sub_components(),
            };
        }
    });
    
    // Track changes from SubIngredientsTable back to edit state
    // Only monitor wrapper_ingredients changes
    use_effect(move || {
        if let Some(wrapper_sub) = wrapper_ingredients.read().get(0).and_then(|i| i.sub_components.as_ref()) {
            // Only update if actually different
            let current_edit_sub = edit_sub_components();
            if current_edit_sub.as_ref() != Some(wrapper_sub) {
                edit_sub_components.set(Some(wrapper_sub.clone()));
            }
        }
    });
    
    let mut update_name = move |new_name: String| {
        // Update local edit state only
        edit_name.set(new_name.clone());
        
        // Check if the new name is in the food database
        let in_database = food_db().iter().any(|(name, _)| name == &new_name);
        is_custom_ingredient.set(!in_database);
        
        // If switching from database to custom or vice versa, update allergen status
        if in_database {
            is_allergen_custom.set(lookup_allergen(&new_name));
        }
    };

    let mut handle_save = move || {
        // Determine allergen status based on database presence
        let in_database = food_db().iter().any(|(name, _)| name == &edit_name());
        let allergen_status = if in_database {
            lookup_allergen(&edit_name())
        } else {
            is_allergen_custom()
        };
        
        let new_ingredient = Ingredient {
            name: edit_name(),
            amount: edit_amount(),
            is_allergen: allergen_status,
            is_namensgebend: Some(edit_is_namensgebend()),
            sub_components: edit_sub_components(),
        };
        
        if props.genesis {
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
            
            // Reset local state for next creation
            edit_name.set(String::new());
            edit_amount.set(0.0);
            edit_is_composite.set(false);
            edit_is_namensgebend.set(false);
            edit_sub_components.set(None);
            is_allergen_custom.set(false);
        } else {
            // Update existing ingredient
            props.ingredients.write()[index] = new_ingredient;
        }
        
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
                        value: "{edit_name}",
                        datalist { id: "ingredients",
                            for item in food_db().clone() {
                                option { 
                                    value: "{item.0}",
                                    // Show allergen marker in label without duplicating the name
                                    label: if item.1 { "(Allergen)" } else { "" }
                                }
                            }
                        }
                    }
                }
                FormField {
                    label: format!("{} (g)", t!("label.menge")),
                    ValidationDisplay {
                        paths: vec![
                            format!("ingredients[{}][amount]", index)
                        ],
                        input {
                            r#type: "number",
                            placeholder: "Menge in Gramm",
                            class: "input input-accent w-full",
                            onchange: move |evt| {
                                if let Ok(amount) = evt.data.value().parse::<f64>() {
                                    edit_amount.set(amount);
                                }
                            },
                            value: "{edit_amount}",
                        }
                    }
                    // Recipe calculator functionality - commented out for now
                    // if !props.genesis {
                    //     label { class: "label cursor-pointer",
                    //         input {
                    //             class: "checkbox",
                    //             r#type: "checkbox",
                    //             checked: "{scale_together}",
                    //             oninput: move |e| scale_together.set(e.value() == "true"),
                    //         }
                    //         span { class: "label-text",
                    //             "{t!(\"nav.verhaeltnisseBeibehalten\")}"
                    //         }
                    //     }
                    //     button {
                    //         class: "btn btn-accent",
                    //         onclick: move |_evt| {
                    //             // Recipe calculator logic
                    //         },
                    //         "{t!(\"nav.anpassen\")}"
                    //     }
                    // }
                }

                br {}
                br {}
                
                // Show allergen status - checkbox for custom ingredients, text for database allergens
                if !edit_name().is_empty() && !edit_is_composite() {
                    if is_custom_ingredient() {
                        // Custom ingredient - show checkbox
                        FormField {
                            help: Some((t!("help.allergenManual")).into()),
                            label: t!("label.allergen"),
                            label { class: "label cursor-pointer",
                                input {
                                    class: "checkbox",
                                    r#type: "checkbox",
                                    checked: "{is_allergen_custom}",
                                    oninput: move |e| {
                                        let is_checked = e.value() == "true";
                                        is_allergen_custom.set(is_checked);
                                        // Don't update immediately - wait for save
                                    },
                                }
                                span { class: "label-text",
                                    {t!("label.allergen")}
                                }
                            }
                        }
                        br {}
                    } else if is_allergen_custom() {
                        // Database allergen - show text only
                        FormField {
                            label: t!("label.allergen"),
                            div { class: "py-2",
                                span { class: "font-bold", "({t!(\"label.allergen\")})" }
                            }
                        }
                        br {}
                    }
                }

                FormField {
                    label: t!("label.zusammengesetzteZutat"),
                    help: Some((t!("help.zusammengesetzteZutaten")).into()),
                    label { class: "label cursor-pointer",
                        input {
                            class: "checkbox",
                            r#type: "checkbox",
                            checked: "{edit_is_composite}",
                            oninput: move |e| edit_is_composite.set(e.value() == "true"),
                        }
                        span { class: "label-text",
                            "{t!(\"label.zusammengesetzteZutat\")}"
                        }
                    }
                    if edit_is_composite() {
                        SubIngredientsTable {
                            ingredients: wrapper_ingredients,
                            index: 0
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
                                checked: "{edit_is_namensgebend}",
                                oninput: move |e| {
                                    edit_is_namensgebend.set(e.value() == "true");
                                    // Don't update immediately - wait for save
                                },
                            }
                            span { class: "label-text",
                                "{t!(\"label.namensgebendeZutat\")}"
                            }
                        }
                    }
                }
                div { class: "modal-action",
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
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| handle_save(),
                        {t!("nav.speichern")},
                    }
                }
            }
            form {
                method: "dialog",
                class: "modal-backdrop",
                onclick: move |_| is_open.set(false),
                button { "" }
            }
        }
    }
}
