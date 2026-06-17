use crate::components::*;
use crate::components::ingredient_path::{IngredientPath, descendant_definitions};
use crate::core::{Ingredient, AmountUnit};
use crate::model::{food_db, lookup_allergen, lookup_agricultural, Country};
use crate::rules::RuleDef;
use crate::services::UnifiedIngredient;
use crate::shared::Validations;
use crate::persistence::{save_composite_ingredient, get_saved_ingredients_list};
use dioxus::prelude::*;
use rust_i18n::t;

/// A single pane of the Miller modal, containing all form fields for editing one ingredient.
/// This is the extracted form body from the old IngredientDetail component.
#[derive(Props, Clone, PartialEq)]
pub struct IngredientPaneProps {
    /// Root ingredient list signal (for mutations and reading).
    /// For genesis mode, this should be a temporary signal with one default ingredient.
    pub ingredients: Signal<Vec<Ingredient>>,
    /// Index into the ingredients signal (0 for genesis mode's wrapper).
    pub index: usize,
    /// Path for this ingredient in the real tree (used for validation display paths).
    /// For genesis, this can be empty.
    pub path: IngredientPath,
    #[props(default = false)]
    pub is_genesis: bool,
    pub rules: Memo<Vec<RuleDef>>,
    /// Called when user clicks "edit" on a sub-ingredient child (fallback if editing_path not set).
    pub on_edit_child: EventHandler<usize>,
    /// Optional: direct access to the editing path signal for drill-down.
    /// When set, clicking edit on a sub-ingredient will auto-save and push onto this path directly,
    /// bypassing the on_edit_child callback chain.
    #[props(default = None)]
    pub editing_path: Option<Signal<IngredientPath>>,
    /// Called when user saves the ingredient. Payload: (new_ingredient, scale_all, factor)
    /// where `factor` is `new_amount / original_amount` (1.0 if unchanged or not applicable).
    pub on_save: EventHandler<(Ingredient, bool, f64)>,
    /// Called when user cancels / closes.
    pub on_close: EventHandler<()>,
    /// Called when the composite checkbox changes. `false` means children were removed.
    #[props(default)]
    pub on_composite_changed: EventHandler<bool>,
    /// When true, the pane is greyed out and non-interactive (used for the left pane in two-pane mode).
    #[props(default = false)]
    pub disabled: bool,
    /// When true, this pane edits a sub-ingredient (shows different title).
    #[props(default = false)]
    pub is_sub_ingredient: bool,
    /// Called when user saves and wants to add the next ingredient (genesis only).
    #[props(default)]
    pub on_save_and_next: EventHandler<(Ingredient, bool, f64)>,
    /// Signal to trigger focus on the name input when it becomes true.
    #[props(default = None)]
    pub focus_trigger: Option<Signal<bool>>,
    /// Depth in the ingredient tree (0 = top-level). Controls visibility of
    /// "merken" and "anteilsmässig übertragen" buttons (shown only at depth 0).
    #[props(default = 0)]
    pub depth: usize,
}

/// True when any descendant of `synth[0]` matches `pred` — i.e. the attribute is
/// already defined on a sub-ingredient, so the composite control is cross-level
/// locked (greyed, read-only). `synth` is a one-element wrapper Vec holding the
/// composite's live children.
fn cross_level_locked(synth: &[Ingredient], pred: &dyn Fn(&Ingredient) -> bool) -> bool {
    !descendant_definitions(synth, &[0], pred).is_empty()
}

