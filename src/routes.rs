use crate::layout::FullLayout;
use crate::layout::SplitLayout;
use crate::pages::bio::Bio;
use crate::pages::impressum::Impressum;
use crate::pages::knospe::Knospe;
use crate::pages::splash_screen::SplashScreen;
use crate::pages::swiss::Swiss;
use dioxus::prelude::*;

#[rustfmt::skip]
#[derive(Clone, Routable, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Route {
    #[layout(FullLayout)]
        #[layout(SplitLayout)]
            #[route("/lebensmittelrecht")]
            Swiss {},

            #[route("/bio")]
            Bio {},

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
        h1 { "Page not found" }
        p { "We are terribly sorry, but the page you requested doesn't exist." }
        pre { color: "red", "log:\nattempted to navigate to: {route:?}" }
    }
}
