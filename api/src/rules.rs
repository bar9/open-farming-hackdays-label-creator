//! Fachregeln des Kurz-Link-Dienstes: Allowlists, CORS-Herkünfte und die
//! Ableitung des Codes aus der Ziel-URL.
//!
//! Warum es diesen Dienst gibt: Fremd-Shortener sind für Declarino
//! unbrauchbar. Provider sperren die Kategorie per DNS, weil ein Dienst, der
//! ohne Konto beliebige Ziele kürzt, das Standardwerkzeug für Phishing-Links
//! ist; andere zeigen Warn-Zwischenseiten oder verlangen einen API-Key, der im
//! WASM-Frontend auslesbar wäre. Ein Kurz-Link unter declarino.ch steht auf
//! keiner solchen Liste, weil die Domain eine eigene Reputation hat. Details
//! und Messwerte in api/README.md.

use sha2::{Digest, Sha256};

/// Basis der Kurz-Links. Immer Produktion: ein in Staging oder lokal
/// erzeugter Link soll auch verschickt funktionieren.
pub const SHORT_BASE: &str = "https://www.declarino.ch";

/// Obergrenze für die Ziel-URL. Ein volles Rezept liegt bei ~4.5 KB; 16 KB
/// lassen viel Luft und verhindern trotzdem, dass jemand den Speicher mit
/// Riesen-URLs füllt. Gezählt in UTF-16-Einheiten wie `String.length` im
/// früheren JavaScript-Dienst, damit die Grenze exakt gleich bleibt.
pub const MAX_URL_LENGTH: usize = 16 * 1024;

/// Länge des Kurz-Codes, und wie weit er bei einer Kollision wachsen darf.
pub const CODE_LENGTH: usize = 7;
pub const MAX_CODE_LENGTH: usize = 12;

/// Hosts, deren URLs gekürzt werden dürfen.
///
/// Diese Liste ist der Missbrauchsschutz: Wer keine fremden Ziele hinterlegen
/// kann, kann den Endpunkt nicht als Phishing-Werkzeug verwenden. Genau das
/// hält declarino.ch von den Sperrlisten fern, an denen die Fremddienste
/// gescheitert sind, und begrenzt zugleich das Datenwachstum.
const ALLOWED_TARGET_HOSTS: &[&str] = &[
    "www.declarino.ch",
    "declarino.ch",
    "bar9.github.io", // Staging
];

/// Herkünfte, die den Endpunkt per Browser aufrufen dürfen.
/// Staging und die lokale Entwicklung müssen mit, sonst ist der Teilen-Button
/// ausserhalb der Produktion tot.
const ALLOWED_ORIGINS: &[&str] = &[
    "https://www.declarino.ch",
    "https://declarino.ch",
    "https://bar9.github.io",
];

/// Ob eine Origin erlaubt ist. localhost/127.0.0.1 mit beliebigem Port gilt
/// als Entwicklung (entspricht `^https?://(localhost|127\.0\.0\.1)(:\d+)?$`).
pub fn is_allowed_origin(origin: &str) -> bool {
    if ALLOWED_ORIGINS.contains(&origin) {
        return true;
    }
    let rest = origin
        .strip_prefix("http://")
        .or_else(|| origin.strip_prefix("https://"));
    let Some(rest) = rest else { return false };
    let port = rest
        .strip_prefix("localhost")
        .or_else(|| rest.strip_prefix("127.0.0.1"));
    match port {
        Some("") => true,
        Some(p) => p
            .strip_prefix(':')
            .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())),
        None => false,
    }
}

/// Prüft, ob `url` auf eine erlaubte Declarino-Adresse zeigt.
pub fn is_allowed_target(url: &str) -> bool {
    let Ok(parsed) = url::Url::parse(url) else {
        return false;
    };
    if parsed.scheme() != "https" && parsed.scheme() != "http" {
        return false;
    }
    parsed
        .host_str()
        .is_some_and(|host| ALLOWED_TARGET_HOSTS.contains(&host))
}

/// Länge wie `String.length` in JavaScript (UTF-16-Einheiten).
pub fn js_length(s: &str) -> usize {
    s.encode_utf16().count()
}

