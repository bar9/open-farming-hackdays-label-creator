use crate::layout::FullLayout;
use crate::layout::SplitLayout;
#[cfg(not(feature = "hidebio"))]
use crate::pages::bio::Bio;
use crate::pages::impressum::Impressum;
#[cfg(not(feature = "hidebio"))]
use crate::pages::knospe::Knospe;
use crate::pages::splash_screen::SplashScreen;
use crate::pages::swiss::Swiss;
use dioxus::prelude::*;
use rust_i18n::t;

#[rustfmt::skip]
#[derive(Clone, Routable, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Route {
    #[layout(FullLayout)]
        #[layout(SplitLayout)]
            #[route("/lebensmittelrecht")]
            Swiss {},

            #[cfg(not(feature = "hidebio"))]
            #[route("/bio")]
            Bio {},

            #[cfg(not(feature = "hidebio"))]
            #[route("/knospe")]
            Knospe {},
        #[end_layout]
        #[route("/impressum")]
        Impressum {},

        #[route("/")]
        SplashScreen {},
    #[end_layout]


    #[route("/:..route")]
    PageNotFound {
        route: Vec<String>,
    },

}

#[component]
fn PageNotFound(route: Vec<String>) -> Element {
    rsx! {
        h1 { {t!("errors.page_not_found").to_string()} }
        p { {t!("errors.page_not_found_message").to_string()} }
        pre { color: "red", {format!("{}\n{:?}", t!("errors.attempted_to_navigate"), route)} }
    }
}
