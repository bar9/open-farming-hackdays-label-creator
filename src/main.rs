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
pub mod core;
mod rules;
mod nl2br;
mod form;

i18n!();

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
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
    #[serde(default = "default_date_prefix")]
    date_prefix: String,
    #[serde(default)]
    date: String,
    #[serde(default)]
    production_country: String,
    #[serde(default)]
    producer_name: String,
    #[serde(default)]
    producer_address: String,
    #[serde(default)]
    producer_phone: String,
    #[serde(default)]
    producer_email: String,
    #[serde(default)]
    producer_website: String,
    #[serde(default)]
    producer_zip: String,
    #[serde(default)]
    producer_city: String,
    #[serde(default)]
    manual_total: Option<f64>,
    #[serde(default)]
    amount_type: AmountType,
    #[serde(default = "default_weight_unit")]
    weight_unit: String,
    #[serde(default = "default_volume_unit")]
    volume_unit: String,
    #[serde(default)]
    amount: Amount,
    #[serde(default)]
    price: Price,
}

fn default_weight_unit() -> String {
    "g".to_string()
}

fn default_volume_unit() -> String {
    "ml".to_string()
}

fn default_date_prefix() -> String {
    "Mindestens haltbar bis".to_string()
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
            date_prefix: String::from("Mindestens haltbar bis "),
            date: String::new(),
            production_country: String::from("Schweiz"),  // Default to "Schweiz"
            producer_name: String::new(),
            producer_address: String::new(),
            producer_email: String::new(),
            producer_website: String::new(),
            producer_phone: String::new(),
            producer_zip: String::new(),
            producer_city: String::new(),
            manual_total: None,
            amount_type: AmountType::Weight,
            weight_unit: "g".to_string(),
            volume_unit: "ml".to_string(),
            amount: Amount::Single(Some(0)),
            price: Price::Single(Some(0))
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
    let producer_name = use_signal(|| initial_form.read().producer_name.clone());
    let producer_address = use_signal(|| initial_form.read().producer_address.clone());
    let producer_email = use_signal(|| initial_form.read().producer_email.clone());
    let producer_website = use_signal(|| initial_form.read().producer_website.clone());
    let producer_phone = use_signal(|| initial_form.read().producer_phone.clone());
    let producer_zip = use_signal(|| initial_form.read().producer_zip.clone());
    let producer_city = use_signal(|| initial_form.read().producer_city.clone());
    let manual_total = use_signal(|| initial_form.read().manual_total.clone());
    let amount_type: Signal<AmountType> = use_signal(|| initial_form.read().amount_type.clone());
    let weight_unit: Signal<String> = use_signal(|| initial_form.read().weight_unit.clone());
    let volume_unit: Signal<String> = use_signal(|| initial_form.read().volume_unit.clone());
    let amount: Signal<Amount> = use_signal(|| initial_form.read().amount.clone());
    let price: Signal<Price> = use_signal(|| initial_form.read().price.clone());

    // let amount_type = use_signal(|| AmountType::Weight);
    // let volume = use_signal(|| String::new());

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
            producer_name: producer_name(),
            producer_address: producer_address(),
            producer_phone: producer_phone(),
            producer_website: producer_website(),
            producer_email: producer_email(),
            producer_zip: producer_zip(),
            producer_city: producer_city(),
            manual_total: manual_total(),
            amount_type: amount_type(),
            weight_unit: weight_unit(),
            volume_unit: volume_unit(),
            amount: amount(),
            price: price()
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

    // let amount_type: Signal<AmountType> = use_signal(|| AmountType::Weight);
    // let weight_unit: Signal<String> = use_signal(|| "g".to_string());
    // let volume_unit: Signal<String> = use_signal(|| "ml".to_string());
    // let amount: Signal<Amount> = use_signal(|| Amount::Single(None));
    // let price: Signal<Price> = use_signal(|| Price::Single(None));
    // let weight_type: Signal<Option<WeightType>> = use_signal(|| Some(WeightType::Gewicht));
    // let volume_type: Signal<Option<VolumeType>> = use_signal(|| Some(VolumeType::Volumen));

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
                            h1 { class: "text-4xl mb-4", "{t!(\"title\")}" }
                            FormField {
                                label: t!("label.produktname"),
                                help: Some(t!("help.produktname").into()),
                                TextInput {
                                    placeholder: t!("placeholder.produktname"),
                                    bound_value: product_title
                                }
                            }
                            FormField {
                                label: t!("label.sachbezeichnung"),
                                help: Some((t!("help.sachbezeichnung")).into()),
                                TextInput {
                                    placeholder: t!("placeholder.sachbezeichnung"),
                                    bound_value: product_subtitle
                                }
                            }
                            SeparatorLine {}
                            FormField {
                                label: t!("label.zutaten"),
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
                                    label: t!("label.datumseingabe"),
                                    help: Some((t!("help.datumseingabe")).into()),
                                    DateInput {
                                        date_value: date,
                                        date_prefix: date_prefix
                                    }

                                }
                                FormField { label: t!("label.zusatzinformationen"),
                                    help: Some((t!("help.zusatzinformationen")).into()),
                                    TextareaInput {
                                        placeholder: t!("placeholder.zusatzinformationen"),
                                        rows: "4",
                                        bound_value: additional_info
                                    }
                                }
                            }
                            FieldGroup2 {
                                FormField {
                                    label: t!("label.aufbewahrungshinweis"),
                                    help: Some((t!("help.aufbewahrungshinweis")).into()),
                                    TextareaInput{
                                        rows: "2",
                                        placeholder: t!("placeholder.aufbewahrungshinweis"),
                                        bound_value: storage_info
                                    }
                                }
                                // FormField { label: t!("label.produktionsland"),
                                //     TextareaInput {
                                //         rows: "2",
                                //         placeholder: t!("placeholder.produktionsland"),
                                //         bound_value: production_country
                                //     }
                                // }
                            }
                            SeparatorLine {}

                            // match amount_type() {
                            //     AmountType::Volume => rsx! {
                            //         "volume"
                            //     },
                            //
                            //     AmountType::Weight => rsx! {
                            //         "weight"
                            //     }
                            // }

                            FieldGroup1 {
                                label: t!("label.gewichtUndPreis"),
                                // FormField {
                                //     label: t!("label.mengenart"),
                                //     AmountTypeSelect {
                                //         amount_type: amount_type
                                //     }
                                // }
                                AmountPrice {
                                    amount_type: amount_type,
                                    weight_unit: weight_unit,
                                    volume_unit: volume_unit,
                                    amount: amount,
                                    price: price,
                                }
                            }
                            SeparatorLine {}
                            FieldGroup1 { label: t!("label.adresse"),
                                FormField {
                                    help: Some((t!("help.name")).into()),
                                    label: t!("label.name"),
                                    TextInput { bound_value: producer_name, placeholder: t!("placeholder.name") }
                                }
                                div { class: "grid grid-cols-3 gap-4",
                                    FormField {
                                        help: Some((t!("help.adresse")).into()),
                                        label: t!("label.adresse"),
                                        TextInput { bound_value: producer_address, placeholder: t!("placeholder.adresse")}
                                    }
                                    FormField {
                                        label: t!("label.plz"),
                                        TextInput { bound_value: producer_zip, placeholder: t!("placeholder.plz")}
                                    }
                                    FormField {
                                        help: Some((t!("help.ort")).into()),
                                        label: t!("label.ort"),
                                        TextInput { bound_value: producer_city, placeholder: t!("placeholder.ort")}
                                    }
                                    FormField {
                                        label: t!("label.telefon"),
                                        TextInput { bound_value: producer_phone, placeholder: t!("placeholder.telefon")}
                                    }
                                    FormField {
                                        label: t!("label.email"),
                                        TextInput { bound_value: producer_email, placeholder: t!("placeholder.email")}
                                    }
                                    FormField {
                                        label: t!("label.website"),
                                        TextInput { bound_value: producer_website, placeholder: t!("placeholder.website")}
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
                producer_name : producer_name,
                producer_address : producer_address,
                producer_zip : producer_zip,
                producer_city : producer_city,
                producer_email: producer_email,
                producer_phone: producer_phone,
                producer_website: producer_website,
                amount_type: amount_type,
                weight_unit: weight_unit,
                volume_unit: volume_unit,
                amount: amount,
                price: price
            }
            div {class: "fixed bottom-2 right-2 flex gap-2",
                span {"Version 0.3.3 vom 10.04.2025"}
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
                    icons::Clipboard{}
                    "{t!(\"nav.linkKopieren\")}"
                }

                button {class: "btn btn-primary",
                    onclick: move |_| config_modal_open.toggle(),
                    crate::icons::Settings {}
                    "{t!(\"nav.konfiguration\")}"
                }

                if config_modal_open() {
                    div { class: "fixed inset-0 bg-black bg-opacity-50 backdrop-blur-md" }
                    dialog {
                        open: "{config_modal_open}", class: "modal",
                        div { class: "modal-box bg-base-100 backdrop-blur-3xl",
                            h3 { class: "font-bold text-lg", "{t!(\"nav.konfiguration\")}" }
                            div {
                                class: "prose",
                                label { "{t!(\"nav.aktiveRegeln\")}" }
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
                                        "{t!(\"nav.schliessen\")}"
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

