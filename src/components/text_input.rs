use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct TextInputProps {
    #[props(into)]
    placeholder: String,
    bound_value: Signal<String>,
    #[props(default=false)]
    required: bool
}
#[component]
pub fn TextInput(mut props: TextInputProps) -> Element {
    rsx! {
        input {
            class: "input bg-white input-bordered w-full invalid:border-red-500",
            r#type: "text",
            placeholder: "{props.placeholder}",
            required: "{props.required}",
            value: "{props.bound_value}",
            oninput: move |evt| props.bound_value.set(evt.data.value())
        }
    }
}