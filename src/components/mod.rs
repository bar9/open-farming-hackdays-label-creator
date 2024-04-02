#![allow(non_snake_case)]

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use dioxus::html::textarea;
use dioxus::prelude::*;
use crate::model::{sorted_ingredient_list, IngredientItem, AdditionalInfo, food_db};

pub fn SeparatorLine() -> Element {
    rsx! {
        hr { class: "border-1 border-dashed border-neutral-400 my-2" }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TextInputProps {
    #[props(into)]
    placeholder: String,
    bound_value: Signal<String>
}
pub fn TextInput(mut props: TextInputProps) -> Element {
    rsx! {
        input {
            class: "input input-bordered w-full",
            r#type: "text",
            placeholder: "{props.placeholder}",
            value: "{props.bound_value}",
            oninput: move |evt| props.bound_value.set(evt.data.value())
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TextareaInputProps {
    #[props(into)]
    placeholder: String,
    bound_value: Signal<String>,
    #[props(into)]
    rows: String
}
pub fn TextareaInput(mut props: TextareaInputProps) -> Element {
    rsx! {
        textarea {
            class: "textarea textarea-bordered w-full",
            rows: "{props.rows}",
            placeholder: "{props.placeholder}",
            value: "{props.bound_value}",
            oninput: move |evt| props.bound_value.set(evt.data.value())
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TextInputDummyProps {
    #[props(into)]
    placeholder: String
}
pub fn TextInputDummy(props: TextInputDummyProps) -> Element {
    rsx! {
        input {
            class: "input input-bordered w-full",
            r#type: "text",
            placeholder: "{props.placeholder}",
        }
    }
}
#[derive(Props, Clone, PartialEq)]
pub struct FormFieldProps {
    #[props(into)]
    label: String,
    children: Element
}
pub fn FormField(props: FormFieldProps) -> Element {
    rsx! {
        div {
            class: "flex gap-2 flex-col",
            label { "{props.label}" }
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct FieldGroup2Props {
    children: Element
}
pub fn FieldGroup2(props: FieldGroup2Props) -> Element {
    rsx! {
        div {
            class: "grid grid-cols-2 gap-4 ",
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct FieldGroup1Props {
    #[props(into)]
    label: String,
    children: Element
}
pub fn FieldGroup1(props: FieldGroup1Props) -> Element {
    rsx! {
        div { class: "flex flex-col gap-4",
            h4 { class: "text-xl mb-2", "{props.label}" }
            {props.children}
        }
    }
}

#[component]
pub fn AddNewIngredientButton(on_click: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button { class: "btn btn-outline",
            onclick: move |evt| on_click.call(evt),
            "Eine Zutat hinzufügen",
        },
    }
}

#[component]
pub fn LabelPreview(
    ingredients: Signal<BTreeMap<usize, IngredientItem>>,
    product_title: Signal<String>,
    additional_info: Signal<String>,
    storage_info: Signal<String>,
) -> Element {
    rsx! {
        div { class: "p-8 flex flex-col bg-base-200",
            h2 { class: "pb-4 text-4xl",
                "Etiketten Vorschau"
            }
            div { class: "bg-white border p-4 grid grid-col-1 gap-4",
                h3 { class: "text-2xl mb-2", "{product_title}" }
                h4 { class: "text-xl mb-2", "Zutaten" }
                span {
                    dangerous_inner_html: "{sorted_ingredient_list(ingredients.read().clone())}"
                }
                if additional_info.to_string() != "" {
                    {
                        rsx! {
                            h4 { class: "text-xl mb-w",
                                "Zusatzinformationen"
                            }
                            span {
                                "{additional_info}"
                            }
                        }
                    }
                }
                if storage_info.to_string() != "" {
                    {
                        rsx! {
                            h4 { class: "text-xl mb-w",
                                "Aufbewahrung + Lagerung"
                            }
                            span {
                                "{storage_info}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct IngredientsTableProps {
    #[props(into)]
    label: String,
    ingredients: Signal<BTreeMap<usize, IngredientItem>>
}
pub fn IngredientsTable(mut props: IngredientsTableProps) -> Element {
    let mut ingredients_lock = props.ingredients.read();
    // let mut delete_callback = |key| ingredients_lock.remove(key);
    let mut name_to_add = use_signal(|| String::new());
    let mut amount_to_add = use_signal(|| 0);
    let mut last_id = use_signal(|| 0);
    rsx! {
        div { class: "flex flex-col gap-4",
            h4 { class: "text-xl mb-2", "{props.label}" }
            table { class: "table border-solid",
                tr { th { "Zutat" } th { "Menge" } }
                {ingredients_lock.iter().map(|(key, ingr)| {
                    rsx! {
                        tr { key: "{key}",
                            td {
                                "{ingr.basicInfo.standard_ingredient.name}"
                            }
                            td {
                                "{ingr.basicInfo.amount} g"
                            }
                            // td {
                            //     button {
                            //         class: "btn btn-square",
                            //         dangerous_inner_html: r###"<svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>"###,
                            //         onclick: move |_| {
                            //             // delete_callback(key);
                            //         }
                            //     }
                            // }
                        }
                    }
                })}
            }
        }
        div { class: "flex flex-row gap-4",
            input {
                list: "ingredients",
                r#type: "flex",
                placeholder: "Name",
                class: "input input-bordered input-accent w-full",
                oninput: move |evt| name_to_add.set(evt.data.value()),
                value: "{name_to_add}",
                datalist {
                    id: "ingredients",
                    for item in food_db().clone() {
                        option { value: "{item.0}" }
                    }
                }
            }
            input {
                r#type: "number",
                placeholder: "Menge",
                class: "input input-bordered input-accent w-full",
                oninput: move |evt| {
                    if let Ok(amount) = evt.data.value().parse::<i32>() {
                        amount_to_add.set(amount);
                    }
                },
                value: "{amount_to_add}"
            }
            "g"
            button { class: "btn btn-accent",
                onclick: move |evt|  {
                    props.ingredients.write().insert(
                        last_id + 1,
                        IngredientItem::from_name_amount((&*name_to_add)(), (&*amount_to_add)())
                    );
                    last_id += 1;
                    name_to_add.set(String::new());
                    amount_to_add.set(0);
                },
                "Hinzufügen"
            }
        }
    }
}
                    // for (key, ingr) in ingredients_lock.iter() {
                    //     {
                    //         { rsx! {
                    //             tr { key: "{key}",
                    //                 td {
                                        // input {
                                        //         r#type: "number",
                                        //         placeholder: "",
                                        //         class: "input input-bordered input-accent",
                                        //         oninput: move |evt| {
                                        //             let mut new_amount_ingredient = ingredient.1.clone();
                                        //             if let Ok(new_amount) = evt.data.value.clone().parse::<i32>() {
                                        //                 new_amount_ingredient.basicInfo.amount = new_amount;
                                        //                 ingredients.write().insert(key, new_amount_ingredient).unwrap();
                                        //             }
                                        //         }
                                        // }
                                        // " g"
    //                                 }
    //
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }


    //                 if ingredients.len() > 0 {
    //                     {rsx! {
    //                     }}
    //                 }
    //                 div {
    //                     if *adding.get() == true {
    //                         {rsx! {
    //
    //                             }
    //                         }}
    //                     } else {
    //                         {rsx! { AddNewIngredientButton{ on_click: move |evt| adding.set(true) } }}
    //                     }
    //                 }