use crate::components::icons;
use crate::components::FieldGroup2;
use crate::components::FormField;
use dioxus::prelude::*;
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::str::FromStr;

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug, Default)]
pub enum AmountType {
    #[default]
    Weight,
    Volume,
}

/// Reference quantity the unit price is stated for: per 100 g/ml for the small
/// units, per 1 kg/l for the large ones. Three components derive this, so the
/// table lives in one place.
pub fn base_factor(amount_type: &AmountType, weight_unit: &str, volume_unit: &str) -> usize {
    match (amount_type, weight_unit, volume_unit) {
        (AmountType::Weight, "mg", _) => 100,
        (AmountType::Weight, "g", _) => 100,
        (AmountType::Weight, "kg", _) => 1,
        (AmountType::Volume, _, "ml") => 100,
        (AmountType::Volume, _, "cl") => 100,
        (AmountType::Volume, _, "l") => 1,
        (_, _, _) => 1,
    }
}

/// The unit that goes with the amount: the weight unit or the volume one.
pub fn display_unit(amount_type: &AmountType, weight_unit: &str, volume_unit: &str) -> String {
    match amount_type {
        AmountType::Weight => weight_unit.to_string(),
        AmountType::Volume => volume_unit.to_string(),
    }
}

/// The declared quantity of the pack.
///
/// `net` is the Nettogewicht/-volumen, `drained` the optional Abtropfgewicht.
/// Both are stored in the unit picked in the form (g/kg, ml/l, ...), not in a
/// canonical one.
///
/// This used to be an enum with `Single`/`Double` variants. That encoded "is the
/// second field filled in" twice — once in the variant and once in the `Option`
/// — and the two could disagree: `Double(Some(x), None)` and `Single(Some(x))`
/// mean the same thing, but the label preview only handled the latter and
/// dropped the weight for the former.
#[derive(Clone, Copy, PartialEq, Serialize, Debug, Default)]
pub struct Amount {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drained: Option<usize>,
}

/// The price of the pack.
///
/// `unit` is the Grundpreis (per `base_factor` units, so per 100 g or per kg),
/// `total` the Gesamtpreis for the whole pack. Both in Rappen. Same story as
/// `Amount`: this was a `Single`/`Double` enum.
#[derive(Clone, Copy, PartialEq, Serialize, Debug, Default)]
pub struct Price {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<usize>,
}

/// Reads both the current shape and the `Single`/`Double` one that shared and
/// printed links still carry. Serialization always writes the current shape.
///
/// Shipped links are permanent: a Kurz-Link can sit on a printed label, so a
/// link written years ago must keep resolving. Dropping the legacy keys here
/// would silently reset the whole form to defaults, because a single field that
/// fails to deserialize takes the entire `Form` with it.
#[derive(Deserialize, Default)]
struct PairCompat {
    #[serde(default)]
    net: Option<usize>,
    #[serde(default)]
    drained: Option<usize>,
    #[serde(default)]
    unit: Option<usize>,
    #[serde(default)]
    total: Option<usize>,
    #[serde(default, rename = "Single")]
    single: Option<Option<usize>>,
    #[serde(default, rename = "Double")]
    double: Option<(Option<usize>, Option<usize>)>,
}

impl PairCompat {
    /// The two slots in order, whichever shape they arrived in.
    fn slots(self) -> (Option<usize>, Option<usize>) {
        if let Some(first) = self.single {
            (first, None)
        } else if let Some((first, second)) = self.double {
            (first, second)
        } else if self.net.is_some() || self.drained.is_some() {
            (self.net, self.drained)
        } else {
            (self.unit, self.total)
        }
    }
}

impl<'de> Deserialize<'de> for Amount {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let (net, drained) = PairCompat::deserialize(d)?.slots();
        Ok(Amount { net, drained })
    }
}

impl<'de> Deserialize<'de> for Price {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let (unit, total) = PairCompat::deserialize(d)?.slots();
        Ok(Price { unit, total })
    }
}

/// Amounts that count as a standard pack size, for which Swiss law does not
/// require a Grundpreis alongside the total price.
const EINHEITSGROESSEN: [usize; 4] = [1, 100, 250, 500];

