use crate::persistence::{get_saved_ingredients, delete_saved_ingredient};
use dioxus::prelude::*;
use rust_i18n::t;

#[component]
pub fn SavedIngredientsManager() -> Element {
    let mut saved_ingredients = use_signal(|| get_saved_ingredients());
    let mut is_open = use_signal(|| false);
    let mut delete_status = use_signal(|| None::<String>);
    
    // Refresh the list when modal opens
    use_effect(move || {
        if is_open() {
            saved_ingredients.set(get_saved_ingredients());
        }
    });
    
    let mut handle_delete = move |name: String| {
        match delete_saved_ingredient(&name) {
            Ok(_) => {
                // Refresh the list
                saved_ingredients.set(get_saved_ingredients());
                delete_status.set(Some(t!("messages.ingredient_deleted", name = name).to_string()));
                
                // Clear status after 2 seconds
                let mut delete_status_clone = delete_status.clone();
                spawn(async move {
                    gloo::timers::future::TimeoutFuture::new(2000).await;
                    delete_status_clone.set(None);
                });
            }
            Err(e) => {
                delete_status.set(Some(t!("messages.error_deleting", error = e).to_string()));
                
                // Clear status after 3 seconds
                let mut delete_status_clone = delete_status.clone();
                spawn(async move {
                    gloo::timers::future::TimeoutFuture::new(3000).await;
                    delete_status_clone.set(None);
                });
            }
        }
    };
    
    rsx! {
        button {
            class: "btn btn-secondary btn-sm",
            onclick: move |_| is_open.toggle(),
            title: t!("tooltips.manage_saved_ingredients").to_string(),
            {t!("nav.saved_ingredients")}
        }
        
        dialog { open: is_open(), class: "modal",
            div { class: "modal-box bg-base-100 max-w-3xl",
                h3 { class: "font-bold text-lg mb-4", 
                    {t!("headers.saved_composite_ingredients")} 
                }
                
                // Show delete status if any
                if let Some(status) = delete_status() {
                    div { class: "alert alert-info mb-4",
                        span { "{status}" }
                    }
                }
                
                if saved_ingredients().is_empty() {
                    div { class: "text-center py-8 text-gray-500",
                        {t!("messages.no_saved_ingredients")}
                    }
                } else {
                    div { class: "overflow-x-auto",
                        table { class: "table table-zebra",
                            thead {
                                tr {
                                    th { {t!("label.zutat")} }
                                    th { {t!("label.components")} }
                                    th { {t!("label.actions")} }
                                }
                            }
                            tbody {
                                for saved in saved_ingredients() {
                                    tr {
                                        td { 
                                            class: if saved.ingredient.is_allergen { "font-bold" } else { "" },
                                            "{saved.ingredient.name}"
                                        }
                                        td {
                                            if let Some(subs) = &saved.ingredient.sub_components {
                                                div { class: "text-sm",
                                                    for sub in subs {
                                                        span {
                                                            class: if sub.is_allergen { "font-bold" } else { "" },
                                                            "{sub.name}"
                                                        }
                                                        if sub != subs.last().unwrap() {
                                                            ", "
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        td {
                                            button {
                                                class: "btn btn-sm btn-error",
                                                onclick: {
                                                    let name = saved.ingredient.name.clone();
                                                    move |_| handle_delete(name.clone())
                                                },
                                                {t!("buttons.delete")}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                div { class: "modal-action",
                    button {
                        class: "btn",
                        onclick: move |_| is_open.set(false),
                        {t!("buttons.close")}
                    }
                }
            }
            
            form {
                method: "dialog",
                class: "modal-backdrop",
                onclick: move |_| is_open.set(false),
                button { "" }
            }
        }
    }
}