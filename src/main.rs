#![allow(non_snake_case)]

use dioxus::prelude::*;
use crate::components::*;
use crate::layout::ThemeLayout;
use crate::model::{IngredientItem};

mod layout;

mod model;
mod components;

const _STYLE: &str = manganis::mg!(file("public/tailwind.css"));

fn main() {
    launch(app);
}

fn app() -> Element {
    let ingredients: Signal<Vec<IngredientItem>> = use_signal(|| Vec::new());
    let product_title = use_signal(|| String::new());
    let product_subtitle = use_signal(|| String::new());
    let additional_info = use_signal(|| String::new());
    let storage_info = use_signal(|| String::new());
    let date_prefix = use_signal(|| String::new());
    let date = use_signal(|| String::new());
    let production_country = use_signal(|| String::from("Schweiz"));
    let net_weight = use_signal(|| String::new());
    let drained_weight = use_signal(|| String::new());
    let producer_name = use_signal(|| String::new());
    let producer_address = use_signal(|| String::new());
    let producer_zip = use_signal(|| String::new());
    let producer_city = use_signal(|| String::new());
    let price_per_100 = use_signal(|| String::new());
    let total_price = use_signal(|| String::new());


    rsx! {
        ThemeLayout {
            div {
                class: "h-screen flex",
                div {
                    class: "flex-1 flex overflow-hidden",
                    div {
                        class: "flex-1 overflow-y-scroll",
                        div { class: "flex flex-col gap-6 p-8 pb-12",
                            h1 { class: "text-4xl text-accent mb-4", "LMK Creator | Lebensmittelkennzeichnung" }
                            FormField {
                                label: "Produktname",
                                help: rsx!{
                                    p { "Markennamen unmittelbar mit Sachbezeichnung ergänzen, gemäss Art. 6 LIV"}
                                },
                                TextInput {
                                    placeholder: "Produktname (optional)",
                                    bound_value: product_title
                                }
                            }
                            FormField {
                                label: "Sachbezeichnung",
                                help: rsx!{
                                    p {"Verkehrsübliche Bezeichnung oder eine beschreibende Bezeichnung, zur Illustration passend"}
                                },
                                TextInput {
                                    placeholder: "Produktname / Produktbeschrieb - z.B. Haferriegel mit Honig",
                                    bound_value: product_subtitle
                                }
                            }
                            SeparatorLine {}
                            FormField {
                                label: "Zutaten",
                                help: rsx!{
                                    p{"Wenn die gesuchte Zutat nicht im Dropdown vorhanden ist, können Sie sie im Textfeld eingeben."}
                                    p{"Falls in Sachbezeichnungen (Bild, Wort) hervorgehoben oder für das Lebensmittel charakteristisch, hat die Angabe in Massenprozenten (Anteil verwendeter Rohware im fertigen Produkt) zu erfolgen. (Art. 12 und Anhang 7 LIV)"}
                                    p{"Produktionsland des Lebensmittels oder Rohstoffes (Art. 15 und 6 LIV): Deklarationspflicht sofern es nicht aus der Adresse oder der Sachbezeichnungen ersichtlich ist. Die Herkunft von Zutaten muss angegeben werden, wenn die Zutat 50 % des Enderzeugnisses oder mehr ausmacht (bei Zutaten tierischer Herkunft: ab 20 %) und die Herkunft des Rohstoffes von jenen des Produktes abweicht."}
                                },
                                IngredientsTable {ingredients: ingredients}
                            }
                            SeparatorLine {}
                            FieldGroup2 {
                                FormField {
                                    label: "Datumseingabe",
                                    help: rsx!{
                                        p{ "Lebensmittel, die kühl gehalten werden müssen -> zu verbrauchen bis.."}
                                        hr{}
                                        p{ "Übrige Lebensmittel: mindestens haltbar bis.."}
                                    },
                                    DateInput {
                                        date_value: date,
                                        date_prefix: date_prefix
                                    }

                                }
                                FormField { label: "Zusatzinformationen",
                                    help: rsx! {
                                        table{ class: "table",
                                            tbody{
                                                tr{td{"Tiefkühlprodukte Bemerkung anbringen: Im Tiefkühler bei -18°C gut verpackt lagern, nach dem Auftauen nicht wieder einfrieren"}}
                                                tr{td{"Gebrauchsanleitung / Zubereitung / Verwendungstipps"}}
                                                tr{td{"Anweisung Aufbewahrung z.B. nach dem Öffnen gekühlt aufbewahren"}}
                                                tr{td{"Alkoholgehalt, Koffeingehalt"}}
                                                tr{td{"Freiwillige Angaben (vegetarisch/allergenfrei etc..)"}}
                                                tr{td{"Nährwertbezogene angaben, Health claims müssen bestimmte Anforderungen erfüllen  -> link anhängen"}}
                                                tr{td{"Nährwertdeklaration: ist obligatorisch (Energiewert, Gehalt an Fett, gesättigte Fettsäuren, Kohlenhydrate, Zucker, Eiweiss und Salz). Ausgenommen von der Nährwertdeklaration sind Lebensmittel nach Anhang 9 (z.B. handwerklich hergestellte Lebensmittel, die lokal abgegeben werden, Radius unter 50 km)"}}
                                            }
                                        }
                                    },
                                    TextareaInput {
                                        placeholder: "Haftungsausschlüsse, Kann Spuren von Nüssen enthalten, Gebrauchsanleitung",
                                        rows: "4",
                                        bound_value: additional_info
                                    }
                                }
                            }
                            FieldGroup2 {
                                FormField {
                                    label: "Aufbewahrungshinweis",
                                    help: rsx!{
                                        br{}
                                        br{}
                                        div {
                                            table { class: "table",
                                                thead {
                                                    tr {
                                                        th { "Aufbewahrungsarten" }
                                                        th { "Bedingungen" }
                                                    }
                                                }
                                                tbody {
                                                    tr {
                                                        td { "Tiefgekühlt" }
                                                        td { "mind. -18°C" }
                                                    }
                                                    tr {
                                                        td { "Gekühlt" }
                                                        td { "+2 - +5°C (8°C Käse, zubereitete Spesen)" }
                                                    }
                                                    tr {
                                                        td { "Kühl lagern" }
                                                        td { "bei Temperaturen bis +15°C" }
                                                    }
                                                    tr {
                                                        td { "Bei Zimmertemperatur lagern" }
                                                        td { "+18 - +22°C" }
                                                    }
                                                    tr {
                                                        td { "Trocken Lagern" }
                                                        td { "an einem trockenen Ort bei max. 70% lagern" }
                                                    }
                                                    tr {
                                                        td { "Lichtgeschützt" }
                                                        td { "vor direktem Lichteinfall" }
                                                    }
                                                    tr {
                                                        td { "Feucht und kühl lagern" }
                                                        td { "bei +6 – 15°C und 70-90% Luftfeuchtigkeit lagern" }
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    TextareaInput{
                                        rows: "2",
                                        placeholder: "z.B. dunkel und kühl bei max. 5°C lagern",
                                        bound_value: storage_info
                                    }
                                }
                                FormField { label: "Produktionsland",
                                    TextareaInput {
                                        rows: "2",
                                        placeholder: "Schweiz",
                                        bound_value: production_country
                                    }
                                }
                            }
                            FieldGroup2 {
                                FormField {
                                    label: "Nettogewicht",
                                    help: rsx!{
                                        p{"Die Aufschrift muss folgende Mindesthöhe haben:"}
                                        ul{
                                            li{"bei einer Nennfüllmenge von mehr als 1000 g oder 100 cl: mindestens 6 mm"}
                                            li{"bei einer Nennfüllmenge von mehr als 200 g oder 20 cl bis 1000 g oder 100 cl: mindestens 4 mm"}
                                            li{"bei einer Nennfüllmenge von mehr als 50 g oder 5 cl bis 200 g oder 20 cl: mindestens 3 mm"}
                                            li{"bei einer Nennfüllmenge von 50 g und darunter oder 5 cl und darunter: mindestens 2 mm"}
                                        }
                                    },
                                    TextInput {
                                        bound_value: net_weight,
                                        placeholder: "300g"
                                    }
                                }
                                FormField { label: "Abtropfgewicht",
                                    TextInput {
                                        bound_value: drained_weight,
                                        placeholder: "125g"
                                    }
                                }
                            }
                            SeparatorLine {}
                            FieldGroup1 { label: "Adresse",
                                FormField {
                                    label: "Vorname / Name / Firma",
                                    TextInput { bound_value: producer_name, placeholder: "Hans Muster AG" }
                                }
                                div { class: "grid grid-cols-3 gap-4",
                                    FormField {
                                    label: "Adresse",
                                        TextInput { bound_value: producer_address, placeholder: "Teststrasse 1" }
                                    }
                                    FormField {
                                        label: "PLZ",
                                        TextInput { bound_value: producer_zip, placeholder: "CH-4001" }
                                    }
                                    FormField {
                                        label: "Ort",
                                        TextInput { bound_value: producer_city, placeholder: "Basel" }
                                    }
                                }
                            }
                            SeparatorLine {}
                            FieldGroup1 { label: "Preis",
                                div { class: "grid grid-cols-2 gap-4",
                                    FormField {
                                        label: "Preis pro 100g",
                                        TextInput {
                                            placeholder: "4.00 CHF",
                                            bound_value: price_per_100
                                        }
                                    }
                                    FormField {
                                        label: "Preis Total",
                                        TextInput {
                                            placeholder: "12.00 CHF",
                                            bound_value: total_price
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            LabelPreview {
                ingredients: ingredients,
                product_title : product_title,
                product_subtitle : product_subtitle,
                additional_info : additional_info,
                storage_info : storage_info,
                production_country : production_country,
                date_prefix : date_prefix,
                date : date,
                net_weight : net_weight,
                drained_weight : drained_weight,
                producer_name : producer_name,
                producer_address : producer_address,
                producer_zip : producer_zip,
                producer_city : producer_city,
                price_per_100: price_per_100,
                total_price: total_price
            }
        }
    }
}
