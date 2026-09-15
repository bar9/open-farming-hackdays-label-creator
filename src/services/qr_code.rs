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
}
