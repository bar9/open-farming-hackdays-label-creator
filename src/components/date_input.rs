use chrono::{DateTime, NaiveDate, TimeDelta, Utc};
use dioxus::prelude::*;
use rust_i18n::t;
use std::ops::Add;

#[derive(Props, Clone, PartialEq)]
pub struct DateInputProps {
    date_prefix: Signal<String>,
    date_value: Signal<String>,
}

pub fn DateInput(mut props: DateInputProps) -> Element {
    let in_a_year: DateTime<Utc> = Utc::now().add(TimeDelta::days(365));
    let default_date = in_a_year.format("%Y-%m-%d").to_string();

    // Initialize from props.date_value if available, converting DD.MM.YYYY -> YYYY-MM-DD
    let initial_ymd = {
        let date_str = props.date_value.peek();
        if !date_str.is_empty() {
            // Try to parse DD.MM.YYYY and convert to YYYY-MM-DD
            NaiveDate::parse_from_str(&date_str, "%d.%m.%Y")
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|_| default_date.clone())
        } else {
            default_date.clone()
        }
    };

    let mut ymd_date: Signal<String> = use_signal(|| initial_ymd);
    use_effect(move || {
        let datestr = &*ymd_date.read();
        // Only update if we have a valid date string
        if let Ok(naive_date) = NaiveDate::parse_from_str(datestr, "%Y-%m-%d") {
            if let Some(datetime_utc) = naive_date.and_hms_opt(0, 0, 0) {
                props.date_value.set(
                    datetime_utc
                        .format("%d.%m.%Y")
                        .to_string(),
                );
            }
        }
        // If parsing fails, keep the previous value
    });

    rsx! {
        select {
            oninput: move |evt| props.date_prefix.set(evt.data.value()),
            class: "select w-full max-w-xs select-bordered bg-base-200 focus:bg-white focus:outline-none focus:ring-2 focus:ring-gray-200",
            value: "{props.date_prefix}",
            option {
                value: "{t!(\"label.mindestensHaltbar\")}",
                selected: *props.date_prefix.read() == t!("label.mindestensHaltbar"),
                "{t!(\"label.mindestensHaltbar\")}"
            }
            option {
                value: "{t!(\"label.zuVerbrauchen\")}",
                selected: *props.date_prefix.read() == t!("label.zuVerbrauchen"),
                "{t!(\"label.zuVerbrauchen\")}"
            }
            option {
                value: "{t!(\"label.keinDatum\")}",
                selected: *props.date_prefix.read() == t!("label.keinDatum"),
                "{t!(\"label.keinDatum\")}"
            }
        }
        if *props.date_prefix.read() != t!("label.keinDatum") {
            input {
                oninput: move |evt| {
                    ymd_date.set(evt.data.value());
                },
                class: "input input-bordered w-full bg-base-200 focus:bg-white focus:outline-none focus:ring-2 focus:ring-gray-200", r#type: "date", value: "{ymd_date}"
            }
        }
    }
}
