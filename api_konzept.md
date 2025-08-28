# API-Konzept für Label Creator

**Autor:** Roland Brand  
**Datum:** 28.08.2025
**Version des Konzepts:** 1
**aktuelle Label-Creator Version:** 0.4.7

## Übersicht

**Zweck:** Dieses Dokument bietet eine Übersicht über aktuelle und künftige Integrationsmöglichkeiten des Label Creators. Es dokumentiert insbesondere das aktuelle Datenschema für die Integration mit externen Systemen.

> **Hinweis zur Schema-Bereitstellung:** Eine systematische Bereitstellung des Schemas (z.B. als JSON Schema) mit dem Deployment wäre möglich und leicht zu implementieren. Dies würde eine automatisierte Validierung und bessere Dokumentation für Integratoren ermöglichen.

Der Integrations- Fokus liegt auf drei Hauptbereichen:

1. **iFrame-Embedding** - Einbettung des Label-Creators in externe Webseiten (bereits möglich)
2. **Marktplatz-Integration** - Datenexport zu E-Commerce-Plattformen (z.B. Biomondo, Markoni, in Spezifikation) 
3. **Lettershop-Anbindung** - Druckdienstleister-Integration (tbd)

> **Hinweis**: In den folgenden Beispielen wird `https://bar9.github.io/open-farming-hackdays-label-creator/` als URL verwendet. Dies ist die **Development-Instanz**. Für den produktiven Einsatz sollte diese URL durch Ihre eigene Domain oder eine beliebige gehostete Instanz ersetzt werden.

## 1. iFrame-Embedding (Bereits implementiert)

### Grundlegende Einbettung

Der Label-Creator kann direkt via iFrame in externe Webseiten eingebettet werden:

```html
<iframe 
  src="https://bar9.github.io/open-farming-hackdays-label-creator/"
  width="100%" 
  height="800"
  frameborder="0">
</iframe>
```

### Einbettung mit vorausgefüllten Daten

Alle Formularfelder können über URL-Parameter vorbelegt werden:

```html
<iframe 
  src="https://bar9.github.io/open-farming-hackdays-label-creator/lebensmittelrecht?product_title=Bergk%C3%A4se&producer_name=Hofmolkerei%20M%C3%BCller"
  width="100%" 
  height="800"
  frameborder="0">
</iframe>
```

### Vollständiges Beispiel mit allen Parametern

Alle Daten können direkt in der URL übergeben werden:

```html
<iframe 
  src="https://bar9.github.io/open-farming-hackdays-label-creator/lebensmittelrecht?product_title=Bio%20Bergk%C3%A4se&product_subtitle=W%C3%BCrzig-kr%C3%A4ftig&producer_name=Alpk%C3%A4serei%20Grindelwald&producer_address=Bergweg%2012&producer_zip=3818&producer_city=Grindelwald&producer_email=info%40alpkaeserei.ch&producer_phone=%2B41%2033%20853%2012%2034&production_country=CH&amount_type=weight&weight_unit=g&ingredients[0][name]=Rohmilch&ingredients[0][amount]=950&ingredients[0][is_allergen]=true&ingredients[1][name]=Salz&ingredients[1][amount]=20&ingredients[1][is_allergen]=false&ingredients[2][name]=Lab&ingredients[2][amount]=5&ingredients[2][is_allergen]=false"
  width="100%" 
  height="900"
  frameborder="0">
</iframe>
```

### Beispiel für dynamische URL-Generierung (optional)

Falls die URL dynamisch generiert werden soll, kann JavaScript verwendet werden:

```javascript
// Produkt-Daten
const product = {
    product_title: 'Bio Bergkäse',
    product_subtitle: 'Würzig-kräftig',
    producer_name: 'Alpkäserei Grindelwald',
    producer_address: 'Bergweg 12',
    producer_zip: '3818',
    producer_city: 'Grindelwald',
    producer_email: 'info@alpkaeserei.ch',
    producer_phone: '+41 33 853 12 34',
    production_country: 'CH',
    amount_type: 'weight',
    weight_unit: 'g',
    'ingredients[0][name]': 'Rohmilch',
    'ingredients[0][amount]': 950,
    'ingredients[0][is_allergen]': true,
    'ingredients[1][name]': 'Salz',
    'ingredients[1][amount]': 20,
    'ingredients[1][is_allergen]': false
};

// URL generieren
const baseUrl = 'https://bar9.github.io/open-farming-hackdays-label-creator/lebensmittelrecht';
const queryString = new URLSearchParams(product).toString();
const fullUrl = `${baseUrl}?${queryString}`;

// iFrame src setzen
document.getElementById('labelCreator').src = fullUrl;
```

