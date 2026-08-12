.PHONY: build test lint check-quality-ratchet full-feature-check png-artifact-smoke release-gate run clean spec spec-static spec-pr spec-contracts verify release-live-smoke validate-skills test-contracts install sync-python-dev
.PHONY: output-footprint
.PHONY: prepare-test prepare-test-contracts test-contracts-prepared prepare-spec

SPEC_PROFILE ?= spec
ROUTINE_CARGO_FEATURES ?= --no-default-features
export ROUTINE_CARGO_FEATURES
PYTEST_WORKERS ?= 4
PYTEST_XDIST_ARGS = -n $(PYTEST_WORKERS) --dist loadfile
SPEC_BIN ?= $(CURDIR)/target/$(SPEC_PROFILE)/biomcp
SPEC_USE_PROVIDED_BIN = $(shell if [ -n "$(BIOMCP_BIN)" ] && [ -x "$(BIOMCP_BIN)" ]; then echo yes; fi)
SPEC_RUN_BIN = $(if $(SPEC_USE_PROVIDED_BIN),$(BIOMCP_BIN),$(SPEC_BIN))
CARGO_WITH_IDENTITY = tools/with-build-identity cargo
ROUTINE_TEST_ARCHIVE = $(CURDIR)/.cache/routine-tests.tar.zst
SPEC_BUILD = $(if $(SPEC_USE_PROVIDED_BIN),,$(CARGO_WITH_IDENTITY) build --locked --profile $(SPEC_PROFILE) $(ROUTINE_CARGO_FEATURES) --bin biomcp --example rmcp_streamable_http_contract)

sync-python-dev:
	uv sync --extra dev --no-install-project

build:
	$(CARGO_WITH_IDENTITY) build --release

prepare-test: prepare-test-contracts
	mkdir -p "$(CURDIR)/.cache"
	$(CARGO_WITH_IDENTITY) nextest archive --locked $(ROUTINE_CARGO_FEATURES) --archive-file "$(ROUTINE_TEST_ARCHIVE)" --zstd-level -7

test: prepare-test
	tools/run-offline -- cargo nextest run --archive-file "$(ROUTINE_TEST_ARCHIVE)"
	$(MAKE) test-contracts-prepared

prepare-test-contracts:
	$(SPEC_BUILD)
	$(MAKE) sync-python-dev

test-contracts: prepare-test-contracts
	$(MAKE) test-contracts-prepared

test-contracts-prepared:
	tools/run-offline -- env BIOMCP_BIN="$(SPEC_RUN_BIN)" uv run --no-sync pytest tests/ -v $(PYTEST_XDIST_ARGS)
	tools/run-offline -- env BIOMCP_BIN="$(SPEC_RUN_BIN)" uv run --no-sync mkdocs build --strict

lint:
	ROUTINE_CARGO_FEATURES="$(ROUTINE_CARGO_FEATURES)" ./bin/lint
	tools/check-quality-ratchet.sh

full-feature-check:
	$(CARGO_WITH_IDENTITY) clippy --locked --all-targets --all-features -- -D warnings
	$(CARGO_WITH_IDENTITY) test --locked --all-features --lib sources::alphagenome::tests
	$(CARGO_WITH_IDENTITY) build --release --locked --all-features --bin biomcp
	tools/check-png-artifact target/release/biomcp

png-artifact-smoke: build
	tools/check-png-artifact target/release/biomcp

release-gate: lint
	$(MAKE) test
	$(MAKE) full-feature-check
	$(MAKE) spec SPEC_PROFILE=release SPEC_BIN="$(CURDIR)/target/release/biomcp"

check-quality-ratchet:
	@bash tools/check-quality-ratchet.sh

output-footprint:
	$(SPEC_BUILD)
	$(MAKE) sync-python-dev
	BIOMCP_BIN="$(SPEC_RUN_BIN)" uv run --no-sync python benchmarks/output-footprint/run.py

run:
	$(CARGO_WITH_IDENTITY) run --

clean:
	cargo clean

install:
	mkdir -p "$(HOME)/.local/bin"
	$(CARGO_WITH_IDENTITY) build --release --locked
	install -m 755 target/release/biomcp "$(HOME)/.local/bin/biomcp"

prepare-spec:
	SPEC_PROFILE="$(SPEC_PROFILE)" BIOMCP_FEATURE_ON_BIN="$(if $(filter release,$(SPEC_PROFILE)),$(SPEC_BIN),)" bash scripts/run-specs.sh prepare-spec

spec: prepare-spec
	tools/run-offline -- env BIOMCP_SPEC_ARTIFACTS_PREPARED=1 SPEC_PROFILE="$(SPEC_PROFILE)" BIOMCP_FEATURE_ON_BIN="$(if $(filter release,$(SPEC_PROFILE)),$(SPEC_BIN),)" bash scripts/run-specs.sh spec
	tools/run-offline -- bash scripts/run-specs.sh spec-static

spec-static:
	bash scripts/run-specs.sh spec-static

spec-pr:
	SPEC_PROFILE="$(SPEC_PROFILE)" BIOMCP_FEATURE_ON_BIN="$(if $(filter release,$(SPEC_PROFILE)),$(SPEC_BIN),)" bash scripts/run-specs.sh spec-pr

spec-contracts:
	SPEC_PROFILE="$(SPEC_PROFILE)" BIOMCP_FEATURE_ON_BIN="$(if $(filter release,$(SPEC_PROFILE)),$(SPEC_BIN),)" bash scripts/run-specs.sh spec-contracts

verify:
	$(CARGO_WITH_IDENTITY) build --release --locked
	$(CARGO_WITH_IDENTITY) nextest run --release --test rmcp_client_contract --run-ignored only
	PATH="$${PWD}/target/release:$$PATH" BIOMCP_BIN="$${PWD}/target/release/biomcp" tools/biomcp-ci discover ERBB1
	PATH="$${PWD}/target/release:$$PATH" BIOMCP_BIN="$${PWD}/target/release/biomcp" tools/biomcp-ci search disease melanoma --limit 3
	PATH="$${PWD}/target/release:$$PATH" BIOMCP_BIN="$${PWD}/target/release/biomcp" tools/biomcp-ci search article -g BRAF --limit 3
	PATH="$${PWD}/target/release:$$PATH" BIOMCP_BIN="$${PWD}/target/release/biomcp" tools/biomcp-ci variant normalize all 'NM_000248.3:c.135del'
	BIOMCP_BIN="$${PWD}/target/release/biomcp" BIOMCP_FEATURE_ON_BIN="$${PWD}/target/release/biomcp" bash scripts/run-specs.sh verify
	BIOMCP_BIN="$${PWD}/target/release/biomcp" BIOMCP_FEATURE_ON_BIN="$${PWD}/target/release/biomcp" tools/biomcp-verify-live nih-reporter -- bash scripts/run-specs.sh verify-nih-reporter

release-live-smoke:
	$(MAKE) verify

validate-skills:
	$(MAKE) sync-python-dev
	PATH="$(CURDIR)/target/release:$(PATH)" \
		uv run --no-sync sh -c 'PATH="$(CURDIR)/target/release:$$PATH" ./scripts/validate-skills.sh'
