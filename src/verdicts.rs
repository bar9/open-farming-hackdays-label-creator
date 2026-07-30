//! TD-1 Stufe 2: typisierte Urteile («Verdicts») des Regelwerks.
//!
//! Die fachlichen Entscheidungen — darf «Bio» in die Sachbezeichnung, welche
//! Knospe erscheint, was sagt «Rezeptur prüfen» — sind hier Werte, keine
//! unabhängigen Bool-Flags. Damit sind die Widersprüche, die der alte
//! `HashMap`-Kontrakt zuliess (allowed **und** not_allowed, ok **und**
//! failed, beide Logo-Varianten), schlicht nicht mehr darstellbar.
//!
//! Die UI liest die Urteile direkt (`VerdictsContext`). Für die Test-Suite
//! bildet `write_conditionals()` sie auf den historischen Schlüssel→Bool-
//! Kontrakt ab (`Output::conditionals()`) — die 90+ Assertions darauf sind
//! das dichteste Sicherheitsnetz des Projekts und bleiben bewusst bestehen
//! (siehe `requirements/TD-1_conditionals_contract.md`).

use crate::conditional_keys as keys;
use std::collections::HashMap;

/// Warum «Bio» in der Sachbezeichnung verweigert wird. Jede Variante hat
/// einen eigenen Hinweistext im Label-Preview.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BioBlockReason {
    /// Bio-CH-Anteil unter 95% (der generische Grund).
    ShareBelow95,
    /// DEC-7: nicht-bio Zutat ohne das Häkchen «Erlaubte nicht-biologische Zutat».
    UndeclaredNonBio,
    /// Erlaubte nicht-bio Zutaten über der 5%-Grenze (Anhang 3 WBF).
    ExceptionOver5Percent,
    /// Zusammengesetztes Produkt mit Umstellbetrieb-Zutat (Excel Zeile 7).
    CompositeUmstellung,
    /// Keine landwirtschaftliche Zutat — nichts zu zertifizieren (DEC-2).
    NothingToCertify,
}

/// Bio-V-Urteil: genau eines von beiden, nie beides.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BioVerdict {
    /// «Bio» darf in die Sachbezeichnung. `umstellung_mono` trägt den
    /// Pflicht-Umstellungshinweis (Monoprodukt aus Umstellbetrieb).
    Allowed { umstellung_mono: bool },
    /// Kein «Bio». Die Gründe speisen die konkreten Hinweistexte;
    /// `ShareBelow95` ist immer mindestens dabei, spezifischere Gründe kommen dazu.
    NotAllowed { reasons: Vec<BioBlockReason> },
}

/// Welche Knospe auf der Etikette erscheint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KnospeLogo {
    /// Mit Schweizerkreuz (≥ 90% Schweizer Anteil der zertifizierten Ware).
    pub swiss_cross: bool,
    /// Umstellungsknospe-Artwork inkl. Umstellungssatz.
    pub umstellung: bool,
}

/// Warum keine Knospe erscheint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KnospeBlockReason {
    /// Nicht alle landwirtschaftlichen Zutaten sind Knospe-konform.
    NotFullyCertified,
    /// DEC-8: erlaubte nicht-bio Zutaten über der 5%-Grenze.
    ExceptionOver5Percent,
    /// Leere Rezeptur oder nichts Landwirtschaftliches (DEC-2).
    NothingToCertify,
}

/// Knospe-Urteil: Logo mit Variante, oder begründetes Nein.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KnospeVerdict {
    /// Knospe-fähig. `bio_suffix` = « Bio» an der Sachbezeichnung (DEC-10;
    /// entfällt für zusammengesetzte Umstellungsprodukte).
    Logo { logo: KnospeLogo, bio_suffix: bool },
    NoLogo { reasons: Vec<KnospeBlockReason> },
}

/// Tri-State des «Rezeptur prüfen»-Buttons. Bei Einzelzutat-Modus (DEC-3)
/// gibt es gar kein Urteil (`None` auf `Verdicts`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckState {
    /// Noch nicht geprüft.
    Pending,
    /// Geprüft und erfüllt.
    Ok,
    /// Geprüft und nicht erfüllt (oder Rezeptur-Fehler offen).
    Failed,
}

/// Gesammelte Urteile eines `execute()`-Laufs. `None` heisst: die zugehörige
/// Regel ist in dieser Konfiguration nicht aktiv.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Verdicts {
    pub bio: Option<BioVerdict>,
    pub knospe: Option<KnospeVerdict>,
    pub bio_check: Option<CheckState>,
    pub knospe_check: Option<CheckState>,
    /// DEC-4: die pauschalen Kennzeichnungsvarianten sind nur zulässig, wenn
    /// keine erlaubte nicht-biologische Ausnahme in der Rezeptur ist.
    pub alternative_marking_allowed: bool,
    /// AP1.3: Eingabefeld für die namensgebende Zutat anzeigen.
    pub namensgebende_zutat_input: bool,
    /// AP1.4: manuelles Total-Eingabefeld anzeigen.
    pub manuelles_total_input: bool,
    /// AP7.1/Fleisch: Indizes der Zutaten, die eine Herkunftsangabe brauchen.
    /// Leer = keine. Ersetzt die dynamische `herkunft_benoetigt_{i}`-Familie.
    pub origin_required_indices: Vec<usize>,
}

