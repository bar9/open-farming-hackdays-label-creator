use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct TextareaInputProps {
    #[props(into)]
    placeholder: String,
    bound_value: Signal<String>,
    #[props(into)]
    rows: String,
}
pub fn TextareaInput(mut props: TextareaInputProps) -> Element {
    rsx! {
        textarea {
            class: "textarea bg-white textarea-bordered w-full",
            rows: "{props.rows}",
            placeholder: "{props.placeholder}",
            value: "{props.bound_value}",
            oninput: move |evt| props.bound_value.set(evt.data.value())
        }
    }
}
