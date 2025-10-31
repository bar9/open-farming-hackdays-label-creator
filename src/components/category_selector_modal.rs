use dioxus::prelude::*;
use rust_i18n::t;
use crate::api::FoodItem;

#[component]
pub fn CategorySelectorModal(
    is_open: Signal<bool>,
    food_suggestions: Signal<Vec<FoodItem>>,
    on_select: EventHandler<String>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut filter_text = use_signal(|| String::new());

    // Filter suggestions based on search text
    let filtered_suggestions = use_memo(move || {
        let filter = filter_text().to_lowercase();
        if filter.is_empty() {
            food_suggestions.read().clone()
        } else {
            food_suggestions.read()
                .iter()
                .filter(|item| {
                    item.food_name.to_lowercase().contains(&filter) ||
                    item.category_names.as_ref()
                        .map_or(false, |cat| cat.to_lowercase().contains(&filter))
                })
                .cloned()
                .collect()
        }
    });

    let mut handle_select = move |category: String| {
        on_select.call(category);
        is_open.set(false);
    };

    let handle_cancel = move |_| {
        on_cancel.call(());
        is_open.set(false);
    };

    rsx! {
        dialog {
            open: is_open(),
            class: "modal",
            div { class: "modal-box w-11/12 max-w-5xl",
                div { class: "flex justify-between items-center mb-4",
                    h3 { class: "font-bold text-lg", {t!("category_selector.title")} }
                    button {
                        class: "btn btn-sm btn-circle btn-ghost",
                        onclick: handle_cancel,
                        "✕"
                    }
                }

                // Show count of suggestions
                div { class: "text-sm text-base-content/70 mb-4",
                    {t!("category_selector.suggestions_count", count = food_suggestions.read().len())}
                }

                // Filter input
                div { class: "mb-4",
                    input {
                        r#type: "text",
                        placeholder: t!("category_selector.filter_placeholder").to_string(),
                        class: "input input-bordered w-full",
                        value: filter_text(),
                        oninput: move |evt| {
                            filter_text.set(evt.value());
                        }
                    }
                }

                // Suggestions list
                div { class: "max-h-96 overflow-y-auto",
                    if filtered_suggestions.read().is_empty() {
                        div { class: "text-center py-8 text-base-content/50",
                            {t!("category_selector.no_matches")}
                        }
                    } else {
                        for (_index, item) in filtered_suggestions.read().iter().enumerate() {
                            div {
                                key: "{item.id}",
                                class: "border rounded-lg p-3 mb-2 hover:bg-base-200 cursor-pointer transition-colors",
                                onclick: {
                                    let category = item.category_names.clone().unwrap_or_else(|| t!("category_selector.no_category").to_string());
                                    move |_| handle_select(category.clone())
                                },
                                div { class: "flex justify-between items-start",
                                    div { class: "flex-1",
                                        div { class: "font-medium text-base-content",
                                            {item.food_name.clone()}
                                        }
                                        if let Some(category) = &item.category_names {
                                            div { class: "text-sm text-primary mt-1",
                                                "→ {category}"
                                            }
                                        } else {
                                            div { class: "text-sm text-base-content/50 mt-1",
                                                {t!("category_selector.no_category")}
                                            }
                                        }
                                    }
                                    div { class: "text-xs text-base-content/50 ml-4",
                                        "ID: {item.id}"
                                    }
                                }
                            }
                        }
                    }
                }

                // Modal actions
                div { class: "modal-action",
                    button {
                        class: "btn btn-ghost",
                        onclick: handle_cancel,
                        {t!("category_selector.cancel")}
                    }
                }
            }
        }
    }
}