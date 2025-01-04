use dioxus::prelude::*;
use crate::core::{Ingredient, SubIngredient};
use crate::model::{food_db};

#[derive(Props, Clone, PartialEq)]
pub struct SubIngredientsTableProps {
    ingredients: Signal<Vec<Ingredient>>,
    index: usize,
}
pub fn SubIngredientsTable(props: SubIngredientsTableProps) -> Element {
    let mut name_to_add = use_signal(|| String::new());

    let mut delete_callback = {
        let mut ingredients = props.ingredients.clone();
        move |index: usize, sub_index: usize| {
            if let Some(mut ingredient) = ingredients.get_mut(index) {
                if let Some(sub_components) = &mut ingredient.sub_components {
                    sub_components.remove(sub_index);
                }
            }
        }
    };

    let add_callback = {
        let mut ingredients = props.ingredients.clone();
        let mut name_to_add = name_to_add.clone();
        move |_evt| {
            if let Some(mut ingredient) = ingredients.get_mut(props.index) {
                if let Some(sub_components) = &mut ingredient.sub_components {
                    sub_components.push(SubIngredient {
                        name: name_to_add(),
                        is_allergen: false,
                    });
                } else {
                    let mut sub_components = Vec::new();
                    sub_components.push(SubIngredient {
                        name: name_to_add(),
                        is_allergen: false,
                    });
                    ingredient.sub_components = Some(sub_components);
                }
            }
            name_to_add.set(String::new());
        }
    };


    rsx! {
        div { class: "flex flex-col gap-4",
            table { class: "table border-solid",
                tr {
                    th { "Zutat (eingeben oder auswählen)" }
                }
                if let Some(sub_components) = props.ingredients.clone().get(props.index).and_then(|ingredient| ingredient.sub_components.clone()) {
                    for (key, ingr) in sub_components.iter().enumerate() {
                        tr { key: "{key}",
                            td { "{ingr.name}" }
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
            button {
                class: "btn btn-accent",
                onclick: add_callback,
                "Hinzufügen"
            }
        }
    }
}
