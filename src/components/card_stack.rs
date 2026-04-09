use crate::components::ingredient_pane::IngredientPane;
use crate::components::ingredient_path::{self, IngredientPath};
use crate::core::Ingredient;
use crate::rules::RuleDef;
use dioxus::prelude::*;
use rust_i18n::t;

/// Stacking card modal for editing ingredients with recursive depth navigation.
///
/// Replaces the two-pane MillerModal with visually stacked cards:
/// - Path length 0: modal closed
/// - Path length 1: single card (top-level ingredient)
/// - Path length N: N stacked cards, each offset, topmost is active
#[derive(Props, Clone, PartialEq)]
pub struct CardStackProps {
    pub ingredients: Signal<Vec<Ingredient>>,
    pub editing_path: Signal<IngredientPath>,
    pub rules: Memo<Vec<RuleDef>>,
}

/// Height of a background card's title bar in pixels.
const TITLE_BAR_HEIGHT: f64 = 40.0;

pub fn CardStack(props: CardStackProps) -> Element {
    let mut editing_path = props.editing_path;
    let path = editing_path.read().clone();
    let is_open = !path.is_empty();
    let depth_count = path.len();

    // Track the initial path length when the modal was opened.
    // When the user opens a sub-ingredient directly from the tree (path length > 1),
    // closing that card should close the modal entirely, not pop to the parent.
    let mut entry_depth = use_signal(|| 0usize);
    use_effect(move || {
        let len = editing_path.read().len();
        if len > 0 && entry_depth() == 0 {
            entry_depth.set(len);
        } else if len == 0 {
            entry_depth.set(0);
        }
    });

    rsx! {
        dialog { open: is_open, class: "modal",
            div {
                class: "modal-box bg-base-100 max-w-2xl w-full p-0",
                style: "overflow: visible; min-height: 80vh; max-height: 90vh;",
                div {
                    class: "relative w-full",
                    // Reserve space: active card height + title bars for all background cards
                    style: "min-height: 80vh;",
                    for depth in 0..depth_count {
                        StackCard {
                            key: "{depth}",
                            ingredients: props.ingredients,
                            editing_path: editing_path,
                            rules: props.rules,
                            depth: depth,
                            total_depth: depth_count,
                            entry_depth: entry_depth,
                        }
                    }
                }
            }
            form {
                method: "dialog",
                class: "modal-backdrop",
                onclick: move |_| {
                    editing_path.set(vec![]);
                },
                button { "" }
            }
        }
    }
}

/// A single card in the stack. Background cards show only a header;
/// the topmost card renders the full IngredientPane.
#[derive(Props, Clone, PartialEq)]
struct StackCardProps {
    ingredients: Signal<Vec<Ingredient>>,
    editing_path: Signal<IngredientPath>,
    rules: Memo<Vec<RuleDef>>,
    depth: usize,
    total_depth: usize,
    entry_depth: Signal<usize>,
}

fn StackCard(props: StackCardProps) -> Element {
    let depth = props.depth;
    let is_topmost = depth == props.total_depth - 1;
    let editing_path = props.editing_path;

    let card_path = {
        let full_path = editing_path.read();
        full_path[..=depth].to_vec()
    };

    let ingredient_name = {
        let root = props.ingredients.read();
        ingredient_path::get_at_path(&root, &card_path)
            .map(|i| if i.name.is_empty() { t!("label.zutatDetails").to_string() } else { i.name.clone() })
            .unwrap_or_default()
    };

    // Each card is offset vertically by the number of background cards above it,
    // so background title bars remain visible above the next card.
    let top_offset = depth as f64 * TITLE_BAR_HEIGHT;

    if is_topmost {
        // Active card: render full IngredientPane with name title bar
        rsx! {
            div {
                class: "absolute left-0 right-0 bottom-0 bg-base-100 rounded-t-lg shadow-xl overflow-y-auto",
                style: "top: {top_offset}px; z-index: {depth};",
                // Title bar with ingredient name
                div { class: "sticky top-0 z-10 bg-base-100 border-b border-base-300 px-4 py-2 rounded-t-lg",
                    h3 { class: "font-bold text-lg truncate", "{ingredient_name}" }
                }
                div { class: "p-4",
                    if depth == 0 {
                        // Top-level ingredient: render IngredientPane directly
                        {rsx! {
                            TopLevelCard {
                                ingredients: props.ingredients,
                                editing_path: editing_path,
                                rules: props.rules,
                                index: card_path[0],
                                entry_depth: props.entry_depth,
                            }
                        }}
                    } else {
                        // Nested ingredient: use NestedPane wrapper for signal sync
                        {rsx! {
                            NestedCard {
                                root_ingredients: props.ingredients,
                                path: card_path,
                                rules: props.rules,
                                editing_path: editing_path,
                                depth: depth,
                                entry_depth: props.entry_depth,
                            }
                        }}
                    }
                }
            }
        }
    } else {
        // Background card: only the title bar is visible (next card covers the rest)
        rsx! {
            div {
                class: "absolute left-0 right-0 bg-base-200 rounded-t-lg shadow-sm pointer-events-none",
                style: "top: {top_offset}px; height: {TITLE_BAR_HEIGHT}px; z-index: {depth};",
                div { class: "font-bold text-lg px-4 py-2 truncate text-base-content/60",
                    "{ingredient_name}"
                }
            }
        }
    }
}

