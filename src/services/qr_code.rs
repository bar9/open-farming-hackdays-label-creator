//! QR-Code für den Teilen-Dialog.
//!
//! Der Kurz-Link soll sich vom Bildschirm aufs Handy holen lassen, ohne ihn
//! abzutippen. Erzeugt wird ein SVG-Pfad statt eines Bildes: er skaliert
//! verlustfrei, braucht keine Datei im Build und lässt sich im Test prüfen.
//!
//! Die Fehlerkorrektur steht auf `Medium`. Das ist der übliche Kompromiss:
//! `Low` verträgt keine Verschmutzung auf Papier, `High` bläht den Code bei
//! langen Links so weit auf, dass die Module auf dem Bildschirm zu klein
//! werden zum Scannen.

use qrcodegen::{QrCode, QrCodeEcc};

/// Rand in Modulen rund um den Code. Die Spezifikation verlangt eine ruhige
/// Zone von vier Modulen; ohne sie finden viele Scanner den Code nicht.
const QUIET_ZONE: i32 = 4;

/// Ein QR-Code, fertig zum Einsetzen in ein `<svg>`.
#[derive(Debug, Clone, PartialEq)]
pub struct QrSvg {
    /// Kantenlänge des viewBox-Quadrats, inklusive ruhiger Zone.
    pub size: i32,
    /// Pfad-Daten (`d`-Attribut) aller dunklen Module.
    pub path: String,
}

/// Baut aus beliebigem Text einen QR-Code als SVG-Pfad.
///
/// `None`, wenn der Text nicht in einen QR-Code passt (Version 40 fasst je
/// nach Zeichensatz gut 2900 Byte). Genau das kann bei einem vollständigen
/// Link mit ganzer Rezeptur im Query-String vorkommen, deshalb ist der Fall
/// ein Rückgabewert und kein Absturz: der Dialog zeigt dann einfach keinen
/// Code, während das Textfeld weiter funktioniert.
pub fn svg_path(text: &str) -> Option<QrSvg> {
    // Leerer Text ergäbe einen gültigen, aber sinnlosen Code.
    if text.is_empty() {
        return None;
    }

    let code = QrCode::encode_text(text, QrCodeEcc::Medium).ok()?;
    let modules = code.size();
    let size = modules + 2 * QUIET_ZONE;

    // Ein Pfad für alle dunklen Module. Ein Rechteck je Modul wäre bei
    // hunderten Modulen deutlich mehr DOM; als Pfad bleibt es ein Element.
    let mut path = String::new();
    for y in 0..modules {
        for x in 0..modules {
            if code.get_module(x, y) {
                if !path.is_empty() {
                    path.push(' ');
                }
                path.push_str(&format!("M{},{}h1v1h-1z", x + QUIET_ZONE, y + QUIET_ZONE));
            }
        }
    }

    Some(QrSvg { size, path })
}

impl QrSvg {
    /// Vollständiges SVG-Dokument mit weissem Hintergrund, für Kopieren und
    /// Herunterladen (R-13). `scale` Pixel pro Modul.
    pub fn standalone_svg(&self, scale: i32) -> String {
        let px = self.size * scale;
        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{px}" height="{px}" viewBox="0 0 {s} {s}" shape-rendering="crispEdges"><path fill="#ffffff" d="M0,0h{s}v{s}h-{s}z"/><path fill="#000000" d="{p}"/></svg>"##,
            s = self.size,
            p = self.path
        )
    }
}

/// Was beim Kopieren des QR-Codes passiert ist.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QrCopyOutcome {
    /// Als Bild in der Zwischenablage.
    Copied,
    /// Der Browser kann keine Bilder kopieren: stattdessen heruntergeladen.
    Downloaded,
    Failed,
}

