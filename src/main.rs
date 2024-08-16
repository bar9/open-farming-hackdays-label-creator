use dioxus::prelude::*;
use std::collections::HashMap;
use std::ops::Deref;
use serde::{Deserialize, Serialize};
use serde_qs::to_string as to_query_string;
use serde_qs::from_str as from_query_string;
use web_sys::window;

fn main() {
    launch(app);
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
struct AppState {
    username: String,
    password: String
}

impl AppState {
    fn from_form_values(form_values: &HashMap<String, FormValue>) -> AppState {
        AppState {
            username: match form_values.get("username") {
                None => "".to_string(),
                Some(form_value) => { form_value.as_value() }
            },
            password: match form_values.get("password") {
                None => "".to_string(),
                Some(form_value) => { form_value.as_value() }
            }
        }
    }
    fn to_form_values(&self) -> HashMap<String, FormValue> {
        let mut map = HashMap::new();
        let _ = &map.insert("username".to_string(), FormValue(vec![self.username.clone()]));
        let _ = &map.insert("password".to_string(), FormValue(vec![self.password.clone()]));
        map
    }

}

impl Default for AppState {
    fn default() -> Self {
        if let Some(window) = web_sys::window() {
            if let Ok(mut query_string) = window.location().search() {
                query_string = query_string.trim_start_matches('?').to_string();
                if let Ok(app_state_from_query_string) = from_query_string::<AppState>(
                    &query_string
                ) {
                    return app_state_from_query_string;
                }
            }
        }
        AppState {username: "admin".to_string(), password: "admin".to_string()}
    }
}

fn app() -> Element {
    let initial_app_state = use_memo(
        move || AppState::default()
    );

    let mut form_values = use_signal(|| initial_app_state().to_form_values());
    let app_state = use_memo(
        move || AppState::from_form_values(&form_values.read())
    );
    let app_state_qs = use_memo(
        move || to_query_string(app_state.read().deref()).unwrap()
    );

    rsx! {
        div {class: "h-screen flex flex-col justify-center items-center gap-3",
            form {
                oninput: move |ev| {
                    form_values.set(ev.values());
                    println!("Input event: {:#?}", ev);
                },
                label { r#for: "username", "Username" }
                input {
                    r#type: "text",
                    name: "username",
                    value: &*app_state.read().username,

                }
                label { r#for: "password", "Password" }
                input {
                    r#type: "text",
                    name: "password",
                    value: &*app_state.read().password,
                }
            }
            pre {"{form_values:#?}"}
            pre {"{app_state:?}"}
            pre {"http://localhost:8080/?{app_state_qs}"}
            button {
                onclick: move |_: MouseEvent| {
                    let window = web_sys::window().expect("no global `window` exists");
                    let navigator = window.navigator();
                    let clipboard = navigator.clipboard();
                    let text = format!("http://localhost:8080/?{app_state_qs}");
                    let  _ = clipboard.write_text(&text);
                },
                "copy to clipboard"
            }

        }
    }
}