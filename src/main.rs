#![allow(non_snake_case)]

use std::collections::BTreeMap;
use dioxus::prelude::*;
use crate::components::{AddNewIngredientButton, FieldGroup1, FieldGroup2, FormField, LabelPreview, SeparatorLine, TextareaInput, TextInput, TextInputDummy};
use crate::layout::ThemeLayout;
use crate::model::{food_db, IngredientItem, sorted_ingredient_list};

mod layout;

mod model;
mod components;
const _STYLE: &str = manganis::mg!(file("public/tailwind.css"));
fn main() {
    launch(app);
}

fn app() -> Element {
    let ingredients = use_signal(|| BTreeMap::<usize, IngredientItem>::new());
    let adding = use_signal(|| false);
    let name_to_add = use_signal(|| String::new());
    let mut last_id = use_signal(|| 0_usize);
    let product_title = use_signal(|| String::new());
    let additional_info = use_signal(|| String::new());
    let storage_info = use_signal(|| String::new());

    rsx! {
        ThemeLayout {
            div { class: "flex flex-col gap-6 bg-neutral p-8 pb-12 h-full",
                h1 { class: "text-4xl text-accent mb-4", "LMK Creator | Lebensmittelkennzeichnung" }
                FormField { label: "Sachbezeichnung",
                    TextInput {
                        placeholder: "Produktname / Produktbeschrieb - z.B. Haferriegel mit Honig",
                        bound_value: product_title
                    }
                }
                FieldGroup2 {
                    FormField { label: "Datumseingabe",
                        input {class: "input input-bordered w-full", r#type: "date", value: "2024-03-23"}
                    }
                    FormField { label: "Zusatzinformationen",
                        TextareaInput {
                            placeholder: "Haftungsausschlüsse, Kann Spuren von Nüssen enthalten, Gebrauchsanleitung",
                            rows: "4",
                            bound_value: additional_info
                        }
                    }
                }
                FieldGroup2 {
                    FormField { label: "Aufbewahrung + Lagerung",
                        TextareaInput{
                            rows: "2",
                            placeholder: "z.B. dunkel und kühl bei max. 5°C lagern",
                            bound_value: storage_info
                        }
                    }
                    FormField { label: "Produktionsland",
                        textarea {class: "textarea textarea-bordered w-full", rows: "2",
                            placeholder: "Schweiz"
                        }
                    }
                }
                FieldGroup2 {
                    FormField { label: "Nettogewicht",
                        input {class: "input input-bordered w-full", r#type: "text", placeholder: "300g", value: ""}
                    }
                    FormField { label: "Abtropfgewicht",
                        input {class: "input input-bordered w-full", r#type: "text", placeholder: "125g", value: ""}
                    }
                }
                SeparatorLine {}
                FieldGroup1 { label: "Adresse",
                    FormField {label: "Vorname / Name / Firma", TextInputDummy { placeholder: "Hans Muster AG" }}
                    div { class: "grid grid-cols-3 gap-4",
                        FormField {label: "Adresse", TextInputDummy { placeholder: "Teststrasse 1" }}
                        FormField {label: "PLZ", TextInputDummy { placeholder: "CH-4001" }}
                        FormField {label: "Ort", TextInputDummy { placeholder: "Basel" }}
                    }
                }
                SeparatorLine {}
                FieldGroup1 { label: "Preis",
                    div { class: "grid grid-cols-2 gap-4",
                        FormField { label: "Preis pro 100g", TextInputDummy { placeholder: "4.00 CHF"}}
                        FormField { label: "Preis Total", TextInputDummy { placeholder: "12.00 CHF"}}
                    }
                }
            },
            LabelPreview{
                ingredients: ingredients,
                product_title: product_title,
                additional_info: additional_info,
                storage_info: storage_info
            }
        }
    }

    // {rsx! {
    //     ThemeLayout{
    //             h2 { class: "pb-4",
    //                 "Zutaten"
    //                 if ingredients.len() > 0 {
    //                     {rsx! {
    //                         table { class: "table border-solid",
    //                             tr {
    //                                 th {
    //                                     "Zutat"
    //                                 }
    //                                 th {
    //                                     "Menge"
    //                                 }
    //                             }
    //                             for ingredient in ingredients {
    //                                 {
    //                                     let (key, ingr) = ingredient.clone();
    //                                     let ingr  = ingr.clone();
    //                                     { rsx! {
    //                                         tr { key: "{key}",
    //                                             td {
    //                                                 {ingr.basicInfo.standard_ingredient.name}
    //                                             }
    //                                             td {
    //                                                 input {
    //                                                         r#type: "number",
    //                                                         placeholder: "",
    //                                                         class: "input input-bordered input-accent",
    //                                                         oninput: move |evt| {
    //                                                             let mut new_amount_ingredient = ingredient.1.clone();
    //                                                             if let Ok(new_amount) = evt.data.value.clone().parse::<i32>() {
    //                                                                 new_amount_ingredient.basicInfo.amount = new_amount;
    //                                                                 ingredients.write().insert(key, new_amount_ingredient).unwrap();
    //                                                             }
    //                                                         }
    //                                                 }
    //                                                 " g"
    //                                             }
    //                                             td {
    //                                                 button {
    //                                                     class: "btn btn-square",
    //                                                     dangerous_inner_html: r###"<svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>"###,
    //                                                     onclick: move |_| {
    //                                                         ingredients.write().remove(&key);
    //                                                     }
    //                                                 }
    //                                             }
    //                                         }
    //                                     }}
    //                                 }
    //
    //                             }
    //                         }
    //                     }}
    //                 }
    //                 div {
    //                     if *adding.get() == true {
    //                         {rsx! {
    //                             div { class: "flex",
    //                             input {
    //                                     list: "ingredients",
    //                                     r#type: "flex",
    //                                     placeholder: "Name",
    //                                     class: "input input-bordered input-accent",
    //                                     oninput: move |evt| name_to_add.set(evt.value.clone()),
    //                                     datalist {
    //                                         id: "ingredients",
    //                                         for item in food_db().clone() {
    //                                             option { value: "{item.0}" }
    //                                         }
    //                                     }
    //                             }
    //                             button { class: "btn btn-outline",
    //                                 onclick: move |evt|  {
    //                                     ingredients.write().insert(
    //                                         last_id + 1,
    //                                         IngredientItem::from_name(String::from(name_to_add.get()))
    //                                     );
    //                                     last_id += 1;
    //                                     adding.set(false);
    //                                 },
    //                                 "Hinzufügen"
    //                             }
    //                             }
    //                         }}
    //                     } else {
    //                         {rsx! { AddNewIngredientButton{ on_click: move |evt| adding.set(true) } }}
    //                     }
    //                 }
    //             }
    //
    // }
}
