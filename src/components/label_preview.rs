use dioxus::prelude::*;
use crate::model::IngredientItem;
use crate::rules::RuleDef;
use crate::rules::RuleDef::{AllGram, AllPercentages, Composite, MaxDetails, PercentagesStartsWithM, V_001_Menge_Immer_Benoetigt};
use crate::model::processed_ingredient_list;

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
    total_price: Signal<String>,
) -> Element {
    let mut active_rules: Signal<Vec<RuleDef>> = use_signal(|| vec![]);

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
                        dangerous_inner_html: "{processed_ingredient_list(ingredients.read().clone(), active_rules.read().clone())}"
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
            h2 { class: "text-primary-content pb-4 pt-8 text-4xl",
                "Regeln"
            }
            div {
                class: "py-2 bg-white",
                form {
                    oninput: move |ev| {
                        let selected_rule: &str = &ev.values().get("radio-rules").unwrap().get(0).unwrap().clone();
                        match selected_rule {
                            "startswith" => *active_rules.write() = vec![V_001_Menge_Immer_Benoetigt, PercentagesStartsWithM],
                            "percent" => *active_rules.write() = vec![V_001_Menge_Immer_Benoetigt, AllPercentages],
                            "debug" => *active_rules.write() = vec![V_001_Menge_Immer_Benoetigt, MaxDetails],
                            "gram" => *active_rules.write() = vec![V_001_Menge_Immer_Benoetigt, AllGram],
                            "composite" => *active_rules.write() = vec![V_001_Menge_Immer_Benoetigt, Composite],
                            "composite-m" => *active_rules.write() = vec![V_001_Menge_Immer_Benoetigt, PercentagesStartsWithM, Composite],
                            _ => *active_rules.write() = vec![V_001_Menge_Immer_Benoetigt]
                       }
                    },
                    div {
                        class: "form-control w-52",
                        label {
                            class: "label cursor-pointer",
                            span {
                                class: "label-text",
                                "Standard"
                            }
                            input {
                                r#type: "radio", name: "radio-rules", class: "radio checked:bg-primary", checked: true, value: "standard"
                            }
                        }
                    }
                    div {
                        class: "form-control w-52",
                        label {
                            class: "label cursor-pointer",
                            span {
                                class: "label-text",
                                "Startbuchstabe 'M' -> %"
                            }
                            input {
                                r#type: "radio", name: "radio-rules", class: "radio checked:bg-primary", value: "startswith"
                            }
                        }
                    }
                    div {
                       class: "form-control w-52",
                       label {
                           class: "label cursor-pointer",
                           span {
                               class: "label-text",
                               "Alle %"
                           }
                           input {
                               r#type: "radio", name: "radio-rules", class: "radio checked:bg-primary", value: "percent"
                           }
                       }
                    }
                    div {
                       class: "form-control w-52",
                       label {
                           class: "label cursor-pointer",
                           span {
                               class: "label-text",
                               "Zusammengesetzt (Beispiel 'Brot')"
                           }
                           input {
                               r#type: "radio", name: "radio-rules", class: "radio checked:bg-primary", value: "composite"
                           }
                       }
                    }
                    div {
                       class: "form-control w-52",
                       label {
                           class: "label cursor-pointer",
                           span {
                               class: "label-text",
                               "Zusammengesetzt Brot + Startbuchstabe M in %"
                           }
                           input {
                               r#type: "radio", name: "radio-rules", class: "radio checked:bg-primary", value: "composite-m"
                           }
                       }
                    }
                    div {
                       class: "form-control w-52",
                       label {
                           class: "label cursor-pointer",
                           span {
                               class: "label-text",
                               "Alle Mengen in g"
                           }
                           input {
                               r#type: "radio", name: "radio-rules", class: "radio checked:bg-primary", value: "gram"
                           }
                       }
                    }
                    div {
                       class: "form-control w-52",
                       label {
                           class: "label cursor-pointer",
                           span {
                               class: "label-text",
                               "Alle Details"
                           }
                           input {
                               r#type: "radio", name: "radio-rules", class: "radio checked:bg-primary", value: "debug"
                           }
                       }
                    }
                }
            }
        }
    }
}