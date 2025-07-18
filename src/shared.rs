use dioxus::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct Validations(pub Memo<HashMap<String, &'static str>>);

#[derive(Clone, Copy)]
pub struct Conditionals(pub Memo<HashMap<String, bool>>);

#[derive(Clone, Copy)]
pub enum Configuration {
    Conventional,
}
