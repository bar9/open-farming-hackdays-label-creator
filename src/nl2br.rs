use dioxus::prelude::*;
use std::ops::Deref;

pub trait Nl2Br: Deref<Target = str> {
    fn nl2br(&self) -> Element {
        rsx! {
            for (i, line) in self.lines().enumerate() {
                "{line}"
                if i != self.lines().count() - 1 {
                    br {}
                }
            }
        }
    }
}

// Blanket implementation for all `Deref<Target = str>`
impl<T: Deref<Target = str>> Nl2Br for T {}