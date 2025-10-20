use crate::components::*;
use crate::core::Ingredient;
use crate::model::{food_db, lookup_allergen, Country};
use crate::rules::RuleDef;
use crate::shared::{Conditionals, Validations};
use dioxus::prelude::*;
use rust_i18n::t;

// TODO: rework save/cancel (stateful modal):
// seems we already have many parts, only the writes via props.inredients.write() are to be delegated to a save() handler

#[derive(Props, Clone, PartialEq)]
pub struct IngredientDetailProps {
    ingredients: Signal<Vec<Ingredient>>,
    index: usize,
    #[props(default = false)]
    genesis: bool,
    rules: Memo<Vec<RuleDef>>,
}
pub fn IngredientDetail(mut props: IngredientDetailProps) -> Element {
    let index: usize;
    let mut ingredients: Signal<Vec<Ingredient>>;
    if props.genesis {
        ingredients = use_signal(|| vec![Ingredient::default()]);
        index = 0;
    } else {
        index = props.index;
        ingredients = props.ingredients;
    }
    let mut is_open = use_signal(|| false);
    let mut scale_together = use_signal(|| false);
    let mut amount_to_edit = use_signal(|| 0.);
    let mut is_composite = use_signal(|| {
        !ingredients
            .get(index)
            .unwrap()
            .clone()
            .sub_components
            .unwrap_or_default()
            .is_empty()
    });
    let mut is_namensgebend = use_signal(|| {
        ingredients
            .get(index)
            .unwrap()
            .is_namensgebend
            .unwrap_or(false)
    });
    let mut selected_origin = use_signal(|| ingredients.get(index).unwrap().origin.clone());

    let ingredient = ingredients.get(index).unwrap().clone();
    let old_ingredient = ingredients.get(index).unwrap().clone();
    let old_ingredient_2 = ingredients.get(index).unwrap().clone();
    let old_ingredient_3 = ingredients.get(index).unwrap().clone();
    let mut update_name = move |new_name: String| {
        ingredients.write()[index] = Ingredient {
            name: new_name.clone(),
            is_allergen: lookup_allergen(&new_name),
            ..old_ingredient.clone()
        };
    };

    let mut handle_genesis = move || {
        let mut new_ingredient = ingredients.get(index).unwrap().clone();
        new_ingredient.amount = amount_to_edit();
        new_ingredient.is_allergen = lookup_allergen(&new_ingredient.name);

        // Check if ingredient with same name and properties already exists (only for non-composite)
        if new_ingredient.sub_components.is_none()
            || new_ingredient.sub_components.as_ref().unwrap().is_empty()
        {
            let mut existing_ingredients = props.ingredients.write();
            if let Some(existing_index) = existing_ingredients.iter().position(|ing| {
                ing.name == new_ingredient.name
                    && ing.is_allergen == new_ingredient.is_allergen
                    && ing.is_namensgebend == new_ingredient.is_namensgebend
                    && (ing.sub_components.is_none()
                        || ing.sub_components.as_ref().unwrap().is_empty())
            }) {
                // Merge amounts instead of adding duplicate
                existing_ingredients[existing_index].amount += new_ingredient.amount;
            } else {
                existing_ingredients.push(new_ingredient);
            }
        } else {
            props.ingredients.write().push(new_ingredient);
        }

        ingredients = use_signal(|| vec![Ingredient::default()]);
        is_open.set(false);
    };

    let herkunft_path = format!("herkunft_benoetigt_{}", index);

    // Check for validation errors for this ingredient
    let validations_context = use_context::<Validations>();
    let has_validation_error = use_memo(move || {
        let validation_entries = (*validations_context.0.read()).clone();
        validation_entries.contains_key(&format!("ingredients[{}][origin]", index))
            || validation_entries.contains_key(&format!("ingredients[{}][amount]", index))
    });

    rsx! {
        if props.genesis {
            button {
                class: "btn btn-accent",
                onclick: move |_| is_open.toggle(),
                onkeydown: move |evt: KeyboardEvent| if evt.key() == Key::Escape { is_open.set(false); },
                "{t!(\"nav.hinzufuegen\")}"
            }
        } else {
            button {
                class: if *has_validation_error.read() {
                    "btn join-item btn-outline btn-error relative"
                } else {
                    "btn join-item btn-outline"
                },
                onclick: move |_| is_open.toggle(),
                onkeydown: move |evt: KeyboardEvent| if evt.key() == Key::Escape { is_open.set(false); },
                icons::ListDetail {}
                if *has_validation_error.read() {
                    span {
                        class: "absolute -top-2 -right-2 bg-error text-error-content rounded-full w-4 h-4 text-xs flex items-center justify-center",
                        "!"
                    }
                }
            }
        }
        if is_open() { div { class: "fixed inset-0 bg-black bg-opacity-50 backdrop-blur-md" } }
        dialog { open: "{is_open}", class: "modal",
            div { class: "modal-box bg-base-100",
                h3 { class: "font-bold text-lg", "{t!(\"label.zutatDetails\")}" }
                FormField {
                    label: t!("label.zutat"),
                    input {
                        list: "ingredients",
                        r#type: "flex",
                        placeholder: t!("placeholder.zutatName").as_ref(),
                        class: "input input-accent w-full",
                        oninput: move |evt| update_name(evt.data.value()),
                        value: "{ingredient.name}",
                        datalist { id: "ingredients",
                            for item in food_db().clone() {
                                option { value: "{item.0}" }
                            }
                        }
                    }
                }
                FormField {
                    label: t!("label.menge"),
                    ValidationDisplay {
                        paths: vec![
                            format!("ingredients[{}][amount]", index)
                        ],
                        input {
                            r#type: "number",
                            placeholder: "Menge",
                            class: "input input-accent w-full",
                            onchange: move |evt| {
                                if let Ok(amount) = evt.data.value().parse::<f64>() {
                                    amount_to_edit.set(amount);
                                }
                            },
                            value: "{ingredient.amount}",
                        }
                    }
                    if !props.genesis {
                        label { class: "label cursor-pointer",
                            input {
                                class: "checkbox",
                                r#type: "checkbox",
                                checked: "{scale_together}",
                                oninput: move |e| scale_together.set(e.value() == "true"),
                            }
                            span { class: "label-text",
                                "{t!(\"nav.verhaeltnisseBeibehalten\")}"
                            }
                        }
                        button {
                            class: "btn btn-accent",
                            onclick: move |_evt| {
                                let old_ingredient = ingredients.get(index).unwrap().clone();
                                if *scale_together.read() {
                                    let factor: f64 = (*amount_to_edit)()
                                        / old_ingredient.amount;
                                    let clonedIngredients = ingredients;
                                    for (key, elem) in clonedIngredients.iter().enumerate() {
                                        let old_ingredient = elem.clone();
                                        ingredients.write()[key] = Ingredient {
                                            amount: (elem.amount * factor),
                                            ..old_ingredient.clone()
                                        }
                                    }
                                } else {
                                    ingredients.write()[index] = Ingredient {
                                        amount: (*amount_to_edit)(),
                                        ..old_ingredient.clone()
                                    }
                                }
                            },
                            "{t!(\"nav.anpassen\")}"
                        }
                    }
                }

                br {}
                br {}

                FormField {
                    label: t!("label.zusammengesetzteZutat"),
                    help: Some((t!("help.zusammengesetzteZutaten")).into()),
                    label { class: "label cursor-pointer",
                        input {
                            class: "checkbox",
                            r#type: "checkbox",
                            checked: "{is_composite}",
                            oninput: move |e| is_composite.set(e.value() == "true"),
                        }
                        span { class: "label-text",
                            "{t!(\"label.zusammengesetzteZutat\")}"
                        }
                    }
                    if is_composite() {
                        SubIngredientsTable {
                            ingredients: ingredients,
                            index:  index
                        }
                    }
                }
                br {}
                ConditionalDisplay {
                    path: "namensgebende_zutat".to_string(),
                    FormField {
                        help: Some((t!("help.namensgebendeZutaten")).into()),
                        label: t!("label.namensgebendeZutat"),
                        label { class: "label cursor-pointer",
                            input {
                                class: "checkbox",
                                r#type: "checkbox",
                                checked: "{is_namensgebend}",
                                oninput: move |e| {
                                    is_namensgebend.set(e.value() == "true");
                                    ingredients.write()[index] = Ingredient {
                                        is_namensgebend: Some(e.value() == "true"),
                                        ..old_ingredient_2.clone()
                                    }
                                },
                            }
                            span { class: "label-text",
                                "{t!(\"label.namensgebendeZutat\")}"
                            }
                        }
                    }
                }
                {
                    // Get context access outside the memo to avoid hook-in-hook violation
                    let conditionals_context = use_context::<Conditionals>();

                    // Check if Bio/Knospe rules are active (always show origin field) or traditional conditional is set
                    let should_show_origin = use_memo(move || {
                        let rules = props.rules.read();
                        let has_bio_knospe = rules.iter().any(|rule|
                            *rule == RuleDef::Bio_Knospe_AlleZutatenHerkunft ||
                            *rule == RuleDef::Knospe_100_Percent_CH_NoOrigin ||
                            *rule == RuleDef::Knospe_90_99_Percent_CH_ShowOrigin
                        );

                        if has_bio_knospe {
                            true
                        } else {
                            // Fall back to traditional conditional check
                            let conditionals = conditionals_context.0.read();
                            *conditionals.get(&herkunft_path).unwrap_or(&false)
                        }
                    });

                    if should_show_origin() {
                        rsx! {
                            FormField {
                                label: "Herkunft",
                                ValidationDisplay {
                                    paths: vec![
                                        format!("ingredients[{}][origin]", index)
                                    ],
                                    select {
                                        class: "select select-bordered w-full",
                                        value: match selected_origin.read().as_ref() {
                                            Some(country) => format!("{:?}", country),
                                            None => "".to_string(),
                                        },
                                        onchange: move |e| {
                                            let country = match e.value().as_str() {
                                                "CH" => Country::CH,
                                                "EU" => Country::EU,
                                                "NoOriginRequired" => Country::NoOriginRequired,
                                                "AD" => Country::AD, "AE" => Country::AE, "AF" => Country::AF,
                                                "AG" => Country::AG, "AI" => Country::AI, "AL" => Country::AL,
                                                "AM" => Country::AM, "AO" => Country::AO, "AQ" => Country::AQ,
                                                "AR" => Country::AR, "AS" => Country::AS, "AT" => Country::AT,
                                                "AU" => Country::AU, "AW" => Country::AW, "AX" => Country::AX,
                                                "AZ" => Country::AZ, "BA" => Country::BA, "BB" => Country::BB,
                                                "BD" => Country::BD, "BE" => Country::BE, "BF" => Country::BF,
                                                "BG" => Country::BG, "BH" => Country::BH, "BI" => Country::BI,
                                                "BJ" => Country::BJ, "BL" => Country::BL, "BM" => Country::BM,
                                                "BN" => Country::BN, "BO" => Country::BO, "BQ" => Country::BQ,
                                                "BR" => Country::BR, "BS" => Country::BS, "BT" => Country::BT,
                                                "BV" => Country::BV, "BW" => Country::BW, "BY" => Country::BY,
                                                "BZ" => Country::BZ, "CA" => Country::CA, "CC" => Country::CC,
                                                "CD" => Country::CD, "CF" => Country::CF, "CG" => Country::CG,
                                                "CI" => Country::CI, "CK" => Country::CK, "CL" => Country::CL,
                                                "CM" => Country::CM, "CN" => Country::CN, "CO" => Country::CO,
                                                "CR" => Country::CR, "CU" => Country::CU, "CV" => Country::CV,
                                                "CW" => Country::CW, "CX" => Country::CX, "CY" => Country::CY,
                                                "CZ" => Country::CZ, "DE" => Country::DE, "DJ" => Country::DJ,
                                                "DK" => Country::DK, "DM" => Country::DM, "DO" => Country::DO,
                                                "DZ" => Country::DZ, "EC" => Country::EC, "EE" => Country::EE,
                                                "EG" => Country::EG, "EH" => Country::EH, "ER" => Country::ER,
                                                "ES" => Country::ES, "ET" => Country::ET, "FI" => Country::FI,
                                                "FJ" => Country::FJ, "FK" => Country::FK, "FM" => Country::FM,
                                                "FO" => Country::FO, "FR" => Country::FR, "GA" => Country::GA,
                                                "GB" => Country::GB, "GD" => Country::GD, "GE" => Country::GE,
                                                "GF" => Country::GF, "GG" => Country::GG, "GH" => Country::GH,
                                                "GI" => Country::GI, "GL" => Country::GL, "GM" => Country::GM,
                                                "GN" => Country::GN, "GP" => Country::GP, "GQ" => Country::GQ,
                                                "GR" => Country::GR, "GS" => Country::GS, "GT" => Country::GT,
                                                "GU" => Country::GU, "GW" => Country::GW, "GY" => Country::GY,
                                                "HK" => Country::HK, "HM" => Country::HM, "HN" => Country::HN,
                                                "HR" => Country::HR, "HT" => Country::HT, "HU" => Country::HU,
                                                "ID" => Country::ID, "IE" => Country::IE, "IL" => Country::IL,
                                                "IM" => Country::IM, "IN" => Country::IN, "IO" => Country::IO,
                                                "IQ" => Country::IQ, "IR" => Country::IR, "IS" => Country::IS,
                                                "IT" => Country::IT, "JE" => Country::JE, "JM" => Country::JM,
                                                "JO" => Country::JO, "JP" => Country::JP, "KE" => Country::KE,
                                                "KG" => Country::KG, "KH" => Country::KH, "KI" => Country::KI,
                                                "KM" => Country::KM, "KN" => Country::KN, "KP" => Country::KP,
                                                "KR" => Country::KR, "KW" => Country::KW, "KY" => Country::KY,
                                                "KZ" => Country::KZ, "LA" => Country::LA, "LB" => Country::LB,
                                                "LC" => Country::LC, "LI" => Country::LI, "LK" => Country::LK,
                                                "LR" => Country::LR, "LS" => Country::LS, "LT" => Country::LT,
                                                "LU" => Country::LU, "LV" => Country::LV, "LY" => Country::LY,
                                                "MA" => Country::MA, "MC" => Country::MC, "MD" => Country::MD,
                                                "ME" => Country::ME, "MF" => Country::MF, "MG" => Country::MG,
                                                "MH" => Country::MH, "MK" => Country::MK, "ML" => Country::ML,
                                                "MM" => Country::MM, "MN" => Country::MN, "MO" => Country::MO,
                                                "MP" => Country::MP, "MQ" => Country::MQ, "MR" => Country::MR,
                                                "MS" => Country::MS, "MT" => Country::MT, "MU" => Country::MU,
                                                "MV" => Country::MV, "MW" => Country::MW, "MX" => Country::MX,
                                                "MY" => Country::MY, "MZ" => Country::MZ, "NA" => Country::NA,
                                                "NC" => Country::NC, "NE" => Country::NE, "NF" => Country::NF,
                                                "NG" => Country::NG, "NI" => Country::NI, "NL" => Country::NL,
                                                "NO" => Country::NO, "NP" => Country::NP, "NR" => Country::NR,
                                                "NU" => Country::NU, "NZ" => Country::NZ, "OM" => Country::OM,
                                                "PA" => Country::PA, "PE" => Country::PE, "PF" => Country::PF,
                                                "PG" => Country::PG, "PH" => Country::PH, "PK" => Country::PK,
                                                "PL" => Country::PL, "PM" => Country::PM, "PN" => Country::PN,
                                                "PR" => Country::PR, "PS" => Country::PS, "PT" => Country::PT,
                                                "PW" => Country::PW, "PY" => Country::PY, "QA" => Country::QA,
                                                "RE" => Country::RE, "RO" => Country::RO, "RS" => Country::RS,
                                                "RU" => Country::RU, "RW" => Country::RW, "SA" => Country::SA,
                                                "SB" => Country::SB, "SC" => Country::SC, "SD" => Country::SD,
                                                "SE" => Country::SE, "SG" => Country::SG, "SH" => Country::SH,
                                                "SI" => Country::SI, "SJ" => Country::SJ, "SK" => Country::SK,
                                                "SL" => Country::SL, "SM" => Country::SM, "SN" => Country::SN,
                                                "SO" => Country::SO, "SR" => Country::SR, "SS" => Country::SS,
                                                "ST" => Country::ST, "SV" => Country::SV, "SX" => Country::SX,
                                                "SY" => Country::SY, "SZ" => Country::SZ, "TC" => Country::TC,
                                                "TD" => Country::TD, "TF" => Country::TF, "TG" => Country::TG,
                                                "TH" => Country::TH, "TJ" => Country::TJ, "TK" => Country::TK,
                                                "TL" => Country::TL, "TM" => Country::TM, "TN" => Country::TN,
                                                "TO" => Country::TO, "TR" => Country::TR, "TT" => Country::TT,
                                                "TV" => Country::TV, "TW" => Country::TW, "TZ" => Country::TZ,
                                                "UA" => Country::UA, "UG" => Country::UG, "UM" => Country::UM,
                                                "US" => Country::US, "UY" => Country::UY, "UZ" => Country::UZ,
                                                "VA" => Country::VA, "VC" => Country::VC, "VE" => Country::VE,
                                                "VG" => Country::VG, "VI" => Country::VI, "VN" => Country::VN,
                                                "VU" => Country::VU, "WF" => Country::WF, "WS" => Country::WS,
                                                "YE" => Country::YE, "YT" => Country::YT, "ZA" => Country::ZA,
                                                "ZM" => Country::ZM, "ZW" => Country::ZW,
                                                _ => return, // Don't update for empty selection
                                            };
                                            selected_origin.set(Some(country.clone()));
                                            ingredients.write()[index] = Ingredient {
                                                origin: Some(country),
                                                ..old_ingredient_3.clone()
                                            }
                                        },
                                        option { value: "", selected: selected_origin.read().is_none(), "Bitte wählen..." }
                                        option { value: "CH", selected: matches!(selected_origin.read().as_ref(), Some(Country::CH)), "Schweiz" }
                                        option { value: "EU", selected: matches!(selected_origin.read().as_ref(), Some(Country::EU)), "EU" }
                                        option { value: "NoOriginRequired", selected: matches!(selected_origin.read().as_ref(), Some(Country::NoOriginRequired)), "Keine Herkunftsangabe benötigt" }

                                        option { value: "AD", selected: matches!(selected_origin.read().as_ref(), Some(Country::AD)), "Andorra" }
                                        option { value: "AE", selected: matches!(selected_origin.read().as_ref(), Some(Country::AE)), "Vereinigte Arabische Emirate" }
                                        option { value: "AF", selected: matches!(selected_origin.read().as_ref(), Some(Country::AF)), "Afghanistan" }
                                        option { value: "AG", selected: matches!(selected_origin.read().as_ref(), Some(Country::AG)), "Antigua und Barbuda" }
                                        option { value: "AI", selected: matches!(selected_origin.read().as_ref(), Some(Country::AI)), "Anguilla" }
                                        option { value: "AL", selected: matches!(selected_origin.read().as_ref(), Some(Country::AL)), "Albanien" }
                                        option { value: "AM", selected: matches!(selected_origin.read().as_ref(), Some(Country::AM)), "Armenien" }
                                        option { value: "AO", selected: matches!(selected_origin.read().as_ref(), Some(Country::AO)), "Angola" }
                                        option { value: "AQ", selected: matches!(selected_origin.read().as_ref(), Some(Country::AQ)), "Antarktis" }
                                        option { value: "AR", selected: matches!(selected_origin.read().as_ref(), Some(Country::AR)), "Argentinien" }
                                        option { value: "AS", selected: matches!(selected_origin.read().as_ref(), Some(Country::AS)), "Amerikanisch-Samoa" }
                                        option { value: "AT", selected: matches!(selected_origin.read().as_ref(), Some(Country::AT)), "Österreich" }
                                        option { value: "AU", selected: matches!(selected_origin.read().as_ref(), Some(Country::AU)), "Australien" }
                                        option { value: "AW", selected: matches!(selected_origin.read().as_ref(), Some(Country::AW)), "Aruba" }
                                        option { value: "AX", selected: matches!(selected_origin.read().as_ref(), Some(Country::AX)), "Åland" }
                                        option { value: "AZ", selected: matches!(selected_origin.read().as_ref(), Some(Country::AZ)), "Aserbaidschan" }
                                        option { value: "BA", selected: matches!(selected_origin.read().as_ref(), Some(Country::BA)), "Bosnien und Herzegowina" }
                                        option { value: "BB", selected: matches!(selected_origin.read().as_ref(), Some(Country::BB)), "Barbados" }
                                        option { value: "BD", selected: matches!(selected_origin.read().as_ref(), Some(Country::BD)), "Bangladesch" }
                                        option { value: "BE", selected: matches!(selected_origin.read().as_ref(), Some(Country::BE)), "Belgien" }
                                        option { value: "BF", selected: matches!(selected_origin.read().as_ref(), Some(Country::BF)), "Burkina Faso" }
                                        option { value: "BG", selected: matches!(selected_origin.read().as_ref(), Some(Country::BG)), "Bulgarien" }
                                        option { value: "BH", selected: matches!(selected_origin.read().as_ref(), Some(Country::BH)), "Bahrain" }
                                        option { value: "BI", selected: matches!(selected_origin.read().as_ref(), Some(Country::BI)), "Burundi" }
                                        option { value: "BJ", selected: matches!(selected_origin.read().as_ref(), Some(Country::BJ)), "Benin" }
                                        option { value: "BL", selected: matches!(selected_origin.read().as_ref(), Some(Country::BL)), "Saint-Barthélemy" }
                                        option { value: "BM", selected: matches!(selected_origin.read().as_ref(), Some(Country::BM)), "Bermuda" }
                                        option { value: "BN", selected: matches!(selected_origin.read().as_ref(), Some(Country::BN)), "Brunei" }
                                        option { value: "BO", selected: matches!(selected_origin.read().as_ref(), Some(Country::BO)), "Bolivien" }
                                        option { value: "BQ", selected: matches!(selected_origin.read().as_ref(), Some(Country::BQ)), "Bonaire, Sint Eustatius und Saba" }
                                        option { value: "BR", selected: matches!(selected_origin.read().as_ref(), Some(Country::BR)), "Brasilien" }
                                        option { value: "BS", selected: matches!(selected_origin.read().as_ref(), Some(Country::BS)), "Bahamas" }
                                        option { value: "BT", selected: matches!(selected_origin.read().as_ref(), Some(Country::BT)), "Bhutan" }
                                        option { value: "BV", selected: matches!(selected_origin.read().as_ref(), Some(Country::BV)), "Bouvetinsel" }
                                        option { value: "BW", selected: matches!(selected_origin.read().as_ref(), Some(Country::BW)), "Botswana" }
                                        option { value: "BY", selected: matches!(selected_origin.read().as_ref(), Some(Country::BY)), "Belarus" }
                                        option { value: "BZ", selected: matches!(selected_origin.read().as_ref(), Some(Country::BZ)), "Belize" }
                                        option { value: "CA", selected: matches!(selected_origin.read().as_ref(), Some(Country::CA)), "Kanada" }
                                        option { value: "CC", selected: matches!(selected_origin.read().as_ref(), Some(Country::CC)), "Kokosinseln" }
                                        option { value: "CD", selected: matches!(selected_origin.read().as_ref(), Some(Country::CD)), "Demokratische Republik Kongo" }
                                        option { value: "CF", selected: matches!(selected_origin.read().as_ref(), Some(Country::CF)), "Zentralafrikanische Republik" }
                                        option { value: "CG", selected: matches!(selected_origin.read().as_ref(), Some(Country::CG)), "Republik Kongo" }
                                        option { value: "CI", selected: matches!(selected_origin.read().as_ref(), Some(Country::CI)), "Elfenbeinküste" }
                                        option { value: "CK", selected: matches!(selected_origin.read().as_ref(), Some(Country::CK)), "Cookinseln" }
                                        option { value: "CL", selected: matches!(selected_origin.read().as_ref(), Some(Country::CL)), "Chile" }
                                        option { value: "CM", selected: matches!(selected_origin.read().as_ref(), Some(Country::CM)), "Kamerun" }
                                        option { value: "CN", selected: matches!(selected_origin.read().as_ref(), Some(Country::CN)), "China" }
                                        option { value: "CO", selected: matches!(selected_origin.read().as_ref(), Some(Country::CO)), "Kolumbien" }
                                        option { value: "CR", selected: matches!(selected_origin.read().as_ref(), Some(Country::CR)), "Costa Rica" }
                                        option { value: "CU", selected: matches!(selected_origin.read().as_ref(), Some(Country::CU)), "Kuba" }
                                        option { value: "CV", selected: matches!(selected_origin.read().as_ref(), Some(Country::CV)), "Kap Verde" }
                                        option { value: "CW", selected: matches!(selected_origin.read().as_ref(), Some(Country::CW)), "Curaçao" }
                                        option { value: "CX", selected: matches!(selected_origin.read().as_ref(), Some(Country::CX)), "Weihnachtsinsel" }
                                        option { value: "CY", selected: matches!(selected_origin.read().as_ref(), Some(Country::CY)), "Zypern" }
                                        option { value: "CZ", selected: matches!(selected_origin.read().as_ref(), Some(Country::CZ)), "Tschechien" }
                                        option { value: "DE", selected: matches!(selected_origin.read().as_ref(), Some(Country::DE)), "Deutschland" }
                                        option { value: "DJ", selected: matches!(selected_origin.read().as_ref(), Some(Country::DJ)), "Dschibuti" }
                                        option { value: "DK", selected: matches!(selected_origin.read().as_ref(), Some(Country::DK)), "Dänemark" }
                                        option { value: "DM", selected: matches!(selected_origin.read().as_ref(), Some(Country::DM)), "Dominica" }
                                        option { value: "DO", selected: matches!(selected_origin.read().as_ref(), Some(Country::DO)), "Dominikanische Republik" }
                                        option { value: "DZ", selected: matches!(selected_origin.read().as_ref(), Some(Country::DZ)), "Algerien" }
                                        option { value: "EC", selected: matches!(selected_origin.read().as_ref(), Some(Country::EC)), "Ecuador" }
                                        option { value: "EE", selected: matches!(selected_origin.read().as_ref(), Some(Country::EE)), "Estland" }
                                        option { value: "EG", selected: matches!(selected_origin.read().as_ref(), Some(Country::EG)), "Ägypten" }
                                        option { value: "EH", selected: matches!(selected_origin.read().as_ref(), Some(Country::EH)), "Westsahara" }
                                        option { value: "ER", selected: matches!(selected_origin.read().as_ref(), Some(Country::ER)), "Eritrea" }
                                        option { value: "ES", selected: matches!(selected_origin.read().as_ref(), Some(Country::ES)), "Spanien" }
                                        option { value: "ET", selected: matches!(selected_origin.read().as_ref(), Some(Country::ET)), "Äthiopien" }
                                        option { value: "FI", selected: matches!(selected_origin.read().as_ref(), Some(Country::FI)), "Finnland" }
                                        option { value: "FJ", selected: matches!(selected_origin.read().as_ref(), Some(Country::FJ)), "Fidschi" }
                                        option { value: "FK", selected: matches!(selected_origin.read().as_ref(), Some(Country::FK)), "Falklandinseln" }
                                        option { value: "FM", selected: matches!(selected_origin.read().as_ref(), Some(Country::FM)), "Mikronesien" }
                                        option { value: "FO", selected: matches!(selected_origin.read().as_ref(), Some(Country::FO)), "Färöer" }
                                        option { value: "FR", selected: matches!(selected_origin.read().as_ref(), Some(Country::FR)), "Frankreich" }
                                        option { value: "GA", selected: matches!(selected_origin.read().as_ref(), Some(Country::GA)), "Gabun" }
                                        option { value: "GB", selected: matches!(selected_origin.read().as_ref(), Some(Country::GB)), "Vereinigtes Königreich" }
                                        option { value: "GD", selected: matches!(selected_origin.read().as_ref(), Some(Country::GD)), "Grenada" }
                                        option { value: "GE", selected: matches!(selected_origin.read().as_ref(), Some(Country::GE)), "Georgien" }
                                        option { value: "GF", selected: matches!(selected_origin.read().as_ref(), Some(Country::GF)), "Französisch-Guayana" }
                                        option { value: "GG", selected: matches!(selected_origin.read().as_ref(), Some(Country::GG)), "Guernsey" }
                                        option { value: "GH", selected: matches!(selected_origin.read().as_ref(), Some(Country::GH)), "Ghana" }
                                        option { value: "GI", selected: matches!(selected_origin.read().as_ref(), Some(Country::GI)), "Gibraltar" }
                                        option { value: "GL", selected: matches!(selected_origin.read().as_ref(), Some(Country::GL)), "Grönland" }
                                        option { value: "GM", selected: matches!(selected_origin.read().as_ref(), Some(Country::GM)), "Gambia" }
                                        option { value: "GN", selected: matches!(selected_origin.read().as_ref(), Some(Country::GN)), "Guinea" }
                                        option { value: "GP", selected: matches!(selected_origin.read().as_ref(), Some(Country::GP)), "Guadeloupe" }
                                        option { value: "GQ", selected: matches!(selected_origin.read().as_ref(), Some(Country::GQ)), "Äquatorialguinea" }
                                        option { value: "GR", selected: matches!(selected_origin.read().as_ref(), Some(Country::GR)), "Griechenland" }
                                        option { value: "GS", selected: matches!(selected_origin.read().as_ref(), Some(Country::GS)), "Südgeorgien und die Südlichen Sandwichinseln" }
                                        option { value: "GT", selected: matches!(selected_origin.read().as_ref(), Some(Country::GT)), "Guatemala" }
                                        option { value: "GU", selected: matches!(selected_origin.read().as_ref(), Some(Country::GU)), "Guam" }
                                        option { value: "GW", selected: matches!(selected_origin.read().as_ref(), Some(Country::GW)), "Guinea-Bissau" }
                                        option { value: "GY", selected: matches!(selected_origin.read().as_ref(), Some(Country::GY)), "Guyana" }
                                        option { value: "HK", selected: matches!(selected_origin.read().as_ref(), Some(Country::HK)), "Hongkong" }
                                        option { value: "HM", selected: matches!(selected_origin.read().as_ref(), Some(Country::HM)), "Heard und McDonaldinseln" }
                                        option { value: "HN", selected: matches!(selected_origin.read().as_ref(), Some(Country::HN)), "Honduras" }
                                        option { value: "HR", selected: matches!(selected_origin.read().as_ref(), Some(Country::HR)), "Kroatien" }
                                        option { value: "HT", selected: matches!(selected_origin.read().as_ref(), Some(Country::HT)), "Haiti" }
                                        option { value: "HU", selected: matches!(selected_origin.read().as_ref(), Some(Country::HU)), "Ungarn" }
                                        option { value: "ID", selected: matches!(selected_origin.read().as_ref(), Some(Country::ID)), "Indonesien" }
                                        option { value: "IE", selected: matches!(selected_origin.read().as_ref(), Some(Country::IE)), "Irland" }
                                        option { value: "IL", selected: matches!(selected_origin.read().as_ref(), Some(Country::IL)), "Israel" }
                                        option { value: "IM", selected: matches!(selected_origin.read().as_ref(), Some(Country::IM)), "Isle of Man" }
                                        option { value: "IN", selected: matches!(selected_origin.read().as_ref(), Some(Country::IN)), "Indien" }
                                        option { value: "IO", selected: matches!(selected_origin.read().as_ref(), Some(Country::IO)), "Britisches Territorium im Indischen Ozean" }
                                        option { value: "IQ", selected: matches!(selected_origin.read().as_ref(), Some(Country::IQ)), "Irak" }
                                        option { value: "IR", selected: matches!(selected_origin.read().as_ref(), Some(Country::IR)), "Iran" }
                                        option { value: "IS", selected: matches!(selected_origin.read().as_ref(), Some(Country::IS)), "Island" }
                                        option { value: "IT", selected: matches!(selected_origin.read().as_ref(), Some(Country::IT)), "Italien" }
                                        option { value: "JE", selected: matches!(selected_origin.read().as_ref(), Some(Country::JE)), "Jersey" }
                                        option { value: "JM", selected: matches!(selected_origin.read().as_ref(), Some(Country::JM)), "Jamaika" }
                                        option { value: "JO", selected: matches!(selected_origin.read().as_ref(), Some(Country::JO)), "Jordanien" }
                                        option { value: "JP", selected: matches!(selected_origin.read().as_ref(), Some(Country::JP)), "Japan" }
                                        option { value: "KE", selected: matches!(selected_origin.read().as_ref(), Some(Country::KE)), "Kenia" }
                                        option { value: "KG", selected: matches!(selected_origin.read().as_ref(), Some(Country::KG)), "Kirgisistan" }
                                        option { value: "KH", selected: matches!(selected_origin.read().as_ref(), Some(Country::KH)), "Kambodscha" }
                                        option { value: "KI", selected: matches!(selected_origin.read().as_ref(), Some(Country::KI)), "Kiribati" }
                                        option { value: "KM", selected: matches!(selected_origin.read().as_ref(), Some(Country::KM)), "Komoren" }
                                        option { value: "KN", selected: matches!(selected_origin.read().as_ref(), Some(Country::KN)), "St. Kitts und Nevis" }
                                        option { value: "KP", selected: matches!(selected_origin.read().as_ref(), Some(Country::KP)), "Nordkorea" }
                                        option { value: "KR", selected: matches!(selected_origin.read().as_ref(), Some(Country::KR)), "Südkorea" }
                                        option { value: "KW", selected: matches!(selected_origin.read().as_ref(), Some(Country::KW)), "Kuwait" }
                                        option { value: "KY", selected: matches!(selected_origin.read().as_ref(), Some(Country::KY)), "Kaimaninseln" }
                                        option { value: "KZ", selected: matches!(selected_origin.read().as_ref(), Some(Country::KZ)), "Kasachstan" }
                                        option { value: "LA", selected: matches!(selected_origin.read().as_ref(), Some(Country::LA)), "Laos" }
                                        option { value: "LB", selected: matches!(selected_origin.read().as_ref(), Some(Country::LB)), "Libanon" }
                                        option { value: "LC", selected: matches!(selected_origin.read().as_ref(), Some(Country::LC)), "St. Lucia" }
                                        option { value: "LI", selected: matches!(selected_origin.read().as_ref(), Some(Country::LI)), "Liechtenstein" }
                                        option { value: "LK", selected: matches!(selected_origin.read().as_ref(), Some(Country::LK)), "Sri Lanka" }
                                        option { value: "LR", selected: matches!(selected_origin.read().as_ref(), Some(Country::LR)), "Liberia" }
                                        option { value: "LS", selected: matches!(selected_origin.read().as_ref(), Some(Country::LS)), "Lesotho" }
                                        option { value: "LT", selected: matches!(selected_origin.read().as_ref(), Some(Country::LT)), "Litauen" }
                                        option { value: "LU", selected: matches!(selected_origin.read().as_ref(), Some(Country::LU)), "Luxemburg" }
                                        option { value: "LV", selected: matches!(selected_origin.read().as_ref(), Some(Country::LV)), "Lettland" }
                                        option { value: "LY", selected: matches!(selected_origin.read().as_ref(), Some(Country::LY)), "Libyen" }
                                        option { value: "MA", selected: matches!(selected_origin.read().as_ref(), Some(Country::MA)), "Marokko" }
                                        option { value: "MC", selected: matches!(selected_origin.read().as_ref(), Some(Country::MC)), "Monaco" }
                                        option { value: "MD", selected: matches!(selected_origin.read().as_ref(), Some(Country::MD)), "Moldau" }
                                        option { value: "ME", selected: matches!(selected_origin.read().as_ref(), Some(Country::ME)), "Montenegro" }
                                        option { value: "MF", selected: matches!(selected_origin.read().as_ref(), Some(Country::MF)), "Saint-Martin" }
                                        option { value: "MG", selected: matches!(selected_origin.read().as_ref(), Some(Country::MG)), "Madagaskar" }
                                        option { value: "MH", selected: matches!(selected_origin.read().as_ref(), Some(Country::MH)), "Marshallinseln" }
                                        option { value: "MK", selected: matches!(selected_origin.read().as_ref(), Some(Country::MK)), "Nordmazedonien" }
                                        option { value: "ML", selected: matches!(selected_origin.read().as_ref(), Some(Country::ML)), "Mali" }
                                        option { value: "MM", selected: matches!(selected_origin.read().as_ref(), Some(Country::MM)), "Myanmar" }
                                        option { value: "MN", selected: matches!(selected_origin.read().as_ref(), Some(Country::MN)), "Mongolei" }
                                        option { value: "MO", selected: matches!(selected_origin.read().as_ref(), Some(Country::MO)), "Macau" }
                                        option { value: "MP", selected: matches!(selected_origin.read().as_ref(), Some(Country::MP)), "Nördliche Marianen" }
                                        option { value: "MQ", selected: matches!(selected_origin.read().as_ref(), Some(Country::MQ)), "Martinique" }
                                        option { value: "MR", selected: matches!(selected_origin.read().as_ref(), Some(Country::MR)), "Mauretanien" }
                                        option { value: "MS", selected: matches!(selected_origin.read().as_ref(), Some(Country::MS)), "Montserrat" }
                                        option { value: "MT", selected: matches!(selected_origin.read().as_ref(), Some(Country::MT)), "Malta" }
                                        option { value: "MU", selected: matches!(selected_origin.read().as_ref(), Some(Country::MU)), "Mauritius" }
                                        option { value: "MV", selected: matches!(selected_origin.read().as_ref(), Some(Country::MV)), "Malediven" }
                                        option { value: "MW", selected: matches!(selected_origin.read().as_ref(), Some(Country::MW)), "Malawi" }
                                        option { value: "MX", selected: matches!(selected_origin.read().as_ref(), Some(Country::MX)), "Mexiko" }
                                        option { value: "MY", selected: matches!(selected_origin.read().as_ref(), Some(Country::MY)), "Malaysia" }
                                        option { value: "MZ", selected: matches!(selected_origin.read().as_ref(), Some(Country::MZ)), "Mosambik" }
                                        option { value: "NA", selected: matches!(selected_origin.read().as_ref(), Some(Country::NA)), "Namibia" }
                                        option { value: "NC", selected: matches!(selected_origin.read().as_ref(), Some(Country::NC)), "Neukaledonien" }
                                        option { value: "NE", selected: matches!(selected_origin.read().as_ref(), Some(Country::NE)), "Niger" }
                                        option { value: "NF", selected: matches!(selected_origin.read().as_ref(), Some(Country::NF)), "Norfolkinsel" }
                                        option { value: "NG", selected: matches!(selected_origin.read().as_ref(), Some(Country::NG)), "Nigeria" }
                                        option { value: "NI", selected: matches!(selected_origin.read().as_ref(), Some(Country::NI)), "Nicaragua" }
                                        option { value: "NL", selected: matches!(selected_origin.read().as_ref(), Some(Country::NL)), "Niederlande" }
                                        option { value: "NO", selected: matches!(selected_origin.read().as_ref(), Some(Country::NO)), "Norwegen" }
                                        option { value: "NP", selected: matches!(selected_origin.read().as_ref(), Some(Country::NP)), "Nepal" }
                                        option { value: "NR", selected: matches!(selected_origin.read().as_ref(), Some(Country::NR)), "Nauru" }
                                        option { value: "NU", selected: matches!(selected_origin.read().as_ref(), Some(Country::NU)), "Niue" }
                                        option { value: "NZ", selected: matches!(selected_origin.read().as_ref(), Some(Country::NZ)), "Neuseeland" }
                                        option { value: "OM", selected: matches!(selected_origin.read().as_ref(), Some(Country::OM)), "Oman" }
                                        option { value: "PA", selected: matches!(selected_origin.read().as_ref(), Some(Country::PA)), "Panama" }
                                        option { value: "PE", selected: matches!(selected_origin.read().as_ref(), Some(Country::PE)), "Peru" }
                                        option { value: "PF", selected: matches!(selected_origin.read().as_ref(), Some(Country::PF)), "Französisch-Polynesien" }
                                        option { value: "PG", selected: matches!(selected_origin.read().as_ref(), Some(Country::PG)), "Papua-Neuguinea" }
                                        option { value: "PH", selected: matches!(selected_origin.read().as_ref(), Some(Country::PH)), "Philippinen" }
                                        option { value: "PK", selected: matches!(selected_origin.read().as_ref(), Some(Country::PK)), "Pakistan" }
                                        option { value: "PL", selected: matches!(selected_origin.read().as_ref(), Some(Country::PL)), "Polen" }
                                        option { value: "PM", selected: matches!(selected_origin.read().as_ref(), Some(Country::PM)), "Saint-Pierre und Miquelon" }
                                        option { value: "PN", selected: matches!(selected_origin.read().as_ref(), Some(Country::PN)), "Pitcairninseln" }
                                        option { value: "PR", selected: matches!(selected_origin.read().as_ref(), Some(Country::PR)), "Puerto Rico" }
                                        option { value: "PS", selected: matches!(selected_origin.read().as_ref(), Some(Country::PS)), "Palästina" }
                                        option { value: "PT", selected: matches!(selected_origin.read().as_ref(), Some(Country::PT)), "Portugal" }
                                        option { value: "PW", selected: matches!(selected_origin.read().as_ref(), Some(Country::PW)), "Palau" }
                                        option { value: "PY", selected: matches!(selected_origin.read().as_ref(), Some(Country::PY)), "Paraguay" }
                                        option { value: "QA", selected: matches!(selected_origin.read().as_ref(), Some(Country::QA)), "Katar" }
                                        option { value: "RE", selected: matches!(selected_origin.read().as_ref(), Some(Country::RE)), "Réunion" }
                                        option { value: "RO", selected: matches!(selected_origin.read().as_ref(), Some(Country::RO)), "Rumänien" }
                                        option { value: "RS", selected: matches!(selected_origin.read().as_ref(), Some(Country::RS)), "Serbien" }
                                        option { value: "RU", selected: matches!(selected_origin.read().as_ref(), Some(Country::RU)), "Russland" }
                                        option { value: "RW", selected: matches!(selected_origin.read().as_ref(), Some(Country::RW)), "Ruanda" }
                                        option { value: "SA", selected: matches!(selected_origin.read().as_ref(), Some(Country::SA)), "Saudi-Arabien" }
                                        option { value: "SB", selected: matches!(selected_origin.read().as_ref(), Some(Country::SB)), "Salomonen" }
                                        option { value: "SC", selected: matches!(selected_origin.read().as_ref(), Some(Country::SC)), "Seychellen" }
                                        option { value: "SD", selected: matches!(selected_origin.read().as_ref(), Some(Country::SD)), "Sudan" }
                                        option { value: "SE", selected: matches!(selected_origin.read().as_ref(), Some(Country::SE)), "Schweden" }
                                        option { value: "SG", selected: matches!(selected_origin.read().as_ref(), Some(Country::SG)), "Singapur" }
                                        option { value: "SH", selected: matches!(selected_origin.read().as_ref(), Some(Country::SH)), "St. Helena" }
                                        option { value: "SI", selected: matches!(selected_origin.read().as_ref(), Some(Country::SI)), "Slowenien" }
                                        option { value: "SJ", selected: matches!(selected_origin.read().as_ref(), Some(Country::SJ)), "Svalbard und Jan Mayen" }
                                        option { value: "SK", selected: matches!(selected_origin.read().as_ref(), Some(Country::SK)), "Slowakei" }
                                        option { value: "SL", selected: matches!(selected_origin.read().as_ref(), Some(Country::SL)), "Sierra Leone" }
                                        option { value: "SM", selected: matches!(selected_origin.read().as_ref(), Some(Country::SM)), "San Marino" }
                                        option { value: "SN", selected: matches!(selected_origin.read().as_ref(), Some(Country::SN)), "Senegal" }
                                        option { value: "SO", selected: matches!(selected_origin.read().as_ref(), Some(Country::SO)), "Somalia" }
                                        option { value: "SR", selected: matches!(selected_origin.read().as_ref(), Some(Country::SR)), "Suriname" }
                                        option { value: "SS", selected: matches!(selected_origin.read().as_ref(), Some(Country::SS)), "Südsudan" }
                                        option { value: "ST", selected: matches!(selected_origin.read().as_ref(), Some(Country::ST)), "São Tomé und Príncipe" }
                                        option { value: "SV", selected: matches!(selected_origin.read().as_ref(), Some(Country::SV)), "El Salvador" }
                                        option { value: "SX", selected: matches!(selected_origin.read().as_ref(), Some(Country::SX)), "Sint Maarten" }
                                        option { value: "SY", selected: matches!(selected_origin.read().as_ref(), Some(Country::SY)), "Syrien" }
                                        option { value: "SZ", selected: matches!(selected_origin.read().as_ref(), Some(Country::SZ)), "Eswatini" }
                                        option { value: "TC", selected: matches!(selected_origin.read().as_ref(), Some(Country::TC)), "Turks- und Caicosinseln" }
                                        option { value: "TD", selected: matches!(selected_origin.read().as_ref(), Some(Country::TD)), "Tschad" }
                                        option { value: "TF", selected: matches!(selected_origin.read().as_ref(), Some(Country::TF)), "Französische Süd- und Antarktisgebiete" }
                                        option { value: "TG", selected: matches!(selected_origin.read().as_ref(), Some(Country::TG)), "Togo" }
                                        option { value: "TH", selected: matches!(selected_origin.read().as_ref(), Some(Country::TH)), "Thailand" }
                                        option { value: "TJ", selected: matches!(selected_origin.read().as_ref(), Some(Country::TJ)), "Tadschikistan" }
                                        option { value: "TK", selected: matches!(selected_origin.read().as_ref(), Some(Country::TK)), "Tokelau" }
                                        option { value: "TL", selected: matches!(selected_origin.read().as_ref(), Some(Country::TL)), "Osttimor" }
                                        option { value: "TM", selected: matches!(selected_origin.read().as_ref(), Some(Country::TM)), "Turkmenistan" }
                                        option { value: "TN", selected: matches!(selected_origin.read().as_ref(), Some(Country::TN)), "Tunesien" }
                                        option { value: "TO", selected: matches!(selected_origin.read().as_ref(), Some(Country::TO)), "Tonga" }
                                        option { value: "TR", selected: matches!(selected_origin.read().as_ref(), Some(Country::TR)), "Türkei" }
                                        option { value: "TT", selected: matches!(selected_origin.read().as_ref(), Some(Country::TT)), "Trinidad und Tobago" }
                                        option { value: "TV", selected: matches!(selected_origin.read().as_ref(), Some(Country::TV)), "Tuvalu" }
                                        option { value: "TW", selected: matches!(selected_origin.read().as_ref(), Some(Country::TW)), "Taiwan" }
                                        option { value: "TZ", selected: matches!(selected_origin.read().as_ref(), Some(Country::TZ)), "Tansania" }
                                        option { value: "UA", selected: matches!(selected_origin.read().as_ref(), Some(Country::UA)), "Ukraine" }
                                        option { value: "UG", selected: matches!(selected_origin.read().as_ref(), Some(Country::UG)), "Uganda" }
                                        option { value: "UM", selected: matches!(selected_origin.read().as_ref(), Some(Country::UM)), "Amerikanische Überseeinseln" }
                                        option { value: "US", selected: matches!(selected_origin.read().as_ref(), Some(Country::US)), "Vereinigte Staaten" }
                                        option { value: "UY", selected: matches!(selected_origin.read().as_ref(), Some(Country::UY)), "Uruguay" }
                                        option { value: "UZ", selected: matches!(selected_origin.read().as_ref(), Some(Country::UZ)), "Usbekistan" }
                                        option { value: "VA", selected: matches!(selected_origin.read().as_ref(), Some(Country::VA)), "Vatikanstadt" }
                                        option { value: "VC", selected: matches!(selected_origin.read().as_ref(), Some(Country::VC)), "St. Vincent und die Grenadinen" }
                                        option { value: "VE", selected: matches!(selected_origin.read().as_ref(), Some(Country::VE)), "Venezuela" }
                                        option { value: "VG", selected: matches!(selected_origin.read().as_ref(), Some(Country::VG)), "Britische Jungferninseln" }
                                        option { value: "VI", selected: matches!(selected_origin.read().as_ref(), Some(Country::VI)), "Amerikanische Jungferninseln" }
                                        option { value: "VN", selected: matches!(selected_origin.read().as_ref(), Some(Country::VN)), "Vietnam" }
                                        option { value: "VU", selected: matches!(selected_origin.read().as_ref(), Some(Country::VU)), "Vanuatu" }
                                        option { value: "WF", selected: matches!(selected_origin.read().as_ref(), Some(Country::WF)), "Wallis und Futuna" }
                                        option { value: "WS", selected: matches!(selected_origin.read().as_ref(), Some(Country::WS)), "Samoa" }
                                        option { value: "YE", selected: matches!(selected_origin.read().as_ref(), Some(Country::YE)), "Jemen" }
                                        option { value: "YT", selected: matches!(selected_origin.read().as_ref(), Some(Country::YT)), "Mayotte" }
                                        option { value: "ZA", selected: matches!(selected_origin.read().as_ref(), Some(Country::ZA)), "Südafrika" }
                                        option { value: "ZM", selected: matches!(selected_origin.read().as_ref(), Some(Country::ZM)), "Sambia" }
                                        option { value: "ZW", selected: matches!(selected_origin.read().as_ref(), Some(Country::ZW)), "Simbabwe" }
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {}
                    }
                }
                div { class: "modal-action",
                    form { method: "dialog",
                        button {
                            class: "btn",
                            onclick: move |_| is_open.toggle(),
                            onkeydown: move |evt| {
                                if evt.key() == Key::Escape {
                                    is_open.set(false);
                                }
                            },
                            "× " {t!("nav.schliessen")},
                        }
                        if props.genesis {
                            button {
                                class: "btn",
                                onclick: move |_| handle_genesis(),
                                {t!("nav.speichern")},
                            }
                        }
                    }
                }
            }
        }
    }
}
