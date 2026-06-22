use crate::components::card_stack::{CardStack, GenesisModal};
use crate::components::ingredient_path::IngredientPath;
use crate::components::*;
use crate::core::Ingredient;
use crate::rules::RuleDef;
use crate::services::{UnifiedIngredient, IngredientSource};
use crate::category_service::*;
use dioxus::prelude::*;
use rust_i18n::t;
use std::collections::HashMap;

/// Convert an Ingredient to UnifiedIngredient for display purposes
fn ingredient_to_unified(ingredient: &Ingredient) -> UnifiedIngredient {
    let (is_meat, is_fish, is_dairy, is_egg, is_honey, is_plant) = if let Some(ref category) = ingredient.category {
        (
            Some(is_meat_category(category)),
            Some(is_fish_category(category)),
            Some(is_dairy_category(category)),
            Some(is_egg_category(category)),
            Some(is_honey_category(category)),
            Some(is_plant_category(category)),
        )
    } else {
        (None, None, None, None, None, None)
    };

    UnifiedIngredient {
        name: ingredient.name.clone(),
        canonical: ingredient.canonical.clone(),
        priority: 0,
        category: ingredient.category.clone(),
        origin: ingredient.origins.as_ref().and_then(|o| o.first().cloned()),
        is_allergen: Some(ingredient.is_allergen),
        is_agricultural: Some(ingredient.is_agricultural),
        is_meat,
        is_fish,
        is_dairy,
        is_egg,
        is_honey,
        is_plant,
        is_bio: ingredient.is_bio,
        source: IngredientSource::Local,
    }
}

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

    // Flatten all validation messages into an ordered (ingredient label, message)
    // list. Keys are either "certification_body" or "ingredients[i][field]" where
    // `i` is the TOP-LEVEL ingredient index (every validator in core.rs iterates
    // `ingredients.iter().enumerate()`), so it maps directly to the ingredient name.
    let issues = use_memo(move || {
        let msgs = props.validation_messages.read();
        let ingredients = props.ingredients.read();
        let mut out: Vec<(usize, String, String)> = Vec::new();
        for (key, messages) in msgs.iter() {
            // Parse the top-level index from "ingredients[i][field]" (None for
            // non-indexed keys such as "certification_body").
            let idx = key
                .strip_prefix("ingredients[")
                .and_then(|rest| rest.split(']').next())
                .and_then(|n| n.parse::<usize>().ok());
            let label = idx
                .and_then(|i| ingredients.get(i))
                .map(|ing| ing.name.clone())
                .unwrap_or_default();
            // `usize::MAX` sorts non-indexed (e.g. cert body) keys last.
            let sort_idx = idx.unwrap_or(usize::MAX);
            for m in messages {
                out.push((sort_idx, label.clone(), m.clone()));
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
            let is_namensgebend = ingr.is_namensgebend.unwrap_or(false);
            let computed_origins = ingr.computed_origins();
            let computed_amount = ingr.computed_amount();
            let unit_key = ingr.computed_unit().translation_key().to_string();
            let unified = ingredient_to_unified(&ingr);
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
                                None => rsx! {},
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
                        IngredientSymbolsCompact {
                            ingredient: unified
                        }
                    }
                    div {
                        class: "text-right",
                        "{computed_amount:.1} " {t!(&unit_key).to_string()}
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
                // Render children recursively (always expanded)
                if let Some(ref children) = children_for_recurse {
                    if !children.is_empty() {
                        {render_ingredient_tree(
                            children,
                            &full_path_for_children,
                            depth + 1,
                            &editing_path_signal,
                            root_ingredients,
                            show_knospe_icon,
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
