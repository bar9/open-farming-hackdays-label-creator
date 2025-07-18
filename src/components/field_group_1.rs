use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct FieldGroup1Props {
    #[props(into)]
    label: String,
    children: Element,
}
pub fn FieldGroup1(props: FieldGroup1Props) -> Element {
    rsx! {
        div { class: "flex flex-col gap-4",
            h4 { class: "text-xl mb-2", "{props.label}" }
            {props.children}
        }
    }
}
