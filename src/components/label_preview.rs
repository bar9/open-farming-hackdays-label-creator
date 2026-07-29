use crate::conditional_keys as keys;
use crate::components::{Amount, AmountType, Price};
use crate::components::icons::{BioSuisseRegular, BioSuisseNoCross, UmstellungsknospeSatzRegular, UmstellungsknospeSatzImport};
use crate::layout::DisclaimerContext;
use crate::shared::{Conditionals, VerdictsContext};
use crate::verdicts::{BioBlockReason, BioVerdict, CheckState, KnospeBlockReason, KnospeVerdict};
use crate::nl2br::Nl2Br;
use dioxus::prelude::*;
use rust_i18n::t;

/// An informational hint below the label (blue box).
///
/// These hints are deliberately outside the white card so users can tell they
/// are guidance, not part of the printed label. The markup was repeated eleven
/// times; naming it keeps the styling in one place and the call sites readable.
#[component]
fn Hint(text: String) -> Element {
    rsx! {
        div { class: "mt-2 p-2 bg-info/30 text-base-content text-xs rounded", {text} }
    }
}

/// A warning hint below the label (amber box with a warning triangle).
/// Used for the failing «Rezeptur prüfen» verdicts.
#[component]
fn WarningHint(text: String) -> Element {
    rsx! {
        div { class: "mt-2 p-2 bg-warning/40 text-base-content text-xs rounded flex items-start gap-2",
            svg {
                class: "w-4 h-4 shrink-0 mt-0.5",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                view_box: "0 0 24 24",
                path {
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    d: "M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z",
                }
            }
            span { {text} }
        }
    }
}

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
    #[props(default)]
    certification_body: Option<Signal<String>>,
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
    fn display_money_exact(cents: Option<usize>) -> String {
        match cents {
            None => String::new(),
            Some(x) => format!("{:.2}", x as f64 / 100.0)
        }
    }

    fn display_money_rounded(cents: Option<usize>) -> String {
        match cents {
            None => String::new(),
            Some(x) => {
                // Round to nearest 5 Rappen (5 cents)
                let rounded_cents = ((x as f64 / 5.0).round() * 5.0) as usize;
                format!("{:.2}", rounded_cents as f64 / 100.0)
            }
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
            .join("\n")
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
    let verdicts = use_context::<VerdictsContext>();
    let mut disclaimer_context = use_context::<Signal<DisclaimerContext>>();
    let disclaimer_accepted = use_memo(move || disclaimer_context.read().accepted);

    use_effect(move || {
        let accepted = disclaimer_accepted();
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item("disclaimer_accepted", if accepted { "true" } else { "false" });
            }
        }
    });

    rsx! {
        div { class: "p-8 flex flex-col bg-base-200",
            if disclaimer_accepted() {
            div { class: "bg-white rounded-lg shadow-lg p-8 mx-4 my-4 relative",
                // Bio Suisse logo display. With Umstellungs ingredients the artwork
                // switches to the official Umstellungsknospe, pre-baked together with
                // the Umstellungssatz text into one combined image (logo left, text
                // right — see `make umstellung-assets`).
                {
                    // TD-1 Stufe 3: the artwork follows the typed Knospe verdict
                    // directly. Logo variant and Umstellung are one value, so the
                    // impossible combinations (both variants, Umstellung without a
                    // logo) are gone by construction. The green "Bio ✓" badge is
                    // the Bio-V counterpart: driven by the recipe verdict, not the
                    // «Rezeptur prüfen» button (DEC-6). Knospe and Bio verdicts
                    // never coexist (different configurations) → no collision.
                    let v = verdicts.0();
                    match (&v.knospe, &v.bio) {
                        (Some(KnospeVerdict::Logo { logo, .. }), _) => {
                            let logo = *logo;
                            rsx! {
                                div { class: "absolute top-2 right-2 flex items-center justify-end",
                                    if logo.umstellung && logo.swiss_cross {
                                        UmstellungsknospeSatzRegular {}
                                    } else if logo.umstellung {
                                        UmstellungsknospeSatzImport {}
                                    } else {
                                        div { class: "w-16 shrink-0",
                                            if logo.swiss_cross {
                                                BioSuisseRegular {}
                                            } else {
                                                BioSuisseNoCross {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        (_, Some(BioVerdict::Allowed { .. })) => rsx! {
                            div { class: "absolute top-2 right-2",
                                span { class: "badge badge-success gap-1",
                                    {t!("badges.bio_qualified").to_string()}
                                }
                            }
                        },
                        _ => rsx! {},
                    }
                }

                div { class: "grid grid-cols-1 divide-y divide-dotted",
                div {
                    class: "py-2",
                    if (*product_subtitle.read()).is_empty() {
                        span {class: "badge badge-warning", {t!("preview.produktnameSachbezeichnung").to_string()}}
                    } else {
                        {
                            // « Bio» after the Sachbezeichnung: granted by either
                            // regime's verdict (Bio-V allowed, or a Knospe logo whose
                            // bio_suffix flag is set — DEC-10).
                            let v = verdicts.0();
                            let suffix_allowed = matches!(v.bio, Some(BioVerdict::Allowed { .. }))
                                || matches!(v.knospe, Some(KnospeVerdict::Logo { bio_suffix: true, .. }));
                            let bio_suffix = if suffix_allowed { " Bio" } else { "" };
                            if !(*product_title.read()).is_empty() {
                                rsx! {
                                    h3 { class: "text-2xl", "{product_title}" }
                                    span { class: "mb-1 text-base", "{product_subtitle}{bio_suffix}" }
                                }
                            } else {
                                rsx! {
                                    h3 { class: "text-2xl mb-1", "{product_subtitle}{bio_suffix}" }
                                }
                            }
                        }
                    }

                }
                if !ignore_ingredients() {
                    div {
                        class: "py-2",
                        if (*label.read()).is_empty() {
                            span { class: "badge badge-warning", {t!("preview.zutatenliste").to_string()} }
                        } else {
                            div { class: "text-sm",
                                "{t!(\"preview.zutaten\").to_string()} "
                                span { dangerous_inner_html: "{label}" }
                            }
                        }
                    }
                }

                if date_prefix() != t!("label.keinDatum") {
                    div {
                        class: "py-2 grid grid-cols-1 gap-4",
                        span {
                            class: "text-sm",
                            span {
                                class: "pr-1",
                                "{date_prefix}"
                            }
                            " {date}"
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
                                    class: "text-sm",
                                    "{calculated_amt} {get_unit()}"
                                }
                            }
                        },
                        // Show raw amounts when no calculation is available
                        (Amount::Single(Some(amt)), None) => rsx! {
                            div {
                                span {
                                    class: "text-sm",
                                    "{amt} {get_unit()}"
                                }
                            }
                        },
                        (Amount::Double(Some(netto), Some(brutto)), None) => rsx! {
                            div {
                                class: "text-sm",
                                span {
                                    span {class: "pr-2", "{t!(\"preview.nettogewicht\").to_string()}" }
                                    " {netto} {get_unit()}"
                                }
                                span {
                                    span {class: "pl-2 pr-2", " {t!(\"preview.abtropfgewicht\").to_string()}" }
                                    " {brutto} {get_unit()}"
                                }
                            }
                        },
                        _ => rsx! {}
                    }
                }

                if !additional_info().is_empty() || !storage_info().is_empty() {
                    div { class: "py-2",
                        if !additional_info().is_empty() {
                            span { class: "text-sm",
                                {additional_info().nl2br()}
                            }
                            br {}
                        }
                        if !storage_info().is_empty() {
                            span { class: "text-sm",
                                {storage_info().nl2br()}
                            }
                            br {}
                        }
                    }
                }


                div { class: "py-2",
                    if !address_combined.read().is_empty() {
                        span {
                            class: "text-sm",
                            {address_combined.read().nl2br()}
                        }
                    } else {
                        span {class: "badge badge-warning", {t!("preview.herstelleradresse").to_string()} }
                    }
                    if !producer_phone.read().is_empty() {
                        div {class: "text-sm",
                            "{t!(\"preview.tel\", phone=producer_phone).to_string()}"
                        }
                    }
                    if !producer_email.read().is_empty() {
                        div {class: "text-sm",
                            "{t!(\"preview.email\", email=producer_email).to_string()}"
                        }
                    }

                    if !producer_website.read().is_empty() {
                        div {class: "text-sm",
                            "{t!(\"preview.website\", website=producer_website).to_string()}"
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
                            span {
                                class: "text-sm",
                                "{display_money_rounded(x)} " {t!("units.chf").to_string()}
                            }
                        },
                        // Handle non-unitary amounts with Price::Single - show both unit price and calculated total
                        (Price::Single(x), _) => {
                            if let Some(unit_price) = x {
                                let total_price_display = if let Some(calc_total) = &calculated_total_price {
                                    if calc_total().0 { Some(calc_total().1) } else { None }
                                } else { None };

                                rsx! (
                                    div {
                                        class: "text-sm",
                                        span {
                                            span {class: "pr-2", {t!("units.chfPro").to_string()} {get_base_factor_and_unit()} }
                                            " {display_money_exact(Some(unit_price))} " {t!("units.chf").to_string()}
                                        }
                                        if let Some(total_price) = total_price_display {
                                            span {
                                                span {class: "pl-2 pr-2", " " {t!("preview.preis").to_string()} }
                                                " {display_money_rounded(Some(total_price))} " {t!("units.chf").to_string()}
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
                            span {
                                class: "text-sm",
                                "{display_money_rounded(x)} " {t!("units.chf").to_string()}
                            }
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
                                        class: "text-sm",
                                        if let Some(unit_price) = unit_price_display {
                                            span {
                                                span {class: "pr-2", {t!("units.chfPro").to_string()} {get_base_factor_and_unit()} }
                                                " {display_money_exact(Some(unit_price))} " {t!("units.chf").to_string()}
                                            }
                                        }
                                        if let Some(total_price) = total_price_display {
                                            span {
                                                span {class: "pl-2 pr-2", " " {t!("preview.preis").to_string()} }
                                                " {display_money_rounded(Some(total_price))} " {t!("units.chf").to_string()}
                                            }
                                        }
                                    }
                                )
                            } else {
                                rsx! {}
                            }
                        }
                    }

                // Display certification body if provided
                if let Some(cert_body_signal) = certification_body {
                    div { class: "py-2",
                        if !cert_body_signal.read().is_empty() {
                            span { class: "text-sm",
                                {t!("preview.bio_zertifizierung", body = cert_body_signal.read()).to_string()}
                            }
                        } else {
                            span { class: "badge badge-warning", {t!("preview.bio_zertifizierungsstelle").to_string()} }
                        }
                    }
                }
                }
            }

            // Bio/Knospe marketing hints: informational texts ABOUT the label,
            // deliberately outside the white card so users see they are not part
            // of the physical label (Testing 25.06.2026).
            div { class: "mx-4",
                // TD-1 Stufe 3: the whole hint section is one function of the
                // typed verdicts. Which hints show, and why, is now readable as
                // a pair of matches instead of eleven independent flag checks.
                {
                    let v = verdicts.0();
                    let alternative_marking = conditionals.is_set(keys::ALTERNATIVE_MARKING_ALLOWED);
                    rsx! {
                        // Bio-V tri-state «Rezeptur prüfen».
                        match (&v.bio_check, &v.bio) {
                            (Some(CheckState::Pending), _) => rsx! {
                                Hint { text: t!("bio_hints.bio_check_pending").to_string() }
                            },
                            (Some(CheckState::Ok), bio) => rsx! {
                                Hint { text: t!("bio_hints.marketing_allowed").to_string() }
                                // DEC-4: only truthful when every agricultural
                                // ingredient is organic.
                                if alternative_marking {
                                    Hint { text: t!("bio_hints.alternative_marking").to_string() }
                                }
                                // Monoprodukt aus Umstellbetrieb: mandatory hint (Zeile 7).
                                if matches!(bio, Some(BioVerdict::Allowed { umstellung_mono: true })) {
                                    Hint { text: t!("bio_hints.umstellbetrieb_mono").to_string() }
                                }
                            },
                            (Some(CheckState::Failed), bio) => rsx! {
                                WarningHint { text: t!("bio_hints.bio_check_failed").to_string() }
                                // The verdict's reasons name what exactly blocks «Bio».
                                if let Some(BioVerdict::NotAllowed { reasons }) = bio {
                                    Hint { text: t!("bio_hints.marketing_not_allowed").to_string() }
                                    if reasons.contains(&BioBlockReason::ExceptionOver5Percent) {
                                        Hint { text: t!("bio_hints.erlaubte_ausnahme_ueber_5_prozent").to_string() }
                                    }
                                    // DEC-7: undeclared non-bio ingredient.
                                    if reasons.contains(&BioBlockReason::UndeclaredNonBio) {
                                        Hint { text: t!("bio_hints.bio_nicht_deklarierte_zutat").to_string() }
                                    }
                                }
                            },
                            (None, _) => rsx! {},
                        }
                        // Knospe tri-state «Rezeptur prüfen».
                        match (&v.knospe_check, &v.knospe) {
                            (Some(CheckState::Pending), _) => rsx! {
                                Hint { text: t!("bio_hints.knospe_check_pending").to_string() }
                            },
                            (Some(CheckState::Ok), _) => rsx! {
                                Hint { text: t!("bio_hints.knospe_check_ok").to_string() }
                                if alternative_marking {
                                    Hint { text: t!("bio_hints.alternative_marking").to_string() }
                                }
                            },
                            (Some(CheckState::Failed), knospe) => rsx! {
                                WarningHint { text: t!("bio_hints.knospe_check_failed").to_string() }
                                // DEC-8: name the 5% cap when that is the reason.
                                if let Some(KnospeVerdict::NoLogo { reasons }) = knospe {
                                    if reasons.contains(&KnospeBlockReason::ExceptionOver5Percent) {
                                        Hint { text: t!("bio_hints.knospe_erlaubte_ausnahme_ueber_5_prozent").to_string() }
                                    }
                                }
                            },
                            (None, _) => rsx! {},
                        }
                    }
                }
            }
            } // end if disclaimer_accepted
            {
                let disclaimer_bg = if disclaimer_accepted() { "bg-success/30" } else { "bg-base-200" };
                let disclaimer_text_class = if disclaimer_accepted() { "text-sm line-clamp-2" } else { "text-sm" };
                rsx! {
                    div { class: "mx-4 mt-4 p-4 {disclaimer_bg} border border-base-300 rounded-lg",
                        label { class: "flex items-start gap-3 cursor-pointer",
                            input {
                                class: "checkbox mt-1",
                                r#type: "checkbox",
                                checked: disclaimer_accepted(),
                                oninput: move |evt: FormEvent| {
                                    disclaimer_context.write().accepted = evt.checked();
                                },
                            }
                            span { class: "{disclaimer_text_class}",
                                {t!("disclaimer.text").to_string().nl2br()}
                            }
                        }
                    }
                }
            }
        }
    }
}
