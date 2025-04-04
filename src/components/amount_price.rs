use std::cmp::PartialEq;
use std::str::FromStr;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use crate::FormField;
use crate::FieldGroup2;
use rust_i18n::t;

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub enum AmountType {
    Weight, Volume
}

impl Default for AmountType {
    fn default() -> Self {
        AmountType::Weight
    }
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum Amount {
    Single(Option<usize>),
    Double(Option<usize>, Option<usize>)
}

impl Amount {
    fn get_value_tuple(self) -> (Option<usize>, Option<usize>) {
        match self {
            Amount::Single(v) => (v, None),
            Amount::Double(v1, v2) => (v1, v2)
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Price {
    Single(Option<usize>),
    Double(Option<usize>, Option<usize>)
}

impl Price {
    fn get_value_tuple(self) -> (Option<usize>, Option<usize>) {
        match self {
            Price::Single(v) => (v, None),
            Price::Double(v1, v2) => (v1, v2)
        }
    }
}

impl Default for Price {
    fn default() -> Self {
        Price::Single(None)
    }
}

impl Default for Amount {
    fn default() -> Self {
        Amount::Single(None)
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AmountPriceProps {
    amount_type: Signal<AmountType>,
    weight_unit: Signal<String>,
    volume_unit: Signal<String>,
    amount: Signal<Amount>,
    price: Signal<Price>,
}

/// Is responsible for reactively rendering amount & price fields
/// takes all state flat as signals from the app state
pub fn AmountPrice (props: AmountPriceProps) -> Element {

    let mut has_abtropfgewicht = use_signal(|| false );
    let amount_type = props.amount_type.clone();
    let weight_unit = props.weight_unit.clone();
    let volume_unit = props.volume_unit.clone();
    let amount = props.amount.clone();
    let price = props.price.clone();



    let get_base_factor = use_memo(move || {
        match (&*amount_type.read(), &*weight_unit.read().as_str(), &*volume_unit.read().as_str()) {
            (AmountType::Weight, "mg", _) => {100_usize}
            (AmountType::Weight, "g", _) => {100_usize}
            (AmountType::Weight, "kg", _) => {1_usize}
            (AmountType::Volume, _, "ml") => {100_usize}
            (AmountType::Volume, _, "cl") => {100_usize}
            (AmountType::Volume, _, "l") => {1_usize}
            (_, _, _) => 1_usize
        }
    });

    let calculated_amount = use_memo(move || {
        match price() {
            Price::Double(Some(unit_price), Some(total_price)) =>
                (true, ((total_price as f64 / unit_price as f64) * get_base_factor() as f64) as usize),
            _ => (false, 0)
        }
    });

    let calculated_total_price = use_memo(move || {
        let net_amount = match amount() {
            Amount::Single(Some(x)) => x,
            Amount::Double(Some(x), _) => x,
            _ => 0
        };
        if net_amount == 0 {
            return (false, 0);
        }
        match price() {
            Price::Double(Some(unit_price), Some(_)) =>
                (true, (unit_price as f64 * (net_amount as f64 / get_base_factor() as f64)) as usize),
            Price::Single(Some(unit_price)) =>
                (true, (unit_price as f64 * (net_amount as f64 / get_base_factor() as f64)) as usize),
            _ => (false, 0)
        }
    });

    let calculated_unit_price = use_memo(move || {
        let net_amount = match amount() {
            Amount::Single(Some(x)) => x,
            Amount::Double(Some(x), _) => x,
            _ => 0
        };
        if net_amount == 0 {
            return (false, 0);
        }
        match price() {
            Price::Double(_, Some(total_price)) =>
                (true, (total_price as f64 / (net_amount as f64 / get_base_factor() as f64)) as usize),
            _ => (false, 0)
        }
    });

    let get_unit = use_memo(move || {
        match (&*amount_type.read(), &*weight_unit.read(), &*volume_unit.read()) {
            (AmountType::Weight, unit, _) => unit.clone(),
            (AmountType::Volume, _, unit) => unit.clone()
        }
    });

    let get_base_factor_and_unit = use_memo(move || {
        match get_base_factor() {
            1 => rsx!("{get_unit()}"),
            _ => rsx!("{get_base_factor()} {get_unit()}")
        }
    });

    let is_einheitsgroesse = use_memo(move || {
        match amount() {
            Amount::Single(x) => {
                return [1_usize, 100_usize, 250_usize, 500_usize].contains(&x.unwrap_or(0))
            }
            Amount::Double(x, _) => {
                return [1_usize, 100_usize, 250_usize, 500_usize].contains(&x.unwrap_or(0))
            }
        }
    });

    let mut einheitsgroesse_input = use_signal(|| display_money(props.price.read().get_value_tuple().0));
    let mut price_input_0 = use_signal(|| display_money(props.price.read().get_value_tuple().0));
    let mut price_input_1 = use_signal(|| display_money(props.price.read().get_value_tuple().1));

    fn set_amount_type(new_amount_type: String, mut amount_type: Signal<AmountType>) {
        match new_amount_type.as_str() {
            "volumen" => {
                amount_type.set(AmountType::Volume);
            },
            "gewicht" => {
                amount_type.set(AmountType::Weight);
            },
            _ => panic!("illegal amount_type")
        };
    }

    fn set_unit(new_unit: String, amount_type: Signal<AmountType>, mut weight_unit: Signal<String>, mut volume_unit: Signal<String>) {
        if *amount_type.read() == AmountType::Weight {
            weight_unit.set(new_unit);
        } else {
            volume_unit.set(new_unit);
        }
    }

    fn set_amount_single(new_amount: String, mut amount: Signal<Amount>) {
        let val = new_amount.parse().ok();
        amount.set(Amount::Single(val));
    }

    fn set_amount_0(new_amount: String, mut amount: Signal<Amount>) {
        let old_amount = amount();
        let val = new_amount.parse().ok();
        match old_amount {
            Amount::Single(_) => {
                amount.set(Amount::Single(val));
            }
            Amount::Double(_, x) => {
                amount.set(Amount::Double(val, x));
            }
        }
    }

    fn set_amount_1(new_amount: String, mut amount: Signal<Amount>) {
        let old_amount = amount();
        let val = new_amount.parse().ok();
        match old_amount {
            Amount::Single(x) => {
                amount.set(Amount::Double(x, val));
            }
            Amount::Double(x, _) => {
                amount.set(Amount::Double(x, val));
            }
        }
    }

    fn display_money(cents: Option<usize>) -> String {
        match cents {
            None => String::new(),
            Some(x) => format!("{:.2}", x as f64 / 100.0)
        }
    }

    fn set_price_0(input: String, mut price: Signal<Price>) {
        let old_price = price();
        if input.is_empty() {
            match old_price {
                Price::Single(_) => {
                    price.set(Price::Single(None));
                }
                Price::Double(_, old) => {
                    price.set(Price::Double(None, old));
                }
            }
        } else {
            let cleaned = input.replace(',', "."); // Handle potential comma input
            if let Some(parsed) = f64::from_str(&cleaned).ok() {
                let cents = (parsed * 100.0) as usize; // Ensure rounding
                match old_price {
                    Price::Single(_) => {
                        price.set(Price::Single(Some(cents))); // Assuming Price::Single(i64)
                    }
                    Price::Double(_, old) => {
                        price.set(Price::Double(Some(cents), old)); // Assuming Price::Single(i64)
                    }
                }
            } else {
                price.set(old_price);
            }
        }
    }

    fn set_price_1(input: String, mut price: Signal<Price>) {
        let old_price = price();
        if input.is_empty() {
            match old_price {
                Price::Single(old) => {
                    price.set(Price::Double(old, None));
                }
                Price::Double(old, _) => {
                    price.set(Price::Double(old, None));
                }
            }
        } else {
            let cleaned = input.replace(',', "."); // Handle potential comma input
            if let Some(parsed) = f64::from_str(&cleaned).ok() {
                let cents = (parsed * 100.0) as usize; // Ensure rounding
                match old_price {
                    Price::Single(old) => {
                        price.set(Price::Double(old, Some(cents))); // Assuming Price::Single(i64)
                    }
                    Price::Double(old, _) => {
                        price.set(Price::Double(old, Some(cents))); // Assuming Price::Single(i64)
                    }
                }
            } else {
                price.set(old_price);
            }
        }
    }

    fn set_price_single(input: String, mut price: Signal<Price>) {
        if input.is_empty(){
            price.set(Price::Single(None));
        } else {
            let cleaned = input.replace(',', "."); // Handle potential comma input
            if let Some(parsed) = f64::from_str(&cleaned).ok() {
                let cents = (parsed * 100.0) as usize; // Ensure rounding
                price.set(Price::Single(Some(cents)));
            }
        }
        // }
    }

    rsx! {
        // pre {
        //     "weight unit: {weight_unit()}"
        // }
        // pre {
        //     "props : weight unit: {(props.weight_unit)()}"
        // }
        // pre {
        //     "volume unit: {volume_unit()}"
        // }
        // pre {
        //     "props : volume unit: {(props.volume_unit)()}"
        // }
        FieldGroup2 {
            // label: t!("label.gewichtUndPreis"),
            FormField {
                label: t!("label.mengenart"),
                select {
                    oninput: move |evt| set_amount_type(evt.data.value(), props.amount_type),
                    class: "select bg-white select-bordered w-full max-w-xs",
                    option {selected: *props.amount_type.read() == AmountType::Weight, value: "gewicht", "Gewicht"}
                    option {selected: *props.amount_type.read() == AmountType::Volume, value: "volumen", "Volumen"}
                }
            }
            FormField {
                label: t!("label.einheit"),
                select {
                    oninput: move |evt| set_unit(evt.data.value(), props.amount_type, props.weight_unit, props.volume_unit),
                    class: "select bg-white select-bordered w-full max-w-xs",

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
                if has_abtropfgewicht() {
                    FormField {
                        label: t!("label.nettogewicht"),
                        help: Some((t!("help.nettogewicht")).into()),
                        div {
                            class: "flex flex-row items-center gap-2",
                            input {
                                class: "input bg-white input-bordered w-1/2",
                                r#type: "number",
                                placeholder: "300",
                                disabled: calculated_amount().0,
                                value: if calculated_amount().0 {"{calculated_amount().1}"} else {props.amount.read().get_value_tuple().0.map(|v| v.to_string()).unwrap_or_default()},
                                oninput: move |evt| set_amount_0(evt.data.value(), props.amount)
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
                            label: t!("label.abtropfgewicht"),
                            help: Some((t!("help.abtropfgewicht")).into()),
                            div {
                                class: "flex flex-row items-center gap-2",
                                input {
                                    class: "input bg-white input-bordered w-1/2",
                                    r#type: "number",
                                    placeholder: "200",
                                    value: props.amount.read().get_value_tuple().1.map(|v| v.to_string()).unwrap_or_default(),
                                    oninput: move |evt| set_amount_1(evt.data.value(), props.amount)
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
                                oninput: move |evt| has_abtropfgewicht.set(evt.checked())
                            }
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                width: "32",
                                "viewBox": "0 0 512 512",
                                height: "32",
                                class: "swap-off fill-current",
                                path { d: "M64,384H448V341.33H64Zm0-106.67H448V234.67H64ZM64,128v42.67H448V128Z" }
                            }
                            svg {
                                width: "32",
                                height: "32",
                                xmlns: "http://www.w3.org/2000/svg",
                                "viewBox": "0 0 512 512",
                                class: "swap-on fill-current",
                                polygon { points: "400 145.49 366.51 112 256 222.51 145.49 112 112 145.49 222.51 256 112 366.51 145.49 400 256 289.49 366.51 400 400 366.51 289.49 256 400 145.49" }
                            }
                        }
                    }
                } else {
                    FormField {
                        label: t!("label.gewicht"),
                        help: Some((t!("help.gewicht")).into()),
                        div {
                            class: "flex flex-row items-center gap-2",
                            input {
                                class: "input bg-white input-bordered w-1/2",
                                r#type: "number",
                                placeholder: "300",
                                disabled: calculated_amount().0,
                                value: if calculated_amount().0 {"{calculated_amount().1}"} else {props.amount.read().get_value_tuple().0.map(|v| v.to_string()).unwrap_or_default()},
                                oninput: move |evt| set_amount_single(evt.data.value(), props.amount)
                            }
                            span {
                                class: "badge",
                                "{props.weight_unit}"
                            }
                        }
                    }
                    label { class: "btn btn-circle swap swap-rotate",
                        input {
                            r#type: "checkbox",
                            checked: has_abtropfgewicht(),
                            oninput: move |evt| has_abtropfgewicht.set(evt.checked())
                        }
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "32",
                            "viewBox": "0 0 512 512",
                            height: "32",
                            class: "swap-off fill-current",
                            path { d: "M64,384H448V341.33H64Zm0-106.67H448V234.67H64ZM64,128v42.67H448V128Z" }
                        }
                        svg {
                            width: "32",
                            height: "32",
                            xmlns: "http://www.w3.org/2000/svg",
                            "viewBox": "0 0 512 512",
                            class: "swap-on fill-current",
                            polygon { points: "400 145.49 366.51 112 256 222.51 145.49 112 112 145.49 222.51 256 112 366.51 145.49 400 256 289.49 366.51 400 400 366.51 289.49 256 400 145.49" }
                        }
                    }
                }
            } else {
                FormField {
                    label: t!("label.volumen"),
                    help: Some((t!("help.volumen")).into()),
                    div {
                        class: "flex flex-row items-center gap-2",
                        input {
                            class: "input bg-white input-bordered w-1/2",
                            r#type: "number",
                            placeholder: "500",
                            disabled: calculated_amount().0,
                            value: if calculated_amount().0 {"{calculated_amount().1}"} else {props.amount.read().get_value_tuple().0.map(|v| v.to_string()).unwrap_or_default()},
                            oninput: move |evt| set_amount_single(evt.data.value(), props.amount)
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
            if is_einheitsgroesse() {
                FormField {
                    help: Some((t!("help.preisProEinheit")).into()),
                    label: t!("label.preisProEinheit"),
                    div {
                        class: "flex flex-row items-center",
                        input {
                            class: "input bg-white input-bordered w-1/2",
                            r#type: "number",
                            step: "any",
                            placeholder: "4.00",
                            //value: display_money(props.price.read().get_value_tuple().0),
                            value: einheitsgroesse_input(),
                            oninput: move |evt| einheitsgroesse_input.set(evt.data.value()),
                            onblur: move |_evt| {set_price_single(einheitsgroesse_input(), props.price); einheitsgroesse_input.set(display_money(props.price.read().get_value_tuple().0));},
                        }
                        span {
                            class: "badge",
                            "CHF"
                        }
                    }
                }
            } else {
                FormField {
                    help: Some((t!("help.preisProX")).into()),
                    label: t!("label.preisProX"),
                    div {
                        class: "flex flex-row items-center",
                        input {
                            class: "input bg-white input-bordered w-1/2",
                            r#type: "number",
                            step: "any",
                            placeholder: "4.00",
                            disabled: calculated_unit_price().0,
                            value: if calculated_unit_price().0 {display_money(Some(calculated_unit_price().1))} else {price_input_0()},
                            oninput: move |evt| price_input_0.set(evt.data.value()),
                            onblur: move |_evt| {set_price_0(price_input_0(), props.price); price_input_0.set(display_money(props.price.read().get_value_tuple().0));}
                        }
                        span {
                            class: "badge",
                            "CHF pro "
                            {get_base_factor_and_unit()}
                        }
                    }
                }
                FormField {
                    help: Some((t!("help.preisTotal")).into()),
                    label: t!("label.preisTotal"),
                    div{
                        class: "flex flex-row items-center",
                        input {
                            class: "input bg-white input-bordered w-1/2",
                            r#type: "number",
                            step: "any",
                            placeholder: "12.00",
                            disabled: calculated_total_price().0,
                            value: if calculated_total_price().0 {display_money(Some(calculated_total_price().1))} else {price_input_1()},
                            oninput: move |evt| price_input_1.set(evt.data.value()),
                            onblur: move |_evt| {set_price_1(price_input_1(), props.price); price_input_1.set(display_money(props.price.read().get_value_tuple().1));}
                        }
                        span {
                            class: "badge",
                            "CHF"
                        }
                    }
                }
            }
        }
    }
}