use crate::components::card_stack::{CardStack, GenesisModal};
use crate::components::ingredient_path::IngredientPath;
use crate::components::*;
use crate::core::Ingredient;
use crate::rules::RuleDef;
use dioxus::prelude::*;
use rust_i18n::t;
use std::collections::HashMap;

#[derive(Props, Clone, PartialEq)]
pub struct IngredientsTableProps {
    ingredients: Signal<Vec<Ingredient>>,
    manual_total: Signal<Option<f64>>,
    validation_messages: Memo<HashMap<String, Vec<String>>>,
    rules: Memo<Vec<RuleDef>>,
    rezeptur_vollstaendig: Signal<bool>,
}
pub fn IngredientsTable(mut props: IngredientsTableProps) -> Element {
    let editing_path: Signal<IngredientPath> = use_signal(Vec::new);

    // Flatten the recipe-scoped validation messages into an ordered (ingredient
    // label, message) list. Only "ingredients[i][field]" keys count — `i` is the
    // TOP-LEVEL ingredient index (every validator in core.rs iterates
    // `ingredients.iter().enumerate()`), so it maps directly to the ingredient
    // name. Non-recipe keys (e.g. "certification_body") stay out of this panel:
    // the yellow placeholder on the label preview covers them (Testing 25.06.2026).
    let issues = use_memo(move || {
        let msgs = props.validation_messages.read();
        let ingredients = props.ingredients.read();
        let mut out: Vec<(usize, String, String)> = Vec::new();
        for (key, messages) in msgs.iter() {
            // Parse the top-level index from "ingredients[i][field]"; skip
            // everything else (non-recipe issues).
            let Some(idx) = key
                .strip_prefix("ingredients[")
                .and_then(|rest| rest.split(']').next())
                .and_then(|n| n.parse::<usize>().ok())
            else {
                continue;
            };
            let label = ingredients
                .get(idx)
                .map(|ing| ing.name.clone())
                .unwrap_or_default();
            for m in messages {
                out.push((idx, label.clone(), m.clone()));
            }
        }
        // Deterministic order: HashMap iteration is unordered.
        out.sort_by(|a, b| a.0.cmp(&b.0).then(a.2.cmp(&b.2)));
        out.into_iter().map(|(_, label, msg)| (label, msg)).collect::<Vec<_>>()
    });

    // Recipe is "valid" once marked complete and no validation errors remain.
    let recipe_valid = use_memo(move || (props.rezeptur_vollstaendig)() && issues().is_empty());

    let total_amount = use_memo(move || {
        props
            .ingredients
            .read()
            .iter()
            .map(|x: &Ingredient| x.computed_amount())
            .sum::<f64>()
    });

    let show_knospe_icon = props.rules.read().contains(&RuleDef::Knospe_ShowBioSuisseLogo);

    rsx! {
        div { class: "grid grid-cols-3 gap-4 items-center border-top",
            GenesisModal {
                ingredients: props.ingredients,
                rules: props.rules
            }
            div {}
            div {}
        }
        div { class: "flex flex-col gap-4",
            // Recursive tree rendering
            {render_ingredient_tree(
                &props.ingredients.read(),
                &[],
                0,
                &editing_path,
                props.ingredients,
                show_knospe_icon,
                None,
            )}

            if props.ingredients.len() > 0 {
                ConditionalDisplay {
                    path: "manuelles_total".to_string(),
                    div {
                        class: "grid grid-cols-3 gap-4",
                        div {{t!("label.total").to_string()}}
                        div {
                            class: "text-right",
                            "{total_amount:.1} " {t!("units.g").to_string()}
                        }

                        FormField {
                            label: "{t!(\"label.manuellesTotal\").to_string()}",
                            help: Some(t!("help.manuellesTotal").to_string()),
                            input {
                                r#type: "number",
                                placeholder: t!("label.manuellesTotal").to_string(),
                                class: "input input-accent w-full",
                                min: "0",
                                onchange: move |evt| {
                                    if let Ok(amount) = evt.data.value().parse::<f64>() {
                                        props.manual_total.set(Some(amount));
                                    } else {
                                        props.manual_total.set(None);
                                    }
                                },
                            }
                        }

                        div {}
                    }
                }
            }
        }
        if props.ingredients.len() > 0 {
            {
                let rezeptur_vollstaendig = (props.rezeptur_vollstaendig)();
                let btn_class = if rezeptur_vollstaendig { "btn btn-disabled" } else { "btn btn-accent" };
                // Always render the feedback; toggle visibility via an interpolated
                // class string (Dioxus 0.7 `if` conditionals don't reliably
                // re-render on Memo changes).
                let valid = recipe_valid();
                let green_vis = if valid { "inline-flex" } else { "hidden" };
                let error_vis = if rezeptur_vollstaendig && !valid { "flex" } else { "hidden" };
                let problems = issues();
                let problem_count = problems.len();
                rsx! {
                    div { class: "mt-4 flex flex-col gap-2",
                        div { class: "flex items-center gap-2",
                            button {
                                class: "{btn_class}",
                                disabled: rezeptur_vollstaendig,
                                onclick: move |_| {
                                    props.rezeptur_vollstaendig.set(true);
                                },
                                "{t!(\"label.rezepturVollstaendig\").to_string()}"
                            }
                            span {
                                class: "{green_vis} items-center gap-1 text-success font-medium",
                                svg {
                                    class: "h-6 w-6",
                                    fill: "none",
                                    view_box: "0 0 24 24",
                                    stroke: "currentColor",
                                    stroke_width: "2.5",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "M5 13l4 4L19 7",
                                    }
                                }
                                "{t!(\"label.rezepturGueltig\").to_string()}"
                            }
                        }
                        div {
                            class: "{error_vis} flex-col gap-1 bg-error/30 rounded p-3 text-sm",
                            div {
                                class: "flex items-center gap-1 font-medium text-error",
                                svg {
                                    class: "h-5 w-5",
                                    fill: "none",
                                    view_box: "0 0 24 24",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z",
                                    }
                                }
                                "{t!(\"label.rezepturProbleme\", count = problem_count).to_string()}"
                            }
                            for (label, msg) in problems {
                                div { class: "flex gap-1 pl-6",
                                    if !label.is_empty() {
                                        span { class: "font-medium", "{label}: " }
                                    }
                                    span { "{msg}" }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Stacking card modal for editing
        CardStack {
            ingredients: props.ingredients,
            editing_path: editing_path,
            rules: props.rules,
        }
    }
}

/// Recursively render the ingredient tree with indentation.
fn render_ingredient_tree(
    ingredients: &[Ingredient],
    path_prefix: &[usize],
    depth: usize,
    editing_path: &Signal<IngredientPath>,
    root_ingredients: Signal<Vec<Ingredient>>,
    show_knospe_icon: bool,
    // Some(is_ch) when an ancestor composite claims the quality — children show
    // the inherited Knospe desaturated ("Pastellfarben", Testing 25.06.2026).
    inherited_knospe: Option<bool>,
) -> Element {
    use crate::model::Country;

    let elements: Vec<Element> = ingredients.iter().enumerate()
        .map(|(i, ingr)| {
            let full_path: IngredientPath = {
                let mut p = path_prefix.to_vec();
                p.push(i);
                p
            };
            let edit_path = full_path.clone();
            let mut editing_path_signal = *editing_path;
            let ingr = ingr.clone();
            let name = ingr.name.clone();
            let is_allergen = ingr.is_allergen;
            let is_agricultural = ingr.is_agricultural;
            let is_namensgebend = ingr.is_namensgebend.unwrap_or(false);
            let computed_origins = ingr.computed_origins();
            let computed_amount = ingr.computed_amount();
            let unit_key = ingr.computed_unit().translation_key().to_string();
            let children = ingr.children.clone();
            let children_for_recurse = children.clone();
            let full_path_for_children = full_path.clone();

            let knospe_variant: Option<bool> = if show_knospe_icon && ingr.computed_bio_status().unwrap_or(false) {
                computed_origins.as_ref()
                    .filter(|o| !o.is_empty())
                    .map(|o| o.contains(&Country::CH))
            } else {
                None
            };

            rsx! {
                div {
                    class: if depth.is_multiple_of(2) { "grid gap-4 grid-cols-3 bg-gray-100 items-center" } else { "grid gap-4 grid-cols-3 bg-white items-center" },
                    style: "padding-left: {depth as f32 * 1.5}rem;",
                    key: "{i}-{name}",
                    div {
                        class: "flex items-center gap-2",
                        div {
                            class: "flex items-center gap-1",
                            if let Some(origins) = &computed_origins {
                                // Skip origins without a flag glyph (e.g. generic `Import`)
                                // so an imported ingredient with no named country shows no flag.
                                for origin in origins.iter().filter(|o| !o.flag_emoji().is_empty()) {
                                    span { class: "text-lg", "{origin.flag_emoji()}" }
                                }
                            }
                            match knospe_variant {
                                Some(true) => rsx! { icons::KnospeCompactCh {} },
                                Some(false) => rsx! { icons::KnospeCompactNoCross {} },
                                // Inherited from a parent-level claim: desaturated icon
                                // (agricultural sub-ingredients only).
                                None => match inherited_knospe.filter(|_| show_knospe_icon && is_agricultural) {
                                    Some(ch) => rsx! {
                                        span {
                                            class: "opacity-40 saturate-50",
                                            title: t!("bio_labels.inherited_quality").to_string(),
                                            if ch { icons::KnospeCompactCh {} } else { icons::KnospeCompactNoCross {} }
                                        }
                                    },
                                    None => rsx! {},
                                },
                            }
                            div {
                                if is_allergen {
                                    span { class: "font-bold", "{name}" }
                                } else {
                                    "{name}"
                                }
                                if is_namensgebend { " ({t!(\"label.namensgebend\").to_string()})" }
                            }
                        }
                    }
                    div {
                        class: "text-right",
                        // Uniform amount display (Testing 25.06.2026, hand note 11):
                        // percent entries without decimals, qualitative zero amounts
                        // as a quiet dash instead of noisy "0.0 %" rows.
                        {
                            let unit_txt = t!(&unit_key).to_string();
                            if computed_amount <= 0.0 {
                                rsx! { span { class: "text-base-content/40", "–" } }
                            } else if unit_txt == "%" {
                                rsx! { "{computed_amount:.0} {unit_txt}" }
                            } else {
                                rsx! { "{computed_amount:.1} {unit_txt}" }
                            }
                        }
                    }
                    div {
                        class: "text-right",
                        div {
                            class: "join",
                            button {
                                class: "btn join-item btn-outline",
                                onclick: move |_| {
                                    editing_path_signal.set(edit_path.clone());
                                },
                                icons::ListDetail {}
                            }
                            if depth == 0 {
                                button {
                                    class: "btn btn-outline join-item",
                                    dangerous_inner_html: r###"<svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>"###,
                                    onclick: {
                                        let mut root_ingredients = root_ingredients;
                                        move |_| {
                                            root_ingredients.write().remove(i);
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
                // Render children recursively (always expanded). A quality claim on
                // this node starts (or continues) the inherited-Knospe display.
                if let Some(ref children) = children_for_recurse {
                    if !children.is_empty() {
                        {render_ingredient_tree(
                            children,
                            &full_path_for_children,
                            depth + 1,
                            &editing_path_signal,
                            root_ingredients,
                            show_knospe_icon,
                            knospe_variant.or(inherited_knospe),
                        )}
                    }
                }
            }
        })
        .collect();

    rsx! {
        {elements.into_iter()}
    }
}
