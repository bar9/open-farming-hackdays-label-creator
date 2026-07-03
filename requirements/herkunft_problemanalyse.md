# Herkunft-Problemanalyse (IST/SOLL) — zur fachlichen Verifikation durch Mirjam

Stand: 03.07.2026, nach Umsetzung des Testing-Feedbacks vom 25.06.2026.
Zweck: Alle Regeln rund um **Herkunftsangaben** (Anzeige auf der Etikette und
Validierung/Fehlermeldungen) an einem Ort dokumentieren, offene Fragen markieren
und fachlich verifizieren lassen. Referenz-Instanz mit dem Stand vom Januar
(v0.8.0) wird separat bereitgestellt.

## 1. Anzeige der Herkunft auf der Etikette (IST nach Umsetzung)

### 1.1 Knospe-Konfiguration — abhängig vom CH-Anteil der landw. Zutaten

| CH-Anteil | Regel | Anzeige |
|---|---|---|
| 100 % | Rule A | Keine Herkunftsangaben |
| 90–99.9 % | Rule B | «(CH)» nur bei landwirtschaftlichen CH-Zutaten |
| < 90 % | Rule C | Zutatenspezifische Regeln (siehe 1.2) |

### 1.2 Knospe < 90 % CH — zutatenspezifische Anzeige (Excel-Referenztabelle)

Herkunft wird angezeigt für:
- Monoprodukte: immer
- Namensgebende Zutaten: immer
- Pflanzliche Zutaten > 50 % Anteil
- Eier / Honig / Fisch & Aquakultur > 10 % Anteil
- Milch & Milchprodukte / Fleisch / Insekten: immer
- Landw. CH-Zutaten ≥ 10 % Anteil (unabhängig von Kategorie)

**Neu seit 25.06-Feedback:** Die Kategorie einer Zutat wird nicht mehr nur von
der BLV-API übernommen, sondern fällt auf eine kuratierte Kategorie-Spalte in
der lokalen Zutaten-DB (`src/food_db.csv`) zurück. Damit zeigt z.B. **Butter**
(lokale DB, Alias → Kochbutter) neu korrekt «(CH)» — vorher zeigte nur Milch
(BLV-Kategorie) die Herkunft. **Bisher kuratiert: nur die Milchprodukt-Familie.**

> **Entscheid Roland 03.07.2026:** Milch**pulver** (Vollmilchpulver,
> Magermilchpulver) sind bewusst NICHT als Milchprodukte kuratiert, damit die
> Excel-Referenz (Vollmilchpulver 4.6 % als Unterzutat: «Nein») weiterhin gilt.
> **→ Frage an Mirjam:** Ist das fachlich richtig, oder gilt «Milch und
> Milchprodukte immer» auch für Pulver?

> **Frage an Mirjam:** Sollen weitere Familien kuratiert werden (Fleisch,
> Eier, Honig, Fisch), damit auch lokal erfasste Zutaten dieser Familien die
> «immer/10 %»-Regeln auslösen? Kandidaten-Spalte findet sich evtl. im
> `Inhaltsverzeichnis_Bio_Zusatz.xlsx` («Herkunftsangabe bei Importknospe benötigt»).

### 1.3 Zusammengesetzte Zutaten

- Herkunft wird auf genau **einer** Ebene deklariert (Einzel-Ebenen-Regel,
  Konflikt wird validiert).
- Deklarieren die **Unterzutaten** die Herkunft: Anzeige bei den Unterzutaten
  (tiefste Ebene), Eltern-Zutat ohne Angabe.
- **Neu (Bugfix 25.06-Feedback):** Deklariert die **Ober-Zutat** die Herkunft
  (top-down), erscheint sie neu nach der Klammer der Unterzutaten —
  z.B. «Himbeerstreusel (Himbeere, Zucker) (CH)». Vorher ging die Angabe
  stillschweigend verloren (BioVo-Bug aus dem Testing).

### 1.4 Alle Konfigurationen