/// Erlaubte Codeform. Grosszügiger als die vergebenen 7 bis 12 Zeichen, damit
/// eine spätere Längenänderung alte Links nicht ungültig macht; alles andere
/// hält Unfug wie Pfad-Traversal aus dem Datenbankschlüssel heraus.
pub fn is_valid_code(code: &str) -> bool {
    (4..=16).contains(&code.len()) && code.bytes().all(|b| b.is_ascii_alphanumeric())
}

/// Base62-Alphabet: Kurz-Codes bleiben so per Doppelklick markierbar und
/// lassen sich am Telefon vorlesen.
const BASE62: &[u8; 62] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// Code für eine URL: die ersten 64 Bit ihres SHA-256 in Base62.
///
/// Bewusst deterministisch statt zufällig: Wer dasselbe Rezept zweimal teilt,
/// bekommt denselben Link, statt einen zweiten Eintrag anzulegen. Das hält
/// den Datenbestand klein und macht die Funktion wiederholbar.
///
/// Bitgenau identisch mit dem früheren JavaScript-Dienst, sonst bekäme eine
/// schon einmal gekürzte URL nach der Migration einen zweiten Code. Die Tests
/// halten das mit Werten aus der alten Implementierung fest.
///
/// 7 Zeichen sind rund 3.5e12 Möglichkeiten. Kollisionen werden beim
/// Schreiben trotzdem erkannt (siehe `app::shorten`), nicht bloss angenommen.
pub fn code_for_url(url: &str, length: usize) -> String {
    let digest = Sha256::digest(url.as_bytes());
    let mut value = u64::from_be_bytes(digest[..8].try_into().expect("8 Bytes"));
    let mut code = vec![0u8; length];
    for slot in code.iter_mut().rev() {
        *slot = BASE62[(value % 62) as usize];
        value /= 62;
    }
    String::from_utf8(code).expect("Base62 ist ASCII")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Werte aus `codeForUrl` der früheren JavaScript-Implementierung.
    #[test]
    fn code_matches_former_javascript_implementation() {
        let cases = [
            ("https://www.declarino.ch/", "ysoSiHG", "0fO6oysoSiHG"),
            (
                "https://www.declarino.ch/lebensmittelrecht?v=2&product_title=Brownies",
                "qgJ7NCs",
                "0gcajqgJ7NCs",
            ),
            ("a", "NODIHBo", "0hoogNODIHBo"),
            ("b", "hMcMKtc", "05kLOhMcMKtc"),
            (
                "https://bar9.github.io/open-farming-hackdays-label-creator/?v=2",
                "IqnmJJR",
                "046fGIqnmJJR",
            ),
        ];
        for (url, seven, twelve) in cases {
            assert_eq!(code_for_url(url, 7), seven, "{url}");
            assert_eq!(code_for_url(url, 12), twelve, "{url}");
        }
    }

    #[test]
    fn origins() {
        for ok in [
            "https://www.declarino.ch",
            "https://declarino.ch",
            "https://bar9.github.io",
            "http://localhost:8080",
            "http://localhost",
            "https://127.0.0.1:3000",
        ] {
            assert!(is_allowed_origin(ok), "{ok}");
        }
        for bad in [
            "https://evil.example",
            "http://localhost.evil.com",
            "http://localhost:",
            "http://localhost:80a",
            "ftp://localhost",
            "https://www.declarino.ch.evil.com",
            "",
        ] {
            assert!(!is_allowed_origin(bad), "{bad}");
        }
    }

    #[test]
    fn targets() {
        assert!(is_allowed_target("https://www.declarino.ch/x"));
        assert!(is_allowed_target("http://declarino.ch/"));
        assert!(is_allowed_target("https://WWW.DECLARINO.CH/x"));
        for bad in [
            "https://example.com/phishing",
            "https://www.declarino.ch.evil.com/x",
            "javascript:alert(1)",
            "notaurl",
            "ftp://www.declarino.ch/",
        ] {
            assert!(!is_allowed_target(bad), "{bad}");
        }
    }

    #[test]
    fn codes() {
        assert!(is_valid_code("UWKJ6Pe"));
        assert!(is_valid_code("abcd"));
        assert!(!is_valid_code("zz"));
        assert!(!is_valid_code("../etc/passwd"));
        assert!(!is_valid_code(""));
        assert!(!is_valid_code("abcdefghijklmnopq"));
    }

    #[test]
    fn length_counts_utf16_like_javascript() {
        assert_eq!(js_length("abc"), 3);
        assert_eq!(js_length("ä"), 1);
        assert_eq!(js_length("😀"), 2);
    }
}
