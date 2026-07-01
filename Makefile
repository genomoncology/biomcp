.PHONY: build test lint check-quality-ratchet release-gate run clean spec spec-pr spec-contracts verify release-live-smoke validate-skills test-contracts install sync-python-dev

SPEC_ROUTINE_PATHS = \
	spec/entity/article.md \
	spec/entity/study.md \
	spec/entity/variant.md \
	spec/surface/mcp.md \
	spec/surface/skills.md \
	spec/surface/cli-contract-ratchet.md \
	spec/surface/trial-action-summary.md \
	spec/surface/ctgov-helper-pivots.md
SPEC_LIVE_PATHS = \
	spec/entity/diagnostic.md \
	spec/entity/disease.md \
	spec/entity/drug.md \
	spec/entity/gene.md \
	spec/entity/pathway.md \
	spec/entity/pgx.md \
	spec/entity/phenotype.md \
	spec/entity/protein.md \
	spec/entity/trial.md \
	spec/entity/vaers.md \
	spec/entity/variant-hotspots.md \
	spec/surface/cli.md \
	spec/surface/discover.md

SPEC_PROFILE ?= spec
SPEC_BIN ?= $(CURDIR)/target/$(SPEC_PROFILE)/biomcp
SPEC_USE_PROVIDED_BIN = $(shell if [ -n "$(BIOMCP_BIN)" ] && [ -x "$(BIOMCP_BIN)" ]; then echo yes; fi)
SPEC_RUN_BIN = $(if $(SPEC_USE_PROVIDED_BIN),$(BIOMCP_BIN),$(SPEC_BIN))
SPEC_BUILD = $(if $(SPEC_USE_PROVIDED_BIN),,cargo build --locked --profile $(SPEC_PROFILE))

sync-python-dev:
	uv sync --extra dev --no-install-project

build:
	cargo build --release

test:
	cargo nextest run
	$(MAKE) test-contracts

test-contracts:
	cargo build --release --locked
	$(MAKE) sync-python-dev
	uv run --no-sync pytest tests/ -v
	uv run --no-sync mkdocs build --strict

lint:
	./bin/lint
	tools/check-quality-ratchet.sh

release-gate: lint test
	$(MAKE) spec SPEC_PROFILE=release SPEC_BIN="$(CURDIR)/target/release/biomcp"

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
	$(SPEC_BUILD)
	BIOMCP_BIN="$(SPEC_RUN_BIN)" bash scripts/run-specs.sh spec

spec-pr:
	$(SPEC_BUILD)
	BIOMCP_BIN="$(SPEC_RUN_BIN)" bash scripts/run-specs.sh spec-pr

spec-contracts:
	$(SPEC_BUILD)
	BIOMCP_BIN="$(SPEC_RUN_BIN)" bash scripts/run-specs.sh spec-contracts

verify:
	cargo build --release --locked
	PATH="$${PWD}/target/release:$$PATH" BIOMCP_BIN="$${PWD}/target/release/biomcp" tools/biomcp-ci discover ERBB1
	PATH="$${PWD}/target/release:$$PATH" BIOMCP_BIN="$${PWD}/target/release/biomcp" tools/biomcp-ci search disease melanoma --limit 3
	PATH="$${PWD}/target/release:$$PATH" BIOMCP_BIN="$${PWD}/target/release/biomcp" tools/biomcp-ci search article -g BRAF --limit 3
	PATH="$${PWD}/target/release:$$PATH" BIOMCP_BIN="$${PWD}/target/release/biomcp" tools/biomcp-ci variant normalize all 'NM_000248.3:c.135del'
	BIOMCP_BIN="$${PWD}/target/release/biomcp" bash scripts/run-specs.sh verify
	BIOMCP_BIN="$${PWD}/target/release/biomcp" tools/biomcp-verify-live cpic -- bash scripts/run-specs.sh verify-cpic
	BIOMCP_BIN="$${PWD}/target/release/biomcp" tools/biomcp-verify-live nih-reporter -- bash scripts/run-specs.sh verify-nih-reporter

release-live-smoke:
	$(MAKE) verify

validate-skills:
	$(MAKE) sync-python-dev
	PATH="$(CURDIR)/target/release:$(PATH)" \
		uv run --no-sync sh -c 'PATH="$(CURDIR)/target/release:$$PATH" ./scripts/validate-skills.sh'
