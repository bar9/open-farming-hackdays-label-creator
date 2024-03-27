#![allow(non_snake_case)]

use std::collections::BTreeMap;
use std::sync::Arc;
use dioxus::prelude::*;
use crate::model::{sorted_ingredient_list, IngredientItem};

#[component]
pub fn SeparatorLine(cx: Scope) -> Element {
    cx.render( rsx! {
        hr { class: "border-1 border-dashed border-neutral-400 my-2" }
    })
}

#[component]
pub fn TextInput<'a>(cx: Scope, placeholder: &'a str, bound_value: &'a UseState<String>) -> Element {
    cx.render( rsx! {
        input {
            class: "input input-bordered w-full",
            r#type: "text",
            placeholder: "{placeholder}",
            value: "{bound_value}",
            oninput: move |evt| bound_value.set(evt.value.clone())
        }
    })
}

#[component]
pub fn TextInputDummy<'a>(cx: Scope, placeholder: &'a str) -> Element {
    cx.render( rsx! {
        input {
            class: "input input-bordered w-full",
            r#type: "text",
            placeholder: "{placeholder}",
        }
    })
}
#[component]
pub fn FormField<'a>(cx: Scope, label: &'a str, children: Element<'a>) -> Element {
    cx.render(rsx! {
        div {
            class: "flex gap-2 flex-col",
            label { "{label}" }
            &children
        }
    })
}

#[component]
pub fn FieldGroup2<'a>(cx: Scope, children: Element<'a>) -> Element {
    cx.render(rsx! {
        div {
            class: "grid grid-cols-2 gap-4 ",
            &children
        }
    })
}

#[component]
pub fn FieldGroup1<'a>(cx: Scope, label: &'a str, children: Element<'a>) -> Element {
    cx.render( rsx! {
        div { class: "flex flex-col gap-4",
            h4 { class: "text-xl mb-2", "{label}" }
            &children
        }
    })
}

#[component]
pub fn AddNewIngredientButton<'a>(cx: Scope, on_click: EventHandler<'a, MouseEvent>) -> Element {
    cx.render(rsx! {
        button { class: "btn btn-outline",
            onclick: move |evt| cx.props.on_click.call(evt),
            "Eine Zutat hinzufügen",
        },
    },)
}

#[component]
pub fn LabelPreview<'a>(
    cx: Scope,
    ingredients: &'a UseRef<BTreeMap<usize, IngredientItem>>,
    product_title: &'a UseState<String>
) -> Element {
    cx.render( rsx! {
        div { class: "p-8 flex flex-col bg-primary",
            h2 { class: "pb-4 text-4xl",
                "Etiketten Vorschau"
            }
            div {
                class: "bg-white border p-4 grid grid-col-1 gap-4",
                h3 { class: "text-2xl mb-2", "{product_title}" }
                h4 { class: "text-xl mb-2", "Zutaten" }
                span {
                    dangerous_inner_html: "{sorted_ingredient_list(ingredients.read().clone())}"
                }
            }
        },
    })
}