### URL-Parameter Schema

| Parameter | Typ | Beschreibung | Beispiel |
|-----------|-----|--------------|----------|
| `product_title` | String | Produktname | `Bio%20Bergkäse` |
| `product_subtitle` | String | Untertitel/Beschreibung | `Würzig-kräftig` |
| `additional_info` | String | Zusatzinformationen | `Aus%20Rohmilch` |
| `storage_info` | String | Lagerungshinweise | `Kühl%20lagern` |
| `date_prefix` | String | Datumsprefix | `Mindestens%20haltbar%20bis` |
| `date` | String | Datum | `31.12.2024` |
| `production_country` | String | Herstellungsland | `CH` |
| `producer_name` | String | Produzent | `Hofmolkerei%20Müller` |
| `producer_address` | String | Adresse | `Hauptstrasse%201` |
| `producer_zip` | String | PLZ | `3800` |
| `producer_city` | String | Ort | `Interlaken` |
| `producer_phone` | String | Telefon | `%2B41%2033%20822%2011%2022` |
| `producer_email` | String | E-Mail | `info%40hofmolkerei.ch` |
| `producer_website` | String | Webseite | `www.hofmolkerei.ch` |
| `manual_total` | Number | Gesamtgewicht (optional) | `1000` |
| `amount_type` | Enum | `weight` oder `volume` | `weight` |
| `weight_unit` | String | Gewichtseinheit | `g` |
| `volume_unit` | String | Volumeneinheit | `ml` |
| `ingredients[n][name]` | String | Name der Zutat | `Milch` |
| `ingredients[n][amount]` | Number | Menge in Gramm | `800` |
| `ingredients[n][is_allergen]` | Boolean | Ist Allergen | `true` |
| `ingredients[n][is_namensgebend]` | Boolean | Ist namensgebend | `false` |

### Responsive Einbettung

```html
<style>
    .label-creator-container {
        position: relative;
        width: 100%;
        max-width: 1200px;
        margin: 0 auto;
    }
    
    .label-creator-iframe {
        width: 100%;
        min-height: 600px;
        height: 80vh;
        border: none;
        box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        border-radius: 8px;
    }
    
    @media (max-width: 768px) {
        .label-creator-iframe {
            height: 100vh;
            border-radius: 0;
        }
    }
</style>

<div class="label-creator-container">
    <iframe 
        class="label-creator-iframe"
        src="https://bar9.github.io/open-farming-hackdays-label-creator/">
    </iframe>
</div>
```

## 2. Marktplatz-Integration (Frontend-Only Konzept)

### Ziel-Marktplätze
- **Biomondo** - Schweizer Bio-Marktplatz
- **Markoni** - Regionaler Online-Marktplatz

### Frontend-basierte Exportmöglichkeiten

Da die App komplett im Browser läuft (WebAssembly), erfolgt der Datenexport client-seitig:

1. **Copy-to-Clipboard**: JSON-Daten in die Zwischenablage kopieren
2. **Download als Datei**: JSON/CSV direkt im Browser generieren und downloaden
3. **PostMessage API**: Kommunikation mit dem einbettenden System
4. **URL-basierter Export**: Daten als URL-Parameter für Weiterleitungen

### Datenexport-Schema (JSON)

