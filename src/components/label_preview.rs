use dioxus::prelude::*;
use crate::model::IngredientItem;

#[component]
pub fn LabelPreview(
    label: Memo<String>,
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
                        dangerous_inner_html: "{label}"
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