use crate::components::*;
use crate::core::{Calculator, Ingredient, Input, Output};
use crate::layout::{CopyLinkContext, ThemeContext};
use crate::rules::{RuleDef, RuleRegistry};
use crate::shared::{restore_params_from_session_storage, Conditionals, Configuration, Validations};
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
    pub certification_body: String,
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
    #[serde(default)]
    pub rezeptur_vollstaendig: bool,
}

fn default_weight_unit() -> String {
    t!("weight_units.g").to_string()
}

fn default_volume_unit() -> String {
    t!("volume_units.ml").to_string()
}

fn default_date_prefix() -> String {
    t!("label.mindestensHaltbar").to_string()
}

impl From<Form> for Input {
    fn from(val: Form) -> Self {
        Input {
            ingredients: val.ingredients,
            total: val.manual_total,
            certification_body: if val.certification_body.is_empty() {
                None
            } else {
                Some(val.certification_body)
            },
            rezeptur_vollstaendig: val.rezeptur_vollstaendig,
            ..Default::default()
        }
    }
}

impl Default for Form {
    fn default() -> Self {
        Form {
            ingredients: Vec::new(),
            ignore_ingredients: false,
            product_title: String::new(),
            product_subtitle: String::new(),
            additional_info: String::new(),
            storage_info: String::new(),
            date_prefix: t!("label.mindestensHaltbar").to_string(),
            date: String::new(),
            production_country: t!("countries.switzerland").to_string(),
            producer_name: String::new(),
            producer_address: String::new(),
            producer_email: String::new(),
            producer_website: String::new(),
            producer_phone: String::new(),
            producer_zip: String::new(),
            producer_city: String::new(),
            certification_body: String::new(),
            manual_total: None,
            amount_type: AmountType::Weight,
            weight_unit: t!("weight_units.g").to_string(),
            volume_unit: t!("volume_units.ml").to_string(),
            amount: Amount::Single(Some(0)),
            price: Price::Single(Some(0)),
            rezeptur_vollstaendig: false,
        }
    }
}

fn parse_form_from_saved_params(params: &str) -> Option<Form> {
    let decoded_query_string = js_sys::decode_uri_component(params)
        .unwrap_or_else(|_| params.into())
        .as_string()
        .unwrap_or(params.to_string());

    from_query_string::<Form>(&decoded_query_string).ok()
}