```json
{
  "version": "1.0",
  "created_at": "2024-01-15T10:30:00Z",
  "label_type": "swiss_food_law",
  "product": {
    "title": "Bio Bergkäse",
    "subtitle": "Würzig-kräftig",
    "description": "Traditionell hergestellter Bergkäse aus Rohmilch",
    "additional_info": "Aus Heumilch hergestellt",
    "storage": {
      "instructions": "Bei 4-8°C lagern",
      "shelf_life": {
        "prefix": "Mindestens haltbar bis",
        "date": "2024-12-31"
      }
    }
  },
  "ingredients": [
    {
      "name": "Rohmilch",
      "amount_g": 950,
      "percentage": 95,
      "is_allergen": true,
      "is_namensgebend": false,
      "sub_components": []
    },
    {
      "name": "Salz",
      "amount_g": 20,
      "percentage": 2,
      "is_allergen": false,
      "is_namensgebend": false,
      "sub_components": []
    }
  ],
  "allergens": ["Milch"],
  "quantity": {
    "type": "weight",
    "value": 1000,
    "unit": "g",
    "display": "1kg"
  },
  "pricing": {
    "currency": "CHF",
    "price": 45.00,
    "unit_price": {
      "value": 4.50,
      "per": "100g"
    }
  },
  "producer": {
    "name": "Alpkäserei Grindelwald",
    "address": {
      "street": "Bergweg 12",
      "zip": "3818",
      "city": "Grindelwald",
      "country": "CH"
    },
    "contact": {
      "phone": "+41 33 853 12 34",
      "email": "info@alpkaeserei.ch",
      "website": "www.alpkaeserei.ch"
    }
  },
  "origin": {
    "country": "CH",
    "region": "Berner Oberland"
  },
  "certifications": [
    {
      "type": "bio",
      "label": "Bio Suisse",
      "number": "CH-BIO-123"
    }
  ],
  "label": {
    "formatted_html": "<b>Rohmilch</b> (95%), Salz (2%), Lab, Käsereikulturen",
    "formatted_text": "ROHMILCH (95%), Salz (2%), Lab, Käsereikulturen"
  }
}
```

### Integration via PostMessage (für iFrame-Embedding)

