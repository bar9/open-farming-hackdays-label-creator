use crate::components::*;
use crate::components::ingredient_path::IngredientPath;
use crate::core::{Ingredient, AmountUnit};
use crate::model::{food_db, lookup_allergen, lookup_agricultural};
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
    /// Called when user saves the ingredient.
    pub on_save: EventHandler<(Ingredient, bool)>,
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
    pub on_save_and_next: EventHandler<(Ingredient, bool)>,
    /// Signal to trigger focus on the name input when it becomes true.
    #[props(default = None)]
    pub focus_trigger: Option<Signal<bool>>,
    /// Depth in the ingredient tree (0 = top-level). Controls visibility of
    /// "merken" and "anteilsmässig übertragen" buttons (shown only at depth 0).
    #[props(default = 0)]
    pub depth: usize,
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

    let mut is_custom_ingredient = use_signal(|| {
        !food_db().iter().any(|(name, _)| name == &edit_name())
    });
    let mut is_allergen_custom = use_signal(|| original_ingredient.is_allergen);

    let amount_has_changed = use_memo(move || {
        if props.is_genesis {
            false
        } else if let Some(current_amount) = edit_amount() {
            let original_amount = original_ingredient.amount;
            (original_amount - current_amount).abs() > 0.01
        } else {
            false
        }
    });

    let scaling_factor = use_memo(move || {
        if props.is_genesis || original_ingredient.amount == 0.0 {
            1.0
        } else if let Some(current_amount) = edit_amount() {
            current_amount / original_ingredient.amount
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
        }]
    });

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
                is_agricultural: if edit_nicht_landwirtschaftlich() { false } else { lookup_agricultural(&edit_name()) },
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

            let new_ing = Ingredient {
                name,
                amount,
                unit,
                is_allergen: allergen,
                is_namensgebend: Some(namensgebend),
                sub_components: None,
                children,
                origins,
                is_agricultural: if edit_nicht_landwirtschaftlich() { false } else { lookup_agricultural(&edit_name()) },
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
        use_effect(move || {
            if !edit_is_composite() { return; }
            if let Some(ing) = ingredients.read().get(pane_index) {
                let ca = ing.computed_amount();
                if edit_amount.peek().is_none_or(|a| (a - ca).abs() > 0.001) {
                    edit_amount.set(Some(ca));
                }
                let cb = ing.computed_bio_status().unwrap_or(false);
                if *edit_is_bio.peek() != cb { edit_is_bio.set(cb); }
                let cbc = ing.computed_bio_ch_status().unwrap_or(false);
                if *edit_bio_ch.peek() != cbc { edit_bio_ch.set(cbc); }
                let co = ing.computed_origins();
                if *edit_origins.peek() != co { edit_origins.set(co); }
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

        match unified_ingredient.source {
            crate::services::IngredientSource::Local => {
                is_custom_ingredient.set(false);
            }
            crate::services::IngredientSource::BLV => {
                is_custom_ingredient.set(true);
            }
            crate::services::IngredientSource::Merged => {
                is_custom_ingredient.set(false);
            }
        }
    };

    let _update_name = move |new_name: String| {
        edit_name.set(new_name.clone());

        let saved_ingredients = get_saved_ingredients_list();
        if let Some(saved) = saved_ingredients.iter().find(|i| i.name == new_name) {
            edit_is_composite.set(true);
            edit_children.set(saved.children.clone());
            is_allergen_custom.set(saved.is_allergen);
            edit_is_namensgebend.set(saved.is_namensgebend.unwrap_or(false));
            if let Some(category) = &saved.category {
                edit_category.set(Some(category.clone()));
            }
            is_custom_ingredient.set(true);
            return;
        }

        let in_database = food_db().iter().any(|(name, _)| name == &new_name);
        is_custom_ingredient.set(!in_database);

        if in_database {
            is_allergen_custom.set(lookup_allergen(&new_name));
        }

        if new_name.is_empty() {
            edit_category.set(None);
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
                is_agricultural: if edit_nicht_landwirtschaftlich() { false } else { lookup_agricultural(&edit_name()) },
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
            // Composite: amount is computed from children; require at least one child with weight
            let children = edit_children();
            let has_weighted_child = children.as_ref().is_some_and(|c| c.iter().any(|child| child.computed_amount() > 0.0));
            if !has_weighted_child { return None; }
            edit_amount().unwrap_or(0.0)
        } else if props.is_sub_ingredient {
            // Sub-ingredients: allow amount 0 (qualitative ingredients)
            edit_amount().unwrap_or(0.0)
        } else {
            match edit_amount() {
                Some(amt) if amt > 0.0 => amt,
                _ => return None,
            }
        };

        let in_database = food_db().iter().any(|(name, _)| name == &edit_name());
        let allergen_status = if in_database {
            lookup_allergen(&edit_name())
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
            is_agricultural: if edit_nicht_landwirtschaftlich() { false } else { lookup_agricultural(&edit_name()) },
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
        })
    };

    let mut handle_save = move |scale_all: bool| {
        if let Some(new_ingredient) = build_ingredient() {
            props.on_save.call((new_ingredient, scale_all));

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
                };
            }
        }
    };

    let mut handle_save_and_next = move || {
        if let Some(new_ingredient) = build_ingredient() {
            props.on_save_and_next.call((new_ingredient, false));

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
            is_custom_ingredient.set(true);
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
            return;
        }
        let Some(orig_ref) = ingredients.get(index) else {
            return;
        };
        let orig = orig_ref.clone();
        edit_name.set(orig.name.clone());
        edit_amount.set(Some(orig.amount));
        edit_unit.set(orig.unit.clone());
        edit_is_composite.set(orig.children.as_ref().is_some_and(|c: &Vec<Ingredient>| !c.is_empty()));
        edit_is_namensgebend.set(orig.is_namensgebend.unwrap_or(false));
        edit_children.set(orig.children.clone());
        is_allergen_custom.set(orig.is_allergen);
        is_custom_ingredient.set(!food_db().iter().any(|(name, _)| name == &orig.name));
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
                    on_edit_child: {
                        let editing_path = props.editing_path;
                        let mut ingredients = props.ingredients;
                        let pane_index = props.index;
                        move |child_index: usize| {
                            // Flush full ingredient state to the real signal so
                            // NestedCard and computed-value sync see current data.
                            let live_children = wrapper_ingredients.read()
                                .first()
                                .and_then(|i| i.children.clone());
                            if let Some(mut ing) = ingredients.get_mut(pane_index) {
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
                            if let Some(mut ep) = editing_path {
                                let mut new_path = ep.read().clone();
                                // If path is empty (genesis mode), seed with the pane index
                                // so NestedCard can find the parent ingredient.
                                if new_path.is_empty() {
                                    new_path.push(pane_index);
                                }
                                new_path.push(child_index);
                                ep.set(new_path);
                            }
                        }
                    }
                }

                // Read-only computed summary as key/value pairs
                br {}
                div { class: "bg-base-200 rounded-lg p-3 space-y-1 text-sm",
                    div { class: "font-semibold mb-2", {t!("label.computedValues").to_string()} }
                    // Amount
                    div { class: "flex justify-between",
                        span { class: "text-base-content/60", {t!("label.menge").to_string()} }
                        span {
                            if let Some(amt) = edit_amount().filter(|a| *a > 0.0) {
                                "{amt} {t!(edit_unit().translation_key()).to_string()}"
                            } else {
                                "—"
                            }
                        }
                    }
                    // Allergen
                    div { class: "flex justify-between",
                        span { class: "text-base-content/60", {t!("label.allergen").to_string()} }
                        span {
                            if is_allergen_custom() {
                                span { class: "font-bold", {t!("label.allergen").to_string()} }
                            } else {
                                {t!("label.keinAllergen").to_string()}
                            }
                        }
                    }
                    // Bio
                    div { class: "flex justify-between",
                        span { class: "text-base-content/60", "Bio" }
                        span {
                            if edit_is_bio() { "✓" } else { "✗" }
                        }
                    }
                    // Bio CH
                    div { class: "flex justify-between",
                        span { class: "text-base-content/60", "Bio CH" }
                        span {
                            if edit_bio_ch() { "✓" } else { "✗" }
                        }
                    }
                    // Origins
                    if let Some(origins) = edit_origins() {
                        div { class: "flex justify-between",
                            span { class: "text-base-content/60", {t!("origin.herkunft").to_string()} }
                            div { class: "flex gap-1 flex-wrap justify-end",
                                for origin in origins.iter() {
                                    span { class: "badge badge-sm badge-outline", "{origin.flag_emoji()} {origin.country_code()}" }
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
                                {t!("messages.scaling_factor", factor = format!("{:.2}", scaling_factor()), before = original_ingredient.amount.to_string(), after = amt.to_string()).to_string()}
                            } else {
                                {t!("messages.please_enter_amount").to_string()}
                            }
                        }
                    }
                }

                br {}

                // Allergen status
                if !edit_name().is_empty() {
                    if is_custom_ingredient() {
                        FormField {
                            help: Some(t!("help.allergenManual").to_string()),
                            label: t!("label.allergen").to_string(),
                            inline_checkbox: true,
                            CheckboxInput {
                                bound_value: is_allergen_custom
                            }
                        }
                        br {}
                    } else if is_allergen_custom() {
                        FormField {
                            label: t!("label.allergen").to_string(),
                            div { class: "py-2",
                                span { class: "font-bold", "({t!(\"label.allergen\").to_string()})" }
                            }
                        }
                        br {}
                    } else {
                        FormField {
                            label: t!("label.allergen").to_string(),
                            div { class: "py-2 text-base-content/50",
                                span { {t!("label.keinAllergen").to_string()} }
                            }
                        }
                        br {}
                    }
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
                        // Derive current bio category from signals
                        let bio_cat = if edit_is_bio() { "knospe" }
                            else if edit_bio_ch() { "bio" }
                            else if edit_nicht_landwirtschaftlich() { "nicht_lw" }
                            else { "andere" };

                        // Helper closure to set all bio signals from a category
                        let mut set_bio_cat = move |cat: &str| {
                            edit_is_bio.set(cat == "knospe");
                            edit_bio_ch.set(cat == "bio");
                            edit_nicht_landwirtschaftlich.set(cat == "nicht_lw");
                            if cat != "knospe" && cat != "bio" {
                                edit_aus_umstellbetrieb.set(false);
                            }
                        };

                        let wildsammlung_step = "aus zertifizierter Wildsammlung";

                        rsx! {
                            // Radio: Bio (Knospe)
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
                            // Radio: Andere
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

                            // Conditional sub-fields below separator
                            if bio_cat == "knospe" {
                                br {}
                                div { class: "border-t border-base-300 pt-2 mt-2",
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
                                            br {}
                                        }
                                    }
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
                                        textarea {
                                            class: "textarea textarea-bordered w-full mt-2",
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
                                        textarea {
                                            class: "textarea textarea-bordered w-full mt-2",
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
                                        textarea {
                                            class: "textarea textarea-bordered w-full mt-2",
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
                    MultiCountrySelect {
                        values: edit_origins.read().clone(),
                        onchange: move |countries| {
                            edit_origins.set(countries);
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
            div { class: "modal-action",
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
                    button {
                        class: "btn btn-info",
                        onclick: handle_save_to_storage,
                        title: t!("tooltips.save_composite_ingredient").to_string(),
                        {t!("buttons.save_to_storage").to_string()}
                    }
                }

                // Show save button:
                // - Sub-ingredient: name non-empty (amount 0 ok for qualitative)
                // - Top-level leaf: name non-empty + amount > 0
                // - Composite: name non-empty + at least one child with weight
                if !edit_name().is_empty() && (
                    props.is_sub_ingredient
                    || (!edit_is_composite() && edit_amount().is_some_and(|a| a > 0.0))
                    || (edit_is_composite() && edit_children().as_ref().is_some_and(|c| c.iter().any(|child| child.computed_amount() > 0.0)))
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
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| handle_save(true),
                        title: t!("buttons.transfer_scaling_title", factor = format!("{:.2}", scaling_factor())).to_string(),
                        {t!("buttons.save_and_transfer").to_string()}
                    }
                }
            }
        }
    }
}