impl Amount {
    /// Is this one of the standard pack sizes?
    pub fn is_einheitsgroesse(&self) -> bool {
        EINHEITSGROESSEN.contains(&self.net.unwrap_or(0))
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AmountPriceProps {
    amount_type: Signal<AmountType>,
    weight_unit: Signal<String>,
    volume_unit: Signal<String>,
    amount: Signal<Amount>,
    price: Signal<Price>,
    /// Sachbezeichnung names eggs, so the pack is declared by count: the
    /// Grundpreis field becomes «Anzahl Eier» in «Stück» and Abtropfgewicht
    /// does not apply (DEC-13).
    #[props(default = false)]
    is_egg_pack: bool,
    /// Number of eggs, used only in egg mode.
    egg_count: Signal<Option<usize>>,
}

/// Is responsible for reactively rendering amount & price fields
/// takes all state flat as signals from the app state
pub fn AmountPrice(props: AmountPriceProps) -> Element {
    let mut has_abtropfgewicht = use_signal(|| false);
    let amount_type = props.amount_type;
    let weight_unit = props.weight_unit;
    let volume_unit = props.volume_unit;
    let mut amount = props.amount;
    let price = props.price;
    let mut is_pristine = use_signal(|| true);

    // Switching to an egg Sachbezeichnung hides the Abtropfgewicht field
    // (DEC-13). Drop any value already entered, otherwise it would linger in
    // the model and keep printing on the label with no field to clear it.
    use_effect(move || {
        if props.is_egg_pack && amount().drained.is_some() {
            amount.with_mut(|a| a.drained = None);
        }
    });
    let invalid_class = use_memo(move || {
        if is_pristine() {
            ""
        } else {
            "invalid:bg-red-50"
        }
    });

    let get_base_factor = use_memo(move || {
        base_factor(
            &amount_type.read(),
            weight_unit.read().as_str(),
            volume_unit.read().as_str(),
        )
    });

    // With both prices known the amount follows from them, so the field is
    // filled in and locked rather than asked for twice.
    let calculated_amount = use_memo(move || match (price().unit, price().total) {
        (Some(unit_price), Some(total_price)) if unit_price > 0 => (
            true,
            ((total_price as f64 / unit_price as f64) * get_base_factor() as f64) as usize,
        ),
        _ => (false, 0),
    });

    let calculated_total_price = use_memo(move || {
        let net = amount().net.unwrap_or(0);
        match price().unit {
            Some(unit_price) if net > 0 => (
                true,
                (unit_price as f64 * (net as f64 / get_base_factor() as f64)) as usize,
            ),
            _ => (false, 0),
        }
    });

    let calculated_unit_price = use_memo(move || {
        let net = amount().net.unwrap_or(0);
        match price().total {
            Some(total_price) if net > 0 => (
                true,
                (total_price as f64 / (net as f64 / get_base_factor() as f64)) as usize,
            ),
            _ => (false, 0),
        }
    });

    let get_unit = use_memo(move || {
        display_unit(
            &amount_type.read(),
            weight_unit.read().as_str(),
            volume_unit.read().as_str(),
        )
    });

    let get_base_factor_and_unit = use_memo(move || match get_base_factor() {
        1 => rsx!("{get_unit()}"),
        _ => rsx!("{get_base_factor()} {get_unit()}"),
    });

    let get_base_factor_and_unit_string = use_memo(move || match get_base_factor() {
        1 => get_unit(),
        _ => format!("{} {}", get_base_factor(), get_unit()),
    });

    let is_einheitsgroesse = use_memo(move || amount().is_einheitsgroesse());

    let mut einheitsgroesse_input = use_signal(|| display_money(props.price.read().unit));
    let mut price_input_0 = use_signal(|| display_money(props.price.read().unit));
    let mut price_input_1 = use_signal(|| display_money(props.price.read().total));

    fn set_amount_type(new_amount_type: String, mut amount_type: Signal<AmountType>) {
        match new_amount_type.as_str() {
            "volumen" => {
                amount_type.set(AmountType::Volume);
            }
            "gewicht" => {
                amount_type.set(AmountType::Weight);
            }
            _ => panic!("illegal amount_type"),
        };
    }

    fn set_unit(
        new_unit: String,
        amount_type: Signal<AmountType>,
        mut weight_unit: Signal<String>,
        mut volume_unit: Signal<String>,
    ) {
        if *amount_type.read() == AmountType::Weight {
            weight_unit.set(new_unit);
        } else {
            volume_unit.set(new_unit);
        }
    }

    fn set_net_amount(new_amount: String, mut amount: Signal<Amount>) {
        let val = new_amount.parse().ok();
        amount.with_mut(|a| a.net = val);
    }

    fn set_drained_amount(new_amount: String, mut amount: Signal<Amount>) {
        let val = new_amount.parse().ok();
        amount.with_mut(|a| a.drained = val);
    }

    fn display_money(cents: Option<usize>) -> String {
        match cents {
            None => String::new(),
            Some(x) => format!("{:.2}", x as f64 / 100.0),
        }
    }

    /// Parse a price field: empty clears it, a comma is accepted as the decimal
    /// separator, and anything unparseable leaves the old value alone rather
    /// than silently zeroing a price that goes on a label.
    fn parse_money(input: &str) -> Result<Option<usize>, ()> {
        if input.is_empty() {
            return Ok(None);
        }
        match f64::from_str(&input.replace(',', ".")) {
            Ok(parsed) => Ok(Some((parsed * 100.0) as usize)),
            Err(_) => Err(()),
        }
    }

    fn set_unit_price(input: String, mut price: Signal<Price>) {
        if let Ok(cents) = parse_money(&input) {
            price.with_mut(|p| p.unit = cents);
        }
    }

    fn set_total_price(input: String, mut price: Signal<Price>) {
        if let Ok(cents) = parse_money(&input) {
            price.with_mut(|p| p.total = cents);
        }
    }

    rsx! {
        FieldGroup2 {
            FormField {
                label: t!("label.mengenart").to_string(),
                required: true,
                select {
                    oninput: move |evt| set_amount_type(evt.data.value(), props.amount_type),
                    class: "select w-full max-w-xs select-bordered bg-base-200",
                    option {selected: *props.amount_type.read() == AmountType::Weight, value: "gewicht", {t!("label.gewicht").to_string()}}
                    option {selected: *props.amount_type.read() == AmountType::Volume, value: "volumen", {t!("label.volumen").to_string()}}
                }
            }
            FormField {
                label: t!("label.einheit").to_string(),
                required: true,
                select {
                    oninput: move |evt| set_unit(evt.data.value(), props.amount_type, props.weight_unit, props.volume_unit),
                    class: "select w-full max-w-xs select-bordered bg-base-200 focus:bg-white",

                    if *props.amount_type.read() == AmountType::Weight {
                        option {selected: *props.weight_unit.read() == "mg", value: "mg", "mg"}
                        option {selected: *props.weight_unit.read() == "g", value: "g", "g"}
                        option {selected: *props.weight_unit.read() == "kg", value: "kg", "kg"}
                    } else {
                        option {selected: *props.volume_unit.read() == "ml", value: "ml", "ml"}
                        option {selected: *props.volume_unit.read() == "cl", value: "cl", "cl"}
                        option {selected: *props.volume_unit.read() == "l", value: "l", "l"}
                    }
                }
            }
            if *props.amount_type.read() == AmountType::Weight {
                if has_abtropfgewicht() && !props.is_egg_pack {
                    FormField {
                        required: true,
                        label: t!("label.nettogewicht").to_string(),
                        help: Some(t!("help.nettogewicht").to_string()),
                        div {
                            class: "flex flex-row items-center gap-2",
                            input {
                                class: "input w-1/2 input-bordered bg-base-200 {invalid_class}",
                                r#type: "number",
                                placeholder: "300",
                                min: "0",
                                required: true,
                                disabled: calculated_amount().0,
                                value: if calculated_amount().0 {"{calculated_amount().1}"} else {props.amount.read().net.map(|v| v.to_string()).unwrap_or_default()},
                                oninput: move |evt| set_net_amount(evt.data.value(), props.amount),
                                onblur: move |_evt| is_pristine.set(true)
                            }
                            span {
                                class: "badge",
                                "{props.weight_unit}"
                            }
                        }
                    }
                    div {
                        class: "relative",
                        FormField {
                            label: t!("label.abtropfgewicht").to_string(),
                            help: Some(t!("help.abtropfgewicht").to_string()),
                            div {
                                class: "flex flex-row items-center gap-2",
                                input {
                                    class: "input input-bordered bg-base-200 w-1/2",
                                    r#type: "number",
                                    placeholder: "200",
                                    value: props.amount.read().drained.map(|v| v.to_string()).unwrap_or_default(),
                                    oninput: move |evt| set_drained_amount(evt.data.value(), props.amount)
                                }
                                span {
                                    class: "badge",
                                    "{props.weight_unit}"
                                }
                            }
                        }
                        label { class: "btn btn-circle swap bordered swap-rotate absolute right-0 bottom-0",
                            input {
                                r#type: "checkbox",
                                checked: has_abtropfgewicht(),
                                oninput: move |evt| {
                                    has_abtropfgewicht.set(evt.checked());
                                    if !evt.checked() {
                                        // Clear abtropfgewicht when hiding the field
                                        amount.with_mut(|a| a.drained = None);
                                    }
                                }
                            }
                            icons::DashedX {}
                        }
                    }
                } else {
                    FormField {
                        label: t!("label.gewicht").to_string(),
                        required: true,
                        help: Some(t!("help.gewicht").to_string()),
                        div {
                            class: "flex flex-row items-center gap-2",
                            input {
                                class: "input w-1/2 input-bordered bg-base-200 {invalid_class}",
                                r#type: "number",
                                placeholder: "300",
                                min: "0",
                                required: true,
                                disabled: calculated_amount().0,
                                value: if calculated_amount().0 {"{calculated_amount().1}"} else {props.amount.read().net.map(|v| v.to_string()).unwrap_or_default()},
                                oninput: move |evt| set_net_amount(evt.data.value(), props.amount),
                                onblur: move |_evt| is_pristine.set(false)
                            }
                            span {
                                class: "badge",
                                "{props.weight_unit}"
                            }
                        }
                    }
                    // Abtropfgewicht does not apply to a pack of eggs (DEC-13),
                    // so the toggle that reveals the field is hidden too.
                    if !props.is_egg_pack {
                        FormField {
                            label: t!("label.abtropfgewicht").to_string(),
                            help: Some(t!("help.abtropfgewicht").to_string()),
                            label { class: "btn btn-circle swap swap-rotate",
                                input {
                                    r#type: "checkbox",
                                    checked: has_abtropfgewicht(),
                                    oninput: move |evt| {
                                        has_abtropfgewicht.set(evt.checked());
                                        if !evt.checked() {
                                            // Clear abtropfgewicht when hiding the field
                                            amount.with_mut(|a| a.drained = None);
                                        }
                                    }
                                }
                                icons::DashedPlus {}
                            }
                        }
                    }
                }
            } else {
                FormField {
                    label: t!("label.volumen").to_string(),
                    help: Some(t!("help.volumen").to_string()),
                    required: true,
                    div {
                        class: "flex flex-row items-center gap-2",
                        input {
                            class: "input w-1/2 input-bordered bg-base-200 {invalid_class}",
                            r#type: "number",
                            placeholder: "500",
                            min: "0",
                            required: true,
                            disabled: calculated_amount().0,
                            value: if calculated_amount().0 {"{calculated_amount().1}"} else {props.amount.read().net.map(|v| v.to_string()).unwrap_or_default()},
                            oninput: move |evt| set_net_amount(evt.data.value(), props.amount),
                            onblur: move |_evt| is_pristine.set(false)
                        }
                        span {
                            class: "badge",
                            "{props.volume_unit}"
                        }
                    }
                }
            }
        }
        FieldGroup2{
            if props.is_egg_pack {
                // DEC-13: eggs are sold by the piece, so the Grundpreis slot
                // carries the count instead of a price per 100 g.
                FormField {
                    help: Some(t!("help.anzahlEier").to_string()),
                    label: t!("label.anzahlEier").to_string(),
                    div {
                        class: "flex flex-row items-center gap-2",
                        input {
                            class: "input input-bordered bg-base-200 w-1/2",
                            r#type: "number",
                            min: "0",
                            step: "1",
                            placeholder: "6",
                            value: (*props.egg_count.read()).map(|v| v.to_string()).unwrap_or_default(),
                            oninput: move |evt| {
                                let raw = evt.data.value();
                                let mut egg_count = props.egg_count;
                                if raw.trim().is_empty() {
                                    egg_count.set(None);
                                } else if let Ok(parsed) = usize::from_str(raw.trim()) {
                                    egg_count.set(Some(parsed));
                                }
                            }
                        }
                        span {
                            class: "badge",
                            {t!("units.stueck").to_string()}
                        }
                    }
                }
                // Only the Grundpreis slot is repurposed; the Detailpreis of the
                // pack stays available and unchanged.
                FormField {
                    help: Some(t!("help.preisTotal").to_string()),
                    label: t!("label.preisTotal").to_string(),
                    div{
                        class: "flex flex-row items-center gap-2",
                        input {
                            class: "input input-bordered bg-base-200 w-1/2",
                            r#type: "number",
                            step: "any",
                            placeholder: "12.00",
                            value: price_input_1(),
                            oninput: move |evt| price_input_1.set(evt.data.value()),
                            onblur: move |_evt| {set_total_price(price_input_1(), props.price); price_input_1.set(display_money(props.price.read().total));}
                        }
                        span {
                            class: "badge",
                            {t!("units.chf").to_string()}
                        }
                    }
                }
            } else if is_einheitsgroesse() {
                FormField {
                    help: Some(t!("help.preisProEinheit").to_string()),
                    label: t!("label.preisProEinheit").to_string(),
                    div {
                        class: "flex flex-row items-center gap-2",
                        input {
                            class: "input input-bordered bg-base-200 w-1/2",
                            r#type: "number",
                            step: "any",
                            placeholder: "4.00",
                            value: einheitsgroesse_input(),
                            oninput: move |evt| einheitsgroesse_input.set(evt.data.value()),
                            onblur: move |_evt| {set_unit_price(einheitsgroesse_input(), props.price); einheitsgroesse_input.set(display_money(props.price.read().unit));},
                        }
                        span {
                            class: "badge",
                            {t!("units.chf").to_string()}
                        }
                    }
                }
            } else {
                FormField {
                    help: Some(t!("help.preisProX", unit = get_base_factor_and_unit_string()).to_string()),
                    label: t!("label.preisProX").to_string(),
                    div {
                        class: "flex flex-row items-center gap-2",
                        input {
                            class: "input input-bordered bg-base-200 w-1/2",
                            r#type: "number",
                            step: "any",
                            placeholder: "4.00",
                            disabled: calculated_unit_price().0,
                            value: if calculated_unit_price().0 {display_money(Some(calculated_unit_price().1))} else {price_input_0()},
                            oninput: move |evt| price_input_0.set(evt.data.value()),
                            onblur: move |_evt| {set_unit_price(price_input_0(), props.price); price_input_0.set(display_money(props.price.read().unit));}
                        }
                        span {
                            class: "badge",
                            {t!("units.chfPro").to_string()}
                            {get_base_factor_and_unit()}
                        }
                    }
                }
                FormField {
                    help: Some(t!("help.preisTotal").to_string()),
                    label: t!("label.preisTotal").to_string(),
                    div{
                        class: "flex flex-row items-center gap-2",
                        input {
                            class: "input input-bordered bg-base-200 w-1/2",
                            r#type: "number",
                            step: "any",
                            placeholder: "12.00",
                            disabled: calculated_total_price().0,
                            value: if calculated_total_price().0 {display_money(Some(calculated_total_price().1))} else {price_input_1()},
                            oninput: move |evt| price_input_1.set(evt.data.value()),
                            onblur: move |_evt| {set_total_price(price_input_1(), props.price); price_input_1.set(display_money(props.price.read().total));}
                        }
                        span {
                            class: "badge",
                            {t!("units.chf").to_string()}
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The unit price is stated per 100 g/ml for the small units and per 1 kg/l
    // for the large ones. Getting this wrong misprices the label, and the table
    // is now shared by three components, so it is pinned here.
    #[test]
    fn base_factor_is_100_for_small_units_and_1_for_large_ones() {
        assert_eq!(base_factor(&AmountType::Weight, "mg", "ml"), 100);
        assert_eq!(base_factor(&AmountType::Weight, "g", "ml"), 100);
        assert_eq!(base_factor(&AmountType::Weight, "kg", "ml"), 1);
        assert_eq!(base_factor(&AmountType::Volume, "g", "ml"), 100);
        assert_eq!(base_factor(&AmountType::Volume, "g", "cl"), 100);
        assert_eq!(base_factor(&AmountType::Volume, "g", "l"), 1);
    }

    // An unknown unit must not silently scale the price by 100.
    #[test]
    fn base_factor_falls_back_to_1_for_unknown_units() {
        assert_eq!(base_factor(&AmountType::Weight, "stk", "ml"), 1);
        assert_eq!(base_factor(&AmountType::Volume, "g", "dl"), 1);
    }

    #[test]
    fn display_unit_follows_the_amount_type() {
        assert_eq!(display_unit(&AmountType::Weight, "kg", "l"), "kg");
        assert_eq!(display_unit(&AmountType::Volume, "kg", "l"), "l");
    }
}

#[cfg(test)]
mod link_compat_tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    // A Kurz-Link can sit on a printed label, so links written before Amount and
    // Price became structs must keep resolving. These tests read the exact query
    // strings the old enum shape produced.
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Form {
        #[serde(default)]
        amount: Amount,
        #[serde(default)]
        price: Price,
    }

    fn parse(query: &str) -> Form {
        serde_qs::Config::new()
            .max_depth(20)
            .deserialize_str::<Form>(query)
            .expect("query string should deserialize")
    }

    fn write(form: &Form) -> String {
        serde_qs::Config::new()
            .max_depth(20)
            .serialize_string(form)
            .expect("form should serialize")
    }

    #[test]
    fn legacy_single_links_still_resolve() {
        let form = parse("amount[Single]=250&price[Single]=450");
        assert_eq!(form.amount.net, Some(250));
        assert_eq!(form.amount.drained, None);
        assert_eq!(form.price.unit, Some(450));
        assert_eq!(form.price.total, None);
    }

    #[test]
    fn legacy_double_links_still_resolve() {
        let form = parse("amount[Double][0]=250&amount[Double][1]=200&price[Double][0]=450&price[Double][1]=1125");
        assert_eq!(form.amount.net, Some(250));
        assert_eq!(form.amount.drained, Some(200));
        assert_eq!(form.price.unit, Some(450));
        assert_eq!(form.price.total, Some(1125));
    }

    // The half-filled state: the Abtropfgewicht field was revealed but left
    // empty, so the old form wrote an empty second slot.
    #[test]
    fn legacy_double_links_with_an_empty_second_slot_still_resolve() {
        let form = parse("amount[Double][0]=250&amount[Double][1]");
        assert_eq!(form.amount.net, Some(250));
        assert_eq!(form.amount.drained, None);
    }

    #[test]
    fn a_link_without_amount_or_price_falls_back_to_empty() {
        let form = parse("");
        assert_eq!(form.amount, Amount::default());
        assert_eq!(form.price, Price::default());
    }

    #[test]
    fn current_links_round_trip() {
        let form = Form {
            amount: Amount {
                net: Some(250),
                drained: Some(200),
            },
            price: Price {
                unit: Some(450),
                total: Some(1125),
            },
        };
        assert_eq!(parse(&write(&form)), form);
    }

    // Empty fields are left out of the query string entirely rather than
    // written as empty keys, keeping shared links short.
    #[test]
    fn empty_fields_are_omitted_from_the_query_string() {
        let written = write(&Form {
            amount: Amount {
                net: Some(250),
                drained: None,
            },
            price: Price::default(),
        });
        assert_eq!(written, "amount[net]=250");
    }
}