Die PostMessage API ([MDN Dokumentation](https://developer.mozilla.org/en-US/docs/Web/API/Window/postMessage)) ermöglicht sichere Kommunikation zwischen dem eingebetteten Label-Creator und der Host-Webseite.

```javascript
// Im Label-Creator (Rust/WASM):
// Sende Produktdaten an das einbettende System
window.parent.postMessage({
  type: 'LABEL_DATA_EXPORT',
  data: productData
}, '*');

// Im einbettenden System:
window.addEventListener('message', (event) => {
  if (event.data.type === 'LABEL_DATA_EXPORT') {
    // Daten an Biomondo/Markoni API senden
    sendToMarketplace(event.data.data);
  }
});
```

### Client-seitiger Download

```javascript
// JSON-Export als Download (im Browser)
function downloadProductData() {
  const data = collectFormData();
  const blob = new Blob([JSON.stringify(data, null, 2)], 
    { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `product_${data.product.title}_${Date.now()}.json`;
  a.click();
  URL.revokeObjectURL(url);
}
```

### URL-basierte Weiterleitung (mit Trade-offs)

URL-basierte Weiterleitung ist die einfachste Integrationsmethode für Frontend-Only Apps, hat aber verschiedene Implementierungsvarianten:

#### Variante 1: Direkte Parameter-Übergabe (Creator-Schema)
```javascript
// Label-Creator definiert das Schema
function redirectToMarketplace(marketplace) {
  const data = collectFormData();
  const params = new URLSearchParams({
    product_title: data.product.title,
    ingredients: JSON.stringify(data.ingredients),
    producer_name: data.producer.name,
    producer_email: data.producer.email
  });
  
  window.open(`https://${marketplace}.ch/import?${params}`, '_blank');
}
```

**Vorteile:**
- Einfache Implementierung
- Keine Anpassungen bei Schema-Änderungen im Creator

**Nachteile:**
- Marktplatz muss Creator-Schema verstehen
- URL-Längenbeschränkung (~2000 Zeichen)
- Sensitive Daten in URL sichtbar

#### Variante 2: Zielsystem-konformes Schema
```javascript
// Mapping auf Marktplatz-spezifisches Schema
function redirectToBiomondo(data) {
  // Biomondo erwartet spezifische Parameter
  const params = new URLSearchParams({
    'item_name': data.product.title,
    'item_ingredients': data.ingredients.map(i => i.name).join(','),
    'vendor': data.producer.name,
    'auth_token': '' // Muss vom User eingegeben werden
  });
  
  window.open(`https://biomondo.ch/quick-import?${params}`, '_blank');
}

function redirectToMarkoni(data) {
  // Markoni hat anderes Schema
  const params = new URLSearchParams({
    'productName': data.product.title,
    'composition': encodeURIComponent(data.label.formatted_text),
    'supplierInfo': `${data.producer.name}|${data.producer.email}`
  });
  
  window.open(`https://markoni.ch/add-product?${params}`, '_blank');
}
```

**Vorteile:**
- Direkte Integration ohne Anpassungen beim Marktplatz
- Optimiert für jeweiliges Zielsystem

**Nachteile:**
- Separates Mapping pro Marktplatz nötig
- Wartungsaufwand bei API-Änderungen

#### Variante 3: Base64-kodierte Payload
```javascript
// Kompakte Datenübertragung via Base64
function redirectWithEncodedData(marketplace, data) {
  const payload = btoa(JSON.stringify(data));
  
  // Kürzere URL, aber Marktplatz muss dekodieren
  window.open(`https://${marketplace}.ch/import?data=${payload}`, '_blank');
}
```

**Vorteile:**
- Strukturierte Daten bleiben erhalten
- Kompakter als einzelne Parameter

**Nachteile:**
- Marktplatz muss Base64 dekodieren können
- Immer noch URL-Längenbeschränkung

#### Variante 4: Two-Step Authentication Flow
```javascript
// Schritt 1: Weiterleitung zur Authentifizierung
function initiateMarketplaceImport(marketplace) {
  const data = collectFormData();
  
  // Temporäre ID generieren oder localStorage nutzen
  const importId = crypto.randomUUID();
  localStorage.setItem(`import_${importId}`, JSON.stringify(data));
  
  // Weiterleitung mit minimalen Daten
  const params = new URLSearchParams({
    'import_id': importId,
    'return_url': window.location.href,
    'action': 'import_product'
  });
  
  window.open(`https://${marketplace}.ch/auth?${params}`, '_blank');
}

// Schritt 2: Nach Auth-Callback Daten via PostMessage senden
window.addEventListener('message', (event) => {
  if (event.origin === 'https://biomondo.ch' && event.data.authenticated) {
    const importId = event.data.import_id;
    const data = localStorage.getItem(`import_${importId}`);
    
    // Jetzt können Daten sicher übertragen werden
    event.source.postMessage({
      type: 'IMPORT_DATA',
      data: JSON.parse(data)
    }, event.origin);
  }
});
```

**Vorteile:**
- Sichere Authentifizierung beim Marktplatz
- Keine Credentials im Creator-Code
- Große Datenmengen möglich

**Nachteile:**
- Komplexere Implementierung
- Marktplatz muss PostMessage unterstützen
- Mehrere Schritte für User

#### Authentifizierungs-Szenarien

| Szenario | Implementierung | Sicherheit | User Experience |
|----------|----------------|------------|-----------------|
| **Keine Auth** | Daten in URL, Marktplatz prüft manuell | ⚠️ Niedrig | ✅ Einfach |
| **Session-basiert** | User muss im Marktplatz eingeloggt sein | ✅ Hoch | ✅ Gut |
| **Token in URL** | User gibt API-Token im Creator ein | ⚠️ Mittel | ❌ Komplex |
| **OAuth Flow** | Redirect zu Marktplatz, dann zurück | ✅ Sehr hoch | ⚠️ Mehrere Schritte |

#### Empfehlung

Für die meisten Fälle empfiehlt sich **Variante 2** (Zielsystem-konformes Schema) kombiniert mit **Session-basierter Authentifizierung**:

```javascript
// Best Practice: Check if user is ready
function safeRedirectToMarketplace(marketplace, data) {
  if (confirm(`Daten an ${marketplace} senden? Bitte stellen Sie sicher, dass Sie dort eingeloggt sind.`)) {
    const params = mapToMarketplaceSchema(marketplace, data);
    window.open(`https://${marketplace}.ch/import?${params}`, '_blank');
  }
}
```

## 3. Export-Funktionen (Frontend-Only)

### Verfügbare Export-Methoden

```rust
// In der Rust/Dioxus App implementiert bzw. zu implementieren:

// 1. Copy to Clipboard
fn copy_json_to_clipboard(form_data: &FormData) {
    let json = serde_json::to_string_pretty(form_data)?;
    // Via web-sys clipboard API
}

// 2. Download als Datei
fn download_as_json(form_data: &FormData) {
    let json = serde_json::to_string_pretty(form_data)?;
    // Trigger browser download
}

// 3. PostMessage an Parent
fn send_to_parent(form_data: &FormData) {
    // window.parent.postMessage()
}

// 4. Generate Share URL
fn generate_share_url(form_data: &FormData) -> String {
    // Serialize to URL params
}
```

### Browser-basierte PDF-Generierung

```javascript
// Option 1: Browser Print API
window.print();

// Option 2: Client-side PDF library (z.B. jsPDF)
// Müsste als WASM-Modul integriert werden
```

## 4. Lettershop-Integration (Frontend-Only Konzept)

### Browser-basierte Druckfunktionen

1. **Browser Print Dialog**
   - CSS Print Styles für optimale Druckausgabe
   - Direkte Druckfunktion über `window.print()`

2. **Download für Druckdienstleister**
   ```javascript
   // JSON-Datei mit Druckdaten generieren
   function generatePrintData() {
     const printData = {
       "type": "food_label",
       "format": "A6",
       "orientation": "landscape",
       "content": {
         "front": {
           "product_title": formData.product_title,
           "ingredients_label": generatedLabel,
           "shelf_life": `${formData.date_prefix} ${formData.date}`,
           "weight": `${formData.amount}${formData.weight_unit}`
         },
         "back": {
           "producer_block": formatProducerAddress(),
           "certifications": getCertifications()
         }
       }
     };
     downloadAsFile('print_data.json', printData);
   }
   ```

3. **Print-optimiertes CSS**
   ```css
   @media print {
     .no-print { display: none; }
     .label-content {
       page-break-inside: avoid;
       width: 148mm;  /* A6 landscape */
       height: 105mm;
     }
   }
   ```

### QR-Code Integration (Client-seitig)

```rust
// QR-Code direkt im Browser generieren
// z.B. mit qrcode-generator WASM library

fn generate_qr_code(form_data: &FormData) -> String {
    // Generate URL with all form data
    let url = generate_share_url(form_data);
    
    // Generate QR code as SVG or Canvas
    // Return as data URL for embedding
    qr_generator::to_svg(&url)
}
```

## Sicherheitsüberlegungen (Frontend-Only)

- **CORS-Konfiguration** für iFrame-Einbettung und PostMessage
- **Origin-Validierung** bei PostMessage-Kommunikation
- **Datenvalidierung** bei allen Eingaben (client-seitig)
- **HTTPS-Only** für die gehostete WebAssembly-App
- **Content Security Policy** für XSS-Schutz
- **Sanitization** von URL-Parametern beim Import

## Nächste Schritte (Frontend-Only Implementierung)

1. **Phase 1**: Dokumentation der bestehenden iFrame-Integration ✅
2. **Phase 2**: Client-seitige Export-Funktionen (JSON, CSV)
3. **Phase 3**: PostMessage API für Marktplatz-Kommunikation
4. **Phase 4**: Browser-basierte Druckfunktion mit CSS Print Styles (oder lettershop api)
5. **Phase 5**: QR-Code-Generierung im Browser (WASM-Library)

## Vorteile der Frontend-Only Architektur

- **Keine Server-Kosten**: Läuft komplett im Browser
- **Datenschutz**: Keine Daten verlassen den Browser des Nutzers
- **Offline-fähig**: Funktioniert auch ohne Internetverbindung
- **Einfaches Deployment**: Statische Files via GitHub Pages, einbetten auf beliebigen websites
- **Skalierbar**: Keine Server-Last, unbegrenzte Nutzer

## Schema-Bereitstellung (Vorschlag)

Für eine bessere Integrationserfahrung könnte das Schema automatisch mit dem Deployment bereitgestellt werden:

```javascript
// Endpoint für Schema-Abruf (statische Datei)
// https://bar9.github.io/open-farming-hackdays-label-creator/schema.json

{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Label Creator Form Data",
  "type": "object",
  "properties": {
    "product_title": {
      "type": "string",
      "description": "Produktname"
    },
    "ingredients": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": { "type": "string" },
          "amount": { "type": "number" },
          "is_allergen": { "type": "boolean" },
          "is_namensgebend": { "type": "boolean" }
        }
      }
    }
    // weitere Felder...
  }
}
```

Dies würde ermöglichen:
- Automatische Validierung von Eingabedaten
- Generierung von Dokumentation
- Type-Safety für Integratoren
- Versionierung des Schemas