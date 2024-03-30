# open-farming-hackdays-label-creator
Lebensmittel Label Creator für Manufakturen und Direktvermarkter (Schweiz)

Challenge: https://hack.farming.opendata.ch/project/111

Live Demo / Prototype: https://jarheadcore.github.io/open-farming-hackdays-label-creator/


## Prerequisites

* install rust platform
* install node platform

```bash
cargo install dioxus-cli
```

```bash
cargo install cargo-make
```

```bash
rustup target add wasm32-unknown-unknown
```

```bash
npm install
```

## Local Development

```bash
cargo make dev
```

## Eingabefelder und Feldtypen

1. Sachbezeichnung / “Name des Produkts" (Textfeld)
2. Zutatenliste → muss mit “Zutaten:” auf der Etikette anfangen
    1. Mengenangabe → Absteigend nach Gewicht stortiert → (Textfeld)
    2. Produktauswahl über API (Schnittstelle) → vordefinierte Liste (Datalist, Textfeld)
    3. Herkunft? (Textfeld)
    4. Allergene herausarbeiten → in 1. Version via “Checkbox” Feld resp. vordefinierte Liste, Ausgabe auf Etikette fett markiert
    5. Typenbezeichnung (Was ist es? “Säuerungsmittel: E303”)
3. Datumangabe → Selektion aus Tag, Monat Jahr (somit wäre Warenlos Angabe hinfällig) | “Zu verbrauchen bis” + “min. haltbar bis” (Info mit genaueren Angaben)
4. Zusatzinformationen → Textfeld (Info: Haftungsausschlüsse, Kann Spuren von Nüssen enthalten, Gebrauchsanleitung etc..)
5. (Nährwerte → in Version 1 nicht benötigt, kann/soll später über API gelöst werden)
6. Aufbewahrungshinweise (Textfeld)
7. (Warenlos / Chargennummer) → evtl. händische Eingabe → wird in Version 1 noch nicht benötigt da Datumsangabe mit Tag / Monat / Jahr angegeben wird
8. Nettogewicht (inkl. Einheit → Gewicht oder Volumen) → Zahl + Einheit per Radio Button
9. Abtropfmenge (Info mit genaueren Angaben)
10. Produktionsland (vorselektiert Schweiz → keine Gültigkeit für Ausland!)
11. Produktionsadresse (Bsp. Hans Muster AG, Teststrasse 1, CH-4000)
12. Preis inkl. MwSt. → 2 Felder, 1x pro 100g, 1x für Totalpreis (Info: kann auch auf Regal angeschrieben werden)
13. Zertifizierungsstelle → vordefinierte Auswahlliste (per API)
