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

/// Collect the paths of the **shallowest** proper descendants of the ingredient at
/// `base` for which `defines` returns true. Recursion stops at a node that defines
/// the attribute (its deeper descendants are not reported separately).
///
/// Foundation for the cross-level rule: when editing a composite, an attribute is
/// "defined on another level" iff this returns a non-empty list — and those paths
/// are exactly where the user can navigate to, or clear.
pub fn descendant_definitions(
    ingredients: &[Ingredient],
    base: &[usize],
    defines: &dyn Fn(&Ingredient) -> bool,
) -> Vec<IngredientPath> {
    let mut out = Vec::new();
    let Some(node) = get_at_path(ingredients, base) else { return out };
    if let Some(children) = node.children.as_ref() {
        for (i, child) in children.iter().enumerate() {
            let mut child_path = base.to_vec();
            child_path.push(i);
            collect_definitions(child, &child_path, defines, &mut out);
        }
    }
    out
}

fn collect_definitions(
    node: &Ingredient,
    path: &[usize],
    defines: &dyn Fn(&Ingredient) -> bool,
    out: &mut Vec<IngredientPath>,
) {
    if defines(node) {
        out.push(path.to_vec());
        return; // shallowest only — don't descend past a defining node
    }
    if let Some(children) = node.children.as_ref() {
        for (i, child) in children.iter().enumerate() {
            let mut child_path = path.to_vec();
            child_path.push(i);
            collect_definitions(child, &child_path, defines, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Country;

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

    /// Tree for cross-level detection: a composite with one origin-bearing child,
    /// one nested origin (grandchild), and one plain child.
    fn origin_tree() -> Vec<Ingredient> {
        vec![Ingredient {
            name: "Composite".into(),
            children: Some(vec![
                // [0,0] direct child WITH an origin
                Ingredient { name: "Apfel".into(), origins: Some(vec![Country::CH]), ..Default::default() },
                // [0,1] composite child WITHOUT its own origin, but [0,1,0] grandchild has one
                Ingredient {
                    name: "Mix".into(),
                    children: Some(vec![
                        Ingredient { name: "Birne".into(), origins: Some(vec![Country::FR]), ..Default::default() },
                    ]),
                    ..Default::default()
                },
                // [0,2] plain child, no origin
                Ingredient { name: "Wasser".into(), ..Default::default() },
            ]),
            ..Default::default()
        }]
    }

    #[test]
    fn descendant_definitions_finds_shallowest() {
        let tree = origin_tree();
        let has_origin = |i: &Ingredient| i.origins.as_ref().is_some_and(|o| !o.is_empty());
        let defs = descendant_definitions(&tree, &[0], &has_origin);
        // Direct child [0,0] defines it; the grandchild [0,1,0] (shallowest below [0,1]) defines it.
        assert!(defs.contains(&vec![0, 0]), "direct origin child should be found: {:?}", defs);
        assert!(defs.contains(&vec![0, 1, 0]), "nested origin grandchild should be found: {:?}", defs);
        assert_eq!(defs.len(), 2, "Wasser has no origin; expected exactly 2 definitions: {:?}", defs);
    }

    #[test]
    fn descendant_definitions_empty_when_none_defined() {
        let tree = make_tree(); // no origins anywhere
        let has_origin = |i: &Ingredient| i.origins.as_ref().is_some_and(|o| !o.is_empty());
        assert!(descendant_definitions(&tree, &[0], &has_origin).is_empty());
    }

    #[test]
    fn descendant_definitions_stops_at_shallowest() {
        // A defining composite child should be reported once, not also its children.
        let tree = vec![Ingredient {
            name: "Root".into(),
            children: Some(vec![Ingredient {
                name: "Sub".into(),
                origins: Some(vec![Country::CH]),
                children: Some(vec![
                    Ingredient { name: "Deep".into(), origins: Some(vec![Country::DE]), ..Default::default() },
                ]),
                ..Default::default()
            }]),
            ..Default::default()
        }];
        let has_origin = |i: &Ingredient| i.origins.as_ref().is_some_and(|o| !o.is_empty());
        let defs = descendant_definitions(&tree, &[0], &has_origin);
        assert_eq!(defs, vec![vec![0, 0]], "should stop at the shallowest defining node");
    }
}
