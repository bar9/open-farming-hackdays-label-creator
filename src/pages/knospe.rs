use crate::components::*;
use crate::core::{Calculator, Ingredient, Input, Output};
use crate::layout::{CopyLinkContext, ThemeContext};
use crate::rules::RuleDef;
use crate::shared::{Conditionals, Configuration, Validations};
use dioxus::prelude::*;
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use serde_qs::from_str as from_query_string;
use serde_qs::to_string as to_query_string;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Form {
    #[serde(default)]
    pub ingredients: Vec<Ingredient>,
    #[serde(default)]
    pub ignore_ingredients: bool,
    #[serde(default)]
    pub product_title: String,
    #[serde(default)]
    pub product_subtitle: String,
    #[serde(default)]
    pub additional_info: String,
    #[serde(default)]
    pub storage_info: String,
    #[serde(default = "default_date_prefix")]
    pub date_prefix: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub production_country: String,
    #[serde(default)]
    pub producer_name: String,
    #[serde(default)]
    pub producer_address: String,
    #[serde(default)]
    pub producer_phone: String,
    #[serde(default)]
    pub producer_email: String,
    #[serde(default)]
    pub producer_website: String,
    #[serde(default)]
    pub producer_zip: String,
    #[serde(default)]
    pub producer_city: String,
    #[serde(default)]
    pub manual_total: Option<f64>,
    #[serde(default)]
    pub amount_type: AmountType,
    #[serde(default = "default_weight_unit")]
    pub weight_unit: String,
    #[serde(default = "default_volume_unit")]
    pub volume_unit: String,
    #[serde(default)]
    pub amount: Amount,
    #[serde(default)]
    pub price: Price,
}

fn default_weight_unit() -> String {
    t!("weight_units.g").to_string()
}

fn default_volume_unit() -> String {
    t!("volume_units.ml").to_string()
}

fn default_date_prefix() -> String {
    t!("date_prefixes.best_before").to_string()
}

impl From<Form> for Input {
    fn from(val: Form) -> Self {
        Input {
            ingredients: val.ingredients,
            total: val.manual_total,
            ..Default::default()
        }
    }
}

impl Default for Form {
    fn default() -> Self {
        if let Some(window) = web_sys::window() {
            if let Ok(mut query_string) = window.location().search() {
                query_string = query_string.trim_start_matches('?').to_string();
                if let Ok(app_state_from_query_string) = from_query_string::<Form>(&query_string) {
                    return app_state_from_query_string;
                }
            }
        }
        Form {
            ingredients: Vec::new(),
            ignore_ingredients: false,
            product_title: String::new(),
            product_subtitle: String::new(),
            additional_info: String::new(),
            storage_info: String::new(),
            date_prefix: format!("{} ", t!("date_prefixes.best_before")),
            date: String::new(),
            production_country: t!("countries.switzerland").to_string(),
            producer_name: String::new(),
            producer_address: String::new(),
            producer_email: String::new(),
            producer_website: String::new(),
            producer_phone: String::new(),
            producer_zip: String::new(),
            producer_city: String::new(),
            manual_total: None,
            amount_type: AmountType::Weight,
            weight_unit: t!("weight_units.g").to_string(),
            volume_unit: t!("volume_units.ml").to_string(),
            amount: Amount::Single(Some(0)),
            price: Price::Single(Some(0)),
        }
    }
}