impl Verdicts {
    /// Bildet die Urteile auf den historischen Schlüssel→Bool-Kontrakt ab.
    ///
    /// Einziger Konsument ist `Output::conditionals()`, also die Test-Suite;
    /// zur Laufzeit liest niemand mehr Schlüssel. Das ist die EINZIGE Stelle,
    /// die Urteil → Schlüssel übersetzt; die Ausschluss-Invarianten (nie
    /// allowed und not_allowed zugleich usw.) folgen aus der Enum-Struktur
    /// statt aus Kontrollfluss-Disziplin.
    pub fn write_conditionals(&self, conditionals: &mut HashMap<String, bool>) {
        match &self.bio {
            None => {}
            Some(BioVerdict::Allowed { umstellung_mono }) => {
                conditionals.insert(keys::BIO_SACHBEZEICHNUNG_SUFFIX.to_string(), true);
                conditionals.insert(keys::BIO_MARKETING_ALLOWED.to_string(), true);
                if *umstellung_mono {
                    conditionals.insert(keys::UMSTELLBETRIEB_HINWEIS.to_string(), true);
                }
            }
            Some(BioVerdict::NotAllowed { reasons }) => {
                conditionals.insert(keys::BIO_MARKETING_NOT_ALLOWED.to_string(), true);
                for reason in reasons {
                    match reason {
                        BioBlockReason::UndeclaredNonBio => {
                            conditionals
                                .insert(keys::BIO_NICHT_DEKLARIERTE_ZUTAT.to_string(), true);
                        }
                        BioBlockReason::ExceptionOver5Percent => {
                            conditionals.insert(
                                keys::BIO_ERLAUBTE_AUSNAHME_UEBER_5_PROZENT.to_string(),
                                true,
                            );
                        }
                        // Der generische Grund und die strukturellen Fälle haben
                        // keinen eigenen Schlüssel; sie äussern sich nur im
                        // "not allowed"-Text.
                        BioBlockReason::ShareBelow95
                        | BioBlockReason::CompositeUmstellung
                        | BioBlockReason::NothingToCertify => {}
                    }
                }
            }
        }

        match &self.knospe {
            None => {}
            Some(KnospeVerdict::Logo { logo, bio_suffix }) => {
                conditionals.insert(keys::KNOSPE_MARKETING_ALLOWED.to_string(), true);
                let variant_key = if logo.swiss_cross {
                    keys::BIO_SUISSE_REGULAR
                } else {
                    keys::BIO_SUISSE_NO_CROSS
                };
                conditionals.insert(variant_key.to_string(), true);
                if logo.umstellung {
                    conditionals.insert(keys::KNOSPE_UMSTELLUNG_LOGO.to_string(), true);
                }
                if *bio_suffix {
                    conditionals.insert(keys::BIO_SACHBEZEICHNUNG_SUFFIX.to_string(), true);
                }
            }
            Some(KnospeVerdict::NoLogo { reasons }) => {
                conditionals.insert(keys::KNOSPE_MARKETING_NOT_ALLOWED.to_string(), true);
                if reasons.contains(&KnospeBlockReason::ExceptionOver5Percent) {
                    conditionals.insert(
                        keys::KNOSPE_ERLAUBTE_AUSNAHME_UEBER_5_PROZENT.to_string(),
                        true,
                    );
                }
            }
        }

        if let Some(state) = self.bio_check {
            let key = match state {
                CheckState::Pending => keys::BIO_CHECK_PENDING,
                CheckState::Ok => keys::BIO_CHECK_OK,
                CheckState::Failed => keys::BIO_CHECK_FAILED,
            };
            conditionals.insert(key.to_string(), true);
        }
        if let Some(state) = self.knospe_check {
            let key = match state {
                CheckState::Pending => keys::KNOSPE_CHECK_PENDING,
                CheckState::Ok => keys::KNOSPE_CHECK_OK,
                CheckState::Failed => keys::KNOSPE_CHECK_FAILED,
            };
            conditionals.insert(key.to_string(), true);
        }

        if self.alternative_marking_allowed {
            conditionals.insert(keys::ALTERNATIVE_MARKING_ALLOWED.to_string(), true);
        }
        if self.namensgebende_zutat_input {
            conditionals.insert(keys::NAMENSGEBENDE_ZUTAT.to_string(), true);
        }
        if self.manuelles_total_input {
            conditionals.insert(keys::MANUELLES_TOTAL.to_string(), true);
        }
        for &index in &self.origin_required_indices {
            conditionals.insert(keys::herkunft_benoetigt(index), true);
        }
        if !self.origin_required_indices.is_empty() {
            conditionals.insert(keys::HERKUNFT_BENOETIGT_UEBER_50_PROZENT.to_string(), true);
        }
    }
}
