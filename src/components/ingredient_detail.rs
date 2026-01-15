use crate::components::*;
use crate::core::{Ingredient, AmountUnit};
use crate::model::{food_db, lookup_allergen, lookup_agricultural};
use crate::rules::RuleDef;
use crate::services::UnifiedIngredient;
use crate::shared::{Conditionals, Validations};
use crate::persistence::{save_composite_ingredient, get_saved_ingredients_list};
use dioxus::prelude::*;
use rust_i18n::t;

// Component for editing or creating ingredients with allergen management and recipe scaling

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
    let ingredients: Signal<Vec<Ingredient>>;
    if props.genesis {
        ingredients = use_signal(|| vec![Ingredient::default()]);
        index = 0;
    } else {
        index = props.index;
        ingredients = props.ingredients;
    }
    let mut is_open = use_signal(|| false);

    // Local state for editing - won't be saved until user clicks save
    let original_ingredient = if let Some(ingredient) = ingredients.get(index) {
        ingredient.clone()
    } else {
        // Fallback to default ingredient if index is invalid
        tracing::warn!("Invalid ingredient index {}, using default", index);
        Ingredient::default()
    };
    let mut edit_name = use_signal(|| original_ingredient.name.clone());
    let mut edit_amount = use_signal(|| {
        if props.genesis {
            None  // Start with blank for new ingredients
        } else {
            Some(original_ingredient.amount)  // Show existing amount for edits
        }
    });
    let mut edit_unit = use_signal(|| original_ingredient.unit.clone());
    let mut edit_is_composite = use_signal(|| {
        original_ingredient.sub_components.as_ref().is_some_and(|s| !s.is_empty())
    });
    let mut edit_is_namensgebend = use_signal(|| {
        original_ingredient.is_namensgebend.unwrap_or(false)
    });
    let mut edit_sub_components = use_signal(|| original_ingredient.sub_components.clone());
    let mut edit_category = use_signal(|| original_ingredient.category.clone());
    let mut edit_origin = use_signal(|| original_ingredient.origin.clone());
    let mut edit_aufzucht_ort = use_signal(|| original_ingredient.aufzucht_ort.clone());
    let mut edit_schlachtungs_ort = use_signal(|| original_ingredient.schlachtungs_ort.clone());
    let mut edit_fangort = use_signal(|| original_ingredient.fangort.clone());
    let mut edit_is_bio = use_signal(|| original_ingredient.is_bio.unwrap_or(false));
    let mut edit_aus_umstellbetrieb = use_signal(|| original_ingredient.aus_umstellbetrieb.unwrap_or(false));
    let mut edit_bio_ch = use_signal(|| original_ingredient.bio_ch.unwrap_or(false));
    let mut save_status = use_signal(|| None::<String>);
    
    // Check if the current name is in the food database
    let mut is_custom_ingredient = use_signal(|| {
        !food_db().iter().any(|(name, _)| name == &edit_name())
    });
    
    // Track allergen status separately for custom ingredients
    let mut is_allergen_custom = use_signal(|| original_ingredient.is_allergen);
    
    // Track if amount has changed and calculate the scaling factor
    let amount_has_changed = use_memo(move || {
        if props.genesis {
            false
        } else if let Some(current_amount) = edit_amount() {
            let original_amount = original_ingredient.amount;
            // Check if amount changed significantly (not just rounding errors)
            (original_amount - current_amount).abs() > 0.01
        } else {
            false
        }
    });
    
    let scaling_factor = use_memo(move || {
        if props.genesis || original_ingredient.amount == 0.0 {
            1.0
        } else if let Some(current_amount) = edit_amount() {
            current_amount / original_ingredient.amount
        } else {
            1.0
        }
    });
    
    // Create a wrapper ingredients signal for SubIngredientsTable
    // Initialize it once and keep it stable
    let mut wrapper_ingredients = use_signal(|| {
        vec![Ingredient {
            name: original_ingredient.name.clone(),
            amount: original_ingredient.amount,
            unit: original_ingredient.unit.clone(),
            is_allergen: original_ingredient.is_allergen,
            is_namensgebend: original_ingredient.is_namensgebend,
            sub_components: original_ingredient.sub_components.clone(),
            origin: original_ingredient.origin.clone(),
            is_agricultural: original_ingredient.is_agricultural,
            is_bio: original_ingredient.is_bio,
            category: original_ingredient.category.clone(),
            aufzucht_ort: original_ingredient.aufzucht_ort.clone(),
            schlachtungs_ort: original_ingredient.schlachtungs_ort.clone(),
            fangort: original_ingredient.fangort.clone(),
            aus_umstellbetrieb: original_ingredient.aus_umstellbetrieb,
            bio_ch: original_ingredient.bio_ch,
        }]
    });
    
    // When composite mode changes, sync the wrapper
    use_effect(move || {
        let _ = edit_is_composite(); // Track this dependency
        if edit_is_composite() {
            // Only copy sub_components if they actually exist, otherwise start fresh
            // This prevents stale sub_components from a previous composite ingredient
            // from appearing when creating a new composite ingredient
            let current_sub_components = edit_sub_components();
            let sub_components_to_use = if current_sub_components.as_ref().is_some_and(|s| !s.is_empty()) {
                current_sub_components
            } else {
                None  // Start with empty sub_components for new composite ingredients
            };

            // Initialize wrapper with current edit state
            wrapper_ingredients.write()[0] = Ingredient {
                name: edit_name(),
                amount: edit_amount().unwrap_or(0.0),  // Use 0 as fallback for sub-ingredients
                unit: edit_unit(),
                is_allergen: is_allergen_custom(),
                is_namensgebend: Some(edit_is_namensgebend()),
                sub_components: sub_components_to_use,
                origin: edit_origin(),
                is_agricultural: lookup_agricultural(&edit_name()),
                is_bio: Some(edit_is_bio()),
                category: edit_category(),
                aufzucht_ort: edit_aufzucht_ort(),
                schlachtungs_ort: edit_schlachtungs_ort(),
                fangort: edit_fangort(),
                aus_umstellbetrieb: Some(edit_aus_umstellbetrieb()),
                bio_ch: Some(edit_bio_ch()),
            };
        } else {
            // Clear wrapper_ingredients sub-components when toggling off composite mode
            wrapper_ingredients.write()[0].sub_components = None;
        }
    });
    
    // Track changes from SubIngredientsTable back to edit state
    // Only monitor wrapper_ingredients changes
    use_effect(move || {
        if let Some(wrapper_sub) = wrapper_ingredients.read().first().and_then(|i| i.sub_components.as_ref()) {
            // Only update if actually different
            let current_edit_sub = edit_sub_components();
            if current_edit_sub.as_ref() != Some(wrapper_sub) {
                edit_sub_components.set(Some(wrapper_sub.clone()));
            }
        }
    });
    
    
    let handle_ingredient_select = move |unified_ingredient: UnifiedIngredient| {
        // Update name
        edit_name.set(unified_ingredient.name.clone());

        // Set category from unified ingredient
        edit_category.set(unified_ingredient.category.clone());

        // Log category to console for debugging (hidden from UI)
        if let Some(category) = &unified_ingredient.category {
            web_sys::console::log_1(&format!("🏷️ Ingredient '{}' category: {}", unified_ingredient.name, category).into());
        }

        // Set allergen status - use local DB data if available, otherwise custom value
        if let Some(is_allergen) = unified_ingredient.is_allergen {
            is_allergen_custom.set(is_allergen);
        }

        // Set bio status if available
        if let Some(is_bio) = unified_ingredient.is_bio {
            edit_is_bio.set(is_bio);
        }

        // Determine if this is a custom ingredient based on source
        match unified_ingredient.source {
            crate::services::IngredientSource::Local => {
                is_custom_ingredient.set(false);
            }
            crate::services::IngredientSource::BLV => {
                is_custom_ingredient.set(true);
            }
            crate::services::IngredientSource::Merged => {
                is_custom_ingredient.set(false); // Has local data
            }
        }

        // Note: Category is now set from unified ingredient
    };

    let _update_name = move |new_name: String| {
        // Update local edit state only
        edit_name.set(new_name.clone());

        // Check if this is a saved composite ingredient
        let saved_ingredients = get_saved_ingredients_list();
        if let Some(saved) = saved_ingredients.iter().find(|i| i.name == new_name) {
            // Load saved ingredient data
            edit_is_composite.set(true);
            edit_sub_components.set(saved.sub_components.clone());
            is_allergen_custom.set(saved.is_allergen);
            edit_is_namensgebend.set(saved.is_namensgebend.unwrap_or(false));
            if let Some(category) = &saved.category {
                edit_category.set(Some(category.clone()));
                // Log saved category to console for debugging (hidden from UI)
                web_sys::console::log_1(&format!("🏷️ Loaded saved ingredient '{}' category: {}", new_name, category).into());
            }
            is_custom_ingredient.set(true);  // Saved ingredients are treated as custom
            return;  // Don't fetch category again
        }

        // Check if the new name is in the food database
        let in_database = food_db().iter().any(|(name, _)| name == &new_name);
        is_custom_ingredient.set(!in_database);

        // If switching from database to custom or vice versa, update allergen status
        if in_database {
            is_allergen_custom.set(lookup_allergen(&new_name));
        }

        // Clear category when name changes (will be set via unified selection)
        if new_name.is_empty() {
            edit_category.set(None);
        }
    };

    
    let handle_save_to_storage = move |_| {
        // Only save composite ingredients with sub-components
        if edit_is_composite() && edit_sub_components().is_some() {
            let ingredient_to_save = Ingredient {
                name: edit_name(),
                amount: 100.0,  // Save with standard amount
                unit: edit_unit(),
                is_allergen: is_allergen_custom(),
                is_namensgebend: Some(edit_is_namensgebend()),
                sub_components: edit_sub_components(),
                origin: edit_origin(),
                is_agricultural: lookup_agricultural(&edit_name()),
                is_bio: Some(edit_is_bio()),
                category: edit_category(),
                aufzucht_ort: edit_aufzucht_ort(),
                schlachtungs_ort: edit_schlachtungs_ort(),
                fangort: edit_fangort(),
                aus_umstellbetrieb: Some(edit_aus_umstellbetrieb()),
                bio_ch: Some(edit_bio_ch()),
            };
            
            match save_composite_ingredient(&ingredient_to_save) {
                Ok(_) => {
                    save_status.set(Some(t!("messages.ingredient_saved_successfully", name = edit_name()).to_string()));
                    // Clear status after 2 seconds
                    let mut save_status_clone = save_status;
                    spawn(async move {
                        gloo::timers::future::TimeoutFuture::new(2000).await;
                        save_status_clone.set(None);
                    });
                }
                Err(e) => {
                    save_status.set(Some(t!("messages.error_generic", error = e).to_string()));
                    // Clear status after 3 seconds
                    let mut save_status_clone = save_status;
                    spawn(async move {
                        gloo::timers::future::TimeoutFuture::new(3000).await;
                        save_status_clone.set(None);
                    });
                }
            }
        }
    };
    
    let mut handle_save = move |scale_all: bool| {
        // Validate that amount is provided
        let amount = match edit_amount() {
            Some(amt) if amt > 0.0 => amt,
            _ => {
                // Don't save if amount is invalid or not provided
                return;
            }
        };
        
        // Determine allergen status based on database presence
        let in_database = food_db().iter().any(|(name, _)| name == &edit_name());
        let allergen_status = if in_database {
            lookup_allergen(&edit_name())
        } else {
            is_allergen_custom()
        };
        
        let new_ingredient = Ingredient {
            name: edit_name(),
            amount,
            unit: edit_unit(),
            is_allergen: allergen_status,
            is_namensgebend: Some(edit_is_namensgebend()),
            sub_components: edit_sub_components(),
            origin: edit_origin(),
            is_agricultural: lookup_agricultural(&edit_name()),
            is_bio: Some(edit_is_bio()),
            category: edit_category(),
            aufzucht_ort: edit_aufzucht_ort(),
            schlachtungs_ort: edit_schlachtungs_ort(),
            fangort: edit_fangort(),
            aus_umstellbetrieb: Some(edit_aus_umstellbetrieb()),
            bio_ch: Some(edit_bio_ch()),
        };
        
        if props.genesis {
            // Check if ingredient with same name and properties already exists (only for non-composite)
            if new_ingredient.sub_components.is_none() || new_ingredient.sub_components.as_ref().unwrap().is_empty() {
                let mut existing_ingredients = props.ingredients.write();
                if let Some(existing_index) = existing_ingredients.iter().position(|ing| {
                    ing.name == new_ingredient.name 
                    && ing.is_allergen == new_ingredient.is_allergen
                    && ing.is_namensgebend == new_ingredient.is_namensgebend
                    && (ing.sub_components.is_none() || ing.sub_components.as_ref().unwrap().is_empty())
                }) {
                    // Merge amounts instead of adding duplicate
                    existing_ingredients[existing_index].amount += new_ingredient.amount;
                    // Update category if the new ingredient has one
                    if new_ingredient.category.is_some() {
                        existing_ingredients[existing_index].category = new_ingredient.category;
                    }
                } else {
                    existing_ingredients.push(new_ingredient);
                }
            } else {
                props.ingredients.write().push(new_ingredient);
            }
            
            // Reset local state for next creation
            edit_name.set(String::new());
            edit_amount.set(None);
            edit_unit.set(AmountUnit::default());
            edit_is_composite.set(false);
            edit_is_namensgebend.set(false);
            edit_sub_components.set(None);
            is_allergen_custom.set(false);
            edit_category.set(None);
            edit_aus_umstellbetrieb.set(false);
            edit_bio_ch.set(false);
            edit_is_bio.set(false);
            // Reset wrapper_ingredients for next creation
            wrapper_ingredients.write()[0] = Ingredient {
                name: String::new(),
                amount: 0.0,
                unit: AmountUnit::default(),
                is_allergen: false,
                is_namensgebend: None,
                sub_components: None,
                origin: None,
                is_agricultural: false,
                is_bio: None,
                category: None,
                aufzucht_ort: None,
                schlachtungs_ort: None,
                fangort: None,
                aus_umstellbetrieb: None,
                bio_ch: None,
            };
        } else {
            // Update existing ingredient
            if scale_all && amount_has_changed() {
                // Apply scaling factor to all ingredients
                let factor = scaling_factor();
                let mut all_ingredients = props.ingredients.write();
                for (i, ingredient) in all_ingredients.iter_mut().enumerate() {
                    if i == index {
                        // Update the current ingredient with all changes
                        *ingredient = new_ingredient.clone();
                    } else {
                        // Scale other ingredients by the same factor
                        ingredient.amount *= factor;
                    }
                }
            } else {
                // Just update the single ingredient
                props.ingredients.write()[index] = new_ingredient;
            }
        }
        is_open.set(false);
    };


    let herkunft_path = format!("herkunft_benoetigt_{}", index);

    // Check for validation errors for this ingredient
    let validations_context = use_context::<Validations>();
    let has_validation_error = use_memo(move || {
        let validation_entries = (*validations_context.0.read()).clone();
        let has_origin_error = validation_entries.get(&format!("ingredients[{}][origin]", index))
            .is_some_and(|v| !v.is_empty());
        let has_amount_error = validation_entries.get(&format!("ingredients[{}][amount]", index))
            .is_some_and(|v| !v.is_empty());
        has_origin_error || has_amount_error
    });

    rsx! {
        if props.genesis {
            button {
                class: "btn btn-accent",
                onclick: move |_| {
                    // Reset edit state when opening in create mode
                    if !is_open() {
                        edit_name.set(String::new());
                        edit_amount.set(None);  // Blank amount field
                        edit_unit.set(AmountUnit::default());
                        edit_is_composite.set(false);
                        edit_is_namensgebend.set(false);
                        edit_sub_components.set(None);
                        is_allergen_custom.set(false);
                        is_custom_ingredient.set(true);
                        edit_category.set(None);
                        edit_aus_umstellbetrieb.set(false);
                        edit_bio_ch.set(false);
                        edit_is_bio.set(false);
                        // Reset wrapper_ingredients to clear any previous sub-components
                        wrapper_ingredients.write()[0] = Ingredient {
                            name: String::new(),
                            amount: 0.0,
                            unit: AmountUnit::default(),
                            is_allergen: false,
                            is_namensgebend: None,
                            sub_components: None,
                            origin: None,
                            is_agricultural: false,
                            is_bio: None,
                            category: None,
                            aufzucht_ort: None,
                            schlachtungs_ort: None,
                            fangort: None,
                            aus_umstellbetrieb: None,
                            bio_ch: None,
                        };
                    }
                    is_open.toggle();
                },
                "{t!(\"nav.hinzufuegen\")}"
            }
        } else {
            button {
                class: if *has_validation_error.read() {
                    "btn join-item btn-outline btn-error relative"
                } else {
                    "btn join-item btn-outline"
                },
                onclick: move |_| {
                    // Reset edit state when opening in edit mode
                    if !is_open() {
                        let orig = ingredients.get(index).unwrap().clone();
                        edit_name.set(orig.name.clone());
                        edit_amount.set(Some(orig.amount));
                        edit_unit.set(orig.unit.clone());
                        edit_is_composite.set(
                            orig.sub_components.as_ref().is_some_and(|s| !s.is_empty())
                        );
                        edit_is_namensgebend.set(
                            orig.is_namensgebend.unwrap_or(false)
                        );
                        edit_sub_components.set(orig.sub_components.clone());
                        is_allergen_custom.set(orig.is_allergen);
                        is_custom_ingredient.set(
                            !food_db().iter().any(|(name, _)| name == &orig.name)
                        );
                        edit_category.set(orig.category.clone());
                        edit_aufzucht_ort.set(orig.aufzucht_ort.clone());
                        edit_schlachtungs_ort.set(orig.schlachtungs_ort.clone());
                        edit_fangort.set(orig.fangort.clone());
                        edit_aus_umstellbetrieb.set(orig.aus_umstellbetrieb.unwrap_or(false));
                        edit_bio_ch.set(orig.bio_ch.unwrap_or(false));
                    }
                    is_open.toggle();
                },
                icons::ListDetail {}
                if *has_validation_error.read() {
                    span {
                        class: "absolute -top-2 -right-2 bg-error text-error-content rounded-full w-4 h-4 text-xs flex items-center justify-center",
                        "!"
                    }
                }
            }
        }
        dialog { open: "{is_open}", class: "modal",
            div { class: "modal-box bg-base-100",
                h3 { class: "font-bold text-lg", "{t!(\"label.zutatDetails\")}" }
                FormField {
                    label: t!("label.zutatEingeben"),
                    UnifiedIngredientInput {
                        bound_value: edit_name,
                        on_ingredient_select: handle_ingredient_select,
                        required: true,
                        placeholder: t!("placeholder.zutatName").to_string(),
                        autofocus: true
                    }
                }
                br {}
                FormField {
                    label: t!("label.menge"),
                    ValidationDisplay {
                        paths: vec![
                            format!("ingredients[{}][amount]", index)
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
                                option { value: "g", selected: edit_unit() == AmountUnit::Gram, {t!("units.g")} }
                                option { value: "ml", selected: edit_unit() == AmountUnit::Milliliter, {t!("units.ml")} }
                            }
                        }
                    }
                    // Show scaling factor when amount changes
                    if !props.genesis && amount_has_changed() {
                        div { class: "text-sm text-info mt-2",
                            if let Some(amt) = edit_amount() {
                                {t!("messages.scaling_factor", factor = format!("{:.2}", scaling_factor()), before = original_ingredient.amount.to_string(), after = amt.to_string())}
                            } else {
                                {t!("messages.please_enter_amount")}
                            }
                        }
                    }
                }

                br {}
                br {}
                
                // Show allergen status - checkbox for custom ingredients, text for database allergens
                if !edit_name().is_empty() && !edit_is_composite() {
                    if is_custom_ingredient() {
                        // Custom ingredient - show checkbox
                        FormField {
                            help: Some((t!("help.allergenManual")).into()),
                            label: t!("label.allergen"),
                            inline_checkbox: true,
                            CheckboxInput {
                                bound_value: is_allergen_custom
                            }
                        }
                        br {}
                    } else if is_allergen_custom() {
                        // Database allergen - show text only
                        FormField {
                            label: t!("label.allergen"),
                            div { class: "py-2",
                                span { class: "font-bold", "({t!(\"label.allergen\")})" }
                            }
                        }
                        br {}
                    }
                }

                FormField {
                    label: t!("label.zusammengesetzteZutat"),
                    help: Some((t!("help.zusammengesetzteZutaten")).into()),
                    inline_checkbox: true,
                    CheckboxInput {
                        bound_value: edit_is_composite
                    }
                }
                if edit_is_composite() {
                    SubIngredientsTable {
                        ingredients: wrapper_ingredients,
                        index: 0
                    }
                }
                br {}
                ConditionalDisplay {
                    path: "namensgebende_zutat".to_string(),
                    FormField {
                        help: Some((t!("help.namensgebendeZutaten")).into()),
                        label: t!("label.namensgebendeZutat"),
                        inline_checkbox: true,
                        CheckboxInput {
                            bound_value: edit_is_namensgebend
                        }
                    }
                }
                // Show save status if any
                if let Some(status) = save_status() {
                    div { class: "alert alert-info mb-4",
                        span { "{status}" }
                    }
                }

                br {}
                {
                    // Check if Bio rule is active and determine configuration
                    let should_show_bio = use_memo(move || {
                        let rules = props.rules.read();
                        rules.contains(&RuleDef::Bio_Knospe_EingabeIstBio)
                    });

                    let is_knospe_config = use_memo(move || {
                        let rules = props.rules.read();
                        rules.contains(&RuleDef::Knospe_ShowBioSuisseLogo)
                    });

                    // Check if ingredient is agricultural (water, salt etc. are not)
                    let is_agricultural = use_memo(move || {
                        lookup_agricultural(&edit_name())
                    });

                    if should_show_bio() && is_agricultural() {
                        if is_knospe_config() {
                            // Knospe configuration: Show both Bio (Knospe) and BioCH with mutual exclusion
                            rsx! {
                                FormField {
                                    help: Some((t!("help.bio_knospe")).into()),
                                    label: t!("bio_labels.bio_knospe"),
                                    inline_checkbox: true,
                                    input {
                                        r#type: "checkbox",
                                        class: "checkbox checkbox-accent",
                                        checked: edit_is_bio(),
                                        disabled: edit_bio_ch() || edit_aus_umstellbetrieb(),
                                        onchange: move |evt| {
                                            edit_is_bio.set(evt.data.value() == "true");
                                        }
                                    }
                                }
                                br {}
                                FormField {
                                    help: Some((t!("help.bio_ch")).into()),
                                    label: t!("bio_labels.bio_ch"),
                                    inline_checkbox: true,
                                    input {
                                        r#type: "checkbox",
                                        class: "checkbox checkbox-accent",
                                        checked: edit_bio_ch(),
                                        disabled: edit_is_bio() || edit_aus_umstellbetrieb(),
                                        onchange: move |evt| {
                                            edit_bio_ch.set(evt.data.value() == "true");
                                        }
                                    }
                                }
                                br {}
                                FormField {
                                    help: Some((t!("help.bio_transitional")).into()),
                                    label: t!("bio_labels.aus_umstellbetrieb"),
                                    inline_checkbox: true,
                                    input {
                                        r#type: "checkbox",
                                        class: "checkbox checkbox-accent",
                                        checked: edit_aus_umstellbetrieb(),
                                        disabled: edit_is_bio() || edit_bio_ch(),
                                        onchange: move |evt| {
                                            edit_aus_umstellbetrieb.set(evt.data.value() == "true");
                                        }
                                    }
                                }
                            }
                        } else {
                            // Bio configuration: Show only BioCH with mutual exclusion to Umstellbetrieb
                            rsx! {
                                FormField {
                                    help: Some((t!("help.bio_ch")).into()),
                                    label: t!("bio_labels.bio_ch"),
                                    inline_checkbox: true,
                                    input {
                                        r#type: "checkbox",
                                        class: "checkbox checkbox-accent",
                                        checked: edit_bio_ch(),
                                        disabled: edit_aus_umstellbetrieb(),
                                        onchange: move |evt| {
                                            edit_bio_ch.set(evt.data.value() == "true");
                                        }
                                    }
                                }
                                br {}
                                FormField {
                                    help: Some((t!("help.bio_transitional")).into()),
                                    label: t!("bio_labels.aus_umstellbetrieb"),
                                    inline_checkbox: true,
                                    input {
                                        r#type: "checkbox",
                                        class: "checkbox checkbox-accent",
                                        checked: edit_aus_umstellbetrieb(),
                                        disabled: edit_bio_ch(),
                                        onchange: move |evt| {
                                            edit_aus_umstellbetrieb.set(evt.data.value() == "true");
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
                    // Get context access outside the memo to avoid hook-in-hook violation
                    let conditionals_context = use_context::<Conditionals>();

                    // Check if Knospe rules are active (always show origin field) or traditional conditional is set
                    let should_show_origin = use_memo(move || {
                        let rules = props.rules.read();
                        let has_knospe = rules.iter().any(|rule|
                            *rule == RuleDef::Knospe_AlleZutatenHerkunft ||
                            *rule == RuleDef::Knospe_100_Percent_CH_NoOrigin ||
                            *rule == RuleDef::Knospe_90_99_Percent_CH_ShowOrigin
                        );

                        if has_knospe {
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
                                label: t!("origin.herkunft"),
                                ValidationDisplay {
                                    paths: vec![
                                        format!("ingredients[{}][origin]", index)
                                    ],
                                    CountrySelect {
                                        value: edit_origin.read().clone(),
                                        onchange: move |country| {
                                            edit_origin.set(country);
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
                    // Check if AP7_4 rule is active and ingredient is beef to show beef specific fields
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
                                label: t!("origin.aufzucht"),
                                help: Some((t!("help.aufzucht_location")).into()),
                                ValidationDisplay {
                                    paths: vec![
                                        format!("ingredients[{}][aufzucht_ort]", index)
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
                                label: t!("origin.schlachtung"),
                                ValidationDisplay {
                                    paths: vec![
                                        format!("ingredients[{}][schlachtungs_ort]", index)
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
                    // Check if AP7_5 rule is active and ingredient is fish to show fish specific field
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
                                label: t!("origin.fangort"),
                                ValidationDisplay {
                                    paths: vec![
                                        format!("ingredients[{}][fangort]", index)
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
                div { class: "modal-action",
                    button {
                        class: "btn",
                        onclick: move |_| {
                            // Reset edit state on cancel
                            let orig = ingredients.get(index).unwrap().clone();
                            edit_name.set(orig.name.clone());
                            edit_amount.set(Some(orig.amount));
                            edit_is_composite.set(
                                !orig.sub_components
                                    .clone()
                                    .unwrap_or_default()
                                    .is_empty()
                            );
                            edit_is_namensgebend.set(
                                orig.is_namensgebend
                                    .unwrap_or(false)
                            );
                            edit_sub_components.set(orig.sub_components.clone());
                            is_allergen_custom.set(orig.is_allergen);
                            is_custom_ingredient.set(
                                !food_db().iter().any(|(name, _)| name == &orig.name)
                            );
                            edit_category.set(orig.category.clone());
                            edit_aufzucht_ort.set(orig.aufzucht_ort.clone());
                            edit_schlachtungs_ort.set(orig.schlachtungs_ort.clone());
                            edit_fangort.set(orig.fangort.clone());
                            edit_aus_umstellbetrieb.set(orig.aus_umstellbetrieb.unwrap_or(false));
                            edit_bio_ch.set(orig.bio_ch.unwrap_or(false));
                            is_open.set(false);
                        },
                        {t!("nav.schliessen")},
                    }
                    
                    // Show "Merken" button only for composite ingredients
                    if edit_is_composite() && edit_sub_components().is_some() {
                        button {
                            class: "btn btn-info",
                            onclick: handle_save_to_storage,
                            title: t!("tooltips.save_composite_ingredient").to_string(),
                            {t!("buttons.save_to_storage")}
                        }
                    }
                    
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| handle_save(false),
                        {t!("nav.speichern")},
                    }
                    if !props.genesis && amount_has_changed() {
                        button {
                            class: "btn btn-secondary",
                            onclick: move |_| handle_save(true),
                            title: t!("buttons.transfer_scaling_title", factor = format!("{:.2}", scaling_factor())).to_string(),
                            {t!("buttons.save_and_transfer")}
                        }
                    }
                }
            }
            form {
                method: "dialog",
                class: "modal-backdrop",
                onclick: move |_| {
                    // Reset edit state on backdrop click
                    let orig = ingredients.get(index).unwrap().clone();
                    edit_name.set(orig.name.clone());
                    edit_amount.set(Some(orig.amount));
                    edit_is_composite.set(
                        orig.sub_components.as_ref().is_some_and(|s| !s.is_empty())
                    );
                    edit_is_namensgebend.set(orig.is_namensgebend.unwrap_or(false));
                    edit_sub_components.set(orig.sub_components.clone());
                    is_allergen_custom.set(orig.is_allergen);
                    is_custom_ingredient.set(
                        !food_db().iter().any(|(name, _)| name == &orig.name)
                    );
                    edit_category.set(orig.category.clone());
                    edit_aufzucht_ort.set(orig.aufzucht_ort.clone());
                    edit_schlachtungs_ort.set(orig.schlachtungs_ort.clone());
                    is_open.set(false);
                },
                button { "" }
            }
        }

    }
}
