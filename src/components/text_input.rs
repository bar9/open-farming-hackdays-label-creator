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
    let mut is_pristine = use_signal(|| true);
    let invalid_class = use_memo(move || {if is_pristine() {""} else {"invalid:bg-red-50"}});
    rsx! {
        input {
            class: "input w-full input-ghost bg-base-200 {invalid_class}",
            r#type: "text",
            placeholder: "{props.placeholder}",
            required: "{props.required}",
            value: "{props.bound_value}",
            oninput: move |evt| props.bound_value.set(evt.data.value()),
            onblur: move |_evt| is_pristine.set(false)
        }
        
    }
}