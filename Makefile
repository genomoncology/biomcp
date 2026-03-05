.PHONY: build test check run clean spec

build:
	cargo build --release

test:
	cargo test

check:
	cargo clippy -- -D warnings
	cargo fmt --check

run:
	cargo run --

clean:
	cargo clean

spec:
	BIOMCP_CACHE_MODE=infinite uv run pytest spec/ --mustmatch-lang bash -v
