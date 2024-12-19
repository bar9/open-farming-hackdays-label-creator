use dioxus::prelude::*;
use crate::Conditionals;

#[derive(Props, Clone, PartialEq)]
pub struct ConditionalDisplayProps{
    path: &'static str,
    children: Element,
}

pub fn ConditionalDisplay(props: ConditionalDisplayProps) -> Element {

    let conditional_context= use_context::<Conditionals>();
    let conditional_entries= (&*conditional_context.0.read()).clone();
    let do_display = *conditional_entries.get(props.path).unwrap_or(&false);

    // rsx! {
    //     "{conditional_entries:?}"
    // }

    if do_display {
        rsx! {
            {props.children}
        }
    } else {
        rsx! {}
    }
}
