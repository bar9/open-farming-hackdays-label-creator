use crate::api::search_food;
use crate::components::*;
use crate::core::Ingredient;
use crate::model::{food_db, lookup_allergen};
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
            category: original_ingredient.category.clone(),
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
                category: edit_category(),
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
                category: edit_category(),
            };
            
            match save_composite_ingredient(&ingredient_to_save) {
                Ok(_) => {
                    save_status.set(Some(format!("'{}' wurde erfolgreich gespeichert", edit_name())));
                    // Clear status after 2 seconds
                    let mut save_status_clone = save_status.clone();
                    spawn(async move {
                        gloo::timers::future::TimeoutFuture::new(2000).await;
                        save_status_clone.set(None);
                    });
                }
                Err(e) => {
                    save_status.set(Some(format!("Error: {}", e)));
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
            category: edit_category(),
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
                        is_fetching_category.set(false);
                    }
                    is_open.toggle();
                },
                "{t!(\"nav.hinzufuegen\")}"
            }
        } else {
            button {
                class: "btn join-item btn-outline",
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
                        is_fetching_category.set(false);
                    }
                    is_open.toggle();
                },
                icons::ListDetail {}
            }
        }
        dialog { open: "{is_open}", class: "modal",
            div { class: "modal-box bg-base-100",
                h3 { class: "font-bold text-lg", "{t!(\"label.zutatDetails\")}" }
                FormField {
                    label: t!("label.zutat"),
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
                                    label: "(Gespeichert)"
                                }
                            }
                            // Then add database ingredients
                            for item in food_db().clone() {
                                option { 
                                    value: "{item.0}",
                                    // Show allergen marker in label without duplicating the name
                                    label: if item.1 { "(Allergen)" } else { "" }
                                }
                            }
                        }
                    }
                    // Show category status - either loading, fetched, or empty
                    if is_fetching_category() {
                        div { class: "text-sm text-info mt-1",
                            "Kategorie wird geladen..."
                        }
                    } else if let Some(category) = &edit_category() {
                        div { class: "text-sm text-success mt-1",
                            "Kategorie: {category}"
                        }
                    }
                }
                FormField {
                    label: format!("{} (g)", t!("label.menge")),
                    ValidationDisplay {
                        paths: vec![
                            format!("ingredients[{}][amount]", index)
                        ],
                        input {
                            r#type: "number",
                            placeholder: "Menge in Gramm",
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
                                "Faktor: ×{scaling_factor():.2} (vorher: {original_ingredient.amount}g → neu: {amt}g)"
                            } else {
                                "Bitte Menge eingeben"
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
                    path: "namensgebende_zutat",
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
                            title: "Diese zusammengesetzte Zutat für spätere Verwendung speichern",
                            "Merken"
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
                            "Speichern und übertragen"
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
                    is_open.set(false);
                },
                button { "" }
            }
        }
    }
}
