use crate::api::search_food;
use crate::components::*;
use crate::core::Ingredient;
use crate::model::{food_db, lookup_allergen, lookup_agricultural, Country};
use crate::rules::RuleDef;
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
    let original_ingredient = ingredients.get(index).unwrap().clone();
    let mut edit_name = use_signal(|| original_ingredient.name.clone());
    let mut edit_amount = use_signal(|| {
        if props.genesis {
            None  // Start with blank for new ingredients
        } else {
            Some(original_ingredient.amount)  // Show existing amount for edits
        }
    });
    let mut edit_is_composite = use_signal(|| {
        original_ingredient.sub_components.as_ref().map_or(false, |s| !s.is_empty())
    });
    let mut edit_is_namensgebend = use_signal(|| {
        original_ingredient.is_namensgebend.unwrap_or(false)
    });
    let mut edit_sub_components = use_signal(|| original_ingredient.sub_components.clone());
    let mut edit_category = use_signal(|| original_ingredient.category.clone());
    let mut is_fetching_category = use_signal(|| false);
    let mut edit_origin = use_signal(|| original_ingredient.origin.clone());
    let mut edit_aufzucht_ort = use_signal(|| original_ingredient.aufzucht_ort.clone());
    let mut edit_schlachtungs_ort = use_signal(|| original_ingredient.schlachtungs_ort.clone());
    let mut edit_fangort = use_signal(|| original_ingredient.fangort.clone());
    let mut edit_is_bio = use_signal(|| original_ingredient.is_bio.unwrap_or(false));
    let mut edit_aus_umstellbetrieb = use_signal(|| original_ingredient.aus_umstellbetrieb.unwrap_or(false));
    let mut edit_bio_nicht_knospe = use_signal(|| original_ingredient.bio_nicht_knospe.unwrap_or(false));
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
            bio_nicht_knospe: original_ingredient.bio_nicht_knospe,
        }]
    });
    
    // When composite mode changes, sync the wrapper
    use_effect(move || {
        let _ = edit_is_composite(); // Track this dependency
        if edit_is_composite() {
            // Initialize wrapper with current edit state
            wrapper_ingredients.write()[0] = Ingredient {
                name: edit_name(),
                amount: edit_amount().unwrap_or(0.0),  // Use 0 as fallback for sub-ingredients
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
                bio_nicht_knospe: Some(edit_bio_nicht_knospe()),
            };
        }
    });
    
    // Track changes from SubIngredientsTable back to edit state
    // Only monitor wrapper_ingredients changes
    use_effect(move || {
        if let Some(wrapper_sub) = wrapper_ingredients.read().get(0).and_then(|i| i.sub_components.as_ref()) {
            // Only update if actually different
            let current_edit_sub = edit_sub_components();
            if current_edit_sub.as_ref() != Some(wrapper_sub) {
                edit_sub_components.set(Some(wrapper_sub.clone()));
            }
        }
    });
    
    
    let mut update_name = move |new_name: String| {
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
            if saved.category.is_some() {
                edit_category.set(saved.category.clone());
                is_fetching_category.set(false);
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
        
        // Fetch category from API
        if !new_name.is_empty() {
            let name_for_api = new_name.clone();
            let mut edit_category_clone = edit_category.clone();
            let mut is_fetching_clone = is_fetching_category.clone();
            
            // Clear previous category and set loading state
            edit_category.set(None);
            is_fetching_category.set(true);
            
            spawn(async move {
                // Get current locale for API call
                let locale = rust_i18n::locale().to_string();
                let lang = if locale.starts_with("fr") {
                    "fr"
                } else if locale.starts_with("it") {
                    "it"
                } else {
                    "de"
                };
                
                match search_food(&name_for_api, lang).await {
                    Ok(category) => {
                        edit_category_clone.set(category);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to fetch category: {}", e);
                    }
                }
                is_fetching_clone.set(false);
            });
        } else {
            edit_category.set(None);
            is_fetching_category.set(false);
        }
    };

    
    let handle_save_to_storage = move |_| {
        // Only save composite ingredients with sub-components
        if edit_is_composite() && edit_sub_components().is_some() {
            let ingredient_to_save = Ingredient {
                name: edit_name(),
                amount: 100.0,  // Save with standard amount
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
                bio_nicht_knospe: Some(edit_bio_nicht_knospe()),
            };
            
            match save_composite_ingredient(&ingredient_to_save) {
                Ok(_) => {
                    save_status.set(Some(t!("messages.ingredient_saved_successfully", name = edit_name()).to_string()));
                    // Clear status after 2 seconds
                    let mut save_status_clone = save_status.clone();
                    spawn(async move {
                        gloo::timers::future::TimeoutFuture::new(2000).await;
                        save_status_clone.set(None);
                    });
                }
                Err(e) => {
                    save_status.set(Some(t!("messages.error_generic", error = e).to_string()));
                    // Clear status after 3 seconds
                    let mut save_status_clone = save_status.clone();
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
            bio_nicht_knospe: Some(edit_bio_nicht_knospe()),
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
            edit_is_composite.set(false);
            edit_is_namensgebend.set(false);
            edit_sub_components.set(None);
            is_allergen_custom.set(false);
            edit_category.set(None);
            edit_aus_umstellbetrieb.set(false);
            edit_bio_nicht_knospe.set(false);
            is_fetching_category.set(false);
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
        validation_entries.contains_key(&format!("ingredients[{}][origin]", index))
            || validation_entries.contains_key(&format!("ingredients[{}][amount]", index))
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
                        edit_is_composite.set(false);
                        edit_is_namensgebend.set(false);
                        edit_sub_components.set(None);
                        is_allergen_custom.set(false);
                        is_custom_ingredient.set(true);
                        edit_category.set(None);
                        edit_aus_umstellbetrieb.set(false);
                        edit_bio_nicht_knospe.set(false);
                        is_fetching_category.set(false);
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
                        edit_is_composite.set(
                            orig.sub_components.as_ref().map_or(false, |s| !s.is_empty())
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
                        edit_bio_nicht_knospe.set(orig.bio_nicht_knospe.unwrap_or(false));
                        is_fetching_category.set(false);
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
                    input {
                        list: "ingredients",
                        r#type: "flex",
                        placeholder: t!("placeholder.zutatName").as_ref(),
                        class: "input input-accent w-full",
                        oninput: move |evt| update_name(evt.data.value()),
                        value: "{edit_name}",
                        datalist { id: "ingredients",
                            // First, add saved composite ingredients
                            for saved_ing in get_saved_ingredients_list() {
                                option { 
                                    value: "{saved_ing.name}",
                                    label: t!("label.saved_indicator").to_string()
                                }
                            }
                            // Then add database ingredients
                            for item in food_db().clone() {
                                option { 
                                    value: "{item.0}",
                                    // Show allergen marker in label without duplicating the name
                                    label: if item.1 { t!("label.allergen_indicator").to_string() } else { "".to_string() }
                                }
                            }
                        }
                    }
                    // Show category status - either loading, fetched, or empty
                    if is_fetching_category() {
                        div { class: "text-sm text-info mt-1",
                            {t!("messages.category_loading")}
                        }
                    } else if let Some(category) = &edit_category() {
                        div { class: "text-sm text-success mt-1",
                            {t!("messages.category_display", category = category)}
                        }
                    }
                }
                br {}
                FormField {
                    label: format!("{} (g)", t!("label.menge")),
                    ValidationDisplay {
                        paths: vec![
                            format!("ingredients[{}][amount]", index)
                        ],
                        input {
                            r#type: "number",
                            placeholder: t!("placeholders.amount_in_grams").to_string(),
                            class: "input input-accent w-full",
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
                            label { class: "label cursor-pointer",
                                input {
                                    class: "checkbox",
                                    r#type: "checkbox",
                                    checked: "{is_allergen_custom}",
                                    oninput: move |e| {
                                        let is_checked = e.value() == "true";
                                        is_allergen_custom.set(is_checked);
                                        // Don't update immediately - wait for save
                                    },
                                }
                                span { class: "label-text",
                                    {t!("label.allergen")}
                                }
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
                    label { class: "label cursor-pointer",
                        input {
                            class: "checkbox",
                            r#type: "checkbox",
                            checked: "{edit_is_composite}",
                            oninput: move |e| edit_is_composite.set(e.value() == "true"),
                        }
                        span { class: "label-text",
                            "{t!(\"label.zusammengesetzteZutat\")}"
                        }
                    }
                    if edit_is_composite() {
                        SubIngredientsTable {
                            ingredients: wrapper_ingredients,
                            index: 0
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
                                checked: "{edit_is_namensgebend}",
                                oninput: move |e| {
                                    edit_is_namensgebend.set(e.value() == "true");
                                    // Don't update immediately - wait for save
                                },
                            }
                            span { class: "label-text",
                                "{t!(\"label.namensgebendeZutat\")}"
                            }
                        }
                    }
                }
                // Show save status if any
                if let Some(status) = save_status() {
                    div { class: "alert alert-info mb-4",
                        span { "{status}" }
                    }
                }

                {
                    // Check if Bio rule is active to show the bio checkbox
                    let should_show_bio = use_memo(move || {
                        let rules = props.rules.read();
                        rules.iter().any(|rule| *rule == RuleDef::Bio_Knospe_EingabeIstBio)
                    });

                    if should_show_bio() {
                        rsx! {
                            FormField {
                                help: Some("Gibt an, ob diese Zutat bio-zertifiziert ist".into()),
                                label: "Bio",
                                label { class: "label cursor-pointer",
                                    input {
                                        class: "checkbox",
                                        r#type: "checkbox",
                                        checked: "{edit_is_bio}",
                                        oninput: move |e| {
                                            let is_checked = e.value() == "true";
                                            edit_is_bio.set(is_checked);
                                        },
                                    }
                                    span { class: "label-text",
                                        "Bio"
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {}
                    }
                }
                {
                    // Check if Bio/Knospe rules are active to show the Umstellung checkboxes
                    let should_show_umstellung = use_memo(move || {
                        let rules = props.rules.read();
                        rules.iter().any(|rule| *rule == RuleDef::Bio_Knospe_EingabeIstBio)
                    });

                    if should_show_umstellung() {
                        rsx! {
                            FormField {
                                help: Some("Gibt an, ob diese Zutat aus einem Umstellbetrieb stammt".into()),
                                label: "Aus Umstellbetrieb",
                                label { class: "label cursor-pointer",
                                    input {
                                        class: "checkbox",
                                        r#type: "checkbox",
                                        checked: "{edit_aus_umstellbetrieb}",
                                        oninput: move |e| {
                                            let is_checked = e.value() == "true";
                                            edit_aus_umstellbetrieb.set(is_checked);
                                        },
                                    }
                                    span { class: "label-text",
                                        "Aus Umstellbetrieb"
                                    }
                                }
                            }
                            FormField {
                                help: Some("Gibt an, ob diese Zutat bio-zertifiziert ist, aber nicht nach Knospe-Standards".into()),
                                label: "Bio (nicht Knospe)",
                                label { class: "label cursor-pointer",
                                    input {
                                        class: "checkbox",
                                        r#type: "checkbox",
                                        checked: "{edit_bio_nicht_knospe}",
                                        oninput: move |e| {
                                            let is_checked = e.value() == "true";
                                            edit_bio_nicht_knospe.set(is_checked);
                                        },
                                    }
                                    span { class: "label-text",
                                        "Bio (nicht Knospe)"
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {}
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
                                        value: match edit_origin.read().as_ref() {
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
                                            edit_origin.set(Some(country));
                                        },
                                        option { value: "", selected: edit_origin.read().is_none(), "Bitte wählen..." }
                                        option { value: "CH", selected: matches!(edit_origin.read().as_ref(), Some(Country::CH)), "Schweiz" }
                                        option { value: "EU", selected: matches!(edit_origin.read().as_ref(), Some(Country::EU)), "EU" }
                                        option { value: "NoOriginRequired", selected: matches!(edit_origin.read().as_ref(), Some(Country::NoOriginRequired)), "Keine Herkunftsangabe benötigt" }

                                        option { value: "AD", selected: matches!(edit_origin.read().as_ref(), Some(Country::AD)), "Andorra" }
                                        option { value: "AE", selected: matches!(edit_origin.read().as_ref(), Some(Country::AE)), "Vereinigte Arabische Emirate" }
                                        option { value: "AF", selected: matches!(edit_origin.read().as_ref(), Some(Country::AF)), "Afghanistan" }
                                        option { value: "AG", selected: matches!(edit_origin.read().as_ref(), Some(Country::AG)), "Antigua und Barbuda" }
                                        option { value: "AI", selected: matches!(edit_origin.read().as_ref(), Some(Country::AI)), "Anguilla" }
                                        option { value: "AL", selected: matches!(edit_origin.read().as_ref(), Some(Country::AL)), "Albanien" }
                                        option { value: "AM", selected: matches!(edit_origin.read().as_ref(), Some(Country::AM)), "Armenien" }
                                        option { value: "AO", selected: matches!(edit_origin.read().as_ref(), Some(Country::AO)), "Angola" }
                                        option { value: "AQ", selected: matches!(edit_origin.read().as_ref(), Some(Country::AQ)), "Antarktis" }
                                        option { value: "AR", selected: matches!(edit_origin.read().as_ref(), Some(Country::AR)), "Argentinien" }
                                        option { value: "AS", selected: matches!(edit_origin.read().as_ref(), Some(Country::AS)), "Amerikanisch-Samoa" }
                                        option { value: "AT", selected: matches!(edit_origin.read().as_ref(), Some(Country::AT)), "Österreich" }
                                        option { value: "AU", selected: matches!(edit_origin.read().as_ref(), Some(Country::AU)), "Australien" }
                                        option { value: "AW", selected: matches!(edit_origin.read().as_ref(), Some(Country::AW)), "Aruba" }
                                        option { value: "AX", selected: matches!(edit_origin.read().as_ref(), Some(Country::AX)), "Åland" }
                                        option { value: "AZ", selected: matches!(edit_origin.read().as_ref(), Some(Country::AZ)), "Aserbaidschan" }
                                        option { value: "BA", selected: matches!(edit_origin.read().as_ref(), Some(Country::BA)), "Bosnien und Herzegowina" }
                                        option { value: "BB", selected: matches!(edit_origin.read().as_ref(), Some(Country::BB)), "Barbados" }
                                        option { value: "BD", selected: matches!(edit_origin.read().as_ref(), Some(Country::BD)), "Bangladesch" }
                                        option { value: "BE", selected: matches!(edit_origin.read().as_ref(), Some(Country::BE)), "Belgien" }
                                        option { value: "BF", selected: matches!(edit_origin.read().as_ref(), Some(Country::BF)), "Burkina Faso" }
                                        option { value: "BG", selected: matches!(edit_origin.read().as_ref(), Some(Country::BG)), "Bulgarien" }
                                        option { value: "BH", selected: matches!(edit_origin.read().as_ref(), Some(Country::BH)), "Bahrain" }
                                        option { value: "BI", selected: matches!(edit_origin.read().as_ref(), Some(Country::BI)), "Burundi" }
                                        option { value: "BJ", selected: matches!(edit_origin.read().as_ref(), Some(Country::BJ)), "Benin" }
                                        option { value: "BL", selected: matches!(edit_origin.read().as_ref(), Some(Country::BL)), "Saint-Barthélemy" }
                                        option { value: "BM", selected: matches!(edit_origin.read().as_ref(), Some(Country::BM)), "Bermuda" }
                                        option { value: "BN", selected: matches!(edit_origin.read().as_ref(), Some(Country::BN)), "Brunei" }
                                        option { value: "BO", selected: matches!(edit_origin.read().as_ref(), Some(Country::BO)), "Bolivien" }
                                        option { value: "BQ", selected: matches!(edit_origin.read().as_ref(), Some(Country::BQ)), "Bonaire, Sint Eustatius und Saba" }
                                        option { value: "BR", selected: matches!(edit_origin.read().as_ref(), Some(Country::BR)), "Brasilien" }
                                        option { value: "BS", selected: matches!(edit_origin.read().as_ref(), Some(Country::BS)), "Bahamas" }
                                        option { value: "BT", selected: matches!(edit_origin.read().as_ref(), Some(Country::BT)), "Bhutan" }
                                        option { value: "BV", selected: matches!(edit_origin.read().as_ref(), Some(Country::BV)), "Bouvetinsel" }
                                        option { value: "BW", selected: matches!(edit_origin.read().as_ref(), Some(Country::BW)), "Botswana" }
                                        option { value: "BY", selected: matches!(edit_origin.read().as_ref(), Some(Country::BY)), "Belarus" }
                                        option { value: "BZ", selected: matches!(edit_origin.read().as_ref(), Some(Country::BZ)), "Belize" }
                                        option { value: "CA", selected: matches!(edit_origin.read().as_ref(), Some(Country::CA)), "Kanada" }
                                        option { value: "CC", selected: matches!(edit_origin.read().as_ref(), Some(Country::CC)), "Kokosinseln" }
                                        option { value: "CD", selected: matches!(edit_origin.read().as_ref(), Some(Country::CD)), "Demokratische Republik Kongo" }
                                        option { value: "CF", selected: matches!(edit_origin.read().as_ref(), Some(Country::CF)), "Zentralafrikanische Republik" }
                                        option { value: "CG", selected: matches!(edit_origin.read().as_ref(), Some(Country::CG)), "Republik Kongo" }
                                        option { value: "CI", selected: matches!(edit_origin.read().as_ref(), Some(Country::CI)), "Elfenbeinküste" }
                                        option { value: "CK", selected: matches!(edit_origin.read().as_ref(), Some(Country::CK)), "Cookinseln" }
                                        option { value: "CL", selected: matches!(edit_origin.read().as_ref(), Some(Country::CL)), "Chile" }
                                        option { value: "CM", selected: matches!(edit_origin.read().as_ref(), Some(Country::CM)), "Kamerun" }
                                        option { value: "CN", selected: matches!(edit_origin.read().as_ref(), Some(Country::CN)), "China" }
                                        option { value: "CO", selected: matches!(edit_origin.read().as_ref(), Some(Country::CO)), "Kolumbien" }
                                        option { value: "CR", selected: matches!(edit_origin.read().as_ref(), Some(Country::CR)), "Costa Rica" }
                                        option { value: "CU", selected: matches!(edit_origin.read().as_ref(), Some(Country::CU)), "Kuba" }
                                        option { value: "CV", selected: matches!(edit_origin.read().as_ref(), Some(Country::CV)), "Kap Verde" }
                                        option { value: "CW", selected: matches!(edit_origin.read().as_ref(), Some(Country::CW)), "Curaçao" }
                                        option { value: "CX", selected: matches!(edit_origin.read().as_ref(), Some(Country::CX)), "Weihnachtsinsel" }
                                        option { value: "CY", selected: matches!(edit_origin.read().as_ref(), Some(Country::CY)), "Zypern" }
                                        option { value: "CZ", selected: matches!(edit_origin.read().as_ref(), Some(Country::CZ)), "Tschechien" }
                                        option { value: "DE", selected: matches!(edit_origin.read().as_ref(), Some(Country::DE)), "Deutschland" }
                                        option { value: "DJ", selected: matches!(edit_origin.read().as_ref(), Some(Country::DJ)), "Dschibuti" }
                                        option { value: "DK", selected: matches!(edit_origin.read().as_ref(), Some(Country::DK)), "Dänemark" }
                                        option { value: "DM", selected: matches!(edit_origin.read().as_ref(), Some(Country::DM)), "Dominica" }
                                        option { value: "DO", selected: matches!(edit_origin.read().as_ref(), Some(Country::DO)), "Dominikanische Republik" }
                                        option { value: "DZ", selected: matches!(edit_origin.read().as_ref(), Some(Country::DZ)), "Algerien" }
                                        option { value: "EC", selected: matches!(edit_origin.read().as_ref(), Some(Country::EC)), "Ecuador" }
                                        option { value: "EE", selected: matches!(edit_origin.read().as_ref(), Some(Country::EE)), "Estland" }
                                        option { value: "EG", selected: matches!(edit_origin.read().as_ref(), Some(Country::EG)), "Ägypten" }
                                        option { value: "EH", selected: matches!(edit_origin.read().as_ref(), Some(Country::EH)), "Westsahara" }
                                        option { value: "ER", selected: matches!(edit_origin.read().as_ref(), Some(Country::ER)), "Eritrea" }
                                        option { value: "ES", selected: matches!(edit_origin.read().as_ref(), Some(Country::ES)), "Spanien" }
                                        option { value: "ET", selected: matches!(edit_origin.read().as_ref(), Some(Country::ET)), "Äthiopien" }
                                        option { value: "FI", selected: matches!(edit_origin.read().as_ref(), Some(Country::FI)), "Finnland" }
                                        option { value: "FJ", selected: matches!(edit_origin.read().as_ref(), Some(Country::FJ)), "Fidschi" }
                                        option { value: "FK", selected: matches!(edit_origin.read().as_ref(), Some(Country::FK)), "Falklandinseln" }
                                        option { value: "FM", selected: matches!(edit_origin.read().as_ref(), Some(Country::FM)), "Mikronesien" }
                                        option { value: "FO", selected: matches!(edit_origin.read().as_ref(), Some(Country::FO)), "Färöer" }
                                        option { value: "FR", selected: matches!(edit_origin.read().as_ref(), Some(Country::FR)), "Frankreich" }
                                        option { value: "GA", selected: matches!(edit_origin.read().as_ref(), Some(Country::GA)), "Gabun" }
                                        option { value: "GB", selected: matches!(edit_origin.read().as_ref(), Some(Country::GB)), "Vereinigtes Königreich" }
                                        option { value: "GD", selected: matches!(edit_origin.read().as_ref(), Some(Country::GD)), "Grenada" }
                                        option { value: "GE", selected: matches!(edit_origin.read().as_ref(), Some(Country::GE)), "Georgien" }
                                        option { value: "GF", selected: matches!(edit_origin.read().as_ref(), Some(Country::GF)), "Französisch-Guayana" }
                                        option { value: "GG", selected: matches!(edit_origin.read().as_ref(), Some(Country::GG)), "Guernsey" }
                                        option { value: "GH", selected: matches!(edit_origin.read().as_ref(), Some(Country::GH)), "Ghana" }
                                        option { value: "GI", selected: matches!(edit_origin.read().as_ref(), Some(Country::GI)), "Gibraltar" }
                                        option { value: "GL", selected: matches!(edit_origin.read().as_ref(), Some(Country::GL)), "Grönland" }
                                        option { value: "GM", selected: matches!(edit_origin.read().as_ref(), Some(Country::GM)), "Gambia" }
                                        option { value: "GN", selected: matches!(edit_origin.read().as_ref(), Some(Country::GN)), "Guinea" }
                                        option { value: "GP", selected: matches!(edit_origin.read().as_ref(), Some(Country::GP)), "Guadeloupe" }
                                        option { value: "GQ", selected: matches!(edit_origin.read().as_ref(), Some(Country::GQ)), "Äquatorialguinea" }
                                        option { value: "GR", selected: matches!(edit_origin.read().as_ref(), Some(Country::GR)), "Griechenland" }
                                        option { value: "GS", selected: matches!(edit_origin.read().as_ref(), Some(Country::GS)), "Südgeorgien und die Südlichen Sandwichinseln" }
                                        option { value: "GT", selected: matches!(edit_origin.read().as_ref(), Some(Country::GT)), "Guatemala" }
                                        option { value: "GU", selected: matches!(edit_origin.read().as_ref(), Some(Country::GU)), "Guam" }
                                        option { value: "GW", selected: matches!(edit_origin.read().as_ref(), Some(Country::GW)), "Guinea-Bissau" }
                                        option { value: "GY", selected: matches!(edit_origin.read().as_ref(), Some(Country::GY)), "Guyana" }
                                        option { value: "HK", selected: matches!(edit_origin.read().as_ref(), Some(Country::HK)), "Hongkong" }
                                        option { value: "HM", selected: matches!(edit_origin.read().as_ref(), Some(Country::HM)), "Heard und McDonaldinseln" }
                                        option { value: "HN", selected: matches!(edit_origin.read().as_ref(), Some(Country::HN)), "Honduras" }
                                        option { value: "HR", selected: matches!(edit_origin.read().as_ref(), Some(Country::HR)), "Kroatien" }
                                        option { value: "HT", selected: matches!(edit_origin.read().as_ref(), Some(Country::HT)), "Haiti" }
                                        option { value: "HU", selected: matches!(edit_origin.read().as_ref(), Some(Country::HU)), "Ungarn" }
                                        option { value: "ID", selected: matches!(edit_origin.read().as_ref(), Some(Country::ID)), "Indonesien" }
                                        option { value: "IE", selected: matches!(edit_origin.read().as_ref(), Some(Country::IE)), "Irland" }
                                        option { value: "IL", selected: matches!(edit_origin.read().as_ref(), Some(Country::IL)), "Israel" }
                                        option { value: "IM", selected: matches!(edit_origin.read().as_ref(), Some(Country::IM)), "Isle of Man" }
                                        option { value: "IN", selected: matches!(edit_origin.read().as_ref(), Some(Country::IN)), "Indien" }
                                        option { value: "IO", selected: matches!(edit_origin.read().as_ref(), Some(Country::IO)), "Britisches Territorium im Indischen Ozean" }
                                        option { value: "IQ", selected: matches!(edit_origin.read().as_ref(), Some(Country::IQ)), "Irak" }
                                        option { value: "IR", selected: matches!(edit_origin.read().as_ref(), Some(Country::IR)), "Iran" }
                                        option { value: "IS", selected: matches!(edit_origin.read().as_ref(), Some(Country::IS)), "Island" }
                                        option { value: "IT", selected: matches!(edit_origin.read().as_ref(), Some(Country::IT)), "Italien" }
                                        option { value: "JE", selected: matches!(edit_origin.read().as_ref(), Some(Country::JE)), "Jersey" }
                                        option { value: "JM", selected: matches!(edit_origin.read().as_ref(), Some(Country::JM)), "Jamaika" }
                                        option { value: "JO", selected: matches!(edit_origin.read().as_ref(), Some(Country::JO)), "Jordanien" }
                                        option { value: "JP", selected: matches!(edit_origin.read().as_ref(), Some(Country::JP)), "Japan" }
                                        option { value: "KE", selected: matches!(edit_origin.read().as_ref(), Some(Country::KE)), "Kenia" }
                                        option { value: "KG", selected: matches!(edit_origin.read().as_ref(), Some(Country::KG)), "Kirgisistan" }
                                        option { value: "KH", selected: matches!(edit_origin.read().as_ref(), Some(Country::KH)), "Kambodscha" }
                                        option { value: "KI", selected: matches!(edit_origin.read().as_ref(), Some(Country::KI)), "Kiribati" }
                                        option { value: "KM", selected: matches!(edit_origin.read().as_ref(), Some(Country::KM)), "Komoren" }
                                        option { value: "KN", selected: matches!(edit_origin.read().as_ref(), Some(Country::KN)), "St. Kitts und Nevis" }
                                        option { value: "KP", selected: matches!(edit_origin.read().as_ref(), Some(Country::KP)), "Nordkorea" }
                                        option { value: "KR", selected: matches!(edit_origin.read().as_ref(), Some(Country::KR)), "Südkorea" }
                                        option { value: "KW", selected: matches!(edit_origin.read().as_ref(), Some(Country::KW)), "Kuwait" }
                                        option { value: "KY", selected: matches!(edit_origin.read().as_ref(), Some(Country::KY)), "Kaimaninseln" }
                                        option { value: "KZ", selected: matches!(edit_origin.read().as_ref(), Some(Country::KZ)), "Kasachstan" }
                                        option { value: "LA", selected: matches!(edit_origin.read().as_ref(), Some(Country::LA)), "Laos" }
                                        option { value: "LB", selected: matches!(edit_origin.read().as_ref(), Some(Country::LB)), "Libanon" }
                                        option { value: "LC", selected: matches!(edit_origin.read().as_ref(), Some(Country::LC)), "St. Lucia" }
                                        option { value: "LI", selected: matches!(edit_origin.read().as_ref(), Some(Country::LI)), "Liechtenstein" }
                                        option { value: "LK", selected: matches!(edit_origin.read().as_ref(), Some(Country::LK)), "Sri Lanka" }
                                        option { value: "LR", selected: matches!(edit_origin.read().as_ref(), Some(Country::LR)), "Liberia" }
                                        option { value: "LS", selected: matches!(edit_origin.read().as_ref(), Some(Country::LS)), "Lesotho" }
                                        option { value: "LT", selected: matches!(edit_origin.read().as_ref(), Some(Country::LT)), "Litauen" }
                                        option { value: "LU", selected: matches!(edit_origin.read().as_ref(), Some(Country::LU)), "Luxemburg" }
                                        option { value: "LV", selected: matches!(edit_origin.read().as_ref(), Some(Country::LV)), "Lettland" }
                                        option { value: "LY", selected: matches!(edit_origin.read().as_ref(), Some(Country::LY)), "Libyen" }
                                        option { value: "MA", selected: matches!(edit_origin.read().as_ref(), Some(Country::MA)), "Marokko" }
                                        option { value: "MC", selected: matches!(edit_origin.read().as_ref(), Some(Country::MC)), "Monaco" }
                                        option { value: "MD", selected: matches!(edit_origin.read().as_ref(), Some(Country::MD)), "Moldau" }
                                        option { value: "ME", selected: matches!(edit_origin.read().as_ref(), Some(Country::ME)), "Montenegro" }
                                        option { value: "MF", selected: matches!(edit_origin.read().as_ref(), Some(Country::MF)), "Saint-Martin" }
                                        option { value: "MG", selected: matches!(edit_origin.read().as_ref(), Some(Country::MG)), "Madagaskar" }
                                        option { value: "MH", selected: matches!(edit_origin.read().as_ref(), Some(Country::MH)), "Marshallinseln" }
                                        option { value: "MK", selected: matches!(edit_origin.read().as_ref(), Some(Country::MK)), "Nordmazedonien" }
                                        option { value: "ML", selected: matches!(edit_origin.read().as_ref(), Some(Country::ML)), "Mali" }
                                        option { value: "MM", selected: matches!(edit_origin.read().as_ref(), Some(Country::MM)), "Myanmar" }
                                        option { value: "MN", selected: matches!(edit_origin.read().as_ref(), Some(Country::MN)), "Mongolei" }
                                        option { value: "MO", selected: matches!(edit_origin.read().as_ref(), Some(Country::MO)), "Macau" }
                                        option { value: "MP", selected: matches!(edit_origin.read().as_ref(), Some(Country::MP)), "Nördliche Marianen" }
                                        option { value: "MQ", selected: matches!(edit_origin.read().as_ref(), Some(Country::MQ)), "Martinique" }
                                        option { value: "MR", selected: matches!(edit_origin.read().as_ref(), Some(Country::MR)), "Mauretanien" }
                                        option { value: "MS", selected: matches!(edit_origin.read().as_ref(), Some(Country::MS)), "Montserrat" }
                                        option { value: "MT", selected: matches!(edit_origin.read().as_ref(), Some(Country::MT)), "Malta" }
                                        option { value: "MU", selected: matches!(edit_origin.read().as_ref(), Some(Country::MU)), "Mauritius" }
                                        option { value: "MV", selected: matches!(edit_origin.read().as_ref(), Some(Country::MV)), "Malediven" }
                                        option { value: "MW", selected: matches!(edit_origin.read().as_ref(), Some(Country::MW)), "Malawi" }
                                        option { value: "MX", selected: matches!(edit_origin.read().as_ref(), Some(Country::MX)), "Mexiko" }
                                        option { value: "MY", selected: matches!(edit_origin.read().as_ref(), Some(Country::MY)), "Malaysia" }
                                        option { value: "MZ", selected: matches!(edit_origin.read().as_ref(), Some(Country::MZ)), "Mosambik" }
                                        option { value: "NA", selected: matches!(edit_origin.read().as_ref(), Some(Country::NA)), "Namibia" }
                                        option { value: "NC", selected: matches!(edit_origin.read().as_ref(), Some(Country::NC)), "Neukaledonien" }
                                        option { value: "NE", selected: matches!(edit_origin.read().as_ref(), Some(Country::NE)), "Niger" }
                                        option { value: "NF", selected: matches!(edit_origin.read().as_ref(), Some(Country::NF)), "Norfolkinsel" }
                                        option { value: "NG", selected: matches!(edit_origin.read().as_ref(), Some(Country::NG)), "Nigeria" }
                                        option { value: "NI", selected: matches!(edit_origin.read().as_ref(), Some(Country::NI)), "Nicaragua" }
                                        option { value: "NL", selected: matches!(edit_origin.read().as_ref(), Some(Country::NL)), "Niederlande" }
                                        option { value: "NO", selected: matches!(edit_origin.read().as_ref(), Some(Country::NO)), "Norwegen" }
                                        option { value: "NP", selected: matches!(edit_origin.read().as_ref(), Some(Country::NP)), "Nepal" }
                                        option { value: "NR", selected: matches!(edit_origin.read().as_ref(), Some(Country::NR)), "Nauru" }
                                        option { value: "NU", selected: matches!(edit_origin.read().as_ref(), Some(Country::NU)), "Niue" }
                                        option { value: "NZ", selected: matches!(edit_origin.read().as_ref(), Some(Country::NZ)), "Neuseeland" }
                                        option { value: "OM", selected: matches!(edit_origin.read().as_ref(), Some(Country::OM)), "Oman" }
                                        option { value: "PA", selected: matches!(edit_origin.read().as_ref(), Some(Country::PA)), "Panama" }
                                        option { value: "PE", selected: matches!(edit_origin.read().as_ref(), Some(Country::PE)), "Peru" }
                                        option { value: "PF", selected: matches!(edit_origin.read().as_ref(), Some(Country::PF)), "Französisch-Polynesien" }
                                        option { value: "PG", selected: matches!(edit_origin.read().as_ref(), Some(Country::PG)), "Papua-Neuguinea" }
                                        option { value: "PH", selected: matches!(edit_origin.read().as_ref(), Some(Country::PH)), "Philippinen" }
                                        option { value: "PK", selected: matches!(edit_origin.read().as_ref(), Some(Country::PK)), "Pakistan" }
                                        option { value: "PL", selected: matches!(edit_origin.read().as_ref(), Some(Country::PL)), "Polen" }
                                        option { value: "PM", selected: matches!(edit_origin.read().as_ref(), Some(Country::PM)), "Saint-Pierre und Miquelon" }
                                        option { value: "PN", selected: matches!(edit_origin.read().as_ref(), Some(Country::PN)), "Pitcairninseln" }
                                        option { value: "PR", selected: matches!(edit_origin.read().as_ref(), Some(Country::PR)), "Puerto Rico" }
                                        option { value: "PS", selected: matches!(edit_origin.read().as_ref(), Some(Country::PS)), "Palästina" }
                                        option { value: "PT", selected: matches!(edit_origin.read().as_ref(), Some(Country::PT)), "Portugal" }
                                        option { value: "PW", selected: matches!(edit_origin.read().as_ref(), Some(Country::PW)), "Palau" }
                                        option { value: "PY", selected: matches!(edit_origin.read().as_ref(), Some(Country::PY)), "Paraguay" }
                                        option { value: "QA", selected: matches!(edit_origin.read().as_ref(), Some(Country::QA)), "Katar" }
                                        option { value: "RE", selected: matches!(edit_origin.read().as_ref(), Some(Country::RE)), "Réunion" }
                                        option { value: "RO", selected: matches!(edit_origin.read().as_ref(), Some(Country::RO)), "Rumänien" }
                                        option { value: "RS", selected: matches!(edit_origin.read().as_ref(), Some(Country::RS)), "Serbien" }
                                        option { value: "RU", selected: matches!(edit_origin.read().as_ref(), Some(Country::RU)), "Russland" }
                                        option { value: "RW", selected: matches!(edit_origin.read().as_ref(), Some(Country::RW)), "Ruanda" }
                                        option { value: "SA", selected: matches!(edit_origin.read().as_ref(), Some(Country::SA)), "Saudi-Arabien" }
                                        option { value: "SB", selected: matches!(edit_origin.read().as_ref(), Some(Country::SB)), "Salomonen" }
                                        option { value: "SC", selected: matches!(edit_origin.read().as_ref(), Some(Country::SC)), "Seychellen" }
                                        option { value: "SD", selected: matches!(edit_origin.read().as_ref(), Some(Country::SD)), "Sudan" }
                                        option { value: "SE", selected: matches!(edit_origin.read().as_ref(), Some(Country::SE)), "Schweden" }
                                        option { value: "SG", selected: matches!(edit_origin.read().as_ref(), Some(Country::SG)), "Singapur" }
                                        option { value: "SH", selected: matches!(edit_origin.read().as_ref(), Some(Country::SH)), "St. Helena" }
                                        option { value: "SI", selected: matches!(edit_origin.read().as_ref(), Some(Country::SI)), "Slowenien" }
                                        option { value: "SJ", selected: matches!(edit_origin.read().as_ref(), Some(Country::SJ)), "Svalbard und Jan Mayen" }
                                        option { value: "SK", selected: matches!(edit_origin.read().as_ref(), Some(Country::SK)), "Slowakei" }
                                        option { value: "SL", selected: matches!(edit_origin.read().as_ref(), Some(Country::SL)), "Sierra Leone" }
                                        option { value: "SM", selected: matches!(edit_origin.read().as_ref(), Some(Country::SM)), "San Marino" }
                                        option { value: "SN", selected: matches!(edit_origin.read().as_ref(), Some(Country::SN)), "Senegal" }
                                        option { value: "SO", selected: matches!(edit_origin.read().as_ref(), Some(Country::SO)), "Somalia" }
                                        option { value: "SR", selected: matches!(edit_origin.read().as_ref(), Some(Country::SR)), "Suriname" }
                                        option { value: "SS", selected: matches!(edit_origin.read().as_ref(), Some(Country::SS)), "Südsudan" }
                                        option { value: "ST", selected: matches!(edit_origin.read().as_ref(), Some(Country::ST)), "São Tomé und Príncipe" }
                                        option { value: "SV", selected: matches!(edit_origin.read().as_ref(), Some(Country::SV)), "El Salvador" }
                                        option { value: "SX", selected: matches!(edit_origin.read().as_ref(), Some(Country::SX)), "Sint Maarten" }
                                        option { value: "SY", selected: matches!(edit_origin.read().as_ref(), Some(Country::SY)), "Syrien" }
                                        option { value: "SZ", selected: matches!(edit_origin.read().as_ref(), Some(Country::SZ)), "Eswatini" }
                                        option { value: "TC", selected: matches!(edit_origin.read().as_ref(), Some(Country::TC)), "Turks- und Caicosinseln" }
                                        option { value: "TD", selected: matches!(edit_origin.read().as_ref(), Some(Country::TD)), "Tschad" }
                                        option { value: "TF", selected: matches!(edit_origin.read().as_ref(), Some(Country::TF)), "Französische Süd- und Antarktisgebiete" }
                                        option { value: "TG", selected: matches!(edit_origin.read().as_ref(), Some(Country::TG)), "Togo" }
                                        option { value: "TH", selected: matches!(edit_origin.read().as_ref(), Some(Country::TH)), "Thailand" }
                                        option { value: "TJ", selected: matches!(edit_origin.read().as_ref(), Some(Country::TJ)), "Tadschikistan" }
                                        option { value: "TK", selected: matches!(edit_origin.read().as_ref(), Some(Country::TK)), "Tokelau" }
                                        option { value: "TL", selected: matches!(edit_origin.read().as_ref(), Some(Country::TL)), "Osttimor" }
                                        option { value: "TM", selected: matches!(edit_origin.read().as_ref(), Some(Country::TM)), "Turkmenistan" }
                                        option { value: "TN", selected: matches!(edit_origin.read().as_ref(), Some(Country::TN)), "Tunesien" }
                                        option { value: "TO", selected: matches!(edit_origin.read().as_ref(), Some(Country::TO)), "Tonga" }
                                        option { value: "TR", selected: matches!(edit_origin.read().as_ref(), Some(Country::TR)), "Türkei" }
                                        option { value: "TT", selected: matches!(edit_origin.read().as_ref(), Some(Country::TT)), "Trinidad und Tobago" }
                                        option { value: "TV", selected: matches!(edit_origin.read().as_ref(), Some(Country::TV)), "Tuvalu" }
                                        option { value: "TW", selected: matches!(edit_origin.read().as_ref(), Some(Country::TW)), "Taiwan" }
                                        option { value: "TZ", selected: matches!(edit_origin.read().as_ref(), Some(Country::TZ)), "Tansania" }
                                        option { value: "UA", selected: matches!(edit_origin.read().as_ref(), Some(Country::UA)), "Ukraine" }
                                        option { value: "UG", selected: matches!(edit_origin.read().as_ref(), Some(Country::UG)), "Uganda" }
                                        option { value: "UM", selected: matches!(edit_origin.read().as_ref(), Some(Country::UM)), "Amerikanische Überseeinseln" }
                                        option { value: "US", selected: matches!(edit_origin.read().as_ref(), Some(Country::US)), "Vereinigte Staaten" }
                                        option { value: "UY", selected: matches!(edit_origin.read().as_ref(), Some(Country::UY)), "Uruguay" }
                                        option { value: "UZ", selected: matches!(edit_origin.read().as_ref(), Some(Country::UZ)), "Usbekistan" }
                                        option { value: "VA", selected: matches!(edit_origin.read().as_ref(), Some(Country::VA)), "Vatikanstadt" }
                                        option { value: "VC", selected: matches!(edit_origin.read().as_ref(), Some(Country::VC)), "St. Vincent und die Grenadinen" }
                                        option { value: "VE", selected: matches!(edit_origin.read().as_ref(), Some(Country::VE)), "Venezuela" }
                                        option { value: "VG", selected: matches!(edit_origin.read().as_ref(), Some(Country::VG)), "Britische Jungferninseln" }
                                        option { value: "VI", selected: matches!(edit_origin.read().as_ref(), Some(Country::VI)), "Amerikanische Jungferninseln" }
                                        option { value: "VN", selected: matches!(edit_origin.read().as_ref(), Some(Country::VN)), "Vietnam" }
                                        option { value: "VU", selected: matches!(edit_origin.read().as_ref(), Some(Country::VU)), "Vanuatu" }
                                        option { value: "WF", selected: matches!(edit_origin.read().as_ref(), Some(Country::WF)), "Wallis und Futuna" }
                                        option { value: "WS", selected: matches!(edit_origin.read().as_ref(), Some(Country::WS)), "Samoa" }
                                        option { value: "YE", selected: matches!(edit_origin.read().as_ref(), Some(Country::YE)), "Jemen" }
                                        option { value: "YT", selected: matches!(edit_origin.read().as_ref(), Some(Country::YT)), "Mayotte" }
                                        option { value: "ZA", selected: matches!(edit_origin.read().as_ref(), Some(Country::ZA)), "Südafrika" }
                                        option { value: "ZM", selected: matches!(edit_origin.read().as_ref(), Some(Country::ZM)), "Sambia" }
                                        option { value: "ZW", selected: matches!(edit_origin.read().as_ref(), Some(Country::ZW)), "Simbabwe" }
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {}
                    }
                }

                {
                    // Check if AP7_4 rule is active and ingredient is beef to show beef specific fields
                    let should_show_beef_fields = use_memo(move || {
                        let rules = props.rules.read();
                        let has_beef_rule = rules.iter().any(|rule| *rule == RuleDef::AP7_4_RindfleischHerkunftDetails);

                        if has_beef_rule {
                            if let Some(category) = &edit_category() {
                                crate::core::is_beef_category(category)
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
                                label: "Aufzucht",
                                help: Some("Ort, wo es die meiste Zeit gelebt hat".into()),
                                ValidationDisplay {
                                    paths: vec![
                                        format!("ingredients[{}][aufzucht_ort]", index)
                                    ],
                                    select {
                                        class: "select select-bordered w-full",
                                        value: match edit_aufzucht_ort.read().as_ref() {
                                            Some(country) => format!("{:?}", country),
                                            None => "".to_string(),
                                        },
                                        onchange: move |e| {
                                            let country = match e.value().as_str() {
                                                "CH" => Country::CH,
                                                "EU" => Country::EU,
                                                "DE" => Country::DE,
                                                "FR" => Country::FR,
                                                "IT" => Country::IT,
                                                "AT" => Country::AT,
                                                _ => return,
                                            };
                                            edit_aufzucht_ort.set(Some(country));
                                        },
                                        option { value: "", selected: edit_aufzucht_ort.read().is_none(), "Bitte wählen..." }
                                        option { value: "CH", selected: matches!(edit_aufzucht_ort.read().as_ref(), Some(Country::CH)), "Schweiz" }
                                        option { value: "EU", selected: matches!(edit_aufzucht_ort.read().as_ref(), Some(Country::EU)), "EU" }
                                        option { value: "DE", selected: matches!(edit_aufzucht_ort.read().as_ref(), Some(Country::DE)), "Deutschland" }
                                        option { value: "FR", selected: matches!(edit_aufzucht_ort.read().as_ref(), Some(Country::FR)), "Frankreich" }
                                        option { value: "IT", selected: matches!(edit_aufzucht_ort.read().as_ref(), Some(Country::IT)), "Italien" }
                                        option { value: "AT", selected: matches!(edit_aufzucht_ort.read().as_ref(), Some(Country::AT)), "Österreich" }
                                    }
                                }
                            }
                            FormField {
                                label: "Schlachtung",
                                ValidationDisplay {
                                    paths: vec![
                                        format!("ingredients[{}][schlachtungs_ort]", index)
                                    ],
                                    select {
                                        class: "select select-bordered w-full",
                                        value: match edit_schlachtungs_ort.read().as_ref() {
                                            Some(country) => format!("{:?}", country),
                                            None => "".to_string(),
                                        },
                                        onchange: move |e| {
                                            let country = match e.value().as_str() {
                                                "CH" => Country::CH,
                                                "EU" => Country::EU,
                                                "DE" => Country::DE,
                                                "FR" => Country::FR,
                                                "IT" => Country::IT,
                                                "AT" => Country::AT,
                                                _ => return,
                                            };
                                            edit_schlachtungs_ort.set(Some(country));
                                        },
                                        option { value: "", selected: edit_schlachtungs_ort.read().is_none(), "Bitte wählen..." }
                                        option { value: "CH", selected: matches!(edit_schlachtungs_ort.read().as_ref(), Some(Country::CH)), "Schweiz" }
                                        option { value: "EU", selected: matches!(edit_schlachtungs_ort.read().as_ref(), Some(Country::EU)), "EU" }
                                        option { value: "DE", selected: matches!(edit_schlachtungs_ort.read().as_ref(), Some(Country::DE)), "Deutschland" }
                                        option { value: "FR", selected: matches!(edit_schlachtungs_ort.read().as_ref(), Some(Country::FR)), "Frankreich" }
                                        option { value: "IT", selected: matches!(edit_schlachtungs_ort.read().as_ref(), Some(Country::IT)), "Italien" }
                                        option { value: "AT", selected: matches!(edit_schlachtungs_ort.read().as_ref(), Some(Country::AT)), "Österreich" }
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {}
                    }
                }

                {
                    // Check if AP7_5 rule is active and ingredient is fish to show fish specific field
                    let should_show_fish_field = use_memo(move || {
                        let rules = props.rules.read();
                        let has_fish_rule = rules.iter().any(|rule| *rule == RuleDef::AP7_5_FischFangort);

                        if has_fish_rule {
                            if let Some(category) = &edit_category() {
                                crate::core::is_fish_category(category)
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
                                label: "Fangort",
                                ValidationDisplay {
                                    paths: vec![
                                        format!("ingredients[{}][fangort]", index)
                                    ],
                                    select {
                                        class: "select select-bordered w-full",
                                        value: match edit_fangort.read().as_ref() {
                                            Some(country) => format!("{:?}", country),
                                            None => "".to_string(),
                                        },
                                        onchange: move |e| {
                                            let country = match e.value().as_str() {
                                                "CH" => Country::CH,
                                                "EU" => Country::EU,
                                                "DE" => Country::DE,
                                                "FR" => Country::FR,
                                                "IT" => Country::IT,
                                                "AT" => Country::AT,
                                                _ => return,
                                            };
                                            edit_fangort.set(Some(country));
                                        },
                                        option { value: "", selected: edit_fangort.read().is_none(), "Bitte wählen..." }
                                        option { value: "CH", selected: matches!(edit_fangort.read().as_ref(), Some(Country::CH)), "Schweiz" }
                                        option { value: "EU", selected: matches!(edit_fangort.read().as_ref(), Some(Country::EU)), "EU" }
                                        option { value: "DE", selected: matches!(edit_fangort.read().as_ref(), Some(Country::DE)), "Deutschland" }
                                        option { value: "FR", selected: matches!(edit_fangort.read().as_ref(), Some(Country::FR)), "Frankreich" }
                                        option { value: "IT", selected: matches!(edit_fangort.read().as_ref(), Some(Country::IT)), "Italien" }
                                        option { value: "AT", selected: matches!(edit_fangort.read().as_ref(), Some(Country::AT)), "Österreich" }
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
                            edit_bio_nicht_knospe.set(orig.bio_nicht_knospe.unwrap_or(false));
                        is_fetching_category.set(false);
                            is_open.set(false);
                        },
                        "× " {t!("nav.schliessen")},
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
                            title: format!("Die Mengenanpassung (×{:.2}) auf das gesamte Rezept übertragen", scaling_factor()),
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
                        orig.sub_components.as_ref().map_or(false, |s| !s.is_empty())
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
