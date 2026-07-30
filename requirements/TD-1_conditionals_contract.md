# TD-1: Der Regel-Kontrakt `conditional_elements: HashMap<String, bool>`

Status: vollständig umgesetzt (Stufen 0–3 + Restschuld) · Datum: 2026-07-29 · Anlass: Analyse nach der DEC-Feedbackrunde

> **Umsetzungsstand** (Branch `td/conditionals-contract`):
> - Stufe 0 ✅ `conditional_invariants.rs` (Matrix-Invarianten + Konsumenten-Pinning), `is_bio_eingabe` entfernt
> - Stufe 1 ✅ `conditional_keys.rs`; kein Schlüssel existiert mehr als freier String
> - Stufe 2 ✅ `verdicts.rs` (BioVerdict/KnospeVerdict/CheckState); `decide_bio/decide_knospe/decide_check` in core.rs; `write_conditionals()` als einzige Urteil→Schlüssel-Stelle; `remove()`-Manöver entfällt; execute() 543→398 Zeilen
> - Stufe 3 ✅ `Output.verdicts` + `VerdictsContext`; `label_preview.rs` liest die Urteile (Logo/Badge, «Bio»-Suffix, Hinweis-Sektion als matches); von 15 `is_set`-Abfragen bleibt eine (`alternative_marking_allowed`)
>
> **Restschuld abgetragen** (Commits `701ecb0`, `578b46a`, `bcec6c0`):
> - `alternative_marking_allowed`, `namensgebende_zutat_input`, `manuelles_total_input` und `origin_required_indices` sind Felder auf `Verdicts`; `execute()` enthält keinen einzigen `conditionals.insert` mehr.
> - `ConditionalDisplay`, der `Conditionals`-Kontext und `is_set()` sind gelöscht; die UI liest ausschliesslich den `VerdictsContext`.
> - `Output` trägt nur noch `verdicts`. Die historische Schlüssel→Bool-Sicht existiert als `Output::conditionals()`, on demand abgeleitet und ausschliesslich von der Test-Suite benutzt (97 Assertions als bewusst erhaltenes Sicherheitsnetz).
> - `execute()`: 543 → 383 Zeilen. Zur Laufzeit existiert kein String-Schlüssel mehr.

## Befund

Die problematischste technische Schuld des Projekts ist nicht die Grösse einer
Datei, sondern ein Vertrag: Die gesamte fachliche Wahrheit des Regelwerks —
darf «Bio» in die Sachbezeichnung, welche Knospe erscheint, was sagt «Rezeptur
prüfen» — verlässt `Calculator::execute()` als **untypisierte
`HashMap<String, bool>`**, aufgebaut imperativ über 543 Zeilen mit
`insert`/`remove` an 32 Stellen.

Warum das die teuerste Schuld ist:

1. **Stiller Fehlermodus.** Ein Tippfehler in einem der ~312 String-Vorkommen
   (Produktion, UI, Tests) ist kein Compilefehler, sondern ein Conditional, das
   still `false` bleibt. Beleg: `is_bio_eingabe` wird seit je produziert und
   von niemandem gelesen — toter Output, den nie jemand bemerkt hat.
2. **Invarianten existieren nur als Kontrollfluss.** `bio_marketing_allowed`
   und `bio_marketing_not_allowed` schliessen sich fachlich aus; nichts im Typ
   verhindert, dass beide gesetzt sind. Der Mono-Umstellbetrieb-Zweig muss
   frühere Schlüsse per `conditionals.remove(...)` **zurücknehmen** — späterer
   Code editiert die Entscheidung früheren Codes, statt dass die Entscheidung
   an einer Stelle fällt. Der Tri-State-Block liest wiederum
   `conditionals.get("bio_marketing_allowed")` — Reihenfolge im Funktionskörper
   ist damit fachlich tragend, ohne dass irgendetwas sie absichert.
3. **Jede Regeländerung landet im selben Monolithen.** Alle vier fachlichen
   DEC-Fixes (DEC-4, 7, 8, 10) mussten in `execute()` operieren. Die Funktion
   ist mit 543 Zeilen die mit Abstand grösste des Projekts und wächst mit jedem
   Ticket; die Wechselwirkungen (Logo-Gate ↔ Prüftext, Suffix ↔ Umstellung)
   sind nur durch Kommentare («sonst widersprechen sich Logo und Prüftext»)
   und einzelne Tests dokumentiert.

Die 94 Test-Assertions auf `conditional_elements` und die UI-Seite
(`Conditionals::is_set`, `ConditionalDisplay`) hängen alle an denselben
Strings.

## Zielbild

