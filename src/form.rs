// use std::fmt::{Debug, Formatter};
// use dioxus::prelude::Signal;
// use serde::{Deserialize, Serialize};
// use crate::form::CompGewicht::Einheitsgewicht;
// use crate::form::MengeUndPreis::GewichtUndPreis;
//
// type Cents = usize;
//
// #[derive(Serialize, Deserialize)]
// pub enum MengeUndPreis {
//     GewichtUndPreis {
//         einheit: Gewichtseinheit,
//         gewicht: CompGewicht,
//     },
//     VolumenUndPreis {
//         einheit: Volumeneinheit,
//         volumen: CompVolumen,
//     },
// }
//
// enum Gewichtseinheit {
//     Mg, G, Kg
// }
//
// enum Volumeneinheit {
//     Ml, Cl, L
// }
//
// enum CompGewicht {
//     Einheitsgewicht {
//         menge: Cents,
//         preis_pro_einheit: Cents,
//     },
//     Gewicht {
//         menge: Cents,
//         preis_pro_basiseinheit: Cents,
//         total_preis: Cents
//     },
//     Abtropfgewicht {
//         total: Cents,
//         abtropf: Cents,
//         preis_pro_basiseinheit: Cents,
//         total_preis: Cents
//     }
// }
//
// enum CompVolumen {
//     Einheitsvolumen {
//         menge: usize,
//         preis_pro_einheit: usize,
//     },
//     Volumen {
//         menge: usize,
//         preis_pro_basiseinheit: usize,
//         total_preis: usize
//     },
// }
//
// // traits for forms, signals?
//
//
// impl MengeUndPreis {
//     //
// }
//
// impl Default for MengeUndPreis {
//     fn default() -> Self {
//         GewichtUndPreis {
//             einheit: Gewichtseinheit::G,
//             gewicht: Einheitsgewicht {
//                 menge: 0,
//                 preis_pro_einheit: 0
//             }
//         }
//     }
// }
//
// struct FormControl<T: Debug> {
//     value: Signal<T>,
//     disabled: Signal<bool>,
//     visible: Signal<bool>
// }
//
// impl<T: Debug + 'static> Debug for FormControl<T> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         todo!()
//     }
// }
//
// impl<T: Debug + 'static> FormControlTrait for FormControl<T> {
//     fn is_disabled(&self) -> bool {
//         self.disabled()
//     }
//
//     fn is_visible(&self) -> bool {
//         self.visible()
//     }
// }
//
// trait FormControlTrait: Debug {
//     fn is_disabled(&self) -> bool;
//     fn is_visible(&self) -> bool;
//
//     fn get_key(&)
// }
//
// // how to link this to my form?
// impl MengeUndPreis {
//     fn getFormControls() -> Vec<FormControl<...>>
// }
