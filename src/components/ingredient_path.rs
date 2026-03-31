#![allow(dead_code)]

use crate::core::Ingredient;

/// A path into the ingredient tree. Each element is an index into a Vec<Ingredient>.
/// `[2]` = top-level ingredient at index 2.
/// `[2, 0]` = child 0 of top-level ingredient 2.
/// `[2, 0, 3]` = grandchild 3 of child 0 of top-level 2.
pub type IngredientPath = Vec<usize>;

/// Get a reference to the ingredient at the given path.
pub fn get_at_path<'a>(ingredients: &'a [Ingredient], path: &[usize]) -> Option<&'a Ingredient> {
    match path {
        [] => None,
        [first] => ingredients.get(*first),
        [first, rest @ ..] => {
            let parent = ingredients.get(*first)?;
            let children = parent.children.as_ref()?;
            get_at_path(children, rest)
        }
    }
}

/// Get a mutable reference to the ingredient at the given path.
pub fn get_at_path_mut<'a>(
    ingredients: &'a mut [Ingredient],
    path: &[usize],
) -> Option<&'a mut Ingredient> {
    match path {
        [] => None,
        [first] => ingredients.get_mut(*first),
        [first, rest @ ..] => {
            let parent = ingredients.get_mut(*first)?;
            let children = parent.children.as_mut()?;
            get_at_path_mut(children, rest)
        }
    }
}

/// Replace the ingredient at the given path.
/// Returns false if the path is invalid.
pub fn set_at_path(ingredients: &mut [Ingredient], path: &[usize], value: Ingredient) -> bool {
    match path {
        [] => false,
        [index] => {
            if *index < ingredients.len() {
                ingredients[*index] = value;
                true
            } else {
                false
            }
        }
        [first, rest @ ..] => {
            if let Some(parent) = ingredients.get_mut(*first) {
                if let Some(children) = &mut parent.children {
                    set_at_path(children, rest, value)
                } else {
                    false
                }
            } else {
                false
            }
        }
    }
}

/// Remove the ingredient at the given path.
/// Returns the removed ingredient, or None if the path is invalid.
pub fn remove_at_path(ingredients: &mut Vec<Ingredient>, path: &[usize]) -> Option<Ingredient> {
    match path {
        [] => None,
        [index] => {
            if *index < ingredients.len() {
                Some(ingredients.remove(*index))
            } else {
                None
            }
        }
        [first, rest @ ..] => {
            let parent = ingredients.get_mut(*first)?;
            let children = parent.children.as_mut()?;
            remove_at_path(children, rest)
        }
    }
}

