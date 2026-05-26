// Shared E2E helpers. Each test binary declares `mod common;` to use this.
//
// Goal: simple, best-effort selectors. Tests are local-only and we accept
// some brittleness — the value is finding panics in real workflows.
//
// Selector strategy (no data-testid in production code):
//   1. placeholder text (i18n-translated German strings)
//   2. visible button text via XPath
//   3. daisyUI class names (.input-accent, .btn-info, ...)
//   4. position (:nth-of-type) as last resort

#![allow(dead_code)]

pub mod recipes;

use fantoccini::elements::Element;
use fantoccini::{Client, ClientBuilder, Locator};
use std::time::Duration;

use recipes::{BioStatus, Config, Recipe, RecipeIngredient};

pub fn app_url() -> String {
    std::env::var("E2E_URL")
        .unwrap_or_else(|_| "http://localhost:8080/open-farming-hackdays-label-creator/".into())
}

pub fn webdriver_url() -> String {
    std::env::var("E2E_WEBDRIVER").unwrap_or_else(|_| "http://localhost:4444".into())
}

pub fn mount_delay() -> Duration {
    let ms: u64 = std::env::var("E2E_MOUNT_DELAY_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1500);
    Duration::from_millis(ms)
}

pub async fn connect() -> Client {
    ClientBuilder::native()
        .connect(&webdriver_url())
        .await
        .expect("WebDriver not reachable — start chromedriver/geckodriver on :4444")
}

pub const ERROR_TRAP: &str = r#"
    if (!window.__trapInstalled) {
        window.__errors = [];
        window.addEventListener('error', e => {
            window.__errors.push('error: ' + String(e.error || e.message));
        });
        window.addEventListener('unhandledrejection', e => {
            window.__errors.push('rejection: ' + String(e.reason));
        });
        const orig = console.error.bind(console);
        console.error = (...args) => {
            window.__errors.push('console.error: ' + args.map(a => {
                try { return typeof a === 'string' ? a : JSON.stringify(a); }
                catch (_) { return String(a); }
            }).join(' '));
            orig(...args);
        };
        window.__trapInstalled = true;
    }
    return null;
"#;

pub async fn install_trap(c: &Client) {
    c.execute(ERROR_TRAP, vec![]).await.expect("install trap");
}

/// Navigate to `path` relative to app_url, install error trap, wait for mount.
pub async fn goto(c: &Client, path: &str) {
    let url = format!("{}{}", app_url(), path);
    c.goto(&url).await.expect("goto");
    install_trap(c).await;
    tokio::time::sleep(mount_delay()).await;
}

pub async fn read_errors(c: &Client) -> Vec<String> {
    c.execute("return window.__errors || [];", vec![])
        .await
        .ok()
        .and_then(|v| {
            v.as_array()
                .map(|a| a.iter().map(|x| x.to_string()).collect())
        })
        .unwrap_or_default()
}

pub async fn clear_errors(c: &Client) {
    c.execute("window.__errors = []; return null;", vec![])
        .await
        .ok();
}

pub async fn assert_no_errors(c: &Client, context: &str) {
    let errs = read_errors(c).await;
    if !errs.is_empty() {
        panic!(
            "JS errors at [{}]:\n  {}",
            context,
            errs.join("\n  ")
        );
    }
}

// ---------- Interaction helpers ----------

/// Find an input by partial placeholder text match (case-sensitive substring).
pub async fn input_by_placeholder(c: &Client, fragment: &str) -> Option<Element> {
    c.find(Locator::Css(&format!("input[placeholder*='{}']", fragment)))
        .await
        .ok()
}

/// Find a button containing the given visible text (XPath, language-sensitive).
pub async fn button_by_text(c: &Client, text: &str) -> Option<Element> {
    c.find(Locator::XPath(&format!(
        "//button[contains(normalize-space(.), '{}')]",
        text
    )))
    .await
    .ok()
}

pub async fn click_button_by_text(c: &Client, text: &str) -> bool {
    if let Some(btn) = button_by_text(c, text).await {
        btn.click().await.ok();
        tokio::time::sleep(Duration::from_millis(300)).await;
        true
    } else {
        false
    }
}

/// First input with the daisyUI accent class (used as the genesis ingredient input).
pub async fn first_accent_input(c: &Client) -> Option<Element> {
    c.find(Locator::Css("input.input-accent")).await.ok()
}

/// First text input on the page (used as product title heuristic).
pub async fn first_text_input(c: &Client) -> Option<Element> {
    c.find(Locator::Css("input[type='text'], input:not([type])"))
        .await
        .ok()
}

/// Find `<dialog open>` count — for sanity checking modals opened/closed.
pub async fn open_dialog_count(c: &Client) -> usize {
    c.execute("return document.querySelectorAll('dialog[open]').length;", vec![])
        .await
        .ok()
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize
}

/// Type into the product-name field. Tries the de-CH placeholder
/// "Sommertraum" first, then falls back to the first text input.
pub async fn set_product_title(c: &Client, name: &str) {
    let el = input_by_placeholder(c, "Sommertraum")
        .await
        .or(first_text_input(c).await);
    if let Some(el) = el {
        let _ = el.click().await;
        let _ = el.send_keys(name).await;
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
}

/// Type into the Sachbezeichnung field (de-CH placeholder
/// "Himbeerkonfitüre" per `locales/de-CH.yml:232`).
pub async fn set_sachbezeichnung(c: &Client, value: &str) {
    if let Some(el) = input_by_placeholder(c, "Himbeerkonfitüre").await {
        let _ = el.click().await;
        let _ = el.send_keys(value).await;
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
}

/// Set `disclaimer_accepted=true` in localStorage so the label preview
/// renders. The whole `.bg-white.rounded-lg.shadow-lg` label container in
/// `label_preview.rs` is gated on this flag (see `if disclaimer_accepted()`
/// at `src/components/label_preview.rs:124`). Without this, `label_html()`
/// returns "" because the container does not mount.
pub async fn accept_disclaimer(c: &Client) {
    let _ = c
        .execute(
            "localStorage.setItem('disclaimer_accepted', 'true'); return null;",
            vec![],
        )
        .await;
}

/// Open the genesis ("add ingredient") modal by clicking the de-CH button text.
pub async fn open_add_ingredient(c: &Client) -> bool {
    click_button_by_text(c, "Zutat hinzufügen").await
}

/// Add a simple ingredient via the genesis modal:
/// - open modal
/// - type name in the .input-accent input
/// - press Enter to commit as custom ingredient
/// - try to fill amount in a numeric input
/// - click a Save / "Speichern und nächste" / "Speichern" button
pub async fn add_simple_ingredient(c: &Client, name: &str, amount_grams: u32) {
    if !open_add_ingredient(c).await {
        return;
    }
    tokio::time::sleep(Duration::from_millis(400)).await;
    if let Some(input) = first_accent_input(c).await {
        let _ = input.click().await;
        let _ = input.send_keys(name).await;
        tokio::time::sleep(Duration::from_millis(250)).await;
        let _ = input.send_keys("\u{E007}").await; // Enter key
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    // Amount input — try [type=number] inside open dialog
    if let Ok(num) = c
        .find(Locator::Css("dialog[open] input[type='number']"))
        .await
    {
        let _ = num.click().await;
        let _ = num.send_keys(&amount_grams.to_string()).await;
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
    // Try to commit. Buttons in IngredientPane are usually labelled "Speichern" or
    // "Speichern und nächste" depending on context.
    for label in &["Speichern und nächste", "Speichern", "Hinzufügen", "OK"] {
        if click_button_by_text(c, label).await {
            break;
        }
    }
    // Close modal if still open
    tokio::time::sleep(Duration::from_millis(300)).await;
    if open_dialog_count(c).await > 0 {
        for label in &["Schliessen", "Schließen", "Abbrechen", "Close"] {
            if click_button_by_text(c, label).await {
                break;
            }
        }
        // Last resort: ESC
        if open_dialog_count(c).await > 0 {
            if let Ok(body) = c.find(Locator::Css("body")).await {
                let _ = body.send_keys("\u{E00C}").await; // ESC
            }
        }
    }
}

/// Read the share URL from the link-share modal. Caller must open the modal first.
/// Robust against `open="false"` attribute weirdness — looks for any readonly
/// input on the page that contains a URL-like value.
pub async fn read_share_url(c: &Client) -> Option<String> {
    c.execute(
        r#"
        const inputs = document.querySelectorAll('input[readonly]');
        for (const i of inputs) {
            if (i.value && (i.value.startsWith('http') || i.value.includes('?'))) {
                return i.value;
            }
        }
        return null;
        "#,
        vec![],
    )
    .await
    .ok()
    .and_then(|v| v.as_str().map(|s| s.to_string()))
}

/// Inject a saved composite ingredient directly into localStorage and reload.
/// Bypasses the brittle UI for creating composites.
/// Key matches `SAVED_INGREDIENTS_KEY` in src/persistence.rs.
pub async fn seed_saved_ingredient_json(c: &Client, json_array: &str) {
    let script = format!(
        "localStorage.setItem('saved_composite_ingredients', `{}`); return null;",
        json_array.replace('`', r"\`")
    );
    c.execute(&script, vec![]).await.ok();
}

pub async fn reload(c: &Client) {
    c.refresh().await.ok();
    install_trap(c).await;
    tokio::time::sleep(mount_delay()).await;
}

// ---------- UX layout assertions ----------

/// Inspect every open dialog plus any descendant scroll container.
/// Reports cases where there is a scrollbar despite room being available
/// in the viewport to grow the modal. Threshold: scroll exists AND
/// modal-box uses less than 95% of the viewport on the scroll axis.
///
/// Returns a list of human-readable issue descriptions (empty = OK).
pub async fn check_modal_sizing(c: &Client) -> Vec<String> {
    let raw = c
        .execute(
            r#"
            function describeEl(el) {
                let s = el.tagName.toLowerCase();
                if (el.id) s += '#' + el.id;
                if (el.className && typeof el.className === 'string') {
                    const cls = el.className.split(/\s+/).filter(Boolean).slice(0, 3).join('.');
                    if (cls) s += '.' + cls;
                }
                return s;
            }
            const issues = [];
            const dialogs = Array.from(document.querySelectorAll('dialog[open]'));
            for (const dlg of dialogs) {
                const dStyle = getComputedStyle(dlg);
                if (dStyle.display === 'none' || dStyle.visibility === 'hidden') continue;
                const box = dlg.querySelector('.modal-box') || dlg;
                const boxRect = box.getBoundingClientRect();
                if (boxRect.width < 50 || boxRect.height < 50) continue;

                const vRoom = window.innerHeight - boxRect.height;
                const hRoom = window.innerWidth - boxRect.width;
                // Threshold: at least 5% viewport unused, OR ≥50px (catches small
                // viewports where 5% is tiny).
                const vRoomThreshold = Math.max(50, window.innerHeight * 0.05);
                const hRoomThreshold = Math.max(50, window.innerWidth * 0.05);

                // Examine the modal-box AND any descendant scroll container
                const candidates = [box, ...dlg.querySelectorAll('*')];
                const seen = new Set();
                for (const el of candidates) {
                    if (seen.has(el)) continue;
                    seen.add(el);
                    const s = getComputedStyle(el);
                    if (s.display === 'none' || s.visibility === 'hidden') continue;
                    const r = el.getBoundingClientRect();
                    if (r.width < 50 || r.height < 50) continue;

                    const oy = s.overflowY;
                    const ox = s.overflowX;
                    const vScrollEnabled = (oy === 'auto' || oy === 'scroll' || el === box);
                    const hScrollEnabled = (ox === 'auto' || ox === 'scroll' || el === box);

                    if (vScrollEnabled && el.scrollHeight > el.clientHeight + 2 && vRoom > vRoomThreshold) {
                        issues.push(
                            'vertical scroll on ' + describeEl(el) +
                            ' with ' + Math.round(vRoom) +
                            'px viewport room unused (el h=' + Math.round(r.height) +
                            ', content h=' + el.scrollHeight +
                            ', modal h=' + Math.round(boxRect.height) +
                            ', viewport h=' + window.innerHeight + ')'
                        );
                    }
                    if (hScrollEnabled && el.scrollWidth > el.clientWidth + 2 && hRoom > hRoomThreshold) {
                        issues.push(
                            'horizontal scroll on ' + describeEl(el) +
                            ' with ' + Math.round(hRoom) +
                            'px viewport room unused (el w=' + Math.round(r.width) +
                            ', content w=' + el.scrollWidth +
                            ', modal w=' + Math.round(boxRect.width) +
                            ', viewport w=' + window.innerWidth + ')'
                        );
                    }
                }
            }
            return issues;
            "#,
            vec![],
        )
        .await
        .ok();
    raw.and_then(|v| {
        v.as_array().map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
    })
    .unwrap_or_default()
}

pub async fn assert_modals_well_sized(c: &Client, context: &str) {
    let issues = check_modal_sizing(c).await;
    if !issues.is_empty() {
        panic!(
            "modal sizing issues at [{}]:\n  - {}",
            context,
            issues.join("\n  - ")
        );
    }
}

/// Set the browser window to a specific size. Useful for testing layout
/// at known viewports (otherwise chromedriver picks an arbitrary size).
pub async fn set_window_size(c: &Client, width: u32, height: u32) {
    c.set_window_rect(0, 0, width, height).await.ok();
}

/// Debug helper: returns a JSON-ish snapshot of every open dialog and the
/// scrollable elements inside it.
pub async fn debug_modal_state(c: &Client) -> String {
    c.execute(
        r#"
        const out = { viewport: { w: window.innerWidth, h: window.innerHeight }, dialogs: [] };
        const dialogs = document.querySelectorAll('dialog');
        for (const d of dialogs) {
            const rect = d.getBoundingClientRect();
            const style = getComputedStyle(d);
            const info = {
                openAttr: d.getAttribute('open'),
                visible: style.display !== 'none' && style.visibility !== 'hidden',
                rect: { w: Math.round(rect.width), h: Math.round(rect.height) },
                scrollers: []
            };
            const all = [d, ...d.querySelectorAll('*')];
            for (const el of all) {
                const s = getComputedStyle(el);
                if (s.display === 'none' || s.visibility === 'hidden') continue;
                const r = el.getBoundingClientRect();
                if (r.width < 50 || r.height < 50) continue;
                const oy = s.overflowY, ox = s.overflowX;
                const vScroll = el.scrollHeight - el.clientHeight;
                const hScroll = el.scrollWidth - el.clientWidth;
                if (vScroll > 2 || hScroll > 2) {
                    info.scrollers.push({
                        tag: el.tagName.toLowerCase(),
                        cls: (el.className && typeof el.className === 'string') ? el.className.split(/\s+/).slice(0,4).join(' ') : '',
                        rect: { w: Math.round(r.width), h: Math.round(r.height) },
                        overflowY: oy, overflowX: ox,
                        vScrollDelta: vScroll, hScrollDelta: hScroll
                    });
                }
            }
            out.dialogs.push(info);
        }
        return JSON.stringify(out, null, 2);
        "#,
        vec![],
    )
    .await
    .ok()
    .and_then(|v| v.as_str().map(|s| s.to_string()))
    .unwrap_or_else(|| "<no output>".to_string())
}

// ---------- Label HTML reads ----------

/// Returns the innerText of the rendered label preview area
/// (the `div.bg-white.rounded-lg.shadow-lg` container in `LabelPreview`).
/// Single source of truth for "what the label says" — substring assertions
/// go against this string.
pub async fn label_html(c: &Client) -> String {
    c.execute(
        r#"
        const preview = document.querySelector('div.bg-white.rounded-lg.shadow-lg');
        return preview ? preview.innerText : '';
        "#,
        vec![],
    )
    .await
    .ok()
    .and_then(|v| v.as_str().map(|s| s.to_string()))
    .unwrap_or_default()
}

pub async fn assert_label_contains(c: &Client, expected: &str, ctx: &str) {
    let html = label_html(c).await;
    if !html.contains(expected) {
        panic!(
            "label missing expected text [{}]\n  expected substring: {}\n  actual label:\n{}",
            ctx, expected, html
        );
    }
}

pub async fn assert_label_not_contains(c: &Client, forbidden: &str, ctx: &str) {
    let html = label_html(c).await;
    if html.contains(forbidden) {
        panic!(
            "label contains forbidden text [{}]\n  forbidden substring: {}\n  actual label:\n{}",
            ctx, forbidden, html
        );
    }
}

/// Detects the BIO SUISSE Knospe with Swiss cross variant via the unique
/// red path style (`fill:#e2001a`) inside the absolutely-positioned logo
/// container. Returns `false` if the no-cross variant is rendered, or if
/// no logo is present.
pub async fn has_bio_suisse_cross(c: &Client) -> bool {
    c.execute(
        r##"
        const matches = document.querySelectorAll(
            'div.absolute.top-2.right-2 svg path[style*="#e2001a"]'
        );
        return matches.length > 0;
        "##,
        vec![],
    )
    .await
    .ok()
    .and_then(|v| v.as_bool())
    .unwrap_or(false)
}

// ---------- Configuration switching ----------

/// Open the configuration dropdown in the header and pick `label`
/// (one of "Lebensmittelrecht" / "Bio" / "Knospe"). If the data-loss
/// warning modal appears, click "Wechseln" when `confirm == true` or
/// "Abbrechen" otherwise. Returns true if the dropdown entry was clicked.
pub async fn select_configuration(c: &Client, label: &str, confirm: bool) -> bool {
    // Open the header dropdown by clicking its trigger ("Konfiguration" label).
    if !click_button_by_text(c, "Konfiguration").await {
        // Fall back to clicking the dropdown trigger by role.
        let trigger = c
            .find(Locator::Css("[role='button'].dropdown-toggle, .dropdown [role='button']"))
            .await
            .ok();
        if let Some(t) = trigger {
            let _ = t.click().await;
        }
    }
    tokio::time::sleep(Duration::from_millis(300)).await;
    // Click the matching menu entry. Try a button match first, then fall
    // back to any element containing the label text.
    let mut clicked = click_button_by_text(c, label).await;
    if !clicked {
        let xpath = format!(
            "//*[self::a or self::li or self::span][contains(normalize-space(.), '{}')]",
            label
        );
        if let Ok(el) = c.find(Locator::XPath(&xpath)).await {
            let _ = el.click().await;
            clicked = true;
        }
    }
    tokio::time::sleep(Duration::from_millis(400)).await;
    // Handle confirm modal if shown.
    if open_dialog_count(c).await > 0 {
        let target = if confirm { "Wechseln" } else { "Abbrechen" };
        click_button_by_text(c, target).await;
        tokio::time::sleep(Duration::from_millis(400)).await;
    }
    clicked
}

// ---------- Ingredient editing ----------

/// Find the row whose visible text contains `name`, then click the
/// first button inside it (the edit / list-detail action). The
/// ingredient list is div-based (`grid grid-cols-3`), not a real
/// `<table>`, so we walk the rows in JS.
pub async fn open_ingredient_edit_by_name(c: &Client, name: &str) -> bool {
    let safe_name = name.replace('\\', "\\\\").replace('\'', "\\'");
    // Scope to rows outside any open dialog — the genesis modal also contains
    // `.grid.grid-cols-3` rows, which would otherwise match first.
    let script = format!(
        r#"
        const rows = document.querySelectorAll('div.grid.grid-cols-3');
        for (const r of rows) {{
            if (r.closest('dialog[open]')) continue;
            if (r.innerText && r.innerText.includes('{}')) {{
                const btn = r.querySelector('button');
                if (btn) {{ btn.click(); return true; }}
            }}
        }}
        return false;
        "#,
        safe_name
    );
    let clicked = c
        .execute(&script, vec![])
        .await
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if clicked {
        tokio::time::sleep(Duration::from_millis(400)).await;
    }
    clicked
}

/// Inside an open dialog, set the first MultiCountrySelect / CountrySelect
/// dropdown to `country_code` (e.g. "CH", "DE", "FR"). Both selects are
/// native `<select>` elements, so we set value + dispatch change directly.
pub async fn set_origin_in_open_dialog(c: &Client, country_code: &str) -> bool {
    let script = format!(
        r#"
        const dlgs = document.querySelectorAll('dialog[open]');
        for (const dlg of dlgs) {{
            const selects = dlg.querySelectorAll('select');
            for (const s of selects) {{
                if (s.querySelector('option[value="{code}"]')) {{
                    s.value = "{code}";
                    s.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    return true;
                }}
            }}
        }}
        return false;
        "#,
        code = country_code
    );
    c.execute(&script, vec![])
        .await
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Clear all selected origins from the MultiCountrySelect inside an
/// open dialog by clicking each badge's × remove button. Scoped to
/// `span.badge button.btn-circle` to avoid hitting unrelated close
/// icons in the dialog header (see `multi_country_select.rs:144-160`).
pub async fn clear_origin_in_open_dialog(c: &Client) -> bool {
    c.execute(
        r#"
        const dlgs = document.querySelectorAll('dialog[open]');
        for (const dlg of dlgs) {
            const removers = dlg.querySelectorAll('span.badge button.btn-circle');
            removers.forEach(b => b.click());
            return true;
        }
        return false;
        "#,
        vec![],
    )
    .await
    .ok()
    .and_then(|v| v.as_bool())
    .unwrap_or(false)
}

// ---------- Recipe seeding via UI ----------

async fn fill_simple_field_by_placeholder(c: &Client, placeholder_fragment: &str, value: &str) {
    if let Some(el) = input_by_placeholder(c, placeholder_fragment).await {
        let _ = el.click().await;
        let _ = el.send_keys(value).await;
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
}

/// Adds an ingredient via the genesis modal with origin and bio status
/// applied. Best-effort: depends on the open-modal selectors used by
/// `add_simple_ingredient`. After this call, the genesis modal is closed.
pub async fn add_full_ingredient(c: &Client, ing: &RecipeIngredient) {
    if !open_add_ingredient(c).await {
        return;
    }
    tokio::time::sleep(Duration::from_millis(400)).await;
    // Name. Two-stage commit because of `unified_ingredient_input.rs:172-203`:
    // pressing Enter selects `search_results[0]` when the dropdown is open,
    // and the dropdown can include near-matches that come before the
    // exact-typed name (e.g. "Buchweizenmehl" before "Weizenmehl"). So we
    // first look for an exact-text dropdown item and click it; if none
    // matches, then Enter creates a custom ingredient from the free text.
    if let Some(input) = first_accent_input(c).await {
        let _ = input.click().await;
        let _ = input.send_keys(ing.name).await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        let safe = ing.name.replace('\\', "\\\\").replace('\'', "\\'");
        let exact_clicked = c
            .execute(
                &format!(
                    r#"
                    const dlgs = document.querySelectorAll('dialog[open]');
                    for (const d of dlgs) {{
                        const items = d.querySelectorAll('div.cursor-pointer');
                        for (const it of items) {{
                            const span = it.querySelector('span.font-medium');
                            const text = span ? span.innerText.trim() : it.innerText.trim();
                            if (text === '{name}') {{ it.click(); return true; }}
                        }}
                    }}
                    return false;
                    "#,
                    name = safe
                ),
                vec![],
            )
            .await
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if !exact_clicked {
            // No exact match — commit typed text as custom ingredient.
            let _ = input.send_keys("\u{E007}").await; // Enter
        }
        tokio::time::sleep(Duration::from_millis(400)).await;
    }
    // Amount
    if let Ok(num) = c.find(Locator::Css("dialog[open] input[type='number']")).await {
        let _ = num.click().await;
        // Some ingredient cards include multiple numeric inputs; the first one
        // is the amount field per ingredient_pane.rs.
        let amount_str = if ing.grams.fract() == 0.0 {
            format!("{}", ing.grams as u64)
        } else {
            format!("{}", ing.grams)
        };
        let _ = num.send_keys(&amount_str).await;
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
    // Origin
    if let Some(code) = ing.origin {
        set_origin_in_open_dialog(c, code).await;
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    // Bio status — labels per locales/de-CH.yml ingredient.* keys.
    let bio_label = match ing.bio {
        BioStatus::Conventional => None,
        BioStatus::BioCh => Some("Bio"),
        BioStatus::BioKnospe => Some("Bio (Knospe)"),
        BioStatus::BioKnospeImport => Some("Bio (Knospe) Import"),
        BioStatus::NichtLandwirtschaftlich => Some("Nicht-landwirtschaftliche Zutat"),
        BioStatus::Andere => Some("Andere"),
    };
    if let Some(label) = bio_label {
        // Click the matching radio / checkbox label inside the open dialog.
        let xpath = format!(
            "//dialog[@open]//label[contains(normalize-space(.), '{}')]",
            label
        );
        if let Ok(el) = c.find(Locator::XPath(&xpath)).await {
            let _ = el.click().await;
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
    }
    // Save and close. ingredient_pane.rs uses "Speichern und nächste Zutat"
    // for the genesis flow and "Speichern" for plain saves.
    for label in &["Speichern und nächste Zutat", "Speichern", "Hinzufügen", "OK"] {
        if click_button_by_text(c, label).await {
            break;
        }
    }
    tokio::time::sleep(Duration::from_millis(300)).await;
    // Close modal if still open
    if open_dialog_count(c).await > 0 {
        for label in &["Schliessen", "Schließen", "Abbrechen", "Close"] {
            if click_button_by_text(c, label).await {
                break;
            }
        }
        if open_dialog_count(c).await > 0 {
            if let Ok(body) = c.find(Locator::Css("body")).await {
                let _ = body.send_keys("\u{E00C}").await; // ESC
            }
        }
    }
    tokio::time::sleep(Duration::from_millis(200)).await;
}

/// Drive the form to enter a full recipe end-to-end. Caller has already
/// navigated to the right route. Slow (~10–30 s per recipe) but doesn't
/// require URL fixtures.
pub async fn seed_recipe_via_ui(c: &Client, recipe: &Recipe) {
    set_product_title(c, recipe.product_name).await;
    set_sachbezeichnung(c, recipe.sachbezeichnung).await;
    // Certification body for Bio / Knospe.
    if let Some(cert) = recipe.certification_body {
        // CertificationBodySelect renders a dropdown; type the code in its input.
        if let Some(el) = input_by_placeholder(c, "CH-BIO-").await {
            let _ = el.click().await;
            let _ = el.send_keys(cert).await;
            tokio::time::sleep(Duration::from_millis(200)).await;
            let _ = el.send_keys("\u{E007}").await; // Enter
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
    }
    // Ingredients
    for ing in recipe.ingredients {
        add_full_ingredient(c, ing).await;
    }
    // Click "Rezeptur vollständig. Überprüfen." for ALL configs: per
    // `core.rs:795`, ingredient validations (AP1_1, AP7_1, AP7_3, AP7_4,
    // AP7_5, Knospe_AlleZutatenHerkunft, Knospe_Under90) are gated on
    // `input.rezeptur_vollstaendig`. Without pressing this, validation
    // banners won't appear when origin is cleared, etc.
    click_button_by_text(c, "Rezeptur vollständig").await;
    tokio::time::sleep(Duration::from_millis(300)).await;
}

/// Navigate to the configuration's route, ensure the disclaimer is
/// accepted (so the label preview actually renders), and wait for mount.
pub async fn goto_config(c: &Client, config: Config) {
    // First navigate so we're on the same origin, then set localStorage
    // and reload to pick up the disclaimer state at mount time.
    goto(c, config.route()).await;
    accept_disclaimer(c).await;
    reload(c).await;
}
