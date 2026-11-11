# Declarino - Swiss Food Label Creator

A web application for creating food labels compliant with Swiss food labeling law (Lebensmittelrecht), built with Dioxus (Rust/WASM). Designed for small manufacturers and direct marketers in Switzerland.

**Production:** https://www.declarino.ch

**Staging:** https://bar9.github.io/open-farming-hackdays-label-creator/

**Origin:** [Open Farming Hackdays Challenge #111](https://hack.farming.opendata.ch/project/111)

## Features

- Three label types: Swiss food law (Lebensmittelrecht), Bio certification, Knospe (Bio Suisse)
- Real-time label preview as you fill in the form
- Ingredient management with allergen detection and bold marking
- Composite ingredients with sub-components
- Country of origin rules (>50%, meat >20%, beef/fish specifics, Knospe tiers)
- Bio certification tracking with Bio Suisse logo logic
- Shareable labels via URL query parameters
- Multilingual: German (de-CH), French (fr-CH), Italian (it-CH)
- No backend for recipe data -- it stays in the browser (URL params + localStorage). The only server-side pieces are the short-link functions in `api/` and cookieless analytics.

## Prerequisites

- **Rust** with wasm target: `rustup target add wasm32-unknown-unknown`
- **Dioxus CLI** v0.7.10 (matches CI): `cargo install dioxus-cli --version 0.7.10`. `dx` refuses to build against a dioxus version other than its own, so `make dev`, `make build` and `make build-production` verify this first; run `make check-dx` to check on its own.
- **Node.js** + npm (for Tailwind CSS + daisyUI)

## Setup

```bash
make setup    # Install npm dependencies
```

## Development

```bash
make dev      # Start Dioxus dev server with hot-reload
```

Open http://localhost:8080/open-farming-hackdays-label-creator

## Build

```bash
make build              # Development build (all pages)
make build-production   # Production build (Bio/Knospe shown as "Coming Soon")
```

The `hidebio` feature flag controls Bio/Knospe page visibility. Production deploys to declarino.ch with this flag enabled.

## Checks

```bash
make check       # Run all: cargo check → clippy → dx build
make check-rust  # Type checking only
make lint        # Clippy with -D warnings
make test        # Unit tests (incl. golden labels and locale parity)
make e2e         # WebDriver suite (needs `make dev` + chromedriver on :4444)
```

## Project Structure

```
src/
  main.rs              # Entry point, locale init
  core.rs              # Calculator, OutputFormatter, percentage logic
  core/tests/          # Rule engine unit + golden tests, by domain
  rules.rs             # RuleDef enum, Rule trait, RuleRegistry
  verdicts.rs          # Typed rule decisions (Bio/Knospe verdicts, check state)
  conditional_keys.rs  # Key constants for the test-only conditionals view
  model.rs             # Country, food DB, aliases
  routes.rs            # Router (cfg-gated Bio/Knospe, FAQ, support)
  layout.rs            # SplitLayout (label editor) + FullLayout
  shared.rs            # Configuration enum, Validations context
  category_service.rs  # Ingredient category detection
  api.rs               # BLV food API (with retry)
  pages/
    swiss.rs           # Swiss food law page
    bio.rs             # Bio certification page
    knospe.rs          # Knospe (Bio Suisse) page
    label_page.rs      # Shared label editor page
    splash_screen.rs   # Landing page
    faq.rs, support.rs # FAQ, support (TWINT)
    impressum.rs       # Legal info
  components/          # Reusable UI components
  services/            # Suggestions, label text, QR, short links, «Bio» suffix
tests/                 # WebDriver e2e suite + locale parity
locales/               # i18n YAML files (de-CH, fr-CH, it-CH)
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | Dioxus 0.7 (Rust, WASM) |
| Styling | Tailwind CSS v4 + daisyUI |
| Build | Dioxus CLI (`dx`) + Cargo |
| Task runner | GNU Make |
| i18n | rust-i18n (YAML) |
| Analytics | Umami Cloud (cookieless, EU) |
| CI/CD | GitHub Actions → GitHub Pages (staging) / Vercel (production) |

## Deployment

Both workflows trigger on push to `main`:

- **Staging** (`deploy.yml`): Builds all pages, deploys to `bar9.github.io/open-farming-hackdays-label-creator/`
- **Production** (`deploy-production.yml`): Builds with `--features hidebio`, deploys to `bar9/declarino` repo → served by Vercel at declarino.ch. Also ships the short-link functions in `api/` (see `api/README.md`)

## Analytics & privacy

Reach is measured with [Umami](https://umami.is) Cloud (cookieless, EU-hosted).
The snippet sits in the `<head>` of `index.html`; `data-domains` limits it to the
production domain, so staging and local development do not skew the numbers.

**`data-exclude-search="true"` is mandatory, not cosmetic.** A share link carries
the whole recipe in the query string, including `producer_name`,
`producer_address`, `producer_phone` and `producer_email`. Without the flag,
opening a shared or printed short link would send those personal details to
Umami. Filling the form is unaffected: the query string only feeds the copy-link
button and never reaches the address bar. The CI step *Check analytics privacy
flags* enforces this, because losing the flag would otherwise be invisible.

**Short links store the recipe.** The «Teilen» dialog shows the full link by
default and sends nothing. Only when the user picks «Kurz-Link» is the full share
URL sent to `/api/shorten` (`link_share_modal.rs`), which stores it without expiry
(see `api/README.md`). That URL contains the recipe and the producer details.
Keep this in mind for the privacy declaration.

The privacy declaration is a [PrivacyBee](https://privacybee.io) document embedded
on `/impressum`; its language follows the UI language. PrivacyBee rescans the site
about every six weeks and lists newly detected services on its own. **After
changing or adding an analytics service, declare it in PrivacyBee right away**
instead of waiting for that scan, otherwise the declaration lags behind reality.

## Architecture Docs

- [Development Loop](requirements/DEVELOPMENT_LOOP.md)
- [Dioxus Patterns](requirements/DIOXUS.md)
- [UI Patterns](requirements/PATTERNS.md)
- [Rules System](requirements/RULES.md)