/// Top-level card (depth 0): renders IngredientPane directly with recursive scaling support.
#[derive(Props, Clone, PartialEq)]
struct TopLevelCardProps {
    ingredients: Signal<Vec<Ingredient>>,
    editing_path: Signal<IngredientPath>,
    rules: Memo<Vec<RuleDef>>,
    index: usize,
    entry_depth: Signal<usize>,
}

fn TopLevelCard(props: TopLevelCardProps) -> Element {
    let mut editing_path = props.editing_path;
    let mut ingredients = props.ingredients;
    let index = props.index;

    rsx! {
        IngredientPane {
            ingredients: props.ingredients,
            index: index,
            path: vec![index],
            is_genesis: false,
            rules: props.rules,
            editing_path: Some(editing_path),
            depth: 0,
            on_edit_child: move |child_index: usize| {
                let mut current = editing_path.read().clone();
                current.push(child_index);
                editing_path.set(current);
            },
            on_save: move |(new_ingredient, scale_all): (Ingredient, bool)| {
                if scale_all {
                    let original_amount = ingredients.read().get(index).map(|i| i.amount).unwrap_or(0.0);
                    if original_amount > 0.0 {
                        let factor = new_ingredient.amount / original_amount;
                        // Scale edited ingredient's children recursively
                        let mut scaled_ingredient = new_ingredient.clone();
                        if let Some(children) = &mut scaled_ingredient.children {
                            for child in children.iter_mut() {
                                child.scale_recursive(factor);
                            }
                        }
                        // Scale all sibling ingredients recursively
                        let mut all = ingredients.write();
                        for (i, ingredient) in all.iter_mut().enumerate() {
                            if i == index {
                                *ingredient = scaled_ingredient.clone();
                            } else {
                                ingredient.scale_recursive(factor);
                            }
                        }
                    } else {
                        ingredients.write()[index] = new_ingredient;
                    }
                } else {
                    ingredients.write()[index] = new_ingredient;
                }
                editing_path.set(vec![]);
            },
            on_close: move |_| {
                editing_path.set(vec![]);
            },
            on_composite_changed: move |is_composite: bool| {
                if !is_composite {
                    editing_path.set(vec![index]);
                }
            },
        }
    }
}

/// Nested card (depth > 0): wraps IngredientPane with bidirectional signal sync
/// between a wrapper signal and the root ingredients signal.
#[derive(Props, Clone, PartialEq)]
struct NestedCardProps {
    root_ingredients: Signal<Vec<Ingredient>>,
    path: Vec<usize>,
    rules: Memo<Vec<RuleDef>>,
    editing_path: Signal<IngredientPath>,
    depth: usize,
    entry_depth: Signal<usize>,
}

