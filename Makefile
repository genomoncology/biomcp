.PHONY: build test lint check check-quality-ratchet release-gate run clean spec spec-pr validate-skills test-contracts install

SPEC_XDIST_ARGS = -n auto --dist loadfile

build:
	cargo build --release

test:
	cargo nextest run

test-contracts:
	cargo build --release --locked
	uv sync --extra dev
	uv run pytest tests/ -v --mcp-cmd "./target/release/biomcp serve"
	uv run mkdocs build --strict

lint:
	./bin/lint

check: lint test test-contracts check-quality-ratchet

release-gate: check spec-pr

check-quality-ratchet:
	@bash tools/check-quality-ratchet.sh

run:
	cargo run --

clean:
	cargo clean

install:
	mkdir -p "$(HOME)/.local/bin"
	cargo build --release --locked
	install -m 755 target/release/biomcp "$(HOME)/.local/bin/biomcp"

spec:
# Keep the protein canary in its existing serialized spec partition.
	PATH="$(CURDIR)/target/release:$(PATH)" BIOMCP_BIN="$(CURDIR)/target/release/biomcp" \
		uv run --extra dev sh -c 'PATH="$(CURDIR)/target/release:$$PATH" BIOMCP_BIN="$(CURDIR)/target/release/biomcp" pytest spec/entity/ spec/surface/ --mustmatch-lang bash --mustmatch-timeout 120 -v $(SPEC_XDIST_ARGS) --deselect spec/entity/protein.md'
	PATH="$(CURDIR)/target/release:$(PATH)" BIOMCP_BIN="$(CURDIR)/target/release/biomcp" \
		uv run --extra dev sh -c 'PATH="$(CURDIR)/target/release:$$PATH" BIOMCP_BIN="$(CURDIR)/target/release/biomcp" pytest spec/entity/protein.md --mustmatch-lang bash --mustmatch-timeout 120 -v'

spec-pr:
# Keep the protein canary in its existing serialized spec partition.
	PATH="$(CURDIR)/target/release:$(PATH)" BIOMCP_BIN="$(CURDIR)/target/release/biomcp" \
		uv run --extra dev sh -c 'PATH="$(CURDIR)/target/release:$$PATH" BIOMCP_BIN="$(CURDIR)/target/release/biomcp" pytest spec/entity/ spec/surface/ --mustmatch-lang bash --mustmatch-timeout 180 -v $(SPEC_XDIST_ARGS) --deselect spec/entity/protein.md'
	PATH="$(CURDIR)/target/release:$(PATH)" BIOMCP_BIN="$(CURDIR)/target/release/biomcp" \
		uv run --extra dev sh -c 'PATH="$(CURDIR)/target/release:$$PATH" BIOMCP_BIN="$(CURDIR)/target/release/biomcp" pytest spec/entity/protein.md --mustmatch-lang bash --mustmatch-timeout 180 -v'

validate-skills:
	PATH="$(CURDIR)/target/release:$(PATH)" \
		uv run --extra dev sh -c 'PATH="$(CURDIR)/target/release:$$PATH" ./scripts/validate-skills.sh'
