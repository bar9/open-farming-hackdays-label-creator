# Herkunft-Problemanalyse (IST/SOLL) — zur fachlichen Verifikation durch Mirjam

Stand: 03.07.2026, nach Umsetzung des Testing-Feedbacks vom 25.06.2026.
Zweck: Alle Regeln rund um **Herkunftsangaben** (Anzeige auf der Etikette und
Validierung/Fehlermeldungen) an einem Ort dokumentieren, offene Fragen markieren
und fachlich verifizieren lassen. Referenz-Instanz mit dem Stand vom Januar
(v0.8.0) wird separat bereitgestellt.

**Terminologie:** Die Fallbezeichnungen A)/B)/C) stammen wörtlich aus dem
`Inhaltsverzeichnis_Bio_Zusatz.xlsx`, Blatt «Inhaltsverzeichnis». Dort gibt es
**zwei getrennte** Aufzählungen, die nicht verwechselt werden dürfen:
- **Zeile 8 «Herkunft der Rohstoffe (Knospe)»** — Typ *Validierungsregel* —
  Fälle A/B/C für die **Herkunftsangabe in der Zutatenliste** (Abschnitt 1.1/1.2).
- **Zeile 9 «Knospe-Logo»** — Typ *Ausgaberegel* — Fälle A/B/C/D für die
  **Logo-Wahl** (Abschnitt 1.5).

## 1. Anzeige der Herkunft auf der Etikette (IST nach Umsetzung)

### 1.1 «Herkunft der Rohstoffe (Knospe)» — Excel Zeile 8, Fälle A/B/C

Excel-Wortlaut (gekürzt) und Implementierungsstatus:

| Fall | Excel-Wortlaut | Implementiert |
|---|---|---|
| **A)** | «Falls 100% der landw. Zutaten aus CH: Herkunft der Zutaten muss nicht angegeben werden.» | ✓ keine Herkunftsangaben |
| **B)** | «Falls mind. 90%–99.99% der landw. Zutaten aus CH: Herkunft der CH-Zutaten angeben in Klammern nach Zutat ("Schweiz" oder "CH")» | ✓ «(CH)» bei landw. CH-Zutaten |
| **C)** | «Bei mehr als 10% landw. Zutaten nicht aus der Schweiz, muss in folgenden Fällen Herkunft einer Zutat angegeben werden: …» (Liste siehe 1.2) | ✓ zutatenspezifisch (1.2) |

> **Frage an Mirjam (steht so bereits im Excel, Fall B):** «Stimmt das? Muss
> Herkunft der anderen Zutaten nicht angegeben werden?» — d.h. ob im Fall B
> ausländische Zutaten wirklich ohne Angabe bleiben. Aktuell implementiert:
> nur CH-Zutaten zeigen «(CH)».

### 1.2 Fall C im Detail — zutatenspezifische Liste (Excel Zeile 8)

Excel-Liste 1:1 (Herkunft angeben für):
- «Schweizer Zutat mit mind. 10% Anteil» → ✓ implementiert (≥ 10 %)
- «pflanzliche Zutat mit mehr als 50% Anteil» → ✓ implementiert (> 50 %)
- «Eier/Honig/Fisch/andere Aquakulturen mit mehr als 10% Anteil» → ✓ implementiert (> 10 %)
- «Monoprodukten» → ✓ implementiert (immer)
- «Milch/Milchprodukte/Fleisch(erzeugnisse)/Insekten(erzeugnisse)» → ✓ implementiert (immer, ohne Schwellenwert)

**Zusätzlich implementiert, NICHT aus der Excel-C-Liste:** Namensgebende
Zutaten zeigen immer die Herkunft. Quelle ist der LIV-Kommentar in derselben
Excel-Zeile (Spalte H, Art. 15/16 LIV: Herkunft der namens-/wertgebenden
Zutat, gilt für alle Qualitäten).

