//! TD-1 Stufe 1: die Namen aller Conditional-Schlüssel an einer Stelle.
//!
//! Die Conditionals verlassen `Calculator::execute()` als
//! `HashMap<String, bool>` und werden von UI und Tests über dieselben
//! String-Schlüssel gelesen. Ein Tippfehler auf einer der Seiten ist kein
//! Compilefehler, sondern ein Conditional, das still `false` bleibt (so ist
//! `is_bio_eingabe` jahrelang produziert und nie gelesen worden).
//!
//! Bis der Kontrakt typisiert ist (TD-1 Stufe 2, siehe
//! `requirements/TD-1_conditionals_contract.md`) sind diese Konstanten die
//! Absicherung: Produktion, UI und Tests referenzieren denselben Namen, und
//! ein toter oder falsch geschriebener Schlüssel fällt beim Kompilieren bzw.
//! in `conditional_invariants.rs` auf.

/// DEC-4: die pauschalen Kennzeichnungsvarianten («Alle landwirtschaftlichen
/// Zutaten…», «Bio-»-Präfix) sind nur zulässig, wenn keine erlaubte
/// nicht-biologische Ausnahme in der Rezeptur ist.
pub const ALTERNATIVE_MARKING_ALLOWED: &str = "alternative_marking_allowed";

/// AP1.3: Eingabefeld für die namensgebende Zutat anzeigen.
pub const NAMENSGEBENDE_ZUTAT: &str = "namensgebende_zutat";

/// AP1.4: manuelles Total-Eingabefeld anzeigen.
pub const MANUELLES_TOTAL: &str = "manuelles_total";

// --- Knospe: Logo und Vermarktung -----------------------------------------

/// Knospe mit Schweizerkreuz (≥ 90% Schweizer Anteil).
pub const BIO_SUISSE_REGULAR: &str = "bio_suisse_regular";
/// Knospe ohne Schweizerkreuz (< 90% Schweizer Anteil).
pub const BIO_SUISSE_NO_CROSS: &str = "bio_suisse_no_cross";
/// Artwork-Variante Umstellungsknospe (mit Umstellungssatz daneben).
pub const KNOSPE_UMSTELLUNG_LOGO: &str = "knospe_umstellung_logo";
/// Produkt darf mit der Knospe vermarktet werden.
pub const KNOSPE_MARKETING_ALLOWED: &str = "knospe_marketing_allowed";
/// Produkt darf NICHT mit der Knospe vermarktet werden.
pub const KNOSPE_MARKETING_NOT_ALLOWED: &str = "knospe_marketing_not_allowed";
/// DEC-8: erlaubte nicht-bio Zutaten über der 5%-Grenze — konkreter Grund.
pub const KNOSPE_ERLAUBTE_AUSNAHME_UEBER_5_PROZENT: &str =
    "knospe_erlaubte_ausnahme_ueber_5_prozent";

/// Tri-State «Rezeptur prüfen» (Knospe).
pub const KNOSPE_CHECK_PENDING: &str = "knospe_check_pending";
pub const KNOSPE_CHECK_OK: &str = "knospe_check_ok";
pub const KNOSPE_CHECK_FAILED: &str = "knospe_check_failed";

// --- Bio-V: Sachbezeichnung und Vermarktung --------------------------------

/// « Bio» wird an die Sachbezeichnung angehängt.
pub const BIO_SACHBEZEICHNUNG_SUFFIX: &str = "bio_sachbezeichnung_suffix";
/// Produkt darf als Bio vermarktet werden.
pub const BIO_MARKETING_ALLOWED: &str = "bio_marketing_allowed";
/// Produkt darf NICHT als Bio vermarktet werden.
pub const BIO_MARKETING_NOT_ALLOWED: &str = "bio_marketing_not_allowed";
/// DEC-7: nicht-bio Zutat ohne Ausnahme-Häkchen — konkreter Grund.
pub const BIO_NICHT_DEKLARIERTE_ZUTAT: &str = "bio_nicht_deklarierte_zutat";
/// Erlaubte nicht-bio Zutaten über der 5%-Grenze — konkreter Grund.
pub const BIO_ERLAUBTE_AUSNAHME_UEBER_5_PROZENT: &str =
    "bio_erlaubte_ausnahme_ueber_5_prozent";
/// Monoprodukt aus Umstellbetrieb: Pflicht-Umstellungshinweis.
pub const UMSTELLBETRIEB_HINWEIS: &str = "umstellbetrieb_hinweis";

/// Tri-State «Rezeptur prüfen» (Bio-V).
pub const BIO_CHECK_PENDING: &str = "bio_check_pending";
pub const BIO_CHECK_OK: &str = "bio_check_ok";
pub const BIO_CHECK_FAILED: &str = "bio_check_failed";

// --- Herkunft ---------------------------------------------------------------

/// Mindestens eine Zutat über 50% braucht eine Herkunftsangabe.
pub const HERKUNFT_BENOETIGT_UEBER_50_PROZENT: &str =
    "herkunft_benoetigt_ueber_50_prozent";

/// Präfix der pro-Zutat-Flags; `herkunft_benoetigt()` hängt den Index an.
pub const HERKUNFT_BENOETIGT_PREFIX: &str = "herkunft_benoetigt_";

/// Pro-Zutat-Flag: Zutat an Index `i` braucht eine Herkunftsangabe.
/// Der einzige dynamisch gebildete Schlüssel des Kontrakts.
pub fn herkunft_benoetigt(index: usize) -> String {
    format!("{HERKUNFT_BENOETIGT_PREFIX}{index}")
}
