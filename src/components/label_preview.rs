use crate::components::{Amount, AmountType, Price};
use crate::components::icons::{BioSuisseRegular, BioSuisseNoCross, Umstellung};
use crate::shared::Conditionals;
use crate::nl2br::Nl2Br;
use dioxus::prelude::*;
use rust_i18n::t;

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
    producer_name: Signal<String>,
    producer_address: Signal<String>,
    producer_zip: Signal<String>,
    producer_city: Signal<String>,
    producer_email: Signal<String>,
    producer_website: Signal<String>,
    producer_phone: Signal<String>,
    amount_type: Signal<AmountType>,
    weight_unit: Signal<String>,
    volume_unit: Signal<String>,
    amount: Signal<Amount>,
    price: Signal<Price>,
    ignore_ingredients: Signal<bool>,
    // Optional calculated values from AmountPrice component
    calculated_amount: Option<Memo<(bool, usize)>>,
    calculated_unit_price: Option<Memo<(bool, usize)>>,
    calculated_total_price: Option<Memo<(bool, usize)>>,
) -> Element {
    fn display_money(cents: Option<usize>) -> String {
        match cents {
            None => String::new(),
            Some(x) => format!("{:.2}", x as f64 / 100.0),
        }
    }

    let address_combined: Memo<String> = use_memo(move || {
        let parts = vec![producer_name(), producer_address(), {
            let zip = producer_zip();
            let city = producer_city();
            if zip.is_empty() {
                city
            } else if city.is_empty() {
                zip
            } else {
                format!("{zip} {city}")
            }
        }];

        parts
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(", ")
    });

    let get_unit = use_memo(move || {
        match (
            &*amount_type.read(),
            &*weight_unit.read(),
            &*volume_unit.read(),
        ) {
            (AmountType::Weight, unit, _) => unit.clone(),
            (AmountType::Volume, _, unit) => unit.clone(),
        }
    });

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

    let get_base_factor_and_unit = use_memo(move || match get_base_factor() {
        1 => rsx!("{get_unit()}"),
        _ => rsx!("{get_base_factor()} {get_unit()}"),
    });

    let conditionals = use_context::<Conditionals>();

    rsx! {
        div { class: "p-8 flex flex-col bg-base-200",
            div { class: "bg-white rounded-lg shadow-lg p-8 mx-4 my-4 relative",
                // Bio Suisse logo display
                if conditionals.0().get("bio_suisse_umstellung").unwrap_or(&false) == &true {
                    div { class: "absolute top-2 right-2 w-16",
                        Umstellung {}
                    }
                } else if conditionals.0().get("bio_suisse_regular").unwrap_or(&false) == &true {
                    div { class: "absolute top-2 right-2 w-16",
                        BioSuisseRegular {}
                    }
                } else if conditionals.0().get("bio_suisse_no_cross").unwrap_or(&false) == &true {
                    div { class: "absolute top-2 right-2 w-16",
                        BioSuisseNoCross {}
                    }
                }

                // Umstellung message display (plain black text, positioned to the left of logo)
                if conditionals.0().get("umstellung_biologische_landwirtschaft").unwrap_or(&false) == &true {
                    div { class: "absolute top-2 left-2 max-w-xs text-xs text-black bg-white",
                        "Hergestellt im Rahmen der Umstellung auf die biologische Landwirtschaft."
                    }
                } else if conditionals.0().get("umstellung_bio_suisse_richtlinien").unwrap_or(&false) == &true {
                    div { class: "absolute top-2 left-2 max-w-xs text-xs text-black bg-white",
                        "Hergestellt im Rahmen der Umstellung auf die Bio Suisse Richtlinien."
                    }
                }

                div { class: "grid grid-col-1 divide-y divide-dotted",
                div {
                    class: "py-2",
                    if (*product_subtitle.read()).is_empty() {
                        span {class: "badge badge-warning", {t!("preview.produktnameSachbezeichnung")}}
                    } else {
                        if !(*product_title.read()).is_empty() {
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

                }
                if !ignore_ingredients() {
                    div {
                        class: "py-2",
                        h4 { class: "font-bold", "{t!(\"preview.zutaten\")}" }
                        if (*label.read()).is_empty() {
                            span { class: "badge badge-warning", {t!("preview.zutatenliste")} }
                        } else {
                            div { class: "text-sm",
                                dangerous_inner_html: "{label}"
                            }
                        }
                    }
                }

                if date_prefix() != t!("label.keinDatum") {
                    div {
                        class: "py-2 grid grid-cols-1 gap-4",
                        span {
                            span {
                                class: "pr-1",
                                b {"{date_prefix}"}
                            }
                            "{date}"
                        }
                    }
                }
                {
                    // Use calculated amount if available, otherwise use raw amount
                    let amount_display = if let Some(calc_amount) = &calculated_amount {
                        if calc_amount().0 { Some(calc_amount().1) } else { None }
                    } else { None };

                    match (amount(), amount_display) {
                        // Show calculated amount when available
                        (_, Some(calculated_amt)) => rsx! {
                            div {
                                span {
                                    "{calculated_amt} {get_unit()}"
                                }
                            }
                        },
                        // Show raw amounts when no calculation is available
                        (Amount::Single(Some(amt)), None) => rsx! {
                            div {
                                span {
                                    "{amt} {get_unit()}"
                                }
                            }
                        },
                        (Amount::Double(Some(netto), Some(brutto)), None) => rsx! {
                            div {
                                span {
                                    span {class: "font-bold pr-2", "{t!(\"preview.nettogewicht\")}" }
                                    "{netto} {get_unit()}"
                                }
                                span {
                                    span {class: "font-bold pl-2 pr-2", "{t!(\"preview.abtropfgewicht\")}" }
                                    "{brutto} {get_unit()}"
                                }
                            }
                        },
                        _ => rsx! {}
                    }
                }

                if !additional_info().is_empty() && !storage_info().is_empty() {
                    div { class: "py-2",
                        span { class: "text-sm",
                            {additional_info().nl2br()}
                        }
                        br {}
                        span { class: "text-sm",
                            {storage_info().nl2br()}
                        }
                        br {}
                    }
                }


                div { class: "py-2",
                    if !address_combined.read().is_empty() {
                        span {
                            class: "text-sm",
                            "{address_combined}"
                        }
                    } else {
                        span {class: "badge badge-warning", {t!("preview.herstelleradresse")} }
                    }
                    if !producer_phone.read().is_empty() {
                        div {class: "text-sm",
                            "{t!(\"preview.tel\", phone=producer_phone)}"
                        }
                    }
                    if !producer_email.read().is_empty() {
                        div {class: "text-sm",
                            "{t!(\"preview.email\", email=producer_email)}"
                        }
                    }

                    if !producer_website.read().is_empty() {
                        div {class: "text-sm",
                            "{t!(\"preview.website\", website=producer_website)}"
                        }
                    }
                }
                    match (price(), amount()) {
                        (Price::Single(None), _) => rsx! {},
                        (Price::Single(x), Amount::Single(Some(1))) |
                        (Price::Single(x), Amount::Single(Some(100))) |
                        (Price::Single(x), Amount::Single(Some(250))) |
                        (Price::Single(x), Amount::Single(Some(500))) |
                        (Price::Single(x), Amount::Double(Some(1), _)) |
                        (Price::Single(x), Amount::Double(Some(100), _)) |
                        (Price::Single(x), Amount::Double(Some(250), _)) |
                        (Price::Single(x), Amount::Double(Some(500), _)) => rsx! {
                            "{display_money(x)} " {t!("units.chf")}
                        },
                        // Handle non-unitary amounts with Price::Single - show both unit price and calculated total
                        (Price::Single(x), _) => {
                            if let Some(unit_price) = x {
                                let total_price_display = if let Some(calc_total) = &calculated_total_price {
                                    if calc_total().0 { Some(calc_total().1) } else { None }
                                } else { None };

                                rsx! (
                                    div {
                                        span {
                                            span {class: "font-bold pr-2", {t!("units.chfPro")} {get_base_factor_and_unit()} }
                                            "{display_money(Some(unit_price))} " {t!("units.chf")}
                                        }
                                        if let Some(total_price) = total_price_display {
                                            span {
                                                span {class: "font-bold pl-2 pr-2", {t!("preview.preis")} }
                                                "{display_money(Some(total_price))} " {t!("units.chf")}
                                            }
                                        }
                                    }
                                )
                            } else {
                                rsx! {}
                            }
                        },
                        (Price::Double(x, _), Amount::Single(Some(1))) |
                        (Price::Double(x, _), Amount::Single(Some(100))) |
                        (Price::Double(x, _), Amount::Single(Some(250))) |
                        (Price::Double(x, _), Amount::Single(Some(500))) |
                        (Price::Double(x, _), Amount::Double(Some(1), _)) |
                        (Price::Double(x, _), Amount::Double(Some(100), _)) |
                        (Price::Double(x, _), Amount::Double(Some(250), _)) |
                        (Price::Double(x, _), Amount::Double(Some(500), _)) => rsx! {
                            "{display_money(x)} " {t!("units.chf")}
                        },
                        (Price::Double(x, y), _) => {
                            // Use calculated values if available, otherwise use raw price values
                            let unit_price_display = if let Some(calc_unit) = &calculated_unit_price {
                                if calc_unit().0 { Some(calc_unit().1) } else { x }
                            } else { x };

                            let total_price_display = if let Some(calc_total) = &calculated_total_price {
                                if calc_total().0 { Some(calc_total().1) } else { y }
                            } else { y };

                            // Show prices if we have either raw values or calculated values
                            if unit_price_display.is_some() || total_price_display.is_some() {
                                rsx! (
                                    div {
                                        if let Some(unit_price) = unit_price_display {
                                            span {
                                                span {class: "font-bold pr-2", {t!("units.chfPro")} {get_base_factor_and_unit()} }
                                                "{display_money(Some(unit_price))} " {t!("units.chf")}
                                            }
                                        }
                                        if let Some(total_price) = total_price_display {
                                            span {
                                                span {class: "font-bold pl-2 pr-2", {t!("preview.preis")} }
                                                "{display_money(Some(total_price))} " {t!("units.chf")}
                                            }
                                        }
                                    }
                                )
                            } else {
                                rsx! {}
                            }
                        }
                    }

                // if !(price_per_100().is_empty() && total_price().is_empty()) {
                //     div { class: "py-2 grid grid-cols-2 gap-4",
                //         div {
                //             span {class: "font-bold pr-2", "{t!(\"preview.preisPro\", amount = 100, unit = \"g\")}"} "{price_per_100} " {t!("units.chf")}
                //         }
                //         div {
                //             span {class: "font-bold pr-2", "{t!(\"preview.preisTotal\")}"} "{total_price} " {t!("units.chf")}
                //         }
                //     }
                // }
                }
            }
        }
    }
}
