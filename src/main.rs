#![allow(non_snake_case)]

use std::collections::BTreeMap;
use dioxus::prelude::*;
use crate::components::{AddNewIngredientButton, FieldGroup1, FieldGroup2, FormField, IngredientsTable, LabelPreview, SeparatorLine, TextareaInput, TextInput, TextInputDummy};
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
            div { class: "flex flex-col gap-6 p-8 pb-12 h-full",
                h1 { class: "text-4xl text-accent mb-4", "LMK Creator | Lebensmittelkennzeichnung" }
                FormField { label: "Sachbezeichnung",
                    TextInput {
                        placeholder: "Produktname / Produktbeschrieb - z.B. Haferriegel mit Honig",
                        bound_value: product_title
                    }
                }
                SeparatorLine {}
                IngredientsTable { label: "Zutaten", ingredients: ingredients}
                SeparatorLine {}
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
}
