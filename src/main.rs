use dioxus::prelude::*;
use std::collections::HashMap;
use std::ops::Deref;
use serde::{Deserialize, Serialize};
use serde_qs::to_string as to_query_string;
use serde_qs::from_str as from_query_string;

fn main() {
    launch(app);
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
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
        &map.insert("username".to_string(), FormValue(vec![self.username.clone()]));
        &map.insert("password".to_string(), FormValue(vec![self.password.clone()]));
        map
    }

    fn from_initial_qs() -> AppState {
        let window = web_sys::window().unwrap();
        let query_string = window.location().search().unwrap_or_else(|_| "?".to_string());
        let query_string = &query_string[1..]; // Skip the '?' at the start
        from_query_string(query_string).unwrap_or_default()
    }
}

impl Default for AppState {
    fn default() -> Self {
        AppState {username: "admin".to_string(), password: "admin".to_string()}
    }
}

fn app() -> Element {
    let mut initial_app_state = AppState::from_initial_qs();
    let mut form_values = use_signal(|| initial_app_state.to_form_values());
    let app_state = use_memo(
        move || AppState::from_form_values(&form_values.read())
    );
    let app_state_qs = use_memo(
        move || to_query_string(app_state.read().deref()).unwrap()
    );

    // let mut should_update_qs = use_signal(|| false);
    //
    // let app_state_qs = use_resource(move || async move {
    //     if *should_update_qs.read() {
    //         let window = web_sys::window().unwrap();
    //
    //         let qs = to_query_string(app_state.read().deref()).unwrap();
    //         if format!("?{}", qs) != window.location().search().unwrap() {
    //             window.location().set_search(&qs);
    //         }
    //         should_update_qs.set(false);
    //     }
    // });

    rsx! {
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
        pre {"{app_state_qs}"}
    }
}