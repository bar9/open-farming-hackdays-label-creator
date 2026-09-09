use crate::built_info;
use crate::routes::Route;
use dioxus::prelude::*;
use rust_i18n::t;

/// Build date as dd.mm.yyyy. `BUILT_TIME_UTC` is RFC 2822; if it ever fails to
/// parse, showing the raw string is better than showing nothing.
fn build_date() -> String {
    let build_time = built_info::BUILT_TIME_UTC;
    match chrono::DateTime::parse_from_rfc2822(build_time) {
        Ok(datetime) => format!("{}", datetime.format("%d.%m.%Y")),
        Err(_) => build_time.to_string(),
    }
}

/// Version line plus the FAQ / Support / Impressum / Release-notes links.
/// Shared by the app layout and the splash screen; they differ only in how the
/// footer is pushed to the bottom, hence `class`.
#[component]
pub fn SiteFooter(
    /// Layout classes for the `<footer>` element, e.g. `mt-auto` vs `flex-none`.
    class: String,
) -> Element {
    rsx! {
        footer {
            class: "bg-base-200 p-4 text-center text-sm border-t border-base-300 {class}",
            div {
                class: "flex justify-center items-center gap-4",
                span {
                    {t!("version.version").to_string()} " " {env!("CARGO_PKG_VERSION")} " "
                    {t!("version.from").to_string()} " " {build_date()}
                }
                Link {
                    to: Route::Faq {},
                    class: "link link-blue hover:link-primary",
                    {t!("app.faq").to_string()}
                }
                Link {
                    to: Route::Support {},
                    class: "link link-blue hover:link-primary",
                    {t!("app.support").to_string()}
                }
                Link {
                    to: Route::Impressum {},
                    class: "link link-blue hover:link-primary",
                    {t!("app.impressum").to_string()}
                }
                a {
                    class: "link link-blue hover:link-primary",
                    href: "https://github.com/bar9/open-farming-hackdays-label-creator/wiki/Release-notes",
                    target: "_blank",
                    {t!("app.release_notes").to_string()}
                }
            }
        }
    }
}
