use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct FieldGroup2Props {
    children: Element,
}
pub fn FieldGroup2(props: FieldGroup2Props) -> Element {
    rsx! {
        div {
            class: "grid grid-cols-2 gap-4 ",
            {props.children}
        }
    }
}
