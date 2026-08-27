// DEC-16 follow-up, from the reported link: an ingredient ticked as
// «aus zertifizierter Wildsammlung» keeps `is_agricultural=true` (that flag
// follows the «Nicht-landwirtschaftlich» quality, not the checkbox). A recipe
// made purely of wild-collected ingredients therefore still printed
// «Alle landwirtschaftlichen Zutaten stammen aus biologischer Landwirtschaft».
mod common;

use common::recipes::Config;
use common::*;
use std::time::Duration;

const BLANKET: &str = "Alle landwirtschaftlichen Zutaten stammen aus biologischer Landwirtschaft";

/// Full page text, since the hint lives outside the white label card.
async fn page_text(c: &fantoccini::Client) -> String {
    c.execute("return document.body.innerText;", vec![])
        .await
        .unwrap()
        .as_str()
        .unwrap_or("")
        .to_string()
}

async fn open_shared(c: &fantoccini::Client, query: &str) {
    // A shared link carries the recipe in the query string on the route itself
    // (see e2e_link_compat). The Bio hints render only once the disclaimer is
    // accepted, which goto_config already did before this call.
    goto(c, &format!("bio?{}", query)).await;
    tokio::time::sleep(mount_delay()).await;
    tokio::time::sleep(Duration::from_millis(2500)).await;
}

/// The exact recipe from the report: one wild-collected, bio-CH ingredient.
const ONLY_WILD: &str = "v=2&ingredients[0][name]=Bärlauch&ingredients[0][is_allergen]=false\
&ingredients[0][amount]=342.0&ingredients[0][unit]=Gram&ingredients[0][is_namensgebend]=false\
&ingredients[0][is_agricultural]=true&ingredients[0][is_bio]=false&ingredients[0][bio_ch]=true\
&ingredients[0][erlaubte_ausnahme_bio]=false&ingredients[0][erlaubte_ausnahme_knospe]=false\
&ingredients[0][processing_steps][0]=aus+zertifizierter+Wildsammlung\
&ingredients[0][aus_umstellbetrieb]=false&rezeptur_vollstaendig=true";

#[tokio::test]
async fn a_purely_wild_collected_recipe_hides_the_blanket_sentence() {
    let c = connect().await;
    goto_config(&c, Config::Bio).await;
    open_shared(&c, ONLY_WILD).await;

    let text = page_text(&c).await;
    assert!(
        text.contains("Wildsammlung"),
        "premise: the recipe should be loaded; text: {}",
        text
    );
    assert!(
        !text.contains(BLANKET),
        "nothing is farmed here, so the blanket sentence must stay hidden; text: {}",
        text
    );

    assert_no_errors(&c, "purely wild-collected recipe").await;
    let _ = c.close().await;
}

// The other half: as soon as one ingredient really is farmed, the sentence is
// meaningful again and must reappear.
#[tokio::test]
async fn adding_a_farmed_ingredient_brings_the_sentence_back() {
    let c = connect().await;
    goto_config(&c, Config::Bio).await;

    let with_farmed = format!(
        "{}&ingredients[1][name]=Rapsöl&ingredients[1][is_allergen]=false\
&ingredients[1][amount]=658.0&ingredients[1][unit]=Gram&ingredients[1][is_agricultural]=true\
&ingredients[1][is_bio]=false&ingredients[1][bio_ch]=true",
        ONLY_WILD
    );
    open_shared(&c, &with_farmed).await;

    let text = page_text(&c).await;
    assert!(
        text.contains(BLANKET),
        "with a farmed ingredient present the sentence applies again; text: {}",
        text
    );

    let _ = c.close().await;
}
