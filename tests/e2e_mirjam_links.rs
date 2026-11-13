// Mirjam (12./13.11.2026): the REAL share link declarino.ch/s/HClTnpf, opened
// in the browser. 0.8.27 passed every hand-built test but the preview of this
// link still showed Butter and Greyerzer without origin, because the link
// carries BLV categories («Fette», «Hartkäse») and is_allergen=false for
// Greyerzer. This test guards the real artifact, not a reconstruction.
mod common;

use common::recipes::Config;
use common::*;
use std::time::Duration;

#[tokio::test]
async fn mirjam_dairy_share_link_preview_shows_origins() {
    let c = connect().await;
    // Accept the disclaimer first so the preview renders on the shared route.
    goto_config(&c, Config::Knospe).await;

    let query = include_str!("fixtures/mirjam_HClTnpf.qs").trim();
    // E2E_SHARE_URL=https://www.declarino.ch/s/HClTnpf checks the deployed
    // short link end to end; by default the fixture runs against the app.
    match std::env::var("E2E_SHARE_URL") {
        Ok(url) => {
            c.goto(&url).await.expect("goto share url");
            tokio::time::sleep(mount_delay()).await;
        }
        Err(_) => goto(&c, &format!("knospe?{}", query)).await,
    }
    tokio::time::sleep(mount_delay()).await;
    tokio::time::sleep(Duration::from_millis(1000)).await;
    let current = c.current_url().await.map(|u| u.to_string()).unwrap_or_default();
    println!("URL: {}", &current[..current.len().min(120)]);

    let text = label_html(&c).await;
    let html: String = c
        .execute(
            "const p = document.querySelector('div.bg-white.rounded-lg.shadow-lg'); return p ? p.innerHTML : '';",
            vec![],
        )
        .await
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();
    println!("PREVIEW: {}", text);
    assert!(
        text.contains("Butter* (CH)"),
        "Butter must show (CH); label: {}",
        text
    );
    assert!(
        text.contains("Greyerzer vollfett* (FR)"),
        "Greyerzer must show (FR); label: {}",
        text
    );
    assert!(
        html.contains("<b>Greyerzer vollfett</b>"),
        "Greyerzer must be bold (allergen); html: {}",
        html
    );
    assert!(
        text.contains("Milch* (CH)") && text.contains("Joghurt* (CH)"),
        "Milch/Joghurt keep their origin; label: {}",
        text
    );

    assert_no_errors(&c, "mirjam dairy share link").await;
    let _ = c.close().await;
}
