.PHONY: dev build check check-rust lint test e2e clean setup css help umstellung-assets

setup:
	npm install

css:
	npx @tailwindcss/cli -i ./input.css -o ./assets/tailwind.css

# Bake the Umstellungssatz text-image + logo into one combined PNG per language
# x variant. Requires Pillow (pip install pillow). Uses placeholder text if the
# official raw images are absent — see scripts/build_umstellung_assets.py.
umstellung-assets:
	python3 scripts/build_umstellung_assets.py

dev:
	dx serve

build:
	dx build --release

build-production:
	dx build --release --features hidebio

check-rust: css
	cargo check

lint: css
	# --all-targets so warnings in tests are caught too; they used to
	# accumulate invisibly because only the binary was linted.
	cargo clippy --all-targets -- -D warnings

test: css
	cargo test --bins --test locale_parity

# Requires `make dev` (port 8080) and geckodriver/chromedriver (port 4444) running.
# Run serially (--test-threads=1). Each test gets its own WebDriver session, so
# state is isolated, but the fixed mount/step sleeps in tests/common make the
# suite timing-sensitive: running 2+ concurrent Chrome instances intermittently
# fails tests that pass in isolation. Serial execution is deterministic; the
# wall-time cost is modest and e2e is excluded from CI anyway.
e2e:
	cargo test --test e2e_smoke --test e2e_recipes --test e2e_label --test e2e_validation --test e2e_flows --test e2e_ux --test e2e_declaration_feedback --test e2e_i18n --test e2e_wildsammlung_layout --test e2e_eggs --test e2e_quality_default --test e2e_namensgebend_layout --test e2e_link_compat -- --nocapture --test-threads=1

e2e-ux:
	cargo test --test e2e_ux -- --nocapture --test-threads=1

check: check-rust lint build
	@echo "All checks passed."

clean:
	cargo clean

help:
	@echo "make setup            Install npm dependencies (Tailwind, daisyUI)"
	@echo "make css              Compile Tailwind CSS"
	@echo "make umstellung-assets Bake Umstellungssatz text + logo into combined PNGs (Pillow)"
	@echo "make dev              Start Dioxus dev server (hot-reload)"
	@echo "make build            Production build (dx build --release)"
	@echo "make build-production Production build with hidebio feature"
	@echo "make check-rust       cargo check"
	@echo "make lint             cargo clippy -D warnings"
	@echo "make test             cargo test"
	@echo "make e2e              Run E2E smoke test (needs dx serve + geckodriver)"
	@echo "make check            All checks (check → clippy → build)"
	@echo "make clean            Clean build artifacts"
