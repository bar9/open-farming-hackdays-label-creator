// use dioxus::prelude::*;
// use strum::IntoEnumIterator;
// use strum_macros::EnumIter;
//
// #[derive(Props, Clone, PartialEq)]
// pub struct AmountTypeSelectProps {
//     amount_type: Signal<AmountType>
// }
//
// #[derive(Clone, PartialEq, EnumIter)]
// pub enum AmountType {
//     Weight, Volume
// }
//
// impl From<String> for AmountType {
//     fn from(value: String) -> Self {
//         if value == "volume" {
//             AmountType::Volume
//         } else {
//             AmountType::Weight
//         }
//     }
// }
//
// pub fn AmountTypeSelect(mut props: AmountTypeSelectProps) -> Element {
//     rsx! {
//         select {
//             oninput: move |evt| props.amount_type.set(evt.data.value().into()),
//             class: "select bg-white select-bordered w-full max-w-xs",
//             option {selected: true, value: "weight", "Gewicht"}
//             option {value: "volume", "Volumen"}
//         }
//     }
// }