fn NestedCard(props: NestedCardProps) -> Element {
    let index = *props.path.last().unwrap();
    let parent_path = props.path[..props.path.len() - 1].to_vec();

    let children = {
        let root = props.root_ingredients.read();
        ingredient_path::get_at_path(&root, &parent_path)
            .and_then(|p| p.children.clone())
            .unwrap_or_default()
    };
    let mut wrapper = use_signal(move || children);

    // Sync wrapper when root ingredients change (root -> wrapper)
    let root_ingredients = props.root_ingredients;
    let parent_path_for_effect = parent_path.clone();
    use_effect(move || {
        let root = root_ingredients.read();
        let new_children = ingredient_path::get_at_path(&root, &parent_path_for_effect)
            .and_then(|p| p.children.clone())
            .unwrap_or_default();
        let current = wrapper.read().clone();
        if current != new_children {
            wrapper.set(new_children);
        }
    });

    // Reverse sync: wrapper -> root_ingredients
    {
        let mut root_ingredients = props.root_ingredients;
        let path_for_reverse = props.path.clone();
        use_effect(move || {
            let wrapper_ingredient = wrapper.read().get(index).cloned();
            if let Some(new_ing) = wrapper_ingredient {
                let current = {
                    let root = root_ingredients.read();
                    ingredient_path::get_at_path(&root, &path_for_reverse).cloned()
                };
                if current.as_ref() != Some(&new_ing) {
                    ingredient_path::set_at_path(
                        &mut root_ingredients.write(),
                        &path_for_reverse,
                        new_ing,
                    );
                }
            }
        });
    }

    let mut editing_path = props.editing_path;
    let entry_depth = props.entry_depth;
    let path_for_save = props.path.clone();
    let path_for_composite = props.path.clone();

    rsx! {
        IngredientPane {
            ingredients: wrapper,
            index: index,
            path: props.path.clone(),
            is_genesis: false,
            rules: props.rules,
            editing_path: Some(props.editing_path),
            depth: props.depth,
            on_edit_child: move |child_index: usize| {
                let mut current = editing_path.read().clone();
                current.push(child_index);
                editing_path.set(current);
            },
            on_save: {
                let path = path_for_save;
                let mut ingredients = props.root_ingredients;
                move |(new_ingredient, _scale_all): (Ingredient, bool)| {
                    ingredient_path::set_at_path(
                        &mut ingredients.write(),
                        &path,
                        new_ingredient,
                    );
                    // Pop one level, but close entirely if we'd go below entry depth
                    let mut current = editing_path.read().clone();
                    current.pop();
                    if current.len() < entry_depth() {
                        editing_path.set(vec![]);
                    } else {
                        editing_path.set(current);
                    }
                }
            },
            on_close: move |_| {
                // Pop one level, but close entirely if we'd go below entry depth
                let mut current = editing_path.read().clone();
                current.pop();
                if current.len() < entry_depth() {
                    editing_path.set(vec![]);
                } else {
                    editing_path.set(current);
                }
            },
            on_composite_changed: {
                let path = path_for_composite;
                move |is_composite: bool| {
                    if !is_composite {
                        editing_path.set(path.clone());
                    }
                }
            },
            is_sub_ingredient: true,
        }
    }
}

/// Genesis modal for adding a new top-level ingredient.
#[derive(Props, Clone, PartialEq)]
pub struct GenesisModalProps {
    pub ingredients: Signal<Vec<Ingredient>>,
    pub rules: Memo<Vec<RuleDef>>,
}