pub fn Knospe() -> Element {
    let initial_form = use_memo(Form::default);
    let ignore_ingredients = use_signal(|| false);
    let ingredients: Signal<Vec<Ingredient>> =
        use_signal(|| initial_form.read().ingredients.clone());
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
    let manual_total = use_signal(|| initial_form.read().manual_total);
    let amount_type: Signal<AmountType> = use_signal(|| initial_form.read().amount_type.clone());
    let weight_unit: Signal<String> = use_signal(|| initial_form.read().weight_unit.clone());
    let volume_unit: Signal<String> = use_signal(|| initial_form.read().volume_unit.clone());
    let amount: Signal<Amount> = use_signal(|| initial_form.read().amount);
    let price: Signal<Price> = use_signal(|| initial_form.read().price);

    let configuration = use_signal(|| Configuration::Conventional);

    let current_state = use_memo(move || Form {
        ingredients: ingredients(),
        ignore_ingredients: ignore_ingredients(),
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
        price: price(),
    });

    let query_string = use_memo(move || {
        format! {"?{}",to_query_string(&current_state()).unwrap()}
    });

    let mut copy_link_context = use_context::<Signal<CopyLinkContext>>();
    let mut theme_context = use_context::<Signal<ThemeContext>>();

    use_effect(move || {
        let qs = query_string();
        copy_link_context.write().query_string = Some(qs);
        theme_context.write().theme = t!("themes.knospe").to_string();
    });

    let rules: Memo<Vec<RuleDef>> = use_memo(move || match configuration() {
        Configuration::Conventional => vec![
            RuleDef::AP1_1_ZutatMengeValidierung,
            RuleDef::AP1_2_ProzentOutputNamensgebend,
            RuleDef::AP1_3_EingabeNamensgebendeZutat,
            RuleDef::AP1_4_ManuelleEingabeTotal,
            RuleDef::AP2_1_ZusammegesetztOutput,
        ],
    });

    let calc_output: Memo<Output> = use_memo(move || {
        let mut calc = Calculator::new();
        calc.rule_defs = rules();
        let form: Form = current_state.read().clone();
        calc.execute(form.into())
    });
    let label: Memo<String> = use_memo(move || calc_output.read().label.clone());
    let validation_messages = use_memo(move || calc_output.read().validation_messages.clone());
    let conditional_display = use_memo(move || calc_output.read().conditional_elements.clone());

    use_context_provider(|| Validations(validation_messages));
    use_context_provider(|| Conditionals(conditional_display));

    let umstellung = use_signal(|| true);

    rsx! {
        div {
            class: "flex h-full",
            div {
                class: "flex-1 flex overflow-hidden",
                div {
                    class: "flex-1 overflow-y-scroll",
                    div { class: "flex flex-col gap-6 p-8 pb-12",
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
                            required: true,
                            TextInput {
                                placeholder: t!("placeholder.sachbezeichnung"),
                                bound_value: product_subtitle,
                                required: true
                            }
                        }
                        FormField {
                            label: t!("Umstellungsbetrieb"),
                            //help: Some(t!("help.ignore_ingredients").into()),
                            CheckboxInput {
                                bound_value: umstellung
                            }
                        }
                        SeparatorLine {}
                        FormField {
                            label: t!("label.ignore_ingredients"),
                            help: Some(t!("help.ignore_ingredients").into()),
                            CheckboxInput {
                                bound_value: ignore_ingredients
                            }
                        }
                        if !ignore_ingredients() {
                            FormField {
                                label: t!("label.zutaten"),
                                help: Some((t!("help.zutaten")).into()),
                                required: true,
                                IngredientsTable {
                                    ingredients: ingredients,
                                    validation_messages: validation_messages,
                                    manual_total: manual_total
                                }
                            }
                        }
                        SeparatorLine {}
                        FieldGroup2 {
                            FormField {
                                required: true,
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
                                    rows: "5",
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
                        }
                        SeparatorLine {}

                        FieldGroup1 {
                            label: t!("label.gewichtUndPreis"),
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
                                required: true,
                                help: Some((t!("help.name")).into()),
                                label: t!("label.name"),
                                TextInput { required: true, bound_value: producer_name, placeholder: t!("placeholder.name") }
                            }
                            div { class: "grid grid-cols-3 gap-4",
                                FormField {
                                    required: true,
                                    help: Some((t!("help.adresse")).into()),
                                    label: t!("label.adresse"),
                                    TextInput { required: true, bound_value: producer_address, placeholder: t!("placeholder.adresse")}
                                }
                                FormField {
                                    required: true,
                                    label: t!("label.plz"),
                                    TextInput { required: true, bound_value: producer_zip, placeholder: t!("placeholder.plz")}
                                }
                                FormField {
                                    required: true,
                                    help: Some((t!("help.ort")).into()),
                                    label: t!("label.ort"),
                                    TextInput { required: true, bound_value: producer_city, placeholder: t!("placeholder.ort")}
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
            price: price,
            calculated_amount: None,
            calculated_unit_price: None,
            calculated_total_price: None,
            ignore_ingredients: ignore_ingredients
        }
    }
}
