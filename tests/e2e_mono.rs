// Manual DEC-2 smoke check via WebDriver: the mono-product quality selector
// must appear when «Keine Zutatenliste» is ticked, and picking the Swiss Knospe
// must put the Knospe logo on the label preview.
mod common;

use common::recipes::Config;
use common::*;
use fantoccini::Locator;

#[tokio::test]
async fn mono_quality_selector_drives_the_knospe_logo() {
    let c = connect().await;
    goto_config(&c, Config::Knospe).await;

    // Fill the Sachbezeichnung so the preview renders a real label.
    if let Ok(el) = c
        .find(Locator::XPath(
            "//input[contains(@placeholder, 'Konfitüre') or contains(@placeholder, 'Sachbezeichnung')]",
        ))
        .await
    {
        let _ = el.click().await;
        let _ = el.send_keys("Weizenmehl").await;
    }

    // Tick «Keine Zutatenliste (Einzelzutat)». Click the checkbox itself; the
    // surrounding <label> is what FormField renders around it.
    let toggled = c
        .find(Locator::XPath(
            "//label[contains(normalize-space(.), 'Keine Zutatenliste')]//input[@type='checkbox']",
        ))
        .await;
    let toggled = match toggled {
        Ok(el) => el.click().await.is_ok(),
        Err(_) => false,
    };
    assert!(toggled, "could not find the «Keine Zutatenliste» checkbox");
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // The quality selector must now be present (DEC-2).
    let has_selector = c
        .find(Locator::XPath("//input[@name='mono_quality']"))
        .await
        .is_ok();
    assert!(
        has_selector,
        "mono-product quality selector missing after enabling «Keine Zutatenliste»"
    );

    // Choose «Bio (Knospe)»; the Swiss Knospe is the default variant.
    let picked = c
        .find(Locator::XPath("//input[@name='mono_quality'][1]"))
        .await;
    let picked = match picked {
        Ok(el) => el.click().await.is_ok(),
        Err(_) => false,
    };
    assert!(picked, "could not select «Bio (Knospe)»");
    tokio::time::sleep(std::time::Duration::from_millis(700)).await;

    assert!(
        has_bio_suisse_cross(&c).await,
        "Swiss Knospe must show the cross logo on the label"
    );

    let _ = c.close().await;
}
