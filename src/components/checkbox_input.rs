use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct CheckboxInputProps {
    #[props(into)]
    bound_value: Signal<bool>,
    #[props(default = false)]
    required: bool,
}
#[component]
pub fn CheckboxInput(mut props: CheckboxInputProps) -> Element {
    let mut is_pristine = use_signal(|| true);
    let invalid_class = use_memo(move || {
        if is_pristine() {
            ""
        } else {
            "invalid:bg-red-50"
        }
    });
    rsx! {
        input {
            class: "toggle bg-base-200 {invalid_class}",
            r#type: "checkbox",
            required: "{props.required}",
            value: "{props.bound_value}",
            oninput: move |evt| props.bound_value.set(evt.data.checked()),
            onblur: move |_evt| is_pristine.set(false)
        }

    }
}