pub fn Knospe() -> Element {
    let mut url_params = use_signal(String::new);

    use_effect(move || {
        if let Some(saved_params) = restore_params_from_session_storage() {
            url_params.set(saved_params);
        }
    });

    let initial_form = use_memo(move || {
        let params = url_params.read().clone();
        if !params.is_empty() {
            if let Some(form_data) = parse_form_from_saved_params(&params) {
                return form_data;
            }
        }
        Form::default()
    });

    let mut ignore_ingredients = use_signal(|| false);
    let mut rezeptur_vollstaendig = use_signal(|| initial_form.read().rezeptur_vollstaendig);
    let mut ingredients: Signal<Vec<Ingredient>> =
        use_signal(|| initial_form.read().ingredients.clone());
    let mut product_title = use_signal(|| initial_form.read().product_title.clone());
    let mut product_subtitle = use_signal(|| initial_form.read().product_subtitle.clone());
    let mut additional_info = use_signal(|| initial_form.read().additional_info.clone());
    let mut storage_info = use_signal(|| initial_form.read().storage_info.clone());
    let mut date_prefix = use_signal(|| initial_form.read().date_prefix.clone());
    let mut date = use_signal(|| initial_form.read().date.clone());
    let mut production_country = use_signal(|| initial_form.read().production_country.clone());
    let mut producer_name = use_signal(|| initial_form.read().producer_name.clone());
    let mut producer_address = use_signal(|| initial_form.read().producer_address.clone());
    let mut producer_email = use_signal(|| initial_form.read().producer_email.clone());
    let mut producer_website = use_signal(|| initial_form.read().producer_website.clone());
    let mut producer_phone = use_signal(|| initial_form.read().producer_phone.clone());
    let mut producer_zip = use_signal(|| initial_form.read().producer_zip.clone());
    let mut producer_city = use_signal(|| initial_form.read().producer_city.clone());
    let mut certification_body = use_signal(|| initial_form.read().certification_body.clone());
    let mut manual_total = use_signal(|| initial_form.read().manual_total);
    let mut amount_type: Signal<AmountType> = use_signal(|| initial_form.read().amount_type.clone());
    let mut weight_unit: Signal<String> = use_signal(|| initial_form.read().weight_unit.clone());
    let mut volume_unit: Signal<String> = use_signal(|| initial_form.read().volume_unit.clone());
    let mut amount: Signal<Amount> = use_signal(|| initial_form.read().amount);
    let mut price: Signal<Price> = use_signal(|| initial_form.read().price);

    use_effect(move || {
        let form_data = initial_form.read();
        if !form_data.product_title.is_empty() || !form_data.product_subtitle.is_empty() {
            ignore_ingredients.set(form_data.ignore_ingredients);
            rezeptur_vollstaendig.set(form_data.rezeptur_vollstaendig);
            ingredients.set(form_data.ingredients.clone());
            product_title.set(form_data.product_title.clone());
            product_subtitle.set(form_data.product_subtitle.clone());
            additional_info.set(form_data.additional_info.clone());
            storage_info.set(form_data.storage_info.clone());
            date_prefix.set(form_data.date_prefix.clone());
            date.set(form_data.date.clone());
            production_country.set(form_data.production_country.clone());
            producer_name.set(form_data.producer_name.clone());
            producer_address.set(form_data.producer_address.clone());
            producer_email.set(form_data.producer_email.clone());
            producer_website.set(form_data.producer_website.clone());
            producer_phone.set(form_data.producer_phone.clone());
            producer_zip.set(form_data.producer_zip.clone());
            producer_city.set(form_data.producer_city.clone());
            certification_body.set(form_data.certification_body.clone());
            manual_total.set(form_data.manual_total);
            amount_type.set(form_data.amount_type.clone());
            weight_unit.set(form_data.weight_unit.clone());
            volume_unit.set(form_data.volume_unit.clone());
            amount.set(form_data.amount);
            price.set(form_data.price);
        }
    });

    let configuration = use_signal(|| Configuration::Knospe);

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
        certification_body: certification_body(),
        manual_total: manual_total(),
        amount_type: amount_type(),
        weight_unit: weight_unit(),
        volume_unit: volume_unit(),
        amount: amount(),
        price: price(),
        rezeptur_vollstaendig: rezeptur_vollstaendig(),
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

    let rules: Memo<Vec<RuleDef>> = use_memo(move || {
        let registry = RuleRegistry::new();
        registry.get_rules_for_config(&configuration())
            .unwrap_or(&vec![])
            .clone()
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

    // Calculate derived values for amount and price
    let get_base_factor = use_memo(move || {
        match (
            &*amount_type.read(),
            weight_unit.read().as_str(),
            volume_unit.read().as_str(),
        ) {
            (AmountType::Weight, "mg", _) => 100_usize,
            (AmountType::Weight, "g", _) => 100_usize,
            (AmountType::Weight, "kg", _) => 1_usize,
            (AmountType::Volume, _, "ml") => 100_usize,
            (AmountType::Volume, _, "cl") => 100_usize,
            (AmountType::Volume, _, "l") => 1_usize,
            (_, _, _) => 1_usize,
        }
    });

    let calculated_amount = use_memo(move || match price() {
        Price::Double(Some(unit_price), Some(total_price)) => (
            true,
            ((total_price as f64 / unit_price as f64) * get_base_factor() as f64) as usize,
        ),
        _ => (false, 0),
    });

    let calculated_total_price = use_memo(move || {
        let net_amount = match amount() {
            Amount::Single(Some(x)) => x,
            Amount::Double(Some(x), _) => x,
            _ => 0,
        };
        if net_amount == 0 {
            return (false, 0);
        }
        match price() {
            // Calculate total price when only unit price is provided
            Price::Double(Some(unit_price), None) => (
                true,
                (unit_price as f64 * (net_amount as f64 / get_base_factor() as f64)) as usize,
            ),
            // For single price fields, calculate total
            Price::Single(Some(unit_price)) => (
                true,
                (unit_price as f64 * (net_amount as f64 / get_base_factor() as f64)) as usize,
            ),
            _ => (false, 0),
        }
    });

    let calculated_unit_price = use_memo(move || {
        let net_amount = match amount() {
            Amount::Single(Some(x)) => x,
            Amount::Double(Some(x), _) => x,
            _ => 0,
        };
        if net_amount == 0 {
            return (false, 0);
        }
        match price() {
            Price::Double(_, Some(total_price)) => (
                true,
                (total_price as f64 / (net_amount as f64 / get_base_factor() as f64)) as usize,
            ),
            _ => (false, 0),
        }
    });

    rsx! {
        div {
            class: "flex h-full",
            div {
                class: "flex-1 flex overflow-hidden",
                div {
                    class: "flex-1 overflow-y-scroll",
                    div { class: "flex flex-col gap-6 p-8 pb-12",
                        FormField {
                            label: t!("label.produktname").to_string(),
                            help: Some(t!("help.produktname").to_string().into()),
                            TextInput {
                                placeholder: t!("placeholder.produktname").to_string(),
                                bound_value: product_title
                            }
                        }
                        FormField {
                            label: t!("label.sachbezeichnung").to_string(),
                            help: Some((t!("help.sachbezeichnung").to_string()).into()),
                            required: true,
                            TextInput {
                                placeholder: t!("placeholder.sachbezeichnung").to_string(),
                                bound_value: product_subtitle,
                                required: true
                            }
                        }
                        SeparatorLine {}
                        FormField {
                            label: t!("label.ignore_ingredients").to_string(),
                            help: Some(t!("help.ignore_ingredients").to_string().into()),
                            inline_checkbox: true,
                            CheckboxInput {
                                bound_value: ignore_ingredients
                            }
                        }
                        if !ignore_ingredients() {
                            FormField {
                                label: t!("label.zutaten").to_string(),
                                help: Some((t!("help.zutaten").to_string()).into()),
                                required: true,
                                IngredientsTable {
                                    ingredients: ingredients,
                                    validation_messages: validation_messages,
                                    manual_total: manual_total,
                                    rules: rules,
                                    rezeptur_vollstaendig: rezeptur_vollstaendig
                                }
                            }
                        }
                        SeparatorLine {}
                        FieldGroup2 {
                            FormField {
                                required: true,
                                label: t!("label.datumseingabe").to_string(),
                                help: Some((t!("help.datumseingabe").to_string()).into()),
                                DateInput {
                                    date_value: date,
                                    date_prefix: date_prefix
                                }

                            }
                            FormField { label: t!("label.zusatzinformationen").to_string(),
                                help: Some((t!("help.zusatzinformationen").to_string()).into()),
                                TextareaInput {
                                    placeholder: t!("placeholder.zusatzinformationen").to_string(),
                                    rows: "5",
                                    bound_value: additional_info
                                }
                            }
                        }
                        FieldGroup2 {
                            FormField {
                                label: t!("label.aufbewahrungshinweis").to_string(),
                                help: Some((t!("help.aufbewahrungshinweis").to_string()).into()),
                                TextareaInput{
                                    rows: "2",
                                    placeholder: t!("placeholder.aufbewahrungshinweis").to_string(),
                                    bound_value: storage_info
                                }
                            }
                        }
                        SeparatorLine {}

                        FieldGroup1 {
                            label: t!("label.gewichtUndPreis").to_string(),
                            AmountPrice {
                                amount_type: amount_type,
                                weight_unit: weight_unit,
                                volume_unit: volume_unit,
                                amount: amount,
                                price: price,
                            }
                        }
                        SeparatorLine {}
                        FieldGroup1 { label: t!("label.adresse").to_string(),
                            FormField {
                                required: true,
                                help: Some((t!("help.name").to_string()).into()),
                                label: t!("label.name").to_string(),
                                TextInput { required: true, bound_value: producer_name, placeholder: t!("placeholder.name").to_string() }
                            }
                            div { class: "grid grid-cols-3 gap-4",
                                FormField {
                                    required: true,
                                    help: Some((t!("help.adresse").to_string()).into()),
                                    label: t!("label.adresse").to_string(),
                                    TextInput { required: true, bound_value: producer_address, placeholder: t!("placeholder.adresse").to_string()}
                                }
                                FormField {
                                    required: true,
                                    label: t!("label.plz").to_string(),
                                    TextInput { required: true, bound_value: producer_zip, placeholder: t!("placeholder.plz").to_string()}
                                }
                                FormField {
                                    required: true,
                                    help: Some((t!("help.ort").to_string()).into()),
                                    label: t!("label.ort").to_string(),
                                    TextInput { required: true, bound_value: producer_city, placeholder: t!("placeholder.ort").to_string()}
                                }
                                FormField {
                                    label: t!("label.telefon").to_string(),
                                    TextInput { bound_value: producer_phone, placeholder: t!("placeholder.telefon").to_string()}
                                }
                                FormField {
                                    label: t!("label.email").to_string(),
                                    TextInput { bound_value: producer_email, placeholder: t!("placeholder.email").to_string()}
                                }
                                FormField {
                                    label: t!("label.website").to_string(),
                                    TextInput { bound_value: producer_website, placeholder: t!("placeholder.website").to_string()}
                                }
                            }
                        }
                        SeparatorLine {}
                        FormField {
                            label: "{t!(\"label.certification_body\").to_string()} *",
                            required: true,
                            help: Some(t!("help.certification_body_knospe").to_string()),
                            CertificationBodySelect {
                                bound_value: certification_body
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
            certification_body: Some(certification_body),
            amount_type: amount_type,
            weight_unit: weight_unit,
            volume_unit: volume_unit,
            amount: amount,
            price: price,
            calculated_amount: Some(calculated_amount),
            calculated_unit_price: Some(calculated_unit_price),
            calculated_total_price: Some(calculated_total_price),
            ignore_ingredients: ignore_ingredients
        }
    }
}
