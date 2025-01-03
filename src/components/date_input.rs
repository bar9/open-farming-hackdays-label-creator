use std::ops::Add;
use chrono::{DateTime, TimeDelta, Utc};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct DateInputProps {
    date_prefix: Signal<String>,
    date_value: Signal<String>,
}

pub fn DateInput(mut props: DateInputProps) -> Element {
    let in_a_year: DateTime<Utc> = Utc::now().add(TimeDelta::days(365));
    let formatted_date = in_a_year.format("%Y-%m-%d").to_string();

    rsx! {
        select {
            oninput: move |evt| props.date_prefix.set(evt.data.value()),
            class: "select bg-white select-bordered w-full max-w-xs",
            option {selected: true, "mindestens haltbar bis"}
            option {"zu verbrauchen bis"}
        }
        input {
            oninput: move |evt| props.date_value.set(evt.data.value()),
            class: "input bg-white input-bordered w-full", r#type: "date", value: "{formatted_date}"}
    }
}
