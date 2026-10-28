// Backwards compatibility: DEC-13 added `egg_count` to the shared Form. Links
// created before that change must still restore correctly.
mod common;

use common::recipes::Config;
use common::*;
use std::time::Duration;

/// Load a query string as if it came from a shared link.
async fn open_shared(c: &fantoccini::Client, query: &str) {
    goto(c, &format!("lebensmittelrecht?{}", query)).await;
    tokio::time::sleep(mount_delay()).await;
    tokio::time::sleep(Duration::from_millis(700)).await;
}

#[tokio::test]
async fn links_created_before_the_egg_count_field_still_load() {
    let c = connect().await;
    // Accept the disclaimer first so the preview renders on the shared route.
    goto_config(&c, Config::Lebensmittelrecht).await;

    // A pre-DEC-13 link: no `egg_count` key anywhere.
    let legacy = "v=2&product_title=Hausmarke&product_subtitle=Konfit%C3%BCre\
&amount_type=Weight&weight_unit=g&amount[Single]=250&price[Single]=450\
&producer_name=Hof%20Muster&production_country=Schweiz";
    open_shared(&c, legacy).await;

    let label = label_html(&c).await;
    assert!(
        label.contains("Konfitüre"),
        "legacy link should restore the Sachbezeichnung; label: {}",
        label
    );
    assert!(
        label.contains("250"),
        "legacy link should restore the amount; label: {}",
        label
    );
    assert!(
        !label.contains("Stück"),
        "a legacy non-egg link must not show a piece count; label: {}",
        label
    );

    assert_no_errors(&c, "legacy link without egg_count").await;
    let _ = c.close().await;
}

// The new field survives a round trip through a shared link.
#[tokio::test]
async fn egg_count_round_trips_through_a_shared_link() {
    let c = connect().await;
    goto_config(&c, Config::Lebensmittelrecht).await;

    let with_eggs = "v=2&product_subtitle=Eier&amount_type=Weight&weight_unit=g\
&amount[Single]=350&egg_count=6";
    open_shared(&c, with_eggs).await;

    let label = label_html(&c).await;
    assert!(
        label.contains("6") && label.contains("Stück"),
        "an egg link should restore the piece count; label: {}",
        label
    );

    assert_no_errors(&c, "egg link round trip").await;
    let _ = c.close().await;
}

// api_konzept.md documents prefilled iFrame URLs with `amount_type=weight`
// (lowercase). Before 28.10.2026 serde rejected that variant, the whole form
// failed to parse and the page opened empty.
#[tokio::test]
async fn documented_iframe_url_prefills_the_form() {
    let c = connect().await;
    goto_config(&c, common::recipes::Config::Lebensmittelrecht).await;
    let query = "product_title=Bio%20Bergk%C3%A4se&product_subtitle=W%C3%BCrzig-kr%C3%A4ftig\
&producer_name=Alpk%C3%A4serei%20Grindelwald&producer_zip=3818&producer_city=Grindelwald\
&production_country=CH&amount_type=weight&weight_unit=g\
&ingredients[0][name]=Rohmilch&ingredients[0][amount]=950&ingredients[0][is_allergen]=true\
&ingredients[1][name]=Salz&ingredients[1][amount]=20&ingredients[1][is_allergen]=false";
    goto(&c, &format!("lebensmittelrecht?{}", query)).await;
    tokio::time::sleep(mount_delay()).await;
    tokio::time::sleep(std::time::Duration::from_millis(700)).await;

    let label = label_html(&c).await;
    assert!(label.contains("Bio Bergkäse"), "title missing. Label:\n{label}");
    assert!(label.contains("Rohmilch"), "ingredient missing. Label:\n{label}");
    assert_no_errors(&c, "documented_iframe_url_prefills_the_form").await;
    let _ = c.close().await;
}
