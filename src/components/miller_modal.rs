use crate::components::ingredient_pane::IngredientPane;
use crate::components::ingredient_path::IngredientPath;
use crate::core::Ingredient;
use crate::rules::RuleDef;
use dioxus::prelude::*;
use rust_i18n::t;

/// Miller Columns modal for editing ingredients with two-pane navigation.
///
/// - Path length 0: modal closed
/// - Path length 1: single-pane (top-level ingredient)
/// - Path length 2+: two-pane (parent on left greyed out, child on right)
#[derive(Props, Clone, PartialEq)]
pub struct MillerModalProps {
    pub ingredients: Signal<Vec<Ingredient>>,
    pub editing_path: Signal<IngredientPath>,
    pub rules: Memo<Vec<RuleDef>>,
}

pub fn MillerModal(props: MillerModalProps) -> Element {
    let mut editing_path = props.editing_path;
    let path = editing_path.read().clone();
    let is_open = !path.is_empty();
    let is_two_pane = path.len() >= 2;

    // Derive left and right pane paths
    let left_path = if is_two_pane {
        path[..path.len() - 1].to_vec()
    } else {
        path.clone()
    };
    let right_path = path.clone();

    // Clone paths for use in closures within RSX
    let left_path_for_save = left_path.clone();
    let left_path_for_composite = left_path.clone();
    let right_path_for_save = right_path.clone();
    let right_path_for_composite = right_path.clone();

    rsx! {
        dialog { open: is_open, class: "modal",
            div {
                class: if is_two_pane {
                    "modal-box bg-base-100 max-w-5xl w-full"
                } else {
                    "modal-box bg-base-100"
                },
                div { class: "flex gap-4",
                    // Left pane
                    if is_open {
                        div { class: if is_two_pane { "flex-1 min-w-0" } else { "flex-1" },
                            if left_path.len() == 1 {
                                IngredientPane {
                                    ingredients: props.ingredients,
                                    index: left_path[0],
                                    path: left_path.clone(),
                                    is_genesis: false,
                                    rules: props.rules,
                                    editing_path: Some(editing_path),
                                    on_edit_child: move |child_index: usize| {
                                        let mut current = editing_path.read().clone();
                                        current.push(child_index);
                                        editing_path.set(current);
                                    },
                                    on_save: {
                                        let left_path = left_path_for_save.clone();
                                        let mut ingredients = props.ingredients;
                                        move |(new_ingredient, scale_all): (Ingredient, bool)| {
                                            if left_path.len() == 1 {
                                                let index = left_path[0];
                                                if scale_all {
                                                    let original_amount = ingredients.get(index).map(|i| i.amount).unwrap_or(0.0);
                                                    if original_amount > 0.0 {
                                                        let factor = new_ingredient.amount / original_amount;
                                                        let mut all = ingredients.write();
                                                        for (i, ingredient) in all.iter_mut().enumerate() {
                                                            if i == index {
                                                                *ingredient = new_ingredient.clone();
                                                            } else {
                                                                ingredient.amount *= factor;
                                                            }
                                                        }
                                                    } else {
                                                        ingredients.write()[index] = new_ingredient;
                                                    }
                                                } else {
                                                    ingredients.write()[index] = new_ingredient;
                                                }
                                            } else {
                                                crate::components::ingredient_path::set_at_path(
                                                    &mut ingredients.write(),
                                                    &left_path,
                                                    new_ingredient,
                                                );
                                            }
                                            editing_path.set(vec![]);
                                        }
                                    },
                                    on_close: move |_| {
                                        editing_path.set(vec![]);
                                    },
                                    on_composite_changed: {
                                        let left_path = left_path_for_composite.clone();
                                        move |is_composite: bool| {
                                            if !is_composite {
                                                editing_path.set(left_path.clone());
                                            }
                                        }
                                    },
                                    disabled: is_two_pane,
                                }
                            } else if !left_path.is_empty() {
                                NestedPane {
                                    root_ingredients: props.ingredients,
                                    path: left_path.clone(),
                                    rules: props.rules,
                                    editing_path: editing_path,
                                    on_edit_child: move |child_index: usize| {
                                        let mut current = editing_path.read().clone();
                                        current.push(child_index);
                                        editing_path.set(current);
                                    },
                                    on_save: {
                                        let left_path = left_path_for_save.clone();
                                        let mut ingredients = props.ingredients;
                                        move |(new_ingredient, _scale_all): (Ingredient, bool)| {
                                            crate::components::ingredient_path::set_at_path(
                                                &mut ingredients.write(),
                                                &left_path,
                                                new_ingredient,
                                            );
                                            editing_path.set(vec![]);
                                        }
                                    },
                                    on_close: move |_| {
                                        editing_path.set(vec![]);
                                    },
                                    on_composite_changed: {
                                        let left_path = left_path_for_composite.clone();
                                        move |is_composite: bool| {
                                            if !is_composite {
                                                editing_path.set(left_path.clone());
                                            }
                                        }
                                    },
                                    disabled: is_two_pane,
                                }
                            }
                        }
                    }

                    // Right pane (only in two-pane mode) — always nested
                    if is_two_pane {
                        div { class: "flex-1 min-w-0 border-l border-base-300 pl-4",
                            NestedPane {
                                root_ingredients: props.ingredients,
                                path: right_path.clone(),
                                rules: props.rules,
                                editing_path: editing_path,
                                on_edit_child: move |child_index: usize| {
                                    let mut current = editing_path.read().clone();
                                    current.push(child_index);
                                    editing_path.set(current);
                                },
                                on_save: {
                                    let right_path = right_path_for_save.clone();
                                    let mut ingredients = props.ingredients;
                                    move |(new_ingredient, _scale_all): (Ingredient, bool)| {
                                        crate::components::ingredient_path::set_at_path(
                                            &mut ingredients.write(),
                                            &right_path,
                                            new_ingredient,
                                        );
                                        let mut current = editing_path.read().clone();
                                        current.pop();
                                        editing_path.set(current);
                                    }
                                },
                                on_close: move |_| {
                                    let mut current = editing_path.read().clone();
                                    current.pop();
                                    editing_path.set(current);
                                },
                                on_composite_changed: {
                                    let right_path = right_path_for_composite.clone();
                                    move |is_composite: bool| {
                                        if !is_composite {
                                            editing_path.set(right_path.clone());
                                        }
                                    }
                                },
                                is_sub_ingredient: true,
                            }
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

/// Component that wraps IngredientPane for a nested path.
/// Needs to be a proper component (not a plain function) because it uses hooks
/// (use_signal/use_effect) and may be conditionally rendered.
#[derive(Props, Clone, PartialEq)]
pub struct NestedPaneProps {
    root_ingredients: Signal<Vec<Ingredient>>,
    path: Vec<usize>,
    rules: Memo<Vec<RuleDef>>,
    editing_path: Signal<IngredientPath>,
    on_edit_child: EventHandler<usize>,
    on_save: EventHandler<(Ingredient, bool)>,
    on_close: EventHandler<()>,
    on_composite_changed: EventHandler<bool>,
    #[props(default = false)]
    disabled: bool,
    #[props(default = false)]
    is_sub_ingredient: bool,
}

fn NestedPane(props: NestedPaneProps) -> Element {
    let index = *props.path.last().unwrap();
    let parent_path = props.path[..props.path.len() - 1].to_vec();

    let children = {
        let root = props.root_ingredients.read();
        crate::components::ingredient_path::get_at_path(&root, &parent_path)
            .and_then(|p| p.children.clone())
            .unwrap_or_default()
    };
    let mut wrapper = use_signal(move || children);

    // Sync wrapper when root ingredients change (root → wrapper)
    let root_ingredients = props.root_ingredients;
    let parent_path_for_effect = parent_path.clone();
    use_effect(move || {
        let root = root_ingredients.read();
        let new_children = crate::components::ingredient_path::get_at_path(&root, &parent_path_for_effect)
            .and_then(|p| p.children.clone())
            .unwrap_or_default();
        let current = wrapper.read().clone();
        if current != new_children {
            wrapper.set(new_children);
        }
    });

    // Reverse sync: wrapper → root_ingredients
    // When IngredientPane writes changes to wrapper (its ingredients signal),
    // propagate them back to root_ingredients so the left pane sees the updates.
    {
        let mut root_ingredients = props.root_ingredients;
        let path_for_reverse = props.path.clone();
        use_effect(move || {
            let wrapper_ingredient = wrapper.read().get(index).cloned();
            if let Some(new_ing) = wrapper_ingredient {
                let current = {
                    let root = root_ingredients.read();
                    crate::components::ingredient_path::get_at_path(&root, &path_for_reverse).cloned()
                };
                if current.as_ref() != Some(&new_ing) {
                    crate::components::ingredient_path::set_at_path(
                        &mut root_ingredients.write(),
                        &path_for_reverse,
                        new_ing,
                    );
                }
            }
        });
    }

    rsx! {
        IngredientPane {
            ingredients: wrapper,
            index: index,
            path: props.path.clone(),
            is_genesis: false,
            rules: props.rules,
            editing_path: Some(props.editing_path),
            on_edit_child: props.on_edit_child,
            on_save: props.on_save,
            on_close: props.on_close,
            on_composite_changed: props.on_composite_changed,
            disabled: props.disabled,
            is_sub_ingredient: props.is_sub_ingredient,
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
    let genesis_ingredients = use_signal(|| vec![Ingredient::default()]);

    rsx! {
        button {
            class: "btn btn-accent",
            onclick: move |_| {
                is_open.toggle();
            },
            "{t!(\"nav.hinzufuegen\").to_string()}"
        }
        dialog { open: "{is_open}", class: "modal",
            div { class: "modal-box bg-base-100",
                if is_open() {
                    IngredientPane {
                        ingredients: genesis_ingredients,
                        index: 0,
                        path: vec![],
                        is_genesis: true,
                        rules: props.rules,
                        on_edit_child: move |_: usize| {},
                        on_save: {
                            let mut ingredients = props.ingredients;
                            move |(new_ingredient, _scale_all): (Ingredient, bool)| {
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
                                is_open.set(false);
                            }
                        },
                        on_save_and_next: {
                            let mut ingredients = props.ingredients;
                            move |(new_ingredient, _scale_all): (Ingredient, bool)| {
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
                                // Modal stays open for next ingredient
                            }
                        },
                        on_close: move |_| {
                            is_open.set(false);
                        },
                        focus_trigger: Some(is_open),
                    }
                }
            }
            form {
                method: "dialog",
                class: "modal-backdrop",
                onclick: move |_| {
                    is_open.set(false);
                },
                button { "" }
            }
        }
    }
}
