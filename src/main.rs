#![allow(non_snake_case)]

use crate::components::*;
use crate::core::{Calculator, Ingredient, Input, Output};
use crate::layout::ThemeLayout;
use crate::rules::RuleDef::{AP1_2_ProzentOutputNamensgebend, AP1_3_EingabeNamensgebendeZutat};
use crate::rules::RuleDef;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use serde_qs::from_str as from_query_string;
use serde_qs::to_string as to_query_string;
use std::collections::HashMap;
use strum_macros::EnumIter;
use web_sys::window;
use rust_i18n::{i18n, t};

mod layout;

mod model;
mod components;
mod core;
mod rules;
mod nl2br;

i18n!();

#[derive(Serialize, Deserialize, PartialEq, Clone)]
struct Form {
    #[serde(default)]
    ingredients: Vec<Ingredient>,
    #[serde(default)]
    product_title: String,
    #[serde(default)]
    product_subtitle: String,
    #[serde(default)]
    additional_info: String,
    #[serde(default)]
    storage_info: String,
    #[serde(default)]
    date_prefix: String,
    #[serde(default)]
    date: String,
    #[serde(default)]
    production_country: String,
    #[serde(default)]
    net_weight: String,
    #[serde(default)]
    drained_weight: String,
    #[serde(default)]
    producer_name: String,
    #[serde(default)]
    producer_address: String,
    #[serde(default)]
    producer_zip: String,
    #[serde(default)]
    producer_city: String,
    #[serde(default)]
    price_per_100: String,
    #[serde(default)]
    total_price: String,
    #[serde(default)]
    manual_total: Option<f64>
}

impl Into<Input> for Form {
    fn into(self) -> Input {
        Input {
            ingredients: self.ingredients,
            total: self.manual_total,
            ..Default::default()
        }
    }
}

