use dioxus::prelude::*;
use rust_i18n::t;
use markdown::{to_html};

#[derive(Props, Clone, PartialEq)]
pub struct ImpressumProps {
}

pub fn Impressum(props: ImpressumProps) -> Element {
    
    let text = t!("impressum");
    
    rsx! {
        div { class: "prose m-auto pt-4 px-4 sm:px-6 lg:px-8", dangerous_inner_html: to_html(&text) }
        div { class: "prose m-auto", 
            iframe {
                src: "https://app.privacybee.io/v/clldi3nqp2313020rttms8eh7y?lang=de&type=dsg",
                style: "width: 100%; height: 1000px; border: 0"
            }
        }
    }
}