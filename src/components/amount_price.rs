use std::cmp::PartialEq;
use dioxus::prelude::*;
use rust_i18n::t;
use crate::components::{FieldGroup1, FieldGroup2, FormField, TextInput};

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

#[derive(Clone, Copy)]
pub enum Amount {
    Single(usize),
    Double(usize, usize)
}

impl Amount {
    fn get_value_tuple(self) -> (usize, usize) {
        match self {
            Amount::Single(v) => (v, 0),
            Amount::Double(v1, v2) => (v1, v2)
        }
    }
}


#[derive(Clone, Copy)]
pub enum Price {
    Single(usize),
    Double(usize, usize)
}

impl Price {
    fn get_value_tuple(self) -> (usize, usize) {
        match self {
            Price::Single(v) => (v, 0),
            Price::Double(v1, v2) => (v1, v2)
        }
    }
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
pub fn AmountPrice (mut props: AmountPriceProps) -> Element {

    let mut has_abtropfgewicht = use_signal(|| false );

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
        FieldGroup2 {
            // label: t!("label.gewichtUndPreis"),
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

                    if *props.amount_type.read() == AmountType::Weight {
                        option {value: "mg", "mg"}
                        option {value: "g", "g"}
                        option {value: "kg", "kg"}
                    } else {
                        option {value: "ml", "ml"}
                        option {value: "cl", "cl"}
                        option {value: "l", "l"}
                    }
                }
            }
            if *props.amount_type.read() == AmountType::Weight {
                if has_abtropfgewicht() {
                    FormField {
                        label: t!("label.nettogewicht"),
                        help: Some((t!("help.nettogewicht")).into()),
                        input {
                            class: "input bg-white input-bordered w-full",
                            r#type: "text",
                            placeholder: t!("placeholder.nettogewicht").as_ref(),
                            value: "{props.amount.read().get_value_tuple().0}",
                            // oninput: move |evt| *props.amount.write() = Amount::Single(evt.data.value().parse().unwrap())
                        }
                    }
                    div {
                        class: "relative",
                        FormField {
                            label: t!("label.abtropfgewicht"),
                            help: Some((t!("help.abtropfgewicht")).into()),
                            input {
                                class: "input bg-white input-bordered w-full",
                                r#type: "text",
                                placeholder: t!("placeholder.abtropfgewicht").as_ref(),
                                value: "{props.amount.read().get_value_tuple().1}",
                                // oninput: move |evt| *props.amount.write() = Amount::Double(evt.data.value().parse().unwrap(), (*props.amount.read()).0, )
                                oninput: move |_| {}
                            }
                        }
                        label { class: "btn btn-circle swap bordered :w\
                        swap-rotate absolute right-0 bottom-0",
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
                        input {
                            class: "input bg-white input-bordered w-full",
                            r#type: "text",
                            placeholder: t!("placeholder.gewicht").as_ref(),
                            value: "{props.amount.read().get_value_tuple().0}",
                            // oninput: move |evt| *props.amount.write() = Amount::Double(evt.data.value().parse().unwrap(), (*props.amount.read()).0, )
                            oninput: move |_| {}
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
                    input {
                        class: "input bg-white input-bordered w-full",
                        r#type: "text",
                        placeholder: t!("placeholder.volumen").as_ref(),
                        value: "{props.amount.read().get_value_tuple().0}",
                        // oninput: move |evt| *props.amount.write() = Amount::Double(evt.data.value().parse().unwrap(), (*props.amount.read()).0, )
                        oninput: move |_| {}
                    }
                }
            }
        }
        FieldGroup2{
            if *props.amount_type.read() == AmountType::Weight {
                FormField {
                    help: Some((t!("help.preisProX")).into()),
                    label: t!("label.preisProX"),
                    input {
                        class: "input bg-white input-bordered w-full",
                        r#type: "text",
                        placeholder: "4.00 CHF",
                        value: "{props.price.read().get_value_tuple().0}",
                        // oninput: move |evt| *props.amount.write() = Amount::Double(evt.data.value().parse().unwrap(), (*props.amount.read()).0, )
                        oninput: move |_| {}
                    }
                }
                FormField {
                    help: Some((t!("help.preisTotal")).into()),
                    label: t!("label.preisTotal"),
                    input {
                        class: "input bg-white input-bordered w-full",
                        r#type: "text",
                        placeholder: "12.00 CHF",
                        value: "{props.price.read().get_value_tuple().1}",
                        // oninput: move |evt| *props.amount.write() = Amount::Double(evt.data.value().parse().unwrap(), (*props.amount.read()).0, )
                        oninput: move |_| {}
                    }
                }
            } else {
                FormField {
                    help: Some((t!("help.preisProX")).into()),
                    label: t!("label.preisProX"),
                    input {
                        class: "input bg-white input-bordered w-full",
                        r#type: "text",
                        placeholder: "4.00 CHF",
                        value: "{props.price.read().get_value_tuple().0}",
                        // oninput: move |evt| *props.amount.write() = Amount::Double(evt.data.value().parse().unwrap(), (*props.amount.read()).0, )
                        oninput: move |_| {}
                    }
                }
                FormField {
                    help: Some((t!("help.preisTotal")).into()),
                    label: t!("label.preisTotal"),
                    input {
                        class: "input bg-white input-bordered w-full",
                        r#type: "text",
                        placeholder: "12.00 CHF",
                        value: "{props.price.read().get_value_tuple().1}",
                        // oninput: move |evt| *props.amount.write() = Amount::Double(evt.data.value().parse().unwrap(), (*props.amount.read()).0, )
                        oninput: move |_| {}
                    }
                }
            }
        }
    }
}