.PHONY: dev build check check-rust lint test e2e clean setup css help

setup:
	npm install

css:
	npx @tailwindcss/cli -i ./input.css -o ./assets/tailwind.css

dev:
	dx serve

build:
	dx build --release

build-production:
	dx build --release --features hidebio

check-rust: css
	cargo check

lint: css
	cargo clippy -- -D warnings

test: css
	cargo test --bins --test locale_parity

# Requires `make dev` (port 8080) and geckodriver/chromedriver (port 4444) running.
# Run serially (--test-threads=1). Each test gets its own WebDriver session, so
# state is isolated, but the fixed mount/step sleeps in tests/common make the
# suite timing-sensitive: running 2+ concurrent Chrome instances intermittently
# fails tests that pass in isolation. Serial execution is deterministic; the
# wall-time cost is modest and e2e is excluded from CI anyway.
e2e:
	cargo test --test e2e_smoke --test e2e_recipes --test e2e_label --test e2e_validation --test e2e_flows --test e2e_ux -- --nocapture --test-threads=1

e2e-ux:
	cargo test --test e2e_ux -- --nocapture --test-threads=1

check: check-rust lint build
	@echo "All checks passed."

clean:
	cargo clean

help:
	@echo "make setup            Install npm dependencies (Tailwind, daisyUI)"
	@echo "make css              Compile Tailwind CSS"
	@echo "make dev              Start Dioxus dev server (hot-reload)"
	@echo "make build            Production build (dx build --release)"
	@echo "make build-production Production build with hidebio feature"
	@echo "make check-rust       cargo check"
	@echo "make lint             cargo clippy -D warnings"
	@echo "make test             cargo test"
	@echo "make e2e              Run E2E smoke test (needs dx serve + geckodriver)"
	@echo "make check            All checks (check → clippy → build)"
	@echo "make clean            Clean build artifacts"
