use dioxus::prelude::*;
use crate::components::FormField;
use crate::model::{food_db, IngredientItem};

#[derive(Props, Clone, PartialEq)]
pub struct IngredientsTableProps {
    ingredients: Signal<Vec<IngredientItem>>
    // TODO: accept validation messages
    // TODO: accept
}
pub fn IngredientsTable(mut props: IngredientsTableProps) -> Element {
    let delete_callback = |index, mut list: Signal<Vec<IngredientItem>>| list.remove(index);
    let mut name_to_add = use_signal(|| String::new());
    let mut amount_to_add = use_signal(|| 0);
    let mut scale_together = use_signal(|| false);
    let mut amount_to_edit = use_signal(|| 0);
    let mut key_to_edit = use_signal(|| 0);
    rsx! {
        div { class: "flex flex-col gap-4",
            table { class: "table border-solid",
                tr {
                    th { "Zutat (eingeben oder auswählen)" }
                    th { "Menge" }
                }
                for (key , & ref ingr) in props.ingredients.read().iter().enumerate() {
                    tr { key: "{key}",
                        td { "{ingr.basicInfo.standard_ingredient.name}" }
                        td { "{ingr.basicInfo.amount} g" }
                        td {
                            IngredientDetail {}
                            // ul { class: "rounded-box menu",
                            //     li {
                            //         details {
                            //             summary { "Menge Anpassen" }
                            //             ul {
                            //                 li {
                            //                     div { class: "form-control w-52",
                            //                         label { class: "label cursor-pointer",
                            //                             span { class: "label-text",
                            //                                 "Verhältnisse beibehalten"
                            //                             }
                            //                             input {
                            //                                 class: "checkbox",
                            //                                 r#type: "checkbox",
                            //                                 checked: "{scale_together}",
                            //                                 oninput: move |e| scale_together.set(e.value() == "true"),
                            //                             }
                            //                         }
                            //                     }
                            //                 }
                            //                 li {
                            //                     input {
                            //                         r#type: "number",
                            //                         placeholder: "Menge",
                            //                         class: "input input-bordered bg-white input-accent w-full",
                            //                         onchange: move |evt| {
                            //                             if let Ok(amount) = evt.data.value().parse::<i32>() {
                            //                                 amount_to_edit.set(amount);
                            //                             }
                            //                         },
                            //                         value: "{ingr.basicInfo.amount}",
                            //                     }
                            //                     button {
                            //                         class: "btn btn-accent",
                            //                         onclick: move |_evt| {
                            //                             if *scale_together.read() {
                            //                                 let factor: f32 = (&*amount_to_edit)() as f32
                            //                                     / props.ingredients.read().get(key).unwrap().basicInfo.amount as f32;
                            //                                 let ingredients = props.ingredients.read().clone();
                            //                                 for (key, elem) in ingredients.iter().enumerate() {
                            //                                     let name = elem.basicInfo.standard_ingredient.name.clone();
                            //                                     props.ingredients.write()[key] = IngredientItem::from_name_amount(
                            //                                         name,
                            //                                         (elem.basicInfo.amount as f32 * factor) as i32,
                            //                                     );
                            //                                 }
                            //                             } else {
                            //                                 let name = (props
                            //                                     .ingredients
                            //                                     .read()
                            //                                     .get(key)
                            //                                     .unwrap()
                            //                                     .basicInfo
                            //                                     .standard_ingredient
                            //                                     .name
                            //                                     .clone());
                            //                                 props.ingredients.write()[key] = IngredientItem::from_name_amount(
                            //                                     name,
                            //                                     (&*amount_to_edit)(),
                            //                                 );
                            //                             }
                            //                         },
                            //                         "Anpassen"
                            //                     }
                            //                 }
                            //             }
                            //         }
                            //     }
                            // }
                        }
                        td {
                            button {
                                class: "btn btn-square",
                                dangerous_inner_html: r###"<svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>"###,
                                onclick: move |_| {
                                    delete_callback(key, props.ingredients.clone());
                                },
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
                datalist { id: "ingredients",
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
                value: "{amount_to_add}",
            }
            "g"
            button {
                class: "btn btn-accent",
                onclick: move |_evt| {
                    props
                        .ingredients
                        .write()
                        .push(
                            IngredientItem::from_name_amount((&*name_to_add)(), (&*amount_to_add)()),
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
pub struct IngredientDetailProps {

}
pub fn IngredientDetail(props: IngredientDetailProps) -> Element {
    let mut is_open = use_signal(|| false);
    rsx! {
        button {
            class: "btn btn-square",
            onclick: move |_| is_open.toggle(),
            dangerous_inner_html: r###"<svg class="h-6 w-6" fill="none" height="24" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" viewBox="0 0 24 24" width="24" xmlns="http://www.w3.org/2000/svg"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>"###,
        }
        if is_open() { div { class: "fixed inset-0 bg-black bg-opacity-50 backdrop-blur-md" } }
        dialog { open: "{is_open}", class: "modal",
            div { class: "modal-box bg-base-100",
                h3 { class: "font-bold text-lg", "Zutat Details" }
                FormField {
                    label: "Zutat",
                    input {
                        list: "ingredients",
                        r#type: "flex",
                        placeholder: "Name",
                        class: "input input-bordered bg-white input-accent w-full",
                        // oninput: move |evt| name_to_add.set(evt.data.value()),
                        // value: "{name_to_add}",
                        datalist { id: "ingredients",
                            for item in food_db().clone() {
                                option { value: "{item.0}" }
                            }
                        }
                    }
                }
                FormField {
                    label: "Menge"
                }
                FormField {
                    label: "Namensgebende Zutat"
                }
                FormField {
                    label: "Zusammengesetyte Zutat"
                }
                // ROW name amount
                // Name
                // Amount
                // cond. ROW change amount
                // ROW namensgebend?
                // ROW composite?
                // Maybe CompositeIngredientsTable
                div { class: "modal-action",
                    form { method: "dialog",
                        button {
                            class: "btn btn-sm",
                            onclick: move |_| is_open.toggle(),
                            onkeydown: move |evt| {
                                match evt.key() {
                                    Key::Escape => {
                                        is_open.set(false);
                                    }
                                    _ => {}
                                }
                            },
                            "× Schliessen",
                        }
                    }
                }
            }
        }
    }
}