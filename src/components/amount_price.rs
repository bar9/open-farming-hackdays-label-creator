use std::cmp::PartialEq;
use dioxus::prelude::*;
use rust_i18n::t;
use crate::components::{FieldGroup1, FormField};

#[derive(PartialEq)]
pub enum AmountType {
    Weight, Volume
}

#[derive(PartialEq)]
pub enum WeightType {
    EinheitsGewicht,
    Gewicht,
    Abtropfgewicht
}

pub enum VolumeType {
    Einheitsvolumen,
    Volumen
}

pub enum Amount {
    Single(usize),
    Double(usize, usize)
}

pub enum Price {
    Single(usize),
    Double(usize, usize)
}

#[derive(Props, Clone, PartialEq)]
pub struct AmountPriceProps {
    amount_type: Signal<AmountType>,
    weight_unit: Signal<String>,
    volume_unit: Signal<String>,
    amount: Signal<Amount>,
    price: Signal<Price>,
    weight_type: Signal<Option<WeightType>>,
    volume_type: Signal<Option<VolumeType>>,
}

/// Is responsible for reactively rendering amount & price fields
/// takes all state flat as signals from the app state
pub fn AmountPrice (props: AmountPriceProps) -> Element {

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

    rsx! {
        FieldGroup1 {
            label: t!("label.gewichtUndPreis"),
            FormField {
                label: t!("label.mengenart"),
                select {
                    oninput: move |evt| set_amount_type(evt.data.value(), props.amount_type),
                    class: "select bg-white select-bordered w-full max-w-xs",
                    option {selected: {*props.amount_type.read() == AmountType::Weight}, value: "gewicht", "Gewicht"}
                    option {selected: {*props.amount_type.read() == AmountType::Volume}, value: "volumen", "Volumen"}
                }
            }
            FormField {
                label: t!("label.einheit"),
                select {
                    oninput: move |evt| set_amount_type(evt.data.value(), props.amount_type),
                    class: "select bg-white select-bordered w-full max-w-xs",

                    option {selected: {*props.amount_type.read() == AmountType::Weight}, value: "mg", "mg"}
                    option {selected: {*props.amount_type.read() == AmountType::Weight}, value: "g", "g"}
                    option {selected: {*props.amount_type.read() == AmountType::Weight}, value: "kg", "kg"}
                    option {selected: {*props.amount_type.read() == AmountType::Volume}, value: "ml", "ml"}
                    option {selected: {*props.amount_type.read() == AmountType::Volume}, value: "cl", "cl"}
                    option {selected: {*props.amount_type.read() == AmountType::Volume}, value: "l", "l"}
                }
            }
        }
    }
}