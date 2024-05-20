#![allow(non_snake_case)]

use std::ops::Add;
use dioxus::prelude::*;
use chrono::prelude::*;
use chrono::TimeDelta;
use crate::model::{sorted_ingredient_list, IngredientItem, food_db};

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
#[component]
pub fn TextInput(mut props: TextInputProps) -> Element {
    rsx! {
        input {
            class: "input bg-white input-bordered w-full",
            r#type: "text",
            placeholder: "{props.placeholder}",
            value: "{props.bound_value}",
            oninput: move |evt| props.bound_value.set(evt.data.value())
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct DateInputProps {
    date_prefix: Signal<String>,
    date_value: Signal<String>
}

pub fn DateInput(mut props: DateInputProps) -> Element {
    let in_a_year: DateTime<Utc> = Utc::now().add(TimeDelta::days(365));
    let formatted_date = in_a_year.format("%Y-%m-%d").to_string();

    rsx! {
        select {
            oninput: move |evt| props.date_prefix.set(evt.data.value()),
            class: "select bg-white select-bordered w-full max-w-xs",
            // oninput: move |evt| props.bound_value.set(evt.data.value()),
            option {selected: true, "mindestens haltbar bis"}
            option {"zu verbrauchen bis"}
        }
        input {
            oninput: move |evt| props.date_value.set(evt.data.value()),
            class: "input bg-white input-bordered w-full", r#type: "date", value: "{formatted_date}"}
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
            class: "textarea bg-white textarea-bordered w-full",
            rows: "{props.rows}",
            placeholder: "{props.placeholder}",
            value: "{props.bound_value}",
            oninput: move |evt| props.bound_value.set(evt.data.value())
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct FormFieldProps {
    #[props(into)]
    label: String,
    help: Option<Element>,
    children: Element
}
pub fn FormField(props: FormFieldProps) -> Element {
    rsx! {
        div {
            class: "flex gap-2 flex-col",
            label {
                "{props.label}"
                {rsx!{
                    FieldHelp {
                        label: props.label,
                        help: props.help.unwrap_or(None)
                    }
                }}
            }
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
    ingredients: Signal<Vec<IngredientItem>>,
    product_title: Signal<String>,
    product_subtitle: Signal<String>,
    additional_info: Signal<String>,
    storage_info: Signal<String>,
    production_country: Signal<String>,
    date_prefix: Signal<String>,
    date: Signal<String>,
    net_weight: Signal<String>,
    drained_weight: Signal<String>,
    producer_name: Signal<String>,
    producer_address: Signal<String>,
    producer_zip: Signal<String>,
    producer_city: Signal<String>,
    price_per_100: Signal<String>,
    total_price: Signal<String>

) -> Element {
    rsx! {
        div { class: "p-8 flex flex-col bg-gradient-to-r from-primary to-secondary",
            h2 { class: "text-primary-content pb-4 text-4xl",
                "Etiketten Vorschau"
            }
            div { class: "bg-white border p-4 grid grid-col-1 divide-y divide-dotted",
                div {
                    class: "py-2",
                    if *product_title.read() != "" {
                        {rsx! {
                            h3 { class: "text-2xl", "{product_title}" }
                            span { class: "mb-1", "{product_subtitle}" }
                        }}
                    } else {
                        {rsx! {
                            h3 { class: "text-2xl mb-1", "{product_subtitle}" }
                        }}
                    }
                }
                div {
                    class: "py-2",
                    h4 { class: "font-bold", "Zutaten:" }
                    div { class: "text-sm",
                        dangerous_inner_html: "{sorted_ingredient_list(ingredients.read().clone())}"
                    }
                }

                div {
                    class: "py-2 grid grid-cols-2 gap-4",
                    h4 { class: "font-bold", "Haltbarkeit" }
                    span {
                        span {class: "font-bold pr-2", "Nettogewicht"}
                        "{net_weight}"
                    }
                    span {
                        span {
                            class: "pr-1",
                            "{date_prefix}"
                        }
                        "{date}"
                    }
                    span {
                        span {class: "font-bold pr-2", "Abtropfgewicht"}
                        "{drained_weight}"
                    }

                }

                div { class: "py-2",
                    span { class: "text-sm",
                        "{additional_info}"
                    }
                    br {}
                    span { class: "text-sm",
                        "{storage_info}"
                    }
                    br {}
                    span{ class: "text-sm pr-1",
                        if (*production_country)() == "Schweiz" {
                            {"Hergestellt in der"}
                        } else {
                            {"Hergestellt in"}
                        }
                    }
                    span {class: "text-sm",
                        "{production_country}"
                    }
                }

                div { class: "py-2",
                    span {class: "text-sm",
                        "{producer_name}, {producer_address}, {producer_zip} {producer_city}"
                    }
                }

                div { class: "py-2 grid grid-cols-2 gap-4",
                    div {
                        span {class: "font-bold pr-2", "Preis pro 100g"} "{price_per_100} CHF"
                    }
                    div {
                        span {class: "font-bold pr-2", "Preis total"} "{total_price} CHF"
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct IngredientsTableProps {
    ingredients: Signal<Vec<IngredientItem>>
}
pub fn IngredientsTable(mut props: IngredientsTableProps) -> Element {
    let delete_callback =
        |index, mut list: Signal<Vec<IngredientItem>>| list.remove(index);
    let mut name_to_add = use_signal(|| String::new());
    let mut amount_to_add = use_signal(|| 0);
    rsx! {
        div { class: "flex flex-col gap-4",
            table { class: "table border-solid",
                tr { th { "Zutat (eingeben oder auswählen)" } th { "Menge" } }
                for (key, &ref ingr) in props.ingredients.read().iter().enumerate() {
                    tr { key: "{key}",
                        td {
                            "{ingr.basicInfo.standard_ingredient.name}"
                        }
                        td {
                            "{ingr.basicInfo.amount} g"
                        }
                        td {
                            button {
                                class: "btn btn-square",
                                dangerous_inner_html: r###"<svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>"###,
                                onclick: move |_| {
                                    delete_callback(key, props.ingredients.clone());
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
                class: "input input-bordered bg-white input-accent w-full",
                oninput: move |evt| {
                    if let Ok(amount) = evt.data.value().parse::<i32>() {
                        amount_to_add.set(amount);
                    }
                },
                value: "{amount_to_add}"
            }
            "g"
            button { class: "btn btn-accent",
                onclick: move |_evt|  {
                    props.ingredients.write().push(
                        IngredientItem::from_name_amount((&*name_to_add)(), (&*amount_to_add)())
                    );
                    name_to_add.set(String::new());
                    amount_to_add.set(0);
                },
                "Hinzufügen"
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct FieldHelpProps{
    #[props(into)]
    label: String,
    help: Element
}
pub fn FieldHelp(props: FieldHelpProps) -> Element {
    let mut is_open = use_signal(|| false);
    if props.help.is_none() {
        None
    } else {
        rsx! {
            button {
                class: "btn btn-xs ml-2",
                onkeydown: move |evt| {
                    match evt.key() {
                        Key::Escape => {
                            is_open.set(false);
                        },
                        _ => {}
                    }
                },
                onclick: move |_| is_open.toggle(),
                InfoIcon{}
            }
            dialog {
                open: "{is_open}",
                class: "modal",
                div {
                    class: "modal-box bg-base-100",
                    h3 { class:"font-bold text-lg", "{props.label}" }
                    {props.help}
                    div {class: "modal-action",
                        form {method: "dialog",
                            button {class:"btn btn-sm",
                                onclick: move |_| is_open.toggle(),
                                "× Schliessen"
                            }
                        }
                    }
                }
            }
        }
    }

}

#[component]
pub fn InfoIcon() -> Element {
    rsx! {
        svg {
            class: "w-6 h-6",
            view_box: "0 0 160 160",
            g {
                fill: "#4b4b4b",
                path {
                    d: "m80 15c-35.88 0-65 29.12-65 65s29.12 65 65 65 65-29.12 65-65-29.12-65-65-65zm0 10c30.36 0 55 24.64 55 55s-24.64 55-55 55-55-24.64-55-55 24.64-55 55-55z"
                }
                path {
                    d: "m57.373 18.231a9.3834 9.1153 0 1 1 -18.767 0 9.3834 9.1153 0 1 1 18.767 0z",
                    transform: "matrix(1.1989 0 0 1.2342 21.214 28.75)"
                }
                path {
                    d: "m90.665 110.96c-0.069 2.73 1.211 3.5 4.327 3.82l5.008 0.1v5.12h-39.073v-5.12l5.503-0.1c3.291-0.1 4.082-1.38 4.327-3.82v-30.813c0.035-4.879-6.296-4.113-10.757-3.968v-5.074l30.665-1.105"
                }
            }
        }
    }
}

