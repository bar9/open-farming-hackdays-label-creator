// The Nettogewicht must survive an empty Abtropfgewicht.
//
// The Abtropfgewicht field is optional and revealed by a toggle. Opening it
// without typing anything is an everyday half-finished state, and it must not
// take the net weight off the label with it.
mod common;

use common::recipes::Config;
use common::*;
use std::time::Duration;

async fn open_shared(c: &fantoccini::Client, query: &str) {
    goto(c, &format!("lebensmittelrecht?{}", query)).await;
    tokio::time::sleep(mount_delay()).await;
    tokio::time::sleep(Duration::from_millis(700)).await;
}

#[tokio::test]
async fn net_weight_still_prints_when_the_drained_weight_is_empty() {
    let c = connect().await;
    goto_config(&c, Config::Lebensmittelrecht).await;

    // Netto entered, Abtropfgewicht revealed but left blank. This is exactly
    // what the app itself serializes for that state: the second slot is present
    // but empty.
    let query = "v=2&product_subtitle=Konfit%C3%BCre&amount_type=Weight&weight_unit=g\
&amount[Double][0]=250&amount[Double][1]";
    open_shared(&c, query).await;

    let label = label_html(&c).await;
    assert!(
        label.contains("250"),
        "the net weight must print even without a drained weight; label: {}",
        label
    );

    assert_no_errors(&c, "net weight without drained weight").await;
    let _ = c.close().await;
}

// Both weights present: the label names them so they cannot be confused.
#[tokio::test]
async fn both_weights_print_with_their_labels() {
    let c = connect().await;
    goto_config(&c, Config::Lebensmittelrecht).await;

    let query = "v=2&product_subtitle=Konfit%C3%BCre&amount_type=Weight&weight_unit=g\
&amount[Double][0]=250&amount[Double][1]=200";
    open_shared(&c, query).await;

    let label = label_html(&c).await;
    assert!(
        label.contains("250") && label.contains("200"),
        "both weights must print; label: {}",
        label
    );

    assert_no_errors(&c, "both weights").await;
    let _ = c.close().await;
}