impl Default for Form {
    fn default() -> Self {
        if let Some(window) = web_sys::window() {
            if let Ok(mut query_string) = window.location().search() {
                query_string = query_string.trim_start_matches('?').to_string();
                if let Ok(app_state_from_query_string) = from_query_string::<Form>(
                    &query_string
                ) {
                    return app_state_from_query_string;
                }
            }
        }
        Form {
            ingredients: Vec::new(),
            product_title: String::new(),
            product_subtitle: String::new(),
            additional_info: String::new(),
            storage_info: String::new(),
            date_prefix: String::new(),
            date: String::new(),
            production_country: String::from("Schweiz"),  // Default to "Schweiz"
            net_weight: String::new(),
            drained_weight: String::new(),
            producer_name: String::new(),
            producer_address: String::new(),
            producer_zip: String::new(),
            producer_city: String::new(),
            price_per_100: String::new(),
            total_price: String::new(),
            manual_total: None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Validations(Memo<HashMap<String, &'static str>>);

#[derive(Clone, Copy)]
pub struct Conditionals(Memo<HashMap<String, bool>>);

#[derive(Clone, Copy, EnumIter)]
pub enum Configuration {
    Conventional
}
fn main() {
    rust_i18n::set_locale("de-CH");
    launch(app);
}

fn app() -> Element {
    let initial_form = use_memo( Form::default );
    let ingredients: Signal<Vec<Ingredient>> = use_signal(|| initial_form.read().ingredients.clone());
    let product_title = use_signal(|| initial_form.read().product_title.clone());
    let product_subtitle = use_signal(|| initial_form.read().product_subtitle.clone());
    let additional_info = use_signal(|| initial_form.read().additional_info.clone());
    let storage_info = use_signal(|| initial_form.read().storage_info.clone());
    let date_prefix = use_signal(|| initial_form.read().date_prefix.clone());
    let date = use_signal(|| initial_form.read().date.clone());
    let production_country = use_signal(|| initial_form.read().production_country.clone());
    let net_weight = use_signal(|| initial_form.read().net_weight.clone());
    let drained_weight = use_signal(|| initial_form.read().drained_weight.clone());
    let producer_name = use_signal(|| initial_form.read().producer_name.clone());
    let producer_address = use_signal(|| initial_form.read().producer_address.clone());
    let producer_zip = use_signal(|| initial_form.read().producer_zip.clone());
    let producer_city = use_signal(|| initial_form.read().producer_city.clone());
    let price_per_100 = use_signal(|| initial_form.read().price_per_100.clone());
    let total_price = use_signal(|| initial_form.read().total_price.clone());
    let manual_total: Signal<Option<f64>> = use_signal(|| None);

    let configuration= use_signal(|| Configuration::Conventional);

    let current_state = use_memo(move || {
        Form {
            ingredients: ingredients(),
            product_title: product_title(),
            product_subtitle: product_subtitle(),
            additional_info: additional_info(),
            storage_info: storage_info(),
            date_prefix: date_prefix(),
            date: date(),
            production_country: production_country(),
            net_weight: net_weight(),
            drained_weight: drained_weight(),
            producer_name: producer_name(),
            producer_address: producer_address(),
            producer_zip: producer_zip(),
            producer_city: producer_city(),
            price_per_100: price_per_100(),
            total_price: total_price(),
            manual_total: manual_total(),
        }
    });

    let query_string = use_memo(move || {
        format!{"?{}",to_query_string(&current_state()).unwrap()}
    });

    let rules:  Memo<Vec<RuleDef>> = use_memo(move || {
        match configuration() {
            Configuration::Conventional =>
                vec![
                    RuleDef::AP1_1_ZutatMengeValidierung,
                    AP1_2_ProzentOutputNamensgebend,
                    AP1_3_EingabeNamensgebendeZutat,
                    RuleDef::AP1_4_ManuelleEingabeTotal,
                    RuleDef::AP2_1_ZusammegesetztOutput
                ]
        }
    });
    //let rules: Signal<Vec<RuleDef>> = use_signal(|| vec![]);
    let calc_output: Memo<Output> = use_memo(move || {
        let mut calc = Calculator::new();
        calc.rule_defs = rules();
        let form: Form = current_state.read().clone();
        calc.execute(form.into())
    });
    let label: Memo<String> = use_memo(move || {
        calc_output.read().label.clone()
    });
    let validation_messages = use_memo(move || {
        calc_output.read().validation_messages.clone()
    });
    let conditional_display = use_memo(move || {
        calc_output.read().conditional_elements.clone()
    });

    use_context_provider(|| Validations(validation_messages));
    use_context_provider(|| Conditionals(conditional_display));

    let mut config_modal_open = use_signal(|| false);

    rsx! {
        document::Stylesheet {
            href: asset!("assets/tailwind.css")
        }
        ThemeLayout {
            div {
                class: "h-screen flex",
                div {
                    class: "flex-1 flex overflow-hidden",
                    div {
                        class: "flex-1 overflow-y-scroll",
                        div { class: "flex flex-col gap-6 p-8 pb-12",
                            h1 { class: "text-4xl text-accent mb-4", "Creator | Lebensmittelkennzeichnung" }
                            FormField {
                                label: {t!("label.produktname")},
                                help: Some((t!("help.produktname")).into()),
                                TextInput {
                                    placeholder: "Produktname (optional)",
                                    bound_value: product_title
                                }
                            }
                            FormField {
                                label: t!("label.sachbezeichnung"),
                                help: Some((t!("help.sachbezeichnung")).into()),
                                TextInput {
                                    placeholder: "Produktname / Produktbeschrieb - z.B. Haferriegel mit Honig",
                                    bound_value: product_subtitle
                                }
                            }
                            SeparatorLine {}
                            FormField {
                                label: "Zutaten",
                                help: Some((t!("help.zutaten")).into()),
                                IngredientsTable {
                                    ingredients: ingredients,
                                    validation_messages: validation_messages,
                                    manual_total: manual_total
                                }
                            }
                            SeparatorLine {}
                            FieldGroup2 {
                                FormField {
                                    label: "Datumseingabe",
                                    help: Some((t!("help.datumseingabe")).into()),
                                    DateInput {
                                        date_value: date,
                                        date_prefix: date_prefix
                                    }

                                }
                                FormField { label: "Zusatzinformationen",
                                    help: Some((t!("help.zusatzinformationen")).into()),
                                    TextareaInput {
                                        placeholder: "Haftungsausschlüsse, Kann Spuren von Nüssen enthalten, Gebrauchsanleitung",
                                        rows: "4",
                                        bound_value: additional_info
                                    }
                                }
                            }
                            FieldGroup2 {
                                FormField {
                                    label: "Aufbewahrungshinweis",
                                    help: Some((t!("help.aufbewahrungshinweis")).into()),
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
                                FormField {
                                    label: "Nettogewicht",
                                    help: Some((t!("help.nettogewicht")).into()),
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
                        }
                    }
                }
            }
            LabelPreview {
                label: label,
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
            div {class: "fixed bottom-2 right-2 flex gap-2",
                span {"Version 0.2.8 vom 06.02.2025"}
                a {class: "link link-blue", href: "https://github.com/bar9/open-farming-hackdays-label-creator/wiki/Release-notes", "Release Notes"}
            }
            div {class: "fixed top-4 right-4 flex gap-2",
                button {class: "btn btn-primary",
                    onclick: move |_: MouseEvent| {
                        let window = window().expect("no global `window` exists");
                        let navigator = window.navigator();
                        let clipboard = navigator.clipboard();
                        let href = window.location().href().unwrap();
                        let text = format!("{href}{query_string}");
                        let  _ = clipboard.write_text(&text);
                    },
                    "📋 Link kopieren"
                }

                button {class: "btn btn-primary",
                    onclick: move |_| config_modal_open.toggle(),
                    "🥬|🍓 Konfiguration"
                }

                if config_modal_open() {
                    div { class: "fixed inset-0 bg-black bg-opacity-50 backdrop-blur-md" }
                    dialog {
                        open: "{config_modal_open}", class: "modal",
                        div { class: "modal-box bg-base-100 backdrop-blur-3xl",
                            h3 { class: "font-bold text-lg", "Konfiguration" }
                            div {
                                class: "prose",
                                label {"Aktive Regeln:"}
                                ul {
                                    {rules().into_iter().map(|rule| {
                                        rsx! {li {"{rule:?}"}}
                                    })}
                                }
                            }
                            div { class: "modal-action",
                                form { method: "dialog",
                                    button {
                                        class: "btn btn-sm",
                                        onclick: move |_| config_modal_open.toggle(),
                                        "× Schliessen"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