/// Rendert das SVG als PNG und legt es in die Zwischenablage (R-13,
/// Livetesting 09.11.2026: «Nicht alle Nutzenden sind versiert mit Snipping
/// Tool»). Kann der Browser keine Bilder kopieren (Firefox ohne
/// `ClipboardItem`, kein sicherer Kontext), wird das PNG heruntergeladen.
pub async fn copy_png(svg: &str, filename: &str) -> QrCopyOutcome {
    run_png_action(svg, filename, true).await
}

/// Lädt den QR-Code als PNG herunter.
pub async fn download_png(svg: &str, filename: &str) -> QrCopyOutcome {
    run_png_action(svg, filename, false).await
}

async fn run_png_action(svg: &str, filename: &str, copy: bool) -> QrCopyOutcome {
    use wasm_bindgen::JsValue;
    use web_sys::js_sys::{Function, Promise};

    // Die Arbeit liegt in JS: Bild laden, auf ein Canvas zeichnen, als PNG
    // ausgeben. In Rust bräuchte das ein halbes Dutzend web-sys-Features für
    // einen einzigen Knopf.
    let f = Function::new_with_args(
        "svg, filename, copy",
        r#"
        return new Promise((resolve) => {
            const img = new Image();
            img.onload = () => {
                const canvas = document.createElement('canvas');
                canvas.width = img.width; canvas.height = img.height;
                canvas.getContext('2d').drawImage(img, 0, 0);
                canvas.toBlob(async (blob) => {
                    if (!blob) { resolve('failed'); return; }
                    const download = () => {
                        const a = document.createElement('a');
                        a.href = URL.createObjectURL(blob);
                        a.download = filename;
                        document.body.appendChild(a); a.click(); a.remove();
                        setTimeout(() => URL.revokeObjectURL(a.href), 1000);
                        resolve('downloaded');
                    };
                    if (copy && window.ClipboardItem && navigator.clipboard && navigator.clipboard.write) {
                        try {
                            await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })]);
                            resolve('copied');
                            return;
                        } catch (e) { /* fällt auf Herunterladen zurück */ }
                    }
                    download();
                }, 'image/png');
            };
            img.onerror = () => resolve('failed');
            img.src = 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(svg);
        });
        "#,
    );
    let Ok(promise) = f.call3(
        &JsValue::NULL,
        &JsValue::from_str(svg),
        &JsValue::from_str(filename),
        &JsValue::from_bool(copy),
    ) else {
        return QrCopyOutcome::Failed;
    };
    let promise: Promise = promise.into();
    match wasm_bindgen_futures::JsFuture::from(promise).await {
        Ok(v) => match v.as_string().as_deref() {
            Some("copied") => QrCopyOutcome::Copied,
            Some("downloaded") => QrCopyOutcome::Downloaded,
            _ => QrCopyOutcome::Failed,
        },
        Err(_) => QrCopyOutcome::Failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_has_no_code() {
        assert_eq!(svg_path(""), None);
    }

    #[test]
    fn a_short_link_becomes_a_scannable_code() {
        let qr = svg_path("https://www.declarino.ch/s/AbC1234").expect("short link must encode");

        // Kleinster QR-Code ist 21x21 Module, plus zweimal die ruhige Zone.
        assert!(
            qr.size >= 21 + 2 * QUIET_ZONE,
            "size {} is smaller than the smallest possible code",
            qr.size
        );
        assert!(!qr.path.is_empty(), "a code must have dark modules");
        assert!(
            qr.path.starts_with('M'),
            "path must be SVG path data, got: {}",
            &qr.path[..qr.path.len().min(20)]
        );
    }

    #[test]
    fn the_quiet_zone_stays_free() {
        let qr = svg_path("https://www.declarino.ch/s/AbC1234").expect("must encode");

        // Kein Modul darf im Rand liegen, sonst ist der Code schwer lesbar.
        // Die erste Zahl jedes Befehls ist x, die zweite y.
        for cmd in qr.path.split(' ') {
            let coords = cmd.trim_start_matches('M');
            let (x, rest) = coords.split_once(',').expect("path command has x,y");
            let y = rest.split('h').next().expect("y before the h command");
            let x: i32 = x.parse().expect("x is a number");
            let y: i32 = y.parse().expect("y is a number");

            assert!(
                x >= QUIET_ZONE && y >= QUIET_ZONE,
                "module at {x},{y} sits in the quiet zone"
            );
            assert!(
                x < qr.size - QUIET_ZONE && y < qr.size - QUIET_ZONE,
                "module at {x},{y} sits in the quiet zone on the far side"
            );
        }
    }

    #[test]
    fn a_longer_link_yields_a_denser_code() {
        let short = svg_path("https://www.declarino.ch/s/AbC1234").expect("must encode");
        let long = svg_path(&format!(
            "https://www.declarino.ch/?{}",
            "produkt=Bergkaese&zutat=Rohmilch&herkunft=CH&".repeat(20)
        ))
        .expect("a full link must still encode");

        assert!(
            long.size > short.size,
            "more data must need a bigger code: {} vs {}",
            long.size,
            short.size
        );
    }

    #[test]
    fn text_too_long_for_any_qr_code_is_refused() {
        // Version 40 fasst rund 2900 Byte; deutlich darüber gibt es keinen
        // gültigen Code mehr. Der Dialog muss das überleben.
        let huge = "x".repeat(5000);
        assert_eq!(svg_path(&huge), None);
    }

    /// Rastert den erzeugten SVG-Pfad zurück in ein Schwarzweissbild.
    ///
    /// Damit lässt sich prüfen, ob der Code wirklich lesbar ist, statt nur
    /// festzustellen, dass irgendein Pfad entstanden ist. Ein vertauschtes
    /// x/y oder eine falsche ruhige Zone fällt sonst nicht auf.
    #[cfg(test)]
    fn render_to_image(qr: &QrSvg, scale: u32) -> image::GrayImage {
        let side = qr.size as u32 * scale;
        let mut img = image::GrayImage::from_pixel(side, side, image::Luma([255u8]));

        for cmd in qr.path.split(' ') {
            let coords = cmd.trim_start_matches('M');
            let (x, rest) = coords.split_once(',').expect("path command has x,y");
            let y = rest.split('h').next().expect("y before the h command");
            let x: u32 = x.parse().expect("x is a number");
            let y: u32 = y.parse().expect("y is a number");

            for dy in 0..scale {
                for dx in 0..scale {
                    img.put_pixel(x * scale + dx, y * scale + dy, image::Luma([0u8]));
                }
            }
        }

        img
    }

    #[test]
    fn a_real_decoder_reads_the_link_back() {
        let link = "https://www.declarino.ch/s/AbC1234";
        let qr = svg_path(link).expect("must encode");

        let img = render_to_image(&qr, 4);
        let mut prepared = rqrr::PreparedImage::prepare(img);
        let grids = prepared.detect_grids();

        assert_eq!(grids.len(), 1, "exactly one QR code must be detectable");
        let (_meta, content) = grids[0].decode().expect("the code must decode");
        assert_eq!(
            content, link,
            "what a scanner reads must be the link we put in"
        );
    }

    #[test]
    fn a_link_with_a_recipe_query_survives_the_round_trip() {
        // Der Regelfall im Dialog ist der Kurz-Link, aber wer den vollen Link
        // wählt, bekommt Umlaute und Prozentkodierung in den Code. Auch das
        // muss ein Scanner unverändert zurückgeben.
        let link = "https://www.declarino.ch/lebensmittelrecht?v=2\
&product_subtitle=Bergk%C3%A4se&zutat=Rohmilch&herkunft=CH&producer_name=Hof%20M%C3%BCller";
        let qr = svg_path(link).expect("must encode");

        let img = render_to_image(&qr, 4);
        let mut prepared = rqrr::PreparedImage::prepare(img);
        let grids = prepared.detect_grids();

        assert_eq!(grids.len(), 1, "the code must be detectable");
        let (_meta, content) = grids[0].decode().expect("the code must decode");
        assert_eq!(content, link, "the query string must survive unchanged");
    }
}
