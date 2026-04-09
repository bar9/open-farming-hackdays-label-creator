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
- No backend -- all data stays in the browser (URL params + localStorage)

## Prerequisites

- **Rust** with wasm target: `rustup target add wasm32-unknown-unknown`
- **Dioxus CLI** v0.7: `cargo install dioxus-cli`
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
make test        # Unit tests
```

## Project Structure

```
src/
  main.rs              # Entry point, locale init
  core.rs              # Calculator, OutputFormatter, percentage logic, tests
  rules.rs             # RuleDef enum, Rule trait, RuleRegistry
  model.rs             # Country, Allergen, data enums
  routes.rs            # Router (4 routes, cfg-gated Bio/Knospe)
  layout.rs            # SplitLayout (label editor) + FullLayout
  shared.rs            # Configuration enum, Validations/Conditionals contexts
  category_service.rs  # Ingredient category detection
  pages/
    swiss.rs           # Swiss food law page
    bio.rs             # Bio certification page
    knospe.rs          # Knospe (Bio Suisse) page
    splash_screen.rs   # Landing page
    impressum.rs       # Legal info
  components/          # 29 reusable UI components
  services/            # Business logic services
locales/               # i18n YAML files (de-CH, fr-CH, it-CH)
requirements/          # Architecture documentation
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | Dioxus 0.7 (Rust, WASM) |
| Styling | Tailwind CSS v4 + daisyUI |
| Build | Dioxus CLI (`dx`) + Cargo |
| Task runner | GNU Make |
| i18n | rust-i18n (YAML) |
| CI/CD | GitHub Actions → GitHub Pages |

## Deployment

Both workflows trigger on push to `main`:

- **Staging** (`deploy.yml`): Builds all pages, deploys to `bar9.github.io/open-farming-hackdays-label-creator/`
- **Production** (`deploy-production.yml`): Builds with `--features hidebio`, deploys to `bar9/declarino` repo → declarino.ch

## Architecture Docs

- [Development Loop](requirements/DEVELOPMENT_LOOP.md)
- [Dioxus Patterns](requirements/DIOXUS.md)
- [UI Patterns](requirements/PATTERNS.md)
- [Rules System](requirements/RULES.md)
- [Three Instances Architecture](requirements/THREE_INSTANCES.md)
