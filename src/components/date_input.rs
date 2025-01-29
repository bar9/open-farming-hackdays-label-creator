use std::ops::Add;
use chrono::{DateTime, NaiveDate, TimeDelta, TimeZone, Utc};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct DateInputProps {
    date_prefix: Signal<String>,
    date_value: Signal<String>,
}

pub fn DateInput(mut props: DateInputProps) -> Element {
    let in_a_year: DateTime<Utc> = Utc::now().add(TimeDelta::days(365));
    let formatted_date = in_a_year.format("%Y-%m-%d").to_string();
    let mut ymd_date: Signal<String> = use_signal(|| formatted_date);
    let mut dmy_date: Memo<String> = use_memo(move || {
        let datestr = &*ymd_date.read();
        let naive_date = NaiveDate::parse_from_str(datestr, "%Y-%m-%d")
            .expect("Failed to parse the date");

        let datetime_utc = Utc.from_utc_date(&naive_date).and_hms(0, 0, 0);
        props.date_value.set(datetime_utc.format("%d.%m.%Y").to_string());
        datetime_utc.format("%d.%m.%Y").to_string()
    });

    rsx! {
        select {
            oninput: move |evt| props.date_prefix.set(evt.data.value()),
            class: "select bg-white select-bordered w-full max-w-xs",
            option {selected: true, "mindestens haltbar bis"}
            option {"zu verbrauchen bis"}
        }
        input {
            oninput: move |evt| {
                ymd_date.set(evt.data.value());
            },
            class: "input bg-white input-bordered w-full", r#type: "date", value: "{ymd_date}"}
    }
}
