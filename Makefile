.PHONY: dev build check check-rust lint test clean setup css help

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

check-rust:
	cargo check

lint:
	cargo clippy -- -D warnings

test:
	cargo test

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
	@echo "make check            All checks (check → clippy → build)"
	@echo "make clean            Clean build artifacts"