/// Add a child ingredient to the ingredient at the given path.
/// Creates the children vec if it doesn't exist.
/// Returns false if the path is invalid.
pub fn add_child_at_path(
    ingredients: &mut [Ingredient],
    path: &[usize],
    child: Ingredient,
) -> bool {
    match path {
        [] => false,
        [index] => {
            if let Some(parent) = ingredients.get_mut(*index) {
                match &mut parent.children {
                    Some(children) => children.push(child),
                    None => parent.children = Some(vec![child]),
                }
                true
            } else {
                false
            }
        }
        [first, rest @ ..] => {
            if let Some(parent) = ingredients.get_mut(*first) {
                if let Some(children) = &mut parent.children {
                    add_child_at_path(children, rest, child)
                } else {
                    false
                }
            } else {
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tree() -> Vec<Ingredient> {
        // Tree structure:
        // [0] "Sauce" -> children: [0] "Salt", [1] "Pepper"
        // [1] "Flour"
        // [2] "Dressing" -> children: [0] "Oil" -> children: [0] "Olive"]
        vec![
            Ingredient {
                name: "Sauce".into(),
                children: Some(vec![
                    Ingredient {
                        name: "Salt".into(),
                        ..Default::default()
                    },
                    Ingredient {
                        name: "Pepper".into(),
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            },
            Ingredient {
                name: "Flour".into(),
                ..Default::default()
            },
            Ingredient {
                name: "Dressing".into(),
                children: Some(vec![Ingredient {
                    name: "Oil".into(),
                    children: Some(vec![Ingredient {
                        name: "Olive".into(),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }]),
                ..Default::default()
            },
        ]
    }

    #[test]
    fn get_at_path_top_level() {
        let tree = make_tree();
        assert_eq!(get_at_path(&tree, &[0]).unwrap().name, "Sauce");
        assert_eq!(get_at_path(&tree, &[1]).unwrap().name, "Flour");
        assert_eq!(get_at_path(&tree, &[2]).unwrap().name, "Dressing");
    }

    #[test]
    fn get_at_path_nested() {
        let tree = make_tree();
        assert_eq!(get_at_path(&tree, &[0, 0]).unwrap().name, "Salt");
        assert_eq!(get_at_path(&tree, &[0, 1]).unwrap().name, "Pepper");
        assert_eq!(get_at_path(&tree, &[2, 0]).unwrap().name, "Oil");
        assert_eq!(get_at_path(&tree, &[2, 0, 0]).unwrap().name, "Olive");
    }

    #[test]
    fn get_at_path_invalid() {
        let tree = make_tree();
        assert!(get_at_path(&tree, &[]).is_none());
        assert!(get_at_path(&tree, &[5]).is_none());
        assert!(get_at_path(&tree, &[1, 0]).is_none()); // Flour has no children
        assert!(get_at_path(&tree, &[0, 5]).is_none());
    }

    #[test]
    fn get_at_path_mut_works() {
        let mut tree = make_tree();
        get_at_path_mut(&mut tree, &[0, 0]).unwrap().name = "Sea Salt".into();
        assert_eq!(get_at_path(&tree, &[0, 0]).unwrap().name, "Sea Salt");
    }

    #[test]
    fn set_at_path_top_level() {
        let mut tree = make_tree();
        let replacement = Ingredient {
            name: "Sugar".into(),
            ..Default::default()
        };
        assert!(set_at_path(&mut tree, &[1], replacement));
        assert_eq!(tree[1].name, "Sugar");
    }

    #[test]
    fn set_at_path_nested() {
        let mut tree = make_tree();
        let replacement = Ingredient {
            name: "Chili".into(),
            ..Default::default()
        };
        assert!(set_at_path(&mut tree, &[0, 1], replacement));
        assert_eq!(get_at_path(&tree, &[0, 1]).unwrap().name, "Chili");
    }

    #[test]
    fn set_at_path_invalid() {
        let mut tree = make_tree();
        let replacement = Ingredient {
            name: "X".into(),
            ..Default::default()
        };
        assert!(!set_at_path(&mut tree, &[], replacement.clone()));
        assert!(!set_at_path(&mut tree, &[10], replacement.clone()));
        assert!(!set_at_path(&mut tree, &[1, 0], replacement)); // Flour has no children
    }

    #[test]
    fn remove_at_path_top_level() {
        let mut tree = make_tree();
        let removed = remove_at_path(&mut tree, &[1]).unwrap();
        assert_eq!(removed.name, "Flour");
        assert_eq!(tree.len(), 2);
    }

    #[test]
    fn remove_at_path_nested() {
        let mut tree = make_tree();
        let removed = remove_at_path(&mut tree, &[0, 0]).unwrap();
        assert_eq!(removed.name, "Salt");
        assert_eq!(tree[0].children.as_ref().unwrap().len(), 1);
        assert_eq!(tree[0].children.as_ref().unwrap()[0].name, "Pepper");
    }

    #[test]
    fn remove_at_path_deep() {
        let mut tree = make_tree();
        let removed = remove_at_path(&mut tree, &[2, 0, 0]).unwrap();
        assert_eq!(removed.name, "Olive");
        assert!(tree[2].children.as_ref().unwrap()[0]
            .children
            .as_ref()
            .unwrap()
            .is_empty());
    }

    #[test]
    fn remove_at_path_invalid() {
        let mut tree = make_tree();
        assert!(remove_at_path(&mut tree, &[]).is_none());
        assert!(remove_at_path(&mut tree, &[10]).is_none());
    }

    #[test]
    fn add_child_at_path_existing_children() {
        let mut tree = make_tree();
        let child = Ingredient {
            name: "Garlic".into(),
            ..Default::default()
        };
        assert!(add_child_at_path(&mut tree, &[0], child));
        let children = tree[0].children.as_ref().unwrap();
        assert_eq!(children.len(), 3);
        assert_eq!(children[2].name, "Garlic");
    }

    #[test]
    fn add_child_at_path_no_children() {
        let mut tree = make_tree();
        let child = Ingredient {
            name: "Yeast".into(),
            ..Default::default()
        };
        assert!(add_child_at_path(&mut tree, &[1], child)); // Flour has no children
        let children = tree[1].children.as_ref().unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "Yeast");
    }

    #[test]
    fn add_child_at_path_deep() {
        let mut tree = make_tree();
        let child = Ingredient {
            name: "Rosemary".into(),
            ..Default::default()
        };
        assert!(add_child_at_path(&mut tree, &[2, 0], child));
        let oil_children = get_at_path(&tree, &[2, 0]).unwrap().children.as_ref().unwrap();
        assert_eq!(oil_children.len(), 2);
        assert_eq!(oil_children[1].name, "Rosemary");
    }

    #[test]
    fn add_child_at_path_invalid() {
        let mut tree = make_tree();
        let child = Ingredient {
            name: "X".into(),
            ..Default::default()
        };
        assert!(!add_child_at_path(&mut tree, &[], child.clone()));
        assert!(!add_child_at_path(&mut tree, &[10], child));
    }
}