pub fn GenesisModal(props: GenesisModalProps) -> Element {
    let mut is_open = use_signal(|| false);
    let mut genesis_ingredients = use_signal(|| vec![Ingredient::default()]);
    // Editing path for drilling into child ingredients within genesis mode.
    // Empty = showing the genesis pane itself, [0, child_idx] = editing a child, etc.
    let mut genesis_editing_path: Signal<IngredientPath> = use_signal(Vec::new);

    let genesis_save = {
        let mut ingredients = props.ingredients;
        move |new_ingredient: Ingredient, close: bool| {
            if new_ingredient.children.is_none() || new_ingredient.children.as_ref().unwrap().is_empty() {
                let mut existing = ingredients.write();
                if let Some(existing_index) = existing.iter().position(|ing| {
                    ing.name == new_ingredient.name
                    && ing.is_allergen == new_ingredient.is_allergen
                    && ing.is_namensgebend == new_ingredient.is_namensgebend
                    && (ing.children.is_none() || ing.children.as_ref().unwrap().is_empty())
                }) {
                    existing[existing_index].amount += new_ingredient.amount;
                    if new_ingredient.category.is_some() {
                        existing[existing_index].category = new_ingredient.category;
                    }
                } else {
                    existing.push(new_ingredient);
                }
            } else {
                ingredients.write().push(new_ingredient);
            }
            if close {
                is_open.set(false);
            }
        }
    };

    let child_depth = genesis_editing_path.read().len();
    let is_editing_child = child_depth > 0;

    // Entry depth for genesis child navigation: children are at path [0, child_idx] (length 2).
    // When popping below this, we should return to the genesis pane (clear path).
    let genesis_entry_depth = use_signal(|| 2usize);

    rsx! {
        button {
            class: "btn btn-accent",
            onclick: move |_| {
                if !is_open() {
                    // Reset to blank ingredient when opening
                    genesis_ingredients.set(vec![Ingredient::default()]);
                    genesis_editing_path.set(vec![]);
                }
                is_open.toggle();
            },
            "{t!(\"nav.hinzufuegen\").to_string()}"
        }
        dialog { open: "{is_open}", class: "modal",
            div {
                class: "modal-box bg-base-100 max-w-2xl w-full p-0",
                style: "overflow: visible; min-height: 80vh; max-height: 90vh;",
                div {
                    class: "relative w-full",
                    style: "min-height: 80vh;",

                    // Background title bar (only when editing a child)
                    if is_editing_child {
                        {
                            let name = genesis_ingredients.read().first()
                                .map(|i| if i.name.is_empty() { t!("label.zutatDetails").to_string() } else { i.name.clone() })
                                .unwrap_or_default();
                            rsx! {
                                div {
                                    class: "absolute left-0 right-0 bg-base-200 rounded-t-lg shadow-sm pointer-events-none",
                                    style: "top: 0; height: {TITLE_BAR_HEIGHT}px; z-index: 0;",
                                    div { class: "font-bold text-lg px-4 py-2 truncate text-base-content/60",
                                        "{name}"
                                    }
                                }
                            }
                        }
                    }

                    // Genesis pane — always mounted to preserve signal state (hidden when editing child)
                    div {
                        class: if is_editing_child { "absolute left-0 right-0 bottom-0 invisible" } else { "absolute left-0 right-0 bottom-0 bg-base-100 rounded-t-lg shadow-xl overflow-y-auto" },
                        style: "top: 0; z-index: 0;",
                        div { class: "p-4",
                            if is_open() {
                                IngredientPane {
                                    ingredients: genesis_ingredients,
                                    index: 0,
                                    path: vec![],
                                    is_genesis: true,
                                    rules: props.rules,
                                    editing_path: Some(genesis_editing_path),
                                    on_edit_child: move |child_index: usize| {
                                        genesis_editing_path.set(vec![0, child_index]);
                                    },
                                    on_save: {
                                        let mut genesis_save = genesis_save;
                                        move |(new_ingredient, _scale_all): (Ingredient, bool)| {
                                            genesis_save(new_ingredient, true);
                                        }
                                    },
                                    on_save_and_next: {
                                        let mut genesis_save = genesis_save;
                                        move |(new_ingredient, _scale_all): (Ingredient, bool)| {
                                            genesis_save(new_ingredient, false);
                                        }
                                    },
                                    on_close: move |_| {
                                        is_open.set(false);
                                    },
                                    focus_trigger: Some(is_open),
                                }
                            }
                        }
                    }

                    // Render child cards on top when drilling into children
                    for depth in 0..child_depth {
                        {
                            let card_path = genesis_editing_path.read()[..=depth].to_vec();
                            let is_topmost = depth == child_depth - 1;
                            let top_offset = (depth + 1) as f64 * TITLE_BAR_HEIGHT;

                            if is_topmost {
                                rsx! {
                                    div {
                                        class: "absolute left-0 right-0 bottom-0 bg-base-100 rounded-t-lg shadow-xl overflow-y-auto",
                                        style: "top: {top_offset}px; z-index: {depth + 1};",
                                        div { class: "sticky top-0 z-10 bg-base-100 border-b border-base-300 px-4 py-2 rounded-t-lg",
                                            {
                                                let name = {
                                                    let root = genesis_ingredients.read();
                                                    ingredient_path::get_at_path(&root, &card_path)
                                                        .map(|i| if i.name.is_empty() { t!("label.zutatDetails").to_string() } else { i.name.clone() })
                                                        .unwrap_or_default()
                                                };
                                                rsx! { h3 { class: "font-bold text-lg truncate", "{name}" } }
                                            }
                                        }
                                        div { class: "p-4",
                                            NestedCard {
                                                root_ingredients: genesis_ingredients,
                                                path: card_path,
                                                rules: props.rules,
                                                editing_path: genesis_editing_path,
                                                depth: depth + 1,
                                                entry_depth: genesis_entry_depth,
                                            }
                                        }
                                    }
                                }
                            } else {
                                let name = {
                                    let root = genesis_ingredients.read();
                                    ingredient_path::get_at_path(&root, &card_path)
                                        .map(|i| if i.name.is_empty() { t!("label.zutatDetails").to_string() } else { i.name.clone() })
                                        .unwrap_or_default()
                                };
                                rsx! {
                                    div {
                                        class: "absolute left-0 right-0 bg-base-200 rounded-t-lg shadow-sm pointer-events-none",
                                        style: "top: {top_offset}px; height: {TITLE_BAR_HEIGHT}px; z-index: {depth + 1};",
                                        div { class: "font-bold text-lg px-4 py-2 truncate text-base-content/60",
                                            "{name}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            form {
                method: "dialog",
                class: "modal-backdrop",
                onclick: move |_| {
                    if is_editing_child {
                        genesis_editing_path.set(vec![]);
                    } else {
                        is_open.set(false);
                    }
                },
                button { "" }
            }
        }
    }
}