> **Frage an Mirjam (steht so bereits im Excel, Fall C):** «gilt das auch bei
> solchen Zutaten, oder nur wenn das ganze Produkt zB ein Milchprodukt ist?»
> — Implementiert ist die Auslegung **pro Zutat** (jede Milch-/Fleisch-Zutat
> zeigt Herkunft). Diese Frage hängt direkt mit der Milchpulver-Frage unten
> zusammen.

**Formatabweichung:** Das Excel erlaubt Herkunft «ausgeschrieben (Paraguay)
oder als Ländercode (ISO-3166 Alpha-2: PY)», Kommentar: «Die Wahl lassen.»
Implementiert ist derzeit **nur der Ländercode** — Wahlmöglichkeit fehlt.

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

### 1.5 «Knospe-Logo» — Excel Zeile 9, Fälle A/B/C/D (Ausgaberegel)

| Fall | Excel-Wortlaut | Implementiert |
|---|---|---|
| **A)** | «Produkte aus mind. 90% Schweizer Knospe-Rohstoffe: Knospe-Logo mit Schweizer Flagge.» | ✓ |
| **B)** | «Produkte mit mehr als 10% ausländischen Knospe-Zutaten: Knospe-Logo ohne Schweizer Flagge.» | ✓ |
| **C)** | «Produkte mit Zutaten aus Knospe-Umstellungsbetrieben: Umstellungsknospe-Logo, mit oder ohne Schweizer Flagge, gleiche Logik wie A) & B). Direkt neben Logo: Hinweis: "Hergestellt im Rahmen der Umstellung auf die biologische Landwirtschaft."» | ✓ **neu seit 25.06-Feedback** (offizielle Bio-Suisse-Logos, Satz links vom Logo, dreisprachig) |
| **D)** | «Für verarbeitete Produkte, die nicht vollständig den Bio Suisse-Verarbeitungsrichtlinien entsprechen (z.B. Rohstoff nach Bio-CH anstatt Knospe, oder nicht erlaubter Verarbeitungsschritt): Umstellungsknospe mit Hinweis "Hergestellt im Rahmen der Umstellung auf die **Bio Suisse Richtlinien**."» | ✗ **NICHT implementiert** |

Zusatzbedingung (implementiert): Ein Logo erscheint nur, wenn 100 % der
landwirtschaftlichen Zutaten Knospe-zertifiziert sind (bzw. erlaubte Ausnahmen).

> **Frage an Mirjam / offener Punkt:** Fall D («Umstellung Verarbeitung», mit
> dem abweichenden Hinweissatz auf die *Richtlinien* statt die *Landwirtschaft*)
> ist noch nicht umgesetzt. Soll das Tool diesen Fall abdecken (z.B. via die
> Checkbox «Umstellung Verarbeitung» aus der Excel-Eingabespalte), und wenn ja,
> wann greift er genau?

## 2. Validierung («Rezeptur vollständig — überprüfen»)

Das Excel klassifiziert «Herkunft der Rohstoffe (Knospe)» (Zeile 8) als
**Validierungsregel**: Die Fallunterscheidung A/B/C aus 1.1 steuert die
Anzeige; die rote Fehlermeldung unten ist die dazugehörige Prüfregel gemäss
Sitzungsentscheid vom 25.06.

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
- Excel-Referenz: `requirements/Inhaltsverzeichnis_Bio_Zusatz.xlsx`, Blatt
  «Inhaltsverzeichnis», Zeile 8 «Herkunft der Rohstoffe (Knospe)» (Fälle A/B/C,
  Validierungsregel) und Zeile 9 «Knospe-Logo» (Fälle A/B/C/D, Ausgaberegel)
- Januar-Vergleichsinstanz (Stand Ende Januar, Commit `f1a2bb9`):
  https://bar9.github.io/open-farming-hackdays-label-creator/v0.8.0/
- Implementierung: `src/core.rs` (`should_show_origin_knospe_under90`,
  `validate_import_knospe_origin`, `format_origin_for_knospe_rules`),
  `src/food_db.csv` (Spalte `category`).
