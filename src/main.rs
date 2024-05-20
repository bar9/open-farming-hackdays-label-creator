#![allow(non_snake_case)]

use dioxus::prelude::*;
use crate::components::*;
use crate::layout::ThemeLayout;
use crate::model::{IngredientItem};

mod layout;

mod model;
mod components;
const _STYLE: &str = manganis::mg!(file("public/tailwind.css"));
fn main() {
    launch(app);
}

fn app() -> Element {
    let ingredients: Signal<Vec<IngredientItem>> = use_signal(|| Vec::new());
    let product_title = use_signal(|| String::new());
    let product_subtitle = use_signal(|| String::new());
    let additional_info = use_signal(|| String::new());
    let storage_info = use_signal(|| String::new());
    let date_prefix = use_signal(|| String::new());
    let date = use_signal(|| String::new());
    let production_country = use_signal(|| String::from("Schweiz"));
    let net_weight = use_signal(|| String::new());
    let drained_weight = use_signal(|| String::new());
    let producer_name = use_signal(|| String::new());
    let producer_address= use_signal(|| String::new());
    let producer_zip= use_signal(|| String::new());
    let producer_city= use_signal(|| String::new());
    let price_per_100 = use_signal(|| String::new());
    let total_price = use_signal(|| String::new());


    rsx! {
        ThemeLayout {
            div { class: "flex flex-col gap-6 p-8 pb-12 h-full",
                h1 { class: "text-4xl text-accent mb-4", "LMK Creator | Lebensmittelkennzeichnung" }
                FormField {
                    label: "Produktname",
                    help: rsx!{div{ "Produktname Hilfe"}},
                    TextInput {
                        placeholder: "Produktname (optional)",
                        bound_value: product_title
                    }
                }
                FormField { label: "Sachbezeichnung",
                    TextInput {
                        placeholder: "Produktname / Produktbeschrieb - z.B. Haferriegel mit Honig",
                        bound_value: product_subtitle
                    }
                }
                SeparatorLine {}
                FormField {
                    label: "Zutaten",
                    help: rsx!{p{"Wählen Sie hier Ihre Zutaten+++"}},
                    IngredientsTable {ingredients: ingredients}
                }
                SeparatorLine {}
                FieldGroup2 {
                    FormField { label: "Datumseingabe",
                        DateInput {
                            date_value: date,
                            date_prefix: date_prefix
                        }

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
                        TextareaInput {
                            rows: "2",
                            placeholder: "Schweiz",
                            bound_value: production_country
                        }
                    }
                }
                FieldGroup2 {
                    FormField { label: "Nettogewicht",
                        TextInput {
                            bound_value: net_weight,
                            placeholder: "300g"
                        }
                    }
                    FormField { label: "Abtropfgewicht",
                        TextInput {
                            bound_value: drained_weight,
                            placeholder: "125g"
                        }
                    }
                }
                SeparatorLine {}
                FieldGroup1 { label: "Adresse",
                    FormField {
                        label: "Vorname / Name / Firma",
                        TextInput { bound_value: producer_name, placeholder: "Hans Muster AG" }
                    }
                    div { class: "grid grid-cols-3 gap-4",
                        FormField {
                        label: "Adresse",
                            TextInput { bound_value: producer_address, placeholder: "Teststrasse 1" }
                        }
                        FormField {
                            label: "PLZ",
                            TextInput { bound_value: producer_zip, placeholder: "CH-4001" }
                        }
                        FormField {
                            label: "Ort",
                            TextInput { bound_value: producer_city, placeholder: "Basel" }
                        }
                    }
                }
                SeparatorLine {}
                FieldGroup1 { label: "Preis",
                    div { class: "grid grid-cols-2 gap-4",
                        FormField {
                            label: "Preis pro 100g",
                            TextInput {
                                placeholder: "4.00 CHF",
                                bound_value: price_per_100
                            }
                        }
                        FormField {
                            label: "Preis Total",
                            TextInput {
                                placeholder: "12.00 CHF",
                                bound_value: total_price
                            }
                        }
                    }
                }
            },
            LabelPreview {
                ingredients: ingredients,
                product_title : product_title,
                product_subtitle : product_subtitle,
                additional_info : additional_info,
                storage_info : storage_info,
                production_country : production_country,
                date_prefix : date_prefix,
                date : date,
                net_weight : net_weight,
                drained_weight : drained_weight,
                producer_name : producer_name,
                producer_address : producer_address,
                producer_zip : producer_zip,
                producer_city : producer_city,
                price_per_100: price_per_100,
                total_price: total_price
            }
        }
    }
}
