use dioxus::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct Validations(pub Memo<HashMap<String, Vec<String>>>);

#[derive(Clone, Copy)]
pub struct Conditionals(pub Memo<HashMap<String, bool>>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Configuration {
    Conventional,
    Bio,
    Knospe,
}