- Die generischen Platzhalter `Import` und `NoOriginRequired` erscheinen nie
  als Text auf der Etikette.
- LMR: Herkunft > 50 % (AP7.1), Fleisch > 20 % (AP7.3), Rind-Details (AP7.4),
  Fisch-Fangort (AP7.5) — unverändert.

## 2. Validierung («Rezeptur vollständig — überprüfen»)

### 2.1 Neu umgesetzte Regel (25.06-Feedback, «roter Text»)

Fehlermeldung «Herkunftsland ist erforderlich …» erscheint **genau dann**, wenn:
1. bei der Zutat **Import-Knospe** oder **Import-Umstellungsknospe** gewählt ist,
2. **kein** konkretes Herkunftsland erfasst ist (nur der generische
   Import-Platzhalter), und
3. die Etikette die **Import-Knospe** zeigt (100 % Knospe, aber < 90 % CH-Anteil).

Zusätzlich:
- **Nicht-landwirtschaftliche Zutaten** (z.B. Dicarbonat) verlangen **nie** eine
  Herkunft (Fehlverhalten aus dem Testing behoben).
- Zutaten ohne Knospe-Auswahl werden von dieser Regel nicht mehr geflaggt
  (vorher: «alle Zutaten brauchen Herkunft»).
- Die Knospe-<90 %-Detailvalidierung (Excel-Tabelle, 1.2) bleibt unverändert
  bestehen.

> **Annahme (zu verifizieren):** Zeigt die Etikette die **CH-Knospe** (≥ 90 %
> CH), gibt es **keinen** roten Fehler für eine Import-Knospe-Zutat ohne Land.
> Ist das richtig, oder soll der Fehler unabhängig vom Logo erscheinen?

> **Offene Frage (Handnotiz 4, unleserlich):** «Importknospe ?? CH > 10 % muss
> Herkunft gegeben werden» — vermutlich die Anzeige-Regel «landw. CH-Zutaten
> ≥ 10 % → Herkunft Schweiz deklarieren» (bereits umgesetzt als Anzeige,
> Abschnitt 1.2 letzter Punkt). **Braucht es dafür zusätzlich eine
> Validierungsmeldung**, wenn eine CH-Zutat ≥ 10 % keine Herkunft erfasst hat
> und die Import-Knospe gezeigt wird?

### 2.2 Zertifizierungsstelle

Nicht mehr Teil der Rezeptur-Prüfung (rotes Panel). Der gelbe Platzhalter
«Bio-Zertifizierungsstelle» auf der Etiketten-Vorschau bleibt, bis eine gültige
Nummer (`CH-BIO-…`) erfasst ist. Eine fehlende Zertifizierungsstelle blockiert
den blauen «Rezeptur OK»-Text **nicht** (Annahme, zu verifizieren).

## 3. Bekannte Grenzen / bewusst nicht automatisiert

- Für Zutaten, die weder eine BLV-Kategorie noch eine kuratierte Kategorie
  haben, können die «immer»-Regeln (1.2) nicht automatisch greifen — die
  RL-Vorgaben sind nicht für beliebige Freitext-Zutaten automatisierbar
  (Feststellung aus der Sitzung vom 25.06).
- Ausländische landw. Zutaten ohne erkannte Kategorie unter 50 % werden bewusst
  nicht angezeigt (Excel-Referenz, z.B. Mandeln (TR), Zucker (PE)).

## 4. Referenzen

- Testing-Notizen: `requirements/Testing Declarino - Notizen aus Besprechung 25.06.2026.docx`
- Excel-Referenz Kategorien/Herkunft: `requirements/Inhaltsverzeichnis_Bio_Zusatz.xlsx`
- Januar-Vergleichsinstanz: v0.8.0 (Commit `9a0113f`), Deployment separat.
- Implementierung: `src/core.rs` (`should_show_origin_knospe_under90`,
  `validate_import_knospe_origin`, `format_origin_for_knospe_rules`),
  `src/food_db.csv` (Spalte `category`).