pub fn IngredientPane(props: IngredientPaneProps) -> Element {
    let index = props.index;
    let ingredients = props.ingredients;

    let original_ingredient = if let Some(ingredient) = ingredients.get(index) {
        ingredient.clone()
    } else {
        tracing::warn!("Invalid ingredient index {}, using default", index);
        Ingredient::default()
    };

    // Local state for editing
    let mut edit_name = use_signal(|| original_ingredient.name.clone());
    let mut edit_amount = use_signal(|| {
        if props.is_genesis {
            None
        } else {
            Some(original_ingredient.amount)
        }
    });
    let mut edit_unit = use_signal(|| original_ingredient.unit.clone());
    let mut edit_is_composite = use_signal(|| {
        original_ingredient.children.as_ref().is_some_and(|s| !s.is_empty())
    });
    let mut edit_is_namensgebend = use_signal(|| {
        original_ingredient.is_namensgebend.unwrap_or(false)
    });
    let mut edit_children = use_signal(|| original_ingredient.children.clone());
    let mut edit_category = use_signal(|| original_ingredient.category.clone());
    // Canonical food_db name when the ingredient name is a curated alias term.
    // Drives allergen/agricultural lookups (the alias name itself isn't in food_db).
    let mut edit_canonical = use_signal(|| original_ingredient.canonical.clone());
    let mut edit_origins = use_signal(|| original_ingredient.origins.clone());
    let mut edit_aufzucht_ort = use_signal(|| original_ingredient.aufzucht_ort.clone());
    let mut edit_schlachtungs_ort = use_signal(|| original_ingredient.schlachtungs_ort.clone());
    let mut edit_fangort = use_signal(|| original_ingredient.fangort.clone());
    let mut edit_is_bio = use_signal(|| original_ingredient.is_bio.unwrap_or(false));
    let mut edit_bio_ch = use_signal(|| original_ingredient.bio_ch.unwrap_or(false));
    let mut edit_erlaubte_ausnahme_bio = use_signal(|| original_ingredient.erlaubte_ausnahme_bio.unwrap_or(false));
    let mut edit_erlaubte_ausnahme_bio_details = use_signal(|| original_ingredient.erlaubte_ausnahme_bio_details.clone().unwrap_or_default());
    let mut edit_erlaubte_ausnahme_knospe = use_signal(|| original_ingredient.erlaubte_ausnahme_knospe.unwrap_or(false));
    let mut edit_erlaubte_ausnahme_knospe_details = use_signal(|| original_ingredient.erlaubte_ausnahme_knospe_details.clone().unwrap_or_default());
    let mut edit_processing_steps = use_signal(|| original_ingredient.processing_steps.clone());
    let mut edit_aus_umstellbetrieb = use_signal(|| original_ingredient.aus_umstellbetrieb.unwrap_or(false));
    let mut edit_nicht_landwirtschaftlich = use_signal(|| {
        !original_ingredient.is_agricultural
            && original_ingredient.is_bio != Some(true)
            && original_ingredient.bio_ch != Some(true)
    });
    let mut save_status = use_signal(|| None::<String>);

    // Track disabled prop as a signal so use_effect closures react to changes
    let mut disabled_sig = use_signal(|| props.disabled);
    if disabled_sig() != props.disabled {
        disabled_sig.set(props.disabled);
    }

    // Derived reactively from the name so it updates as the user types (not just
    // on explicit selection): "custom" == name not an exact match in the food DB.
    let is_custom_ingredient = use_memo(move || {
        // An alias resolves to a canonical food_db entry, so it is not "custom".
        let typed = edit_name();
        let lookup = edit_canonical().unwrap_or(typed);
        !food_db().iter().any(|(name, _)| name == &lookup)
    });
    let mut is_allergen_custom = use_signal(|| original_ingredient.is_allergen);

    // Captured once at mount. Not reactive to `ingredients` changes so the
    // auto-save effect below can't overwrite it with the user's new value.
    let original_amount = use_signal(|| original_ingredient.computed_amount());

    let amount_has_changed = use_memo(move || {
        if props.is_genesis {
            false
        } else if let Some(current_amount) = edit_amount() {
            (original_amount() - current_amount).abs() > 0.01
        } else {
            false
        }
    });

    let scaling_factor = use_memo(move || {
        let orig = original_amount();
        if props.is_genesis || orig == 0.0 {
            1.0
        } else if let Some(current_amount) = edit_amount() {
            current_amount / orig
        } else {
            1.0
        }
    });

    // Wrapper ingredients signal for SubIngredientsTable
    let mut wrapper_ingredients = use_signal(|| {
        vec![Ingredient {
            name: original_ingredient.name.clone(),
            amount: original_ingredient.amount,
            unit: original_ingredient.unit.clone(),
            is_allergen: original_ingredient.is_allergen,
            is_namensgebend: original_ingredient.is_namensgebend,
            sub_components: None,
            children: original_ingredient.children.clone(),
            origins: original_ingredient.origins.clone(),
            is_agricultural: original_ingredient.is_agricultural,
            is_bio: original_ingredient.is_bio,
            category: original_ingredient.category.clone(),
            aufzucht_ort: original_ingredient.aufzucht_ort.clone(),
            schlachtungs_ort: original_ingredient.schlachtungs_ort.clone(),
            fangort: original_ingredient.fangort.clone(),
            bio_ch: original_ingredient.bio_ch,
            erlaubte_ausnahme_bio: original_ingredient.erlaubte_ausnahme_bio,
            erlaubte_ausnahme_bio_details: original_ingredient.erlaubte_ausnahme_bio_details.clone(),
            erlaubte_ausnahme_knospe: original_ingredient.erlaubte_ausnahme_knospe,
            erlaubte_ausnahme_knospe_details: original_ingredient.erlaubte_ausnahme_knospe_details.clone(),
            processing_steps: original_ingredient.processing_steps.clone(),
            aus_umstellbetrieb: original_ingredient.aus_umstellbetrieb,
            override_children: None,
            canonical: original_ingredient.canonical.clone(),
        }]
    });

    // Flush this pane's edits to the tree, then drill into the descendant at the
    // given path relative to this composite (e.g. [2] = child 2, [2, 0] = its
    // grandchild). Shared by the sub-ingredient table and the cross-level popovers
    // so the composite's in-progress edits survive the round-trip into a child card.
    let goto_descendant = use_callback(move |rel: IngredientPath| {
        let live_children = wrapper_ingredients.read().first().and_then(|i| i.children.clone());
        let mut root_ingredients = ingredients;
        if let Some(mut ing) = root_ingredients.get_mut(index) {
            ing.name = edit_name();
            ing.amount = edit_amount().unwrap_or(0.0);
            ing.unit = edit_unit();
            ing.is_allergen = is_allergen_custom();
            ing.is_namensgebend = Some(edit_is_namensgebend());
            ing.children = live_children;
            ing.origins = edit_origins();
            ing.is_bio = Some(edit_is_bio());
            ing.bio_ch = Some(edit_bio_ch());
            ing.category = edit_category();
        }
        if let Some(mut ep) = props.editing_path {
            let mut new_path = ep.read().clone();
            if new_path.is_empty() { new_path.push(index); }
            new_path.extend(rel);
            ep.set(new_path);
        }
    });
    let goto_child = use_callback(move |child_index: usize| { goto_descendant.call(vec![child_index]); });

    // Enforce: aus_umstellbetrieb requires bio or bio_ch
    use_effect(move || {
        if !edit_is_bio() && !edit_bio_ch() && edit_aus_umstellbetrieb() {
            edit_aus_umstellbetrieb.set(false);
        }
    });

    // When composite mode changes, sync the wrapper
    use_effect(move || {
        let _ = edit_is_composite();
        if edit_is_composite() {
            let current_children = edit_children();
            let children_to_use = if current_children.as_ref().is_some_and(|s| !s.is_empty()) {
                current_children
            } else {
                None
            };
            wrapper_ingredients.write()[0] = Ingredient {
                name: edit_name(),
                amount: edit_amount().unwrap_or(0.0),
                unit: edit_unit(),
                is_allergen: is_allergen_custom(),
                is_namensgebend: Some(edit_is_namensgebend()),
                sub_components: None,
                children: children_to_use,
                origins: edit_origins(),
                is_agricultural: if edit_nicht_landwirtschaftlich() { false } else { let typed = edit_name(); lookup_agricultural(&edit_canonical().unwrap_or(typed)) },
                is_bio: Some(edit_is_bio()),
                category: edit_category(),
                aufzucht_ort: edit_aufzucht_ort(),
                schlachtungs_ort: edit_schlachtungs_ort(),
                fangort: edit_fangort(),
                bio_ch: Some(edit_bio_ch()),
                erlaubte_ausnahme_bio: Some(edit_erlaubte_ausnahme_bio()),
                erlaubte_ausnahme_bio_details: if edit_erlaubte_ausnahme_bio_details().is_empty() { None } else { Some(edit_erlaubte_ausnahme_bio_details()) },
                erlaubte_ausnahme_knospe: Some(edit_erlaubte_ausnahme_knospe()),
                erlaubte_ausnahme_knospe_details: if edit_erlaubte_ausnahme_knospe_details().is_empty() { None } else { Some(edit_erlaubte_ausnahme_knospe_details()) },
                processing_steps: edit_processing_steps(),
                aus_umstellbetrieb: Some(edit_aus_umstellbetrieb()),
                override_children: None,
                canonical: edit_canonical(),
            };
        } else {
            wrapper_ingredients.write()[0].children = None;
        }
    });

    // Track changes from SubIngredientsTable back to edit state (children only).
    {
        use_effect(move || {
            if let Some(wrapper_children) = wrapper_ingredients.read().first().and_then(|i| i.children.as_ref()) {
                let current_edit_children = edit_children();
                if current_edit_children.as_ref() != Some(wrapper_children) {
                    edit_children.set(Some(wrapper_children.clone()));
                }
            }
        });
    }

    // Auto-sync all edit fields back to the ingredients signal for live updates.
    // This enables the left pane to reflect right-pane changes immediately.
    {
        let mut ingredients = props.ingredients;
        let pane_index = props.index;
        let is_genesis = props.is_genesis;
        use_effect(move || {
            if is_genesis || disabled_sig() {
                return;
            }
            // Read all edit signals to subscribe to their changes
            let name = edit_name();
            let amount = edit_amount().unwrap_or(0.0);
            let unit = edit_unit();
            let children = edit_children();
            let allergen = is_allergen_custom();
            let namensgebend = edit_is_namensgebend();
            let category = edit_category();
            let origins = edit_origins();
            let aufzucht_ort = edit_aufzucht_ort();
            let schlachtungs_ort = edit_schlachtungs_ort();
            let fangort = edit_fangort();
            let is_bio = edit_is_bio();
            let bio_ch = edit_bio_ch();
            let erlaubte_ausnahme_bio = edit_erlaubte_ausnahme_bio();
            let erlaubte_ausnahme_bio_details = edit_erlaubte_ausnahme_bio_details();
            let erlaubte_ausnahme_knospe = edit_erlaubte_ausnahme_knospe();
            let erlaubte_ausnahme_knospe_details = edit_erlaubte_ausnahme_knospe_details();
            let processing_steps = edit_processing_steps();
            let aus_umstellbetrieb = edit_aus_umstellbetrieb();
            let canonical = edit_canonical();
            // Resolve allergen/agricultural against the canonical entry when the
            // displayed name is an alias (the alias itself isn't in food_db).
            let lookup_name = canonical.clone().unwrap_or_else(|| name.clone());

            let new_ing = Ingredient {
                name,
                amount,
                unit,
                is_allergen: allergen,
                is_namensgebend: Some(namensgebend),
                sub_components: None,
                children,
                origins,
                is_agricultural: if edit_nicht_landwirtschaftlich() { false } else { lookup_agricultural(&lookup_name) },
                is_bio: Some(is_bio),
                category,
                aufzucht_ort,
                schlachtungs_ort,
                fangort,
                bio_ch: Some(bio_ch),
                erlaubte_ausnahme_bio: Some(erlaubte_ausnahme_bio),
                erlaubte_ausnahme_bio_details: if erlaubte_ausnahme_bio_details.is_empty() { None } else { Some(erlaubte_ausnahme_bio_details) },
                erlaubte_ausnahme_knospe: Some(erlaubte_ausnahme_knospe),
                erlaubte_ausnahme_knospe_details: if erlaubte_ausnahme_knospe_details.is_empty() { None } else { Some(erlaubte_ausnahme_knospe_details) },
                processing_steps,
                aus_umstellbetrieb: Some(aus_umstellbetrieb),
                override_children: None,
                canonical,
            };

            let needs_update = ingredients.get(pane_index)
                .map(|current| *current != new_ing)
                .unwrap_or(false);
            if needs_update {
                ingredients.write()[pane_index] = new_ing;
            }
        });
    }

    // Computed-value sync: when composite (has children), update edit signals
    // from the ingredient's computed values so the parent reflects aggregated
    // child data (weight sum, bio AND, origins union, allergen any).
    // Computed-value sync: when the ingredients signal changes and we're composite,
    // update edit signals from computed values. Uses peek() for edit signals so this
    // effect only re-runs when `ingredients` or `edit_is_composite` change — NOT when
    // edit signals change (which would fight with SubIngredientsTable additions).
    {
        let ingredients = props.ingredients;
        let pane_index = props.index;
        let is_genesis = props.is_genesis;
        use_effect(move || {
            if !edit_is_composite() { return; }
            // Source of truth for the composite differs by mode. In genesis the
            // ingredient isn't committed to `props.ingredients` yet (the auto-sync
            // effect skips genesis), so its computed values and children live in
            // `wrapper_ingredients`. Reading the empty committed slot here would
            // clobber the children/origins/bio that a saved-composite recall
            // (handle_ingredient_select) just restored into the edit_* signals.
            let source = if is_genesis {
                wrapper_ingredients.read().first().cloned()
            } else {
                ingredients.read().get(pane_index).cloned()
            };
            if let Some(ing) = source {
                let ca = ing.computed_amount();
                if edit_amount.peek().is_none_or(|a| (a - ca).abs() > 0.001) {
                    edit_amount.set(Some(ca));
                }
                let cu = ing.computed_unit();
                if *edit_unit.peek() != cu { edit_unit.set(cu); }
                let cb = ing.computed_bio_status().unwrap_or(false);
                if *edit_is_bio.peek() != cb { edit_is_bio.set(cb); }
                let cbc = ing.computed_bio_ch_status().unwrap_or(false);
                if *edit_bio_ch.peek() != cbc { edit_bio_ch.set(cbc); }
                // NOTE: origin is intentionally NOT mirrored up from children here.
                // Origin is single-level: a composite's parent origin must stay
                // purely user-authored (top-down) and empty for bottom-up composites,
                // so it doesn't create a spurious two-level conflict. The label still
                // shows the aggregated origin via `computed_origins()`.
                let ch = ing.children.clone();
                if *edit_children.peek() != ch { edit_children.set(ch); }
                // Allergen: true if any child is allergen
                let any_allergen = ing.children.as_ref()
                    .is_some_and(|children| children.iter().any(|c| c.is_allergen));
                if *is_allergen_custom.peek() != any_allergen { is_allergen_custom.set(any_allergen); }
            }
        });
    }

    // Reverse sync: when the real ingredients signal changes (e.g. child card saved),
    // update wrapper_ingredients so SubIngredientsTable reflects the change.
    // Use peek() for wrapper to avoid subscribing — otherwise wrapper-originated changes
    // (like delete) would trigger this effect before auto-sync propagates, undoing the delete.
    {
        let ingredients = props.ingredients;
        let pane_index = props.index;
        use_effect(move || {
            let real_children = ingredients.read()
                .get(pane_index)
                .and_then(|i| i.children.clone());
            let wrapper_children = wrapper_ingredients.peek()
                .first()
                .and_then(|i| i.children.clone());
            if real_children != wrapper_children {
                if let Some(children) = real_children {
                    wrapper_ingredients.write()[0].children = Some(children);
                }
            }
        });
    }

    let handle_ingredient_select = move |unified_ingredient: UnifiedIngredient| {
        edit_name.set(unified_ingredient.name.clone());
        edit_canonical.set(unified_ingredient.canonical.clone());
        edit_category.set(unified_ingredient.category.clone());

        if let Some(category) = &unified_ingredient.category {
            web_sys::console::log_1(&format!("Ingredient '{}' category: {}", unified_ingredient.name, category).into());
        }

        if let Some(is_allergen) = unified_ingredient.is_allergen {
            is_allergen_custom.set(is_allergen);
        }

        if let Some(is_bio) = unified_ingredient.is_bio {
            edit_is_bio.set(is_bio);
        }

        // If this name matches a user-saved composite ingredient, restore its hierarchy.
        // Saved entries always have children (handle_save_to_storage enforces it), so the
        // is_composite flag can be set unconditionally on a hit.
        let saved_ingredients = get_saved_ingredients_list();
        if let Some(saved) = saved_ingredients.into_iter().find(|i| i.name == unified_ingredient.name) {
            edit_is_composite.set(true);
            edit_children.set(saved.children.clone());
            edit_is_namensgebend.set(saved.is_namensgebend.unwrap_or(false));
            edit_unit.set(saved.unit.clone());
            edit_origins.set(saved.origins.clone());
            is_allergen_custom.set(saved.is_allergen);
            if saved.category.is_some() {
                edit_category.set(saved.category.clone());
            }
        }
    };

    let handle_save_to_storage = move |_| {
        if edit_is_composite() && edit_children().is_some() {
            let ingredient_to_save = Ingredient {
                name: edit_name(),
                amount: 100.0,
                unit: edit_unit(),
                is_allergen: is_allergen_custom(),
                is_namensgebend: Some(edit_is_namensgebend()),
                sub_components: None,
                children: edit_children(),
                origins: edit_origins(),
                is_agricultural: if edit_nicht_landwirtschaftlich() { false } else { let typed = edit_name(); lookup_agricultural(&edit_canonical().unwrap_or(typed)) },
                is_bio: Some(edit_is_bio()),
                category: edit_category(),
                aufzucht_ort: edit_aufzucht_ort(),
                schlachtungs_ort: edit_schlachtungs_ort(),
                fangort: edit_fangort(),
                bio_ch: Some(edit_bio_ch()),
                erlaubte_ausnahme_bio: Some(edit_erlaubte_ausnahme_bio()),
                erlaubte_ausnahme_bio_details: if edit_erlaubte_ausnahme_bio_details().is_empty() { None } else { Some(edit_erlaubte_ausnahme_bio_details()) },
                erlaubte_ausnahme_knospe: Some(edit_erlaubte_ausnahme_knospe()),
                erlaubte_ausnahme_knospe_details: if edit_erlaubte_ausnahme_knospe_details().is_empty() { None } else { Some(edit_erlaubte_ausnahme_knospe_details()) },
                processing_steps: edit_processing_steps(),
                aus_umstellbetrieb: Some(edit_aus_umstellbetrieb()),
                override_children: None,
                canonical: edit_canonical(),
            };

            match save_composite_ingredient(&ingredient_to_save) {
                Ok(_) => {
                    save_status.set(Some(t!("messages.ingredient_saved_successfully", name = edit_name()).to_string()));
                    let mut save_status_clone = save_status;
                    spawn(async move {
                        gloo::timers::future::TimeoutFuture::new(2000).await;
                        save_status_clone.set(None);
                    });
                }
                Err(e) => {
                    save_status.set(Some(t!("messages.error_generic", error = e).to_string()));
                    let mut save_status_clone = save_status;
                    spawn(async move {
                        gloo::timers::future::TimeoutFuture::new(3000).await;
                        save_status_clone.set(None);
                    });
                }
            }
        }
    };

    let build_ingredient = move || -> Option<Ingredient> {
        let amount = if edit_is_composite() {
            let children = edit_children();
            let has_weighted_child = children
                .as_ref()
                .is_some_and(|c| c.iter().any(|child| child.computed_amount() > 0.0));
            if has_weighted_child {
                // Bottom-up weight: the total is the sum of the children. Derive it
                // directly (don't depend on edit_amount being synced yet — e.g. right
                // after a saved-composite recall).
                let tmp = Ingredient { name: edit_name(), children: children.clone(), ..Default::default() };
                tmp.computed_amount()
            } else {
                // Top-down weight: weightless (qualitative) children mean the parent
                // supplies the total directly, so a positive amount is required.
                match edit_amount() {
                    Some(amt) if amt > 0.0 => amt,
                    _ => return None,
                }
            }
        } else if props.is_sub_ingredient {
            // Sub-ingredients: allow amount 0 (qualitative ingredients)
            edit_amount().unwrap_or(0.0)
        } else {
            match edit_amount() {
                Some(amt) if amt > 0.0 => amt,
                _ => return None,
            }
        };

        // Alias names ("Mehl") aren't in food_db; resolve flags via the canonical.
        let canonical = edit_canonical();
        let typed = edit_name();
        let lookup_name = canonical.clone().unwrap_or(typed);
        let in_database = food_db().iter().any(|(name, _)| name == &lookup_name);
        let allergen_status = if in_database {
            lookup_allergen(&lookup_name)
        } else {
            is_allergen_custom()
        };

        Some(Ingredient {
            name: edit_name(),
            amount,
            unit: edit_unit(),
            is_allergen: allergen_status,
            is_namensgebend: Some(edit_is_namensgebend()),
            sub_components: None,
            children: edit_children(),
            origins: edit_origins(),
            is_agricultural: if edit_nicht_landwirtschaftlich() { false } else { lookup_agricultural(&lookup_name) },
            is_bio: Some(edit_is_bio()),
            category: edit_category(),
            aufzucht_ort: edit_aufzucht_ort(),
            schlachtungs_ort: edit_schlachtungs_ort(),
            fangort: edit_fangort(),
            bio_ch: Some(edit_bio_ch()),
            erlaubte_ausnahme_bio: Some(edit_erlaubte_ausnahme_bio()),
            erlaubte_ausnahme_bio_details: if edit_erlaubte_ausnahme_bio_details().is_empty() { None } else { Some(edit_erlaubte_ausnahme_bio_details()) },
            erlaubte_ausnahme_knospe: Some(edit_erlaubte_ausnahme_knospe()),
            erlaubte_ausnahme_knospe_details: if edit_erlaubte_ausnahme_knospe_details().is_empty() { None } else { Some(edit_erlaubte_ausnahme_knospe_details()) },
            processing_steps: edit_processing_steps(),
            aus_umstellbetrieb: Some(edit_aus_umstellbetrieb()),
            override_children: None,
            canonical,
        })
    };

    let mut handle_save = move |scale_all: bool| {
        if let Some(new_ingredient) = build_ingredient() {
            props.on_save.call((new_ingredient, scale_all, scaling_factor()));

            if props.is_genesis {
                // Reset local state for next creation
                edit_name.set(String::new());
                edit_amount.set(None);
                edit_unit.set(AmountUnit::default());
                edit_is_composite.set(false);
                edit_is_namensgebend.set(false);
                edit_children.set(None);
                is_allergen_custom.set(false);
                edit_category.set(None);
                edit_origins.set(None);
                edit_aufzucht_ort.set(None);
                edit_schlachtungs_ort.set(None);
                edit_fangort.set(None);
                edit_bio_ch.set(false);
                edit_is_bio.set(false);
                edit_erlaubte_ausnahme_bio.set(false);
                edit_erlaubte_ausnahme_bio_details.set(String::new());
                edit_erlaubte_ausnahme_knospe.set(false);
                edit_erlaubte_ausnahme_knospe_details.set(String::new());
                edit_processing_steps.set(None);
                edit_canonical.set(None);
                wrapper_ingredients.write()[0] = Ingredient {
                    name: String::new(),
                    amount: 0.0,
                    unit: AmountUnit::default(),
                    is_allergen: false,
                    is_namensgebend: None,
                    sub_components: None,
                    children: None,
                    origins: None,
                    is_agricultural: false,
                    is_bio: None,
                    category: None,
                    aufzucht_ort: None,
                    schlachtungs_ort: None,
                    fangort: None,
                    bio_ch: None,
                    erlaubte_ausnahme_bio: None,
                    erlaubte_ausnahme_bio_details: None,
                    erlaubte_ausnahme_knospe: None,
                    erlaubte_ausnahme_knospe_details: None,
                    processing_steps: None,
                    aus_umstellbetrieb: None,
                    override_children: None,
                    canonical: None,
                };
            }
        }
    };

    let mut handle_save_and_next = move || {
        if let Some(new_ingredient) = build_ingredient() {
            props.on_save_and_next.call((new_ingredient, false, 1.0));

            // Reset local state for next creation (same as handle_save genesis reset)
            edit_name.set(String::new());
            edit_amount.set(None);
            edit_unit.set(AmountUnit::default());
            edit_is_composite.set(false);
            edit_is_namensgebend.set(false);
            edit_children.set(None);
            is_allergen_custom.set(false);
            edit_category.set(None);
            edit_origins.set(None);
            edit_aufzucht_ort.set(None);
            edit_schlachtungs_ort.set(None);
            edit_fangort.set(None);
            edit_bio_ch.set(false);
            edit_is_bio.set(false);
            edit_erlaubte_ausnahme_bio.set(false);
            edit_erlaubte_ausnahme_bio_details.set(String::new());
            edit_erlaubte_ausnahme_knospe.set(false);
            edit_erlaubte_ausnahme_knospe_details.set(String::new());
            edit_processing_steps.set(None);
            edit_canonical.set(None);
            wrapper_ingredients.write()[0] = Ingredient::default();
        }
    };

    // Reset all edit signals to match the ingredient from the source signal.
    let is_genesis = props.is_genesis;
    let mut reset_to_original = move || {
        if is_genesis {
            // In genesis mode, just clear everything
            edit_name.set(String::new());
            edit_amount.set(None);
            edit_unit.set(AmountUnit::default());
            edit_is_composite.set(false);
            edit_is_namensgebend.set(false);
            edit_children.set(None);
            is_allergen_custom.set(false);
            edit_category.set(None);
            edit_origins.set(None);
            edit_aufzucht_ort.set(None);
            edit_schlachtungs_ort.set(None);
            edit_fangort.set(None);
            edit_bio_ch.set(false);
            edit_is_bio.set(false);
            edit_erlaubte_ausnahme_bio.set(false);
            edit_erlaubte_ausnahme_bio_details.set(String::new());
            edit_erlaubte_ausnahme_knospe.set(false);
            edit_erlaubte_ausnahme_knospe_details.set(String::new());
            edit_processing_steps.set(None);
            edit_canonical.set(None);
            return;
        }
        let Some(orig_ref) = ingredients.get(index) else {
            return;
        };
        let orig = orig_ref.clone();
        edit_name.set(orig.name.clone());
        edit_canonical.set(orig.canonical.clone());
        edit_amount.set(Some(orig.amount));
        edit_unit.set(orig.unit.clone());
        edit_is_composite.set(orig.children.as_ref().is_some_and(|c: &Vec<Ingredient>| !c.is_empty()));
        edit_is_namensgebend.set(orig.is_namensgebend.unwrap_or(false));
        edit_children.set(orig.children.clone());
        is_allergen_custom.set(orig.is_allergen);
        edit_category.set(orig.category.clone());
        edit_origins.set(orig.origins.clone());
        edit_aufzucht_ort.set(orig.aufzucht_ort.clone());
        edit_schlachtungs_ort.set(orig.schlachtungs_ort.clone());
        edit_fangort.set(orig.fangort.clone());
        edit_bio_ch.set(orig.bio_ch.unwrap_or(false));
        edit_is_bio.set(orig.is_bio.unwrap_or(false));
        edit_erlaubte_ausnahme_bio.set(orig.erlaubte_ausnahme_bio.unwrap_or(false));
        edit_erlaubte_ausnahme_bio_details.set(orig.erlaubte_ausnahme_bio_details.clone().unwrap_or_default());
        edit_erlaubte_ausnahme_knospe.set(orig.erlaubte_ausnahme_knospe.unwrap_or(false));
        edit_erlaubte_ausnahme_knospe_details.set(orig.erlaubte_ausnahme_knospe_details.clone().unwrap_or_default());
        edit_processing_steps.set(orig.processing_steps.clone());
        edit_aus_umstellbetrieb.set(orig.aus_umstellbetrieb.unwrap_or(false));
        edit_nicht_landwirtschaftlich.set(!orig.is_agricultural && orig.is_bio != Some(true) && orig.bio_ch != Some(true));
    };

    // Knospe config detection (used by bio section and Wildsammlung)
    let is_knospe_config = use_memo(move || {
        let rules = props.rules.read();
        rules.contains(&RuleDef::Knospe_ShowBioSuisseLogo)
    });

    // Plain "Knospe" (Bio Suisse, Swiss) locks the origin to CH; "Knospe Import"
    // keeps it editable (defaults to the generic `Import` origin). Lock the origin
    // control whenever the current quality is Knospe with a Swiss origin.
    let origin_locked_ch = use_memo(move || {
        is_knospe_config()
            && edit_is_bio()
            && edit_origins().as_ref().is_some_and(|o| o.contains(&Country::CH))
    });

    // Does this composite have at least one child carrying a weight? When it does,
    // the parent amount is the (read-only) sum of children (bottom-up). When it
    // doesn't, the children are qualitative and the parent supplies the total
    // weight top-down — so the amount field is editable.
    let composite_has_weighted_child = use_memo(move || {
        edit_children()
            .as_ref()
            .is_some_and(|c| c.iter().any(|child| child.computed_amount() > 0.0))
    });

    // Use the path for validation display paths, falling back to index
    let validation_index = if props.path.is_empty() { index } else { props.path[0] };

    // Check for validation errors
    let validations_context = use_context::<Validations>();
    let _has_validation_error = use_memo(move || {
        let validation_entries = (*validations_context.0.read()).clone();
        let has_origin_error = validation_entries.get(&format!("ingredients[{}][origin]", validation_index))
            .is_some_and(|v| !v.is_empty());
        let has_amount_error = validation_entries.get(&format!("ingredients[{}][amount]", validation_index))
            .is_some_and(|v| !v.is_empty());
        has_origin_error || has_amount_error
    });

    // Focus trigger signal (use provided or create a dummy)
    let focus_signal = props.focus_trigger.unwrap_or_else(|| use_signal(|| false));

    rsx! {
        div { class: if props.disabled { "opacity-50 pointer-events-none" } else { "" },
            h3 { class: "font-bold text-lg text-left",
                if props.is_genesis {
                    "{t!(\"label.zutatDetails\").to_string()}"
                } else if props.is_sub_ingredient {
                    "{t!(\"label.unterzutatBearbeiten\").to_string()}"
                } else {
                    "{t!(\"nav.bearbeiten\").to_string()}"
                }
            }
            FormField {
                label: t!("label.zutatEingeben").to_string(),
                UnifiedIngredientInput {
                    bound_value: edit_name,
                    on_ingredient_select: handle_ingredient_select,
                    required: true,
                    placeholder: t!("placeholder.zutatName").to_string(),
                    autofocus: true,
                    focus_when_true: Some(focus_signal)
                }
            }

            // Composite toggle — right after name, as daisyUI toggle
            br {}
            FormField {
                label: t!("label.zusammengesetzteZutat").to_string(),
                help: Some(t!("help.zusammengesetzteZutaten").to_string()),
                inline_checkbox: true,
                input {
                    class: "toggle toggle-accent",
                    r#type: "checkbox",
                    checked: edit_is_composite(),
                    oninput: move |evt| {
                        let is_composite = evt.data.checked();
                        if is_composite {
                            // Enabling composite: clear attribute fields (will be computed from children)
                            edit_amount.set(None);
                            edit_unit.set(AmountUnit::default());
                            edit_origins.set(None);
                            edit_is_bio.set(false);
                            edit_bio_ch.set(false);
                            is_allergen_custom.set(false);
                            edit_aufzucht_ort.set(None);
                            edit_schlachtungs_ort.set(None);
                            edit_fangort.set(None);
                            edit_erlaubte_ausnahme_bio.set(false);
                            edit_erlaubte_ausnahme_bio_details.set(String::new());
                            edit_erlaubte_ausnahme_knospe.set(false);
                            edit_erlaubte_ausnahme_knospe_details.set(String::new());
                            edit_processing_steps.set(None);
                            edit_aus_umstellbetrieb.set(false);
                            edit_nicht_landwirtschaftlich.set(false);
                        } else {
                            // Disabling composite: transfer computed/bubbled values to own fields
                            if let Some(children) = edit_children() {
                                if !children.is_empty() {
                                    let tmp = Ingredient {
                                        name: edit_name(),
                                        children: Some(children.clone()),
                                        ..Default::default()
                                    };
                                    edit_amount.set(Some(tmp.computed_amount()));
                                    if let Some(origins) = tmp.computed_origins() {
                                        edit_origins.set(Some(origins));
                                    }
                                    if let Some(bio) = tmp.computed_bio_status() {
                                        edit_is_bio.set(bio);
                                    }
                                    if let Some(bio_ch) = tmp.computed_bio_ch_status() {
                                        edit_bio_ch.set(bio_ch);
                                    }
                                    let any_allergen = children.iter().any(|c| c.is_allergen);
                                    is_allergen_custom.set(any_allergen);
                                }
                            }
                            // Remove children
                            edit_children.set(None);
                        }
                        edit_is_composite.set(is_composite);
                        props.on_composite_changed.call(is_composite);
                    }
                }
            }

            if edit_is_composite() {
                // === COMPOSITE MODE: children list + read-only computed summary ===
                br {}
                SubIngredientsTable {
                    ingredients: wrapper_ingredients,
                    index: 0,
                    on_edit_child: move |child_index: usize| {
                        goto_child.call(child_index);
                    }
                }

                // Weight is top-down: when the sub-ingredients carry no weights the
                // parent supplies the total directly (editable); when they do, the
                // total is their (read-only) sum shown in the summary below.
                if !composite_has_weighted_child() {
                    br {}
                    FormField {
                        label: format!("{} (g)", t!("label.menge").to_string()),
                        help: Some(t!("help.menge").to_string()),
                        ValidationDisplay {
                            paths: vec![
                                format!("ingredients[{}][amount]", validation_index)
                            ],
                            div { class: "flex gap-2",
                                input {
                                    r#type: "number",
                                    placeholder: t!("placeholders.amount_in_grams").to_string(),
                                    class: "input input-accent flex-1",
                                    min: "0",
                                    step: "any",
                                    oninput: move |evt| {
                                        let value = evt.data.value();
                                        if value.is_empty() {
                                            edit_amount.set(None);
                                        } else if let Ok(amount) = value.parse::<f64>() {
                                            edit_amount.set(Some(amount));
                                        }
                                    },
                                    value: edit_amount().map_or(String::new(), |v| v.to_string()),
                                }
                                select {
                                    class: "select select-accent w-20",
                                    value: if edit_unit() == AmountUnit::Gram { "g" } else { "ml" },
                                    onchange: move |evt| {
                                        let value = evt.data.value();
                                        if value == "ml" {
                                            edit_unit.set(AmountUnit::Milliliter);
                                        } else {
                                            edit_unit.set(AmountUnit::Gram);
                                        }
                                    },
                                    option { value: "g", selected: edit_unit() == AmountUnit::Gram, {t!("units.g").to_string()} }
                                    option { value: "ml", selected: edit_unit() == AmountUnit::Milliliter, {t!("units.ml").to_string()} }
                                }
                            }
                        }
                    }
                } else {
                    // Weighted children: the composite weight is their (read-only) sum.
                    // Greyed, with go-to / clear-weight on each weighted sub-ingredient.
                    br {}
                    {
                        let synth = vec![Ingredient { children: edit_children(), ..Default::default() }];
                        let weight_locked = cross_level_locked(&synth, &|c: &Ingredient| c.amount > 0.0);
                        let total = synth[0].computed_amount();
                        let unit_key = synth[0].computed_unit().translation_key();
                        rsx! {
                            FormField {
                                label: format!("{} (g)", t!("label.menge").to_string()),
                                help: Some(t!("help.menge").to_string()),
                                CrossLevelLock {
                                    locked: weight_locked,
                                    div { class: "flex gap-2",
                                        input {
                                            r#type: "number",
                                            class: "input input-accent flex-1",
                                            value: "{total}",
                                            disabled: true,
                                        }
                                        span { class: "select select-accent w-20 flex items-center justify-center",
                                            {t!(unit_key).to_string()}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Quality (Bio/Knospe) at composite level. Cross-level rule + Phase 9
                // "parent claim overrides": editable here (the whole Oberzutat can be
                // declared Knospe) when no sub-ingredient claims quality; greyed with
                // go-to/clear when one does. Decoupled from origin — Swiss/Import comes
                // from the separate Herkunft field above.
                {
                    let should_show_bio = props.rules.read().contains(&RuleDef::Bio_Knospe_EingabeIstBio);
                    let claims_quality = |c: &Ingredient| c.is_bio == Some(true) || c.bio_ch == Some(true)
                        || c.erlaubte_ausnahme_bio == Some(true) || c.erlaubte_ausnahme_knospe == Some(true);
                    let synth = vec![Ingredient { children: edit_children(), ..Default::default() }];
                    let locked = cross_level_locked(&synth, &claims_quality);
                    let derived = Ingredient { children: edit_children(), ..Default::default() };
                    let derived_label = if derived.is_knospe_compliant() { t!("bio_labels.bio_knospe").to_string() }
                        else if derived.computed_bio_ch_status() == Some(true) { t!("bio_labels.bio_ch").to_string() }
                        else { t!("bio_labels.andere").to_string() };
                    // Decoupled composite quality setter — sets the quality flags only,
                    // never origins (the Herkunft field owns origin).
                    let mut set_q = move |cat: &str| {
                        edit_is_bio.set(cat == "knospe");
                        edit_bio_ch.set(cat == "bio");
                        edit_nicht_landwirtschaftlich.set(cat == "nicht_lw");
                        if cat != "knospe" && cat != "bio" { edit_aus_umstellbetrieb.set(false); }
                    };
                    // Composite Knospe logo chooser (decoupled from origin): Knospe vs
                    // Umstellungsknospe; the Schweiz/Import split lives on the Herkunft field.
                    let mut set_comp_knospe = move |umstellung: bool| {
                        edit_is_bio.set(true);
                        edit_bio_ch.set(false);
                        edit_nicht_landwirtschaftlich.set(false);
                        edit_aus_umstellbetrieb.set(umstellung);
                    };
                    let cur = if edit_is_bio() { "knospe" } else if edit_bio_ch() { "bio" }
                        else if edit_nicht_landwirtschaftlich() { "nicht_lw" } else { "andere" };
                    rsx! {
                        if should_show_bio {
                            br {}
                            FormField {
                                label: t!("bio_labels.quality").to_string(),
                                CrossLevelLock {
                                    locked: locked,
                                    if locked {
                                        span { class: "badge badge-outline", "{derived_label}" }
                                    } else {
                                        div { class: "flex flex-col gap-1",
                                            for (key, label) in [
                                                ("knospe", t!("bio_labels.bio_knospe").to_string()),
                                                ("bio", t!("bio_labels.bio_ch").to_string()),
                                                ("nicht_lw", t!("bio_labels.nicht_landwirtschaftlich").to_string()),
                                                ("andere", t!("bio_labels.andere").to_string()),
                                            ].into_iter() {
                                                label { class: "flex items-center gap-2 cursor-pointer",
                                                    input {
                                                        r#type: "radio",
                                                        name: "comp_quality",
                                                        class: "radio radio-primary radio-sm",
                                                        checked: cur == key,
                                                        onchange: move |_| { set_q(key); }
                                                    }
                                                    span { "{label}" }
                                                }
                                            }
                                            // Knospe logo chooser (decoupled from origin): Knospe vs Umstellungsknospe.
                                            if cur == "knospe" {
                                                div { class: "flex gap-2 mt-2",
                                                    for (umst, label) in [
                                                        (false, t!("bio_labels.knospe").to_string()),
                                                        (true, t!("bio_labels.umstellungsknospe").to_string()),
                                                    ].into_iter() {
                                                        {
                                                            let selected = edit_aus_umstellbetrieb() == umst;
                                                            rsx! {
                                                                button {
                                                                    r#type: "button",
                                                                    class: if selected { "flex flex-col items-center gap-1 p-2 rounded-lg border-2 border-primary bg-primary/5" } else { "flex flex-col items-center gap-1 p-2 rounded-lg border-2 border-base-300 hover:border-base-content/30" },
                                                                    onclick: move |_| { set_comp_knospe(umst); },
                                                                    div { class: if umst { "opacity-60" } else { "" },
                                                                        crate::components::icons::BioSuisseRegular {}
                                                                    }
                                                                    span { class: "text-xs text-center leading-tight font-medium", "{label}" }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Allergen — bottom-up from children; greyed (a composite's allergen
                // status is always derived), with go-to to the allergen-bearing
                // sub-ingredient(s). Not clearable here (food safety).
                br {}
                {
                    let synth = vec![Ingredient { children: edit_children(), ..Default::default() }];
                    // A composite's allergen status is always derived: locked iff any child is an allergen.
                    let is_allergen = cross_level_locked(&synth, &|c: &Ingredient| c.is_allergen);
                    rsx! {
                        FormField {
                            label: t!("label.allergen").to_string(),
                            inline_checkbox: true,
                            CrossLevelLock {
                                locked: is_allergen,
                                input {
                                    r#type: "checkbox",
                                    class: "checkbox checkbox-accent",
                                    checked: is_allergen,
                                    disabled: true,
                                }
                            }
                        }
                    }
                }

                // Herkunft — editable on the composite itself, UNLESS a sub-ingredient
                // already declares an origin (single-level rule). When locked, the
                // greyed control shows the children's origins (change them on the
                // sub-ingredient itself).
                br {}
                {
                    let has_origin = |c: &Ingredient| c.origins.as_ref().is_some_and(|o| !o.is_empty());
                    let synth = vec![Ingredient { children: edit_children(), ..Default::default() }];
                    let locked = cross_level_locked(&synth, &has_origin);
                    let display_origins = Ingredient { children: edit_children(), ..Default::default() }.computed_origins();
                    rsx! {
                        FormField {
                            label: t!("origin.herkunft").to_string(),
                            help: Some(t!("help.herkunft_liv_art_16").to_string()),
                            CrossLevelLock {
                                locked: locked,
                                if locked {
                                    div { class: "flex gap-1 flex-wrap",
                                        if let Some(origins) = display_origins {
                                            for origin in origins.iter() {
                                                span { class: "badge badge-outline", "{origin.flag_emoji()} {origin.country_code()}" }
                                            }
                                        }
                                    }
                                } else {
                                    MultiCountrySelect {
                                        values: edit_origins.read().clone(),
                                        onchange: move |countries| { edit_origins.set(countries); }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                // === LEAF MODE: editable form fields ===
                br {}
                FormField {
                    label: format!("{} (g)", t!("label.menge").to_string()),
                    help: Some(t!("help.menge").to_string()),
                    ValidationDisplay {
                        paths: vec![
                            format!("ingredients[{}][amount]", validation_index)
                        ],
                        div { class: "flex gap-2",
                            input {
                                r#type: "number",
                                placeholder: t!("placeholders.amount_in_grams").to_string(),
                                class: "input input-accent flex-1",
                                min: "0",
                                step: "any",
                                oninput: move |evt| {
                                    let value = evt.data.value();
                                    if value.is_empty() {
                                        edit_amount.set(None);
                                    } else if let Ok(amount) = value.parse::<f64>() {
                                        edit_amount.set(Some(amount));
                                    }
                                },
                                value: edit_amount().map_or(String::new(), |v| v.to_string()),
                            }
                            select {
                                class: "select select-accent w-20",
                                value: if edit_unit() == AmountUnit::Gram { "g" } else { "ml" },
                                onchange: move |evt| {
                                    let value = evt.data.value();
                                    if value == "ml" {
                                        edit_unit.set(AmountUnit::Milliliter);
                                    } else {
                                        edit_unit.set(AmountUnit::Gram);
                                    }
                                },
                                option { value: "g", selected: edit_unit() == AmountUnit::Gram, {t!("units.g").to_string()} }
                                option { value: "ml", selected: edit_unit() == AmountUnit::Milliliter, {t!("units.ml").to_string()} }
                            }
                        }
                    }
                    if props.depth == 0 && !props.is_genesis && amount_has_changed() {
                        div { class: "text-sm text-info mt-2",
                            if let Some(amt) = edit_amount() {
                                {t!("messages.scaling_factor", factor = format!("{:.2}", scaling_factor()), before = original_amount().to_string(), after = amt.to_string()).to_string()}
                            } else {
                                {t!("messages.please_enter_amount").to_string()}
                            }
                        }
                    }
                }

                br {}

                // Allergen status
                if !edit_name().is_empty() {
                    FormField {
                        help: if is_custom_ingredient() { Some(t!("help.allergenManual").to_string()) } else { None },
                        label: t!("label.allergen").to_string(),
                        inline_checkbox: true,
                        CheckboxInput {
                            bound_value: is_allergen_custom,
                            disabled: !is_custom_ingredient(),
                        }
                    }
                    br {}
                }
            }

            br {}
            ConditionalDisplay {
                path: "namensgebende_zutat".to_string(),
                FormField {
                    help: Some(t!("help.namensgebendeZutaten").to_string()),
                    label: t!("label.namensgebendeZutat").to_string(),
                    inline_checkbox: true,
                    CheckboxInput {
                        bound_value: edit_is_namensgebend
                    }
                }
            }
            // Save status
            if let Some(status) = save_status() {
                div { class: "alert alert-info mb-4",
                    span { "{status}" }
                }
            }

            // Bio, origins, beef, fish fields — only shown in leaf mode
            if !edit_is_composite() {
            br {}
            {
                // Bio rule check
                let should_show_bio = use_memo(move || {
                    let rules = props.rules.read();
                    rules.contains(&RuleDef::Bio_Knospe_EingabeIstBio)
                });

                if should_show_bio() {
                    if is_knospe_config() {
                        // Derive current bio category from signals. Knospe splits into
                        // CH-Knospe ("knospe") and "knospe_import" by whether the origin
                        // is Swiss — quality is the bio flag, origin is the discriminator.
                        let origins_have_ch = edit_origins().as_ref().is_some_and(|o| o.contains(&Country::CH));
                        // Top-level quality. Variante b: every Knospe variant collapses to
                        // "knospe"; Swiss/Import and Umstellung are picked via the logo row.
                        let bio_cat = if edit_is_bio() { "knospe" }
                            else if edit_bio_ch() { "bio" }
                            else if edit_nicht_landwirtschaftlich() { "nicht_lw" }
                            else { "andere" };

                        // Which Knospe logo is active, derived from origin + Umstellbetrieb.
                        let knospe_variant = match (origins_have_ch, edit_aus_umstellbetrieb()) {
                            (true, false) => "knospe_ch",
                            (false, false) => "knospe_import",
                            (true, true) => "umstellung_ch",
                            (false, true) => "umstellung_import",
                        };

                        let mut set_bio_cat = move |cat: &str| {
                            edit_is_bio.set(cat == "knospe");
                            edit_bio_ch.set(cat == "bio");
                            edit_nicht_landwirtschaftlich.set(cat == "nicht_lw");
                            if cat == "knospe" {
                                // Default a fresh Knospe selection to the Swiss Knospe.
                                edit_origins.set(Some(vec![Country::CH]));
                                edit_aus_umstellbetrieb.set(false);
                            } else if cat == "nicht_lw" {
                                edit_origins.set(None);
                                edit_aus_umstellbetrieb.set(false);
                            } else if cat == "andere" {
                                edit_aus_umstellbetrieb.set(false);
                            }
                            // "bio" (non-Knospe) keeps its own Umstellbetrieb checkbox.
                        };

                        // Pick the specific Knospe logo: encodes is_bio + origin + Umstellung,
                        // replacing the old Knospe-Import radio and Umstellbetrieb checkbox.
                        let mut set_knospe_variant = move |variant: &str| {
                            edit_is_bio.set(true);
                            edit_bio_ch.set(false);
                            edit_nicht_landwirtschaftlich.set(false);
                            edit_aus_umstellbetrieb.set(matches!(variant, "umstellung_ch" | "umstellung_import"));
                            if matches!(variant, "knospe_ch" | "umstellung_ch") {
                                edit_origins.set(Some(vec![Country::CH]));
                            } else {
                                let keep = edit_origins().filter(|o| !o.is_empty() && !o.contains(&Country::CH));
                                edit_origins.set(keep.or(Some(vec![Country::Import])));
                            }
                        };

                        let wildsammlung_step = "aus zertifizierter Wildsammlung";

                        rsx! {
                            // Radio: Bio (Knospe) — Swiss, origin locked to CH
                            FormField {
                                help: Some(t!("help.bio_knospe").to_string()),
                                label: t!("bio_labels.bio_knospe").to_string(),
                                inline_checkbox: true,
                                input {
                                    r#type: "radio",
                                    name: "bio_category",
                                    class: "radio radio-primary",
                                    checked: bio_cat == "knospe",
                                    onchange: move |_| { set_bio_cat("knospe"); }
                                }
                            }
                            // Radio: Bio
                            FormField {
                                help: Some(t!("help.bio_ch").to_string()),
                                label: t!("bio_labels.bio_ch").to_string(),
                                inline_checkbox: true,
                                input {
                                    r#type: "radio",
                                    name: "bio_category",
                                    class: "radio radio-primary",
                                    checked: bio_cat == "bio",
                                    onchange: move |_| { set_bio_cat("bio"); }
                                }
                            }
                            // Radio: Andere (Nicht-biologisch)
                            FormField {
                                help: Some(t!("help.andere").to_string()),
                                label: t!("bio_labels.andere").to_string(),
                                inline_checkbox: true,
                                input {
                                    r#type: "radio",
                                    name: "bio_category",
                                    class: "radio radio-primary",
                                    checked: bio_cat == "andere",
                                    onchange: move |_| { set_bio_cat("andere"); }
                                }
                            }
                            // Radio: Nicht-landwirtschaftliche Zutat
                            FormField {
                                help: Some(t!("help.nicht_landwirtschaftlich").to_string()),
                                label: t!("bio_labels.nicht_landwirtschaftlich").to_string(),
                                inline_checkbox: true,
                                input {
                                    r#type: "radio",
                                    name: "bio_category",
                                    class: "radio radio-primary",
                                    checked: bio_cat == "nicht_lw",
                                    onchange: move |_| { set_bio_cat("nicht_lw"); }
                                }
                            }

                            // Variante b: when "Bio (Knospe)" is chosen, pick WHICH Knospe
                            // via the logo row (Swiss/Import × Knospe/Umstellung) + Wildsammlung.
                            if bio_cat == "knospe" {
                                br {}
                                div { class: "border-t border-base-300 pt-2 mt-2",
                                    div { class: "grid grid-cols-2 sm:grid-cols-4 gap-2 mb-3",
                                        for (key, label) in [
                                            ("knospe_ch", t!("bio_labels.knospe_ch").to_string()),
                                            ("knospe_import", t!("bio_labels.knospe_import").to_string()),
                                            ("umstellung_ch", t!("bio_labels.umstellung_ch").to_string()),
                                            ("umstellung_import", t!("bio_labels.umstellung_import").to_string()),
                                        ].into_iter() {
                                            {
                                                let selected = knospe_variant == key;
                                                let umstellung = key.starts_with("umstellung");
                                                let ch = key.ends_with("_ch");
                                                rsx! {
                                                    button {
                                                        r#type: "button",
                                                        class: if selected { "flex flex-col items-center gap-1 p-2 rounded-lg border-2 border-primary bg-primary/5" } else { "flex flex-col items-center gap-1 p-2 rounded-lg border-2 border-base-300 hover:border-base-content/30" },
                                                        onclick: move |_| { set_knospe_variant(key); },
                                                        div { class: if umstellung { "opacity-60" } else { "" },
                                                            if ch { crate::components::icons::BioSuisseRegular {} } else { crate::components::icons::BioSuisseNoCross {} }
                                                        }
                                                        span { class: "text-xs text-center leading-tight font-medium", "{label}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    {
                                        let is_wildsammlung_checked = edit_processing_steps()
                                            .as_ref()
                                            .is_some_and(|s| s.contains(&wildsammlung_step.to_string()));
                                        rsx! {
                                            FormField {
                                                help: Some(t!("help.wildsammlung").to_string()),
                                                label: t!("bio_labels.wildsammlung").to_string(),
                                                inline_checkbox: true,
                                                input {
                                                    r#type: "checkbox",
                                                    class: "checkbox checkbox-accent",
                                                    checked: is_wildsammlung_checked,
                                                    onchange: move |evt: dioxus::prelude::Event<dioxus::prelude::FormData>| {
                                                        let mut current = edit_processing_steps().unwrap_or_default();
                                                        if evt.data.value() == "true" {
                                                            if !current.contains(&wildsammlung_step.to_string()) {
                                                                current.push(wildsammlung_step.to_string());
                                                            }
                                                        } else {
                                                            current.retain(|s| s != wildsammlung_step);
                                                        }
                                                        edit_processing_steps.set(
                                                            if current.is_empty() { None } else { Some(current) }
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else if bio_cat == "bio" {
                                br {}
                                div { class: "border-t border-base-300 pt-2 mt-2",
                                    FormField {
                                        help: Some(t!("help.aus_umstellbetrieb").to_string()),
                                        label: t!("bio_labels.aus_umstellbetrieb").to_string(),
                                        inline_checkbox: true,
                                        input {
                                            r#type: "checkbox",
                                            class: "checkbox checkbox-accent",
                                            checked: edit_aus_umstellbetrieb(),
                                            onchange: move |evt| {
                                                edit_aus_umstellbetrieb.set(evt.data.value() == "true");
                                            }
                                        }
                                    }
                                    br {}
                                    FormField {
                                        help: Some(t!("help.erlaubte_ausnahme_knospe").to_string()),
                                        label: t!("bio_labels.erlaubte_ausnahme_knospe").to_string(),
                                        inline_checkbox: true,
                                        input {
                                            r#type: "checkbox",
                                            class: "checkbox checkbox-accent",
                                            checked: edit_erlaubte_ausnahme_knospe(),
                                            onchange: move |evt| {
                                                edit_erlaubte_ausnahme_knospe.set(evt.data.value() == "true");
                                            }
                                        }
                                    }
                                    if edit_erlaubte_ausnahme_knospe() {
                                        div { class: "flex justify-end mt-2",
                                            InternalNoteMark {}
                                        }
                                        textarea {
                                            class: "textarea textarea-bordered w-full",
                                            placeholder: t!("bio_labels.erlaubte_ausnahme_details_placeholder").to_string(),
                                            rows: 2,
                                            value: "{edit_erlaubte_ausnahme_knospe_details}",
                                            oninput: move |evt| {
                                                edit_erlaubte_ausnahme_knospe_details.set(evt.data.value());
                                            }
                                        }
                                    }
                                }
                            } else if bio_cat == "andere" {
                                br {}
                                div { class: "border-t border-base-300 pt-2 mt-2",
                                    FormField {
                                        help: Some(t!("help.erlaubte_ausnahme_bio").to_string()),
                                        label: t!("bio_labels.erlaubte_ausnahme_bio").to_string(),
                                        inline_checkbox: true,
                                        input {
                                            r#type: "checkbox",
                                            class: "checkbox checkbox-accent",
                                            checked: edit_erlaubte_ausnahme_bio(),
                                            onchange: move |evt| {
                                                edit_erlaubte_ausnahme_bio.set(evt.data.value() == "true");
                                            }
                                        }
                                    }
                                    if edit_erlaubte_ausnahme_bio() {
                                        div { class: "flex justify-end mt-2",
                                            InternalNoteMark {}
                                        }
                                        textarea {
                                            class: "textarea textarea-bordered w-full",
                                            placeholder: t!("bio_labels.erlaubte_ausnahme_details_placeholder").to_string(),
                                            rows: 2,
                                            value: "{edit_erlaubte_ausnahme_bio_details}",
                                            oninput: move |evt| {
                                                edit_erlaubte_ausnahme_bio_details.set(evt.data.value());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {
                            FormField {
                                help: Some(t!("help.bio_ch").to_string()),
                                label: t!("bio_labels.bio_ch").to_string(),
                                inline_checkbox: true,
                                input {
                                    r#type: "checkbox",
                                    class: "checkbox checkbox-accent",
                                    checked: edit_bio_ch(),
                                    onchange: move |evt| {
                                        edit_bio_ch.set(evt.data.value() == "true");
                                    }
                                }
                            }
                            if edit_bio_ch() {
                                br {}
                                FormField {
                                    help: Some(t!("help.aus_umstellbetrieb").to_string()),
                                    label: t!("bio_labels.aus_umstellbetrieb").to_string(),
                                    inline_checkbox: true,
                                    input {
                                        r#type: "checkbox",
                                        class: "checkbox checkbox-accent",
                                        checked: edit_aus_umstellbetrieb(),
                                        onchange: move |evt| {
                                            edit_aus_umstellbetrieb.set(evt.data.value() == "true");
                                        }
                                    }
                                }
                            }
                            if !edit_bio_ch() {
                                br {}
                                div { class: "border-t border-base-300 pt-2 mt-2",
                                    FormField {
                                        help: Some(t!("help.erlaubte_ausnahme_bio").to_string()),
                                        label: t!("bio_labels.erlaubte_ausnahme_bio").to_string(),
                                        inline_checkbox: true,
                                        input {
                                            r#type: "checkbox",
                                            class: "checkbox checkbox-accent",
                                            checked: edit_erlaubte_ausnahme_bio(),
                                            onchange: move |evt| {
                                                edit_erlaubte_ausnahme_bio.set(evt.data.value() == "true");
                                            }
                                        }
                                    }
                                    if edit_erlaubte_ausnahme_bio() {
                                        div { class: "flex justify-end mt-2",
                                            InternalNoteMark {}
                                        }
                                        textarea {
                                            class: "textarea textarea-bordered w-full",
                                            placeholder: t!("bio_labels.erlaubte_ausnahme_details_placeholder").to_string(),
                                            rows: 2,
                                            value: "{edit_erlaubte_ausnahme_bio_details}",
                                            oninput: move |evt| {
                                                edit_erlaubte_ausnahme_bio_details.set(evt.data.value());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    rsx! {}
                }
            }
            // Processing steps (Knospe mode only)
            {
                let should_show_processing = use_memo(move || {
                    let rules = props.rules.read();
                    let is_knospe = rules.contains(&RuleDef::Knospe_ShowBioSuisseLogo);

                    if is_knospe {
                        if let Some(category) = &edit_category() {
                            crate::processing_service::has_processing_steps_for_blv_category(category)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                });

                let available_steps = use_memo(move || {
                    if let Some(category) = &edit_category() {
                        crate::processing_service::get_steps_for_blv_category(category)
                    } else {
                        vec![]
                    }
                });

                if should_show_processing() {
                    rsx! {
                        br {}
                        FormField {
                            label: t!("label.verarbeitungsschritte").to_string(),
                            help: Some(t!("help.verarbeitungsschritte").to_string()),
                            div { class: "flex flex-col gap-2",
                                for step in available_steps.read().iter().filter(|s| s.step_de != "aus zertifizierter Wildsammlung") {
                                    label { class: "flex items-center gap-2 cursor-pointer",
                                        input {
                                            r#type: "checkbox",
                                            class: "checkbox checkbox-accent checkbox-sm",
                                            checked: edit_processing_steps()
                                                .as_ref()
                                                .is_some_and(|s| s.contains(&step.step_de)),
                                            onchange: {
                                                let step_name = step.step_de.clone();
                                                move |evt: dioxus::prelude::Event<dioxus::prelude::FormData>| {
                                                    let mut current = edit_processing_steps()
                                                        .unwrap_or_default();
                                                    if evt.data.value() == "true" {
                                                        if !current.contains(&step_name) {
                                                            current.push(step_name.clone());
                                                        }
                                                    } else {
                                                        current.retain(|s| s != &step_name);
                                                    }
                                                    edit_processing_steps.set(
                                                        if current.is_empty() { None } else { Some(current) }
                                                    );
                                                }
                                            }
                                        }
                                        span { "{step.step_de}" }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    rsx! {}
                }
            }
            br {}
            FormField {
                label: t!("origin.herkunft").to_string(),
                help: Some(t!("help.herkunft_liv_art_16").to_string()),
                ValidationDisplay {
                    paths: vec![
                        format!("ingredients[{}][origin]", validation_index)
                    ],
                    if origin_locked_ch() {
                        // Plain Knospe is Swiss by definition — origin is fixed to CH.
                        // Shown as a static badge (no editable country picker).
                        div { class: "flex items-center gap-2",
                            span { class: "badge badge-lg badge-outline",
                                "{Country::CH.flag_emoji()} {Country::CH.country_code()}"
                            }
                        }
                    } else {
                        MultiCountrySelect {
                            values: edit_origins.read().clone(),
                            onchange: move |countries| {
                                edit_origins.set(countries);
                            }
                        }
                    }
                }
            }
            br {}
            {
                // Beef fields
                let should_show_beef_fields = use_memo(move || {
                    let rules = props.rules.read();
                    let has_beef_rule = rules.contains(&RuleDef::AP7_4_RindfleischHerkunftDetails);
                    if has_beef_rule {
                        if let Some(category) = &edit_category() {
                            crate::category_service::is_beef_category(category)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                });

                if should_show_beef_fields() {
                    rsx! {
                        FormField {
                            label: t!("origin.aufzucht").to_string(),
                            help: Some(t!("help.aufzucht_location").to_string()),
                            ValidationDisplay {
                                paths: vec![
                                    format!("ingredients[{}][aufzucht_ort]", validation_index)
                                ],
                                CountrySelect {
                                    value: edit_aufzucht_ort.read().clone(),
                                    include_all_countries: true,
                                    onchange: move |country| {
                                        edit_aufzucht_ort.set(country);
                                    }
                                }
                            }
                        }
                        FormField {
                            label: t!("origin.schlachtung").to_string(),
                            ValidationDisplay {
                                paths: vec![
                                    format!("ingredients[{}][schlachtungs_ort]", validation_index)
                                ],
                                CountrySelect {
                                    value: edit_schlachtungs_ort.read().clone(),
                                    include_all_countries: true,
                                    onchange: move |country| {
                                        edit_schlachtungs_ort.set(country);
                                    }
                                }
                            }
                        }
                    }
                } else {
                    rsx! {}
                }
            }
            br {}
            {
                // Fish field
                let should_show_fish_field = use_memo(move || {
                    let rules = props.rules.read();
                    let has_fish_rule = rules.contains(&RuleDef::AP7_5_FischFangort);
                    if has_fish_rule {
                        if let Some(category) = &edit_category() {
                            crate::category_service::is_fish_category(category)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                });

                if should_show_fish_field() {
                    rsx! {
                        FormField {
                            label: t!("origin.fangort").to_string(),
                            ValidationDisplay {
                                paths: vec![
                                    format!("ingredients[{}][fangort]", validation_index)
                                ],
                                CountrySelect {
                                    value: edit_fangort.read().clone(),
                                    include_all_countries: true,
                                    onchange: move |country| {
                                        edit_fangort.set(country);
                                    }
                                }
                            }
                        }
                    }
                } else {
                    rsx! {}
                }
            }
            } // end if !edit_is_composite() for bio/origins/beef/fish
            div { class: "modal-action sticky bottom-0 bg-base-100 -mx-4 px-4 pt-3 pb-2 border-t border-base-300 mt-0",
                button {
                    class: "btn",
                    onclick: move |_| {
                        reset_to_original();
                        props.on_close.call(());
                    },
                    {t!("nav.schliessen").to_string()},
                }

                // "Merken" button for composite ingredients (top-level only)
                if props.depth == 0 && edit_is_composite() && edit_children().is_some() {
                    span { class: "tooltip", "data-tip": t!("tooltips.save_composite_ingredient").to_string(),
                        button {
                            class: "btn btn-info",
                            onclick: handle_save_to_storage,
                            {t!("buttons.save_to_storage").to_string()}
                        }
                    }
                }

                // Show save button (mirror `build_ingredient`'s acceptance rules):
                // - Sub-ingredient: name non-empty (amount 0 ok for qualitative)
                // - Top-level leaf: name non-empty + amount > 0
                // - Composite: name non-empty + EITHER a weighted child (bottom-up sum)
                //   OR a positive parent amount (top-down weight, weightless children)
                if !edit_name().is_empty() && (
                    props.is_sub_ingredient
                    || (!edit_is_composite() && edit_amount().is_some_and(|a| a > 0.0))
                    || (edit_is_composite() && (
                        edit_children().as_ref().is_some_and(|c| c.iter().any(|child| child.computed_amount() > 0.0))
                        || edit_amount().is_some_and(|a| a > 0.0)
                    ))
                ) {
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| handle_save(false),
                        {t!("nav.speichern").to_string()},
                    }
                    if props.is_genesis && !edit_is_composite() {
                        button {
                            class: "btn btn-secondary",
                            onclick: move |_| handle_save_and_next(),
                            {t!("nav.speichernUndNaechste").to_string()}
                        }
                    }
                }
                if props.depth == 0 && !props.is_genesis && !edit_is_composite() && amount_has_changed() {
                    span { class: "tooltip", "data-tip": t!("buttons.transfer_scaling_title", factor = format!("{:.2}", scaling_factor())).to_string(),
                        button {
                            class: "btn btn-secondary",
                            onclick: move |_| handle_save(true),
                            {t!("buttons.save_and_transfer").to_string()}
                        }
                    }
                }
            }
        }
    }
}
