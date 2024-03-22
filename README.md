# open-farming-hackdays-label-creator
Lebensmittel Label Creator für Manufakturen und Direktvermarkter (Schweiz)


## prerequisites

* install rust platform
* install node platform

```bash
cargo install dioxus-cli
```

```bash
rustup target add wasm32-unknown-unknown
```

```bash
npm install
```

## local dev mode

```bash
npx tailwindcss -i input.css -o public/tailwind.css && dx serve --hot-reload
```