`execute()` berechnet **typisierte Urteile** («verdicts»); die HashMap bleibt
als abgeleitete Kompatibilitätsschicht bestehen, erzeugt aus den Typen durch
eine erschöpfende Abbildung. Die Fachlogik kann dann nicht mehr
widersprüchlich sein, weil die Widersprüche nicht mehr darstellbar sind:

```rust
/// Ergebnis der Bio-V-Beurteilung — ein Wert, nicht vier unabhängige Bools.
pub enum BioVerdict {
    Allowed { umstellung_mono: bool },
    NotAllowed { reasons: Vec<BioBlockReason> },   // Ausnahme >5%, nicht deklariert, …
}

pub enum KnospeVerdict {
    Logo { variant: KnospeLogo },                  // Regular/NoCross × Umstellung
    NoLogo { reasons: Vec<KnospeBlockReason> },
}

pub enum CheckState { Pending, Ok, Failed }        // «Rezeptur prüfen», je Regime

pub struct Verdicts {
    pub bio: Option<BioVerdict>,
    pub knospe: Option<KnospeVerdict>,
    pub bio_check: Option<CheckState>,
    pub knospe_check: Option<CheckState>,
    // …
}
```

- `bio_suisse_regular`/`bio_suisse_no_cross` gleichzeitig: **nicht darstellbar**.
- `check_ok` und `check_failed` gleichzeitig: **nicht darstellbar**.
- Das `remove()`-Manöver entfällt: der Mono-Umstellbetrieb-Fall ist ein Zweig
  **in** der Entscheidung, kein Patch **nach** der Entscheidung.
- Der Tippfehler-Fehlermodus wandert vom Laufzeit-Schweigen in den Compiler.

## Migrationsplan (inkrementell, jede Stufe grün)

**Stufe 0 — Netz spannen (klein, sofort machbar)**
Invariantentest: `execute()` über eine Matrix von Rezepturen laufen lassen und
die Ausschluss-Paare (`*_allowed`/`*_not_allowed`, `check_*`, Logo-Varianten)
als Properties prüfen. Dazu die zwei bekannten Leichen aufräumen
(`is_bio_eingabe` entfernen). Sichert die Stufen 1–3 ab, bevor irgendetwas
umgebaut wird.

**Stufe 1 — Schlüssel zentralisieren (mechanisch)**
Ein Modul `conditional_keys` mit `pub const`-Namen; Produktion, UI und Tests
verwenden die Konstanten. Kein Verhalten geändert, aber ab hier ist jeder
Schlüssel referenzierbar, grep-bar und vom Compiler geprüft. (~312 Stellen,
skriptbar wie die bisherigen mechanischen Umbauten.)

**Stufe 2 — Urteile extrahieren, Map ableiten**
Je Verdict eine reine Funktion (`decide_bio(...) -> Option<BioVerdict>` usw.),
aus `execute()` herausgelöst; am Ende **eine** Funktion
`Verdicts::to_conditionals()` mit erschöpfendem `match`, die die bisherige
HashMap erzeugt. `execute()` schrumpft auf Orchestrierung; UI und alle 94
Assertions laufen unverändert weiter. Die bestehenden 304 Unit- und 54
E2E-Tests sind das Sicherheitsnetz — sie prüfen genau die Map bzw. das
gerenderte Resultat.

**Stufe 3 — UI auf Typen umstellen (optional, pro Komponente)**
`Output` bekommt zusätzlich `verdicts: Verdicts`; `label_preview.rs` liest die
Enums direkt (ein `match` statt acht `is_set`-Abfragen). Die HashMap bleibt für
`ConditionalDisplay`-Pfade, bis auch diese migriert sind, und kann dann fallen.

## Aufwand & Risiko

| Stufe | Aufwand | Risiko |
|---|---|---|
| 0 | ~1 h | keins (nur Tests + tote Zeile) |
| 1 | ~2 h | minimal, mechanisch, skriptbar |
| 2 | ~1 Tag | moderat; abgesichert durch Stufe 0 + volle Testsuite |
| 3 | inkrementell | gering, pro Komponente entscheidbar |

Abbruchsicher: Nach jeder Stufe ist der Zustand besser als davor und
eigenständig haltbar. Stufe 2 ist der eigentliche Gewinn; Stufe 3 ist Kür.

## Bewusst nicht gewählte Alternativen

- **Nur `execute()` in Unterfunktionen schneiden:** lindert die Länge, lässt
  aber den stillen String-Fehlermodus und die per-Hand-Invarianten bestehen.
- **Grosser Umbau in einem Schritt:** verwirft das dichteste Testnetz des
  Projekts (94 Map-Assertions) genau in dem Moment, in dem man es braucht.
