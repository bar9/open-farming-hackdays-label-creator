#![allow(non_snake_case)]

use std::collections::BTreeMap;
use std::sync::Arc;
use dioxus::html::textarea;
use dioxus::prelude::*;
use crate::model::{sorted_ingredient_list, IngredientItem, AdditionalInfo};

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