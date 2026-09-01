# Archiv

Abgelegte Dateien, die nicht mehr Teil des Builds sind, aber als Beleg oder
Vorlage aufbewahrt werden. Nichts hier wird kompiliert, deployt oder getestet.

| Datei | Herkunft | Warum archiviert |
|---|---|---|
| `template.html` | Design-Mockup, ursprünglich `879bd98` (2024-03) | Statischer HTML-Entwurf der Oberfläche, entstanden vor der Dioxus-Umsetzung. Wird von keinem Build-Schritt referenziert. Inhaltlich überholt: lädt daisyUI 4.8.0 per CDN (das Projekt nutzt 5.5.19 aus `package.json`) und zeigt den früheren Produktnamen "LMK Creator". Die tatsächliche Oberfläche liegt in `src/components/` und `src/pages/`. |

Löschen ist jederzeit möglich, die Historie behält den Inhalt.
