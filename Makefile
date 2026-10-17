SHELL := /bin/bash

.PHONY: build test lint check-quality-ratchet full-feature-check png-artifact-smoke release-gate release-gate-markers run clean spec spec-static spec-pr spec-contracts verify release-live-smoke validate-skills test-contracts install sync-python-dev test-rust test-python stress
.PHONY: landing-gate main-gate repo-audit
.PHONY: output-footprint
.PHONY: prepare-test prepare-test-contracts prepare-routine-test-tmp test-contracts-prepared prepare-spec

SPEC_PROFILE ?= dev
FULL_FEATURE_PROFILE ?= dev
FULL_FEATURE_BIN = target/$(if $(filter dev,$(FULL_FEATURE_PROFILE)),debug,$(FULL_FEATURE_PROFILE))/biomcp
ROUTINE_CARGO_FEATURES ?= --no-default-features
export ROUTINE_CARGO_FEATURES
PYTEST_WORKERS ?= 4
PYTEST_XDIST_ARGS = -n $(PYTEST_WORKERS) --dist loadfile
# Cargo maps the dev profile to target/debug; every other profile name is
# its own directory under target/.
SPEC_BIN ?= $(CURDIR)/target/$(if $(filter dev,$(SPEC_PROFILE)),debug,$(SPEC_PROFILE))/biomcp
SPEC_USE_PROVIDED_BIN = $(shell if [ -n "$(BIOMCP_BIN)" ] && [ -x "$(BIOMCP_BIN)" ]; then echo yes; fi)
SPEC_RUN_BIN = $(if $(SPEC_USE_PROVIDED_BIN),$(BIOMCP_BIN),$(SPEC_BIN))
CARGO_WITH_IDENTITY = tools/with-build-identity cargo
ROUTINE_TEST_ARCHIVE = $(CURDIR)/.cache/routine-tests.tar.zst
ROUTINE_TEST_TMPDIR = $(CURDIR)/.cache/routine-test-tmp
PYTEST_BASETEMP = $(ROUTINE_TEST_TMPDIR)/pytest
# Every goal that runs the Python contract suite joins the routine
# lane: the suite's temp-path contract holds only while TMPDIR and
# pytest's basetemp stay in the worktree, and the offline sandbox
# otherwise points TMPDIR at its own /tmp (2148).
ifneq (,$(filter test test-rust test-python test-contracts test-contracts-prepared,$(MAKECMDGOALS)))
export TMPDIR = $(ROUTINE_TEST_TMPDIR)
export BIOMCP_ROUTINE_TEST_LANE = 1
endif
SPEC_BUILD = $(if $(SPEC_USE_PROVIDED_BIN),,$(CARGO_WITH_IDENTITY) build --locked --profile $(SPEC_PROFILE) $(ROUTINE_CARGO_FEATURES) --bin biomcp --example rmcp_streamable_http_contract)

sync-python-dev:
	uv sync --extra dev --no-install-project

build:
	$(CARGO_WITH_IDENTITY) build --release

prepare-routine-test-tmp:
	mkdir -p "$(ROUTINE_TEST_TMPDIR)"

# The nextest archive builds the bin, the MCP example, and every test unit
# in one dev-profile cargo invocation; the contract build after it is a
# freshness check that compiles nothing unless the archive did not run.
prepare-test: prepare-routine-test-tmp
	mkdir -p "$(CURDIR)/.cache"
	$(CARGO_WITH_IDENTITY) nextest archive --locked $(ROUTINE_CARGO_FEATURES) --archive-file "$(ROUTINE_TEST_ARCHIVE)" --zstd-level -7
	$(MAKE) prepare-test-contracts

# Each lane gets a bounded share of the host; callers can tune these after
# measuring their workload. CPU discovery also works on supported non-Linux hosts.
TEST_LANE_WORKERS ?= $(shell python3 -c 'import os; print(min(4, max(1, (os.cpu_count() or 1) // 2)))')
NEXTEST_JOBS ?= $(TEST_LANE_WORKERS)
NEXTEST_JOBS_ARG = -j $(NEXTEST_JOBS)

test:
	$(MAKE) prepare-test
	@set -eu; \
	logs="$$(mktemp -d)"; rust_pid=; python_pid=; \
	trap 'kill $$rust_pid $$python_pid 2>/dev/null || true; rm -rf "$$logs"' EXIT; \
	trap 'exit 130' INT; trap 'exit 143' TERM; \
	$(MAKE) test-rust >"$$logs/rust" 2>&1 & rust_pid=$$!; \
	$(MAKE) test-python >"$$logs/python" 2>&1 & python_pid=$$!; \
	rust_status=0; python_status=0; \
	wait $$rust_pid || rust_status=$$?; rust_pid=; \
	wait $$python_pid || python_status=$$?; python_pid=; \
	printf '\n=== Rust lane (exit %s) ===\n' "$$rust_status"; cat "$$logs/rust"; \
	printf '\n=== Python lane (exit %s) ===\n' "$$python_status"; cat "$$logs/python"; \
	test "$$rust_status" -eq 0 && test "$$python_status" -eq 0

# Both lanes consume the artifacts prepared once by `make test`. Each can
# also run independently against an existing prepared build.
test-rust: prepare-routine-test-tmp
	tools/run-offline -- cargo nextest run --archive-file "$(ROUTINE_TEST_ARCHIVE)" $(NEXTEST_JOBS_ARG) --config-file "$(CURDIR)/.config/nextest.toml"

test-python: prepare-routine-test-tmp
	$(MAKE) test-contracts-prepared

prepare-test-contracts:
	$(SPEC_BUILD)
	$(MAKE) sync-python-dev

# The contract gate runs in the routine lane shape (2148): the tmp
# dir must exist before the lane's exported TMPDIR carries the build.
test-contracts: prepare-routine-test-tmp prepare-test-contracts
	$(MAKE) test-contracts-prepared

test-contracts-prepared:
	tools/run-offline -- env $(if $(BIOMCP_ROUTINE_TEST_LANE),TMPDIR="$(TMPDIR)") BIOMCP_BIN="$(SPEC_RUN_BIN)" uv run --no-sync pytest tests/ -v $(PYTEST_XDIST_ARGS) -m "not release_gate and not repo_audit" $(if $(BIOMCP_ROUTINE_TEST_LANE),--basetemp "$(PYTEST_BASETEMP)")
	tools/run-offline -- env NO_MKDOCS_2_WARNING=1 $(if $(BIOMCP_ROUTINE_TEST_LANE),TMPDIR="$(TMPDIR)") BIOMCP_BIN="$(SPEC_RUN_BIN)" uv run --no-sync mkdocs build --strict

lint:
	@tool_dir="$$(tools/bootstrap-lint-tools)" && \
		PATH="$$tool_dir:$$PATH" ROUTINE_CARGO_FEATURES="$(ROUTINE_CARGO_FEATURES)" ./bin/lint
	tools/check-quality-ratchet.sh

# Keep commands in recipes so even make -j preserves the landing order.
# The existing landing gate runs on the merge candidate; main adds the
# shipped-feature build and behavior checks before the next landing.
landing-gate:
	$(MAKE) lint
	$(MAKE) test
	$(MAKE) spec

main-gate:
	$(MAKE) landing-gate
	$(MAKE) repo-audit
	$(MAKE) full-feature-check

repo-audit: sync-python-dev
	tools/run-offline -- uv run --no-sync pytest tests/ -v $(PYTEST_XDIST_ARGS) -m "repo_audit and not release_gate"

full-feature-check:
	$(CARGO_WITH_IDENTITY) clippy --locked --all-targets --all-features -- -D warnings
	$(CARGO_WITH_IDENTITY) test --locked --all-features --lib sources::alphagenome::tests
	$(CARGO_WITH_IDENTITY) build --locked --profile $(FULL_FEATURE_PROFILE) --all-features --bin biomcp
	tools/check-png-artifact $(FULL_FEATURE_BIN)

png-artifact-smoke: build
	tools/check-png-artifact target/release/biomcp

# The marker set alone (ticket 2125): the packaged-crate cold
# compile behind -m release_gate and any later check that joins it.
# The release workflow's release-gate job runs this target before it
# builds any archive; `make release-gate` still runs the same
# selection after the full suite with the dev build's binary on
# BIOMCP_BIN.
release-gate-markers:
	$(MAKE) repo-audit
	$(MAKE) sync-python-dev
	tools/run-offline -- uv run --no-sync pytest tests/ -m release_gate -v $(PYTEST_XDIST_ARGS)

release-gate: lint
	$(MAKE) test
	$(MAKE) repo-audit
	tools/run-offline -- env BIOMCP_BIN="$(SPEC_RUN_BIN)" uv run --no-sync pytest tests/ -m release_gate -v $(PYTEST_XDIST_ARGS)
	$(MAKE) full-feature-check FULL_FEATURE_PROFILE=release
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

spec:
	$(MAKE) prepare-spec
	$(MAKE) sync-python-dev
	tools/run-offline -- env BIOMCP_SPEC_ARTIFACTS_PREPARED=1 SPEC_PROFILE="$(SPEC_PROFILE)" BIOMCP_FEATURE_ON_BIN="$(if $(filter release,$(SPEC_PROFILE)),$(SPEC_BIN),)" bash scripts/run-specs.sh spec
	tools/run-offline -- $(MAKE) spec-static

spec-static:
	bash scripts/run-specs.sh spec-static

spec-pr:
	SPEC_PROFILE="$(SPEC_PROFILE)" BIOMCP_FEATURE_ON_BIN="$(if $(filter release,$(SPEC_PROFILE)),$(SPEC_BIN),)" bash scripts/run-specs.sh spec-pr

spec-contracts:
	SPEC_PROFILE="$(SPEC_PROFILE)" BIOMCP_FEATURE_ON_BIN="$(if $(filter release,$(SPEC_PROFILE)),$(SPEC_BIN),)" bash scripts/run-specs.sh spec-contracts

verify:
	$(CARGO_WITH_IDENTITY) build --release --locked
	$(CARGO_WITH_IDENTITY) nextest run --release --test rmcp_client_contract --run-ignored only
	$(CARGO_WITH_IDENTITY) nextest run --release --test rmcp_client_contract rmcp_stdio_recovers_from_tool_panic
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

# Stress lane: run the known load-flaky tests pinned to a two-CPU set
# with forced worker parallelism, repeated BIOMCP_STRESS_REPEAT times
# (default 3). The lane runs the prepared dev build — the CI job
# downloads the build job's archive instead of rebuilding it, and a
# local caller runs make prepare-test first (ticket 2075). The test
# invocations alone are pinned, because the runners auto-serialize on
# the detected CPU count.
# One-CPU pinning is avoided deliberately: it deadlocks the pipe
# handshake child deterministically (issue
# sdlc/issues/2026-09-25-single-cpu-affinity-deadlocks-the-handshake-child.md).
# The fixture-server case is the replay server behind the golden suite:
# its readiness handshake blocks on the server's greeting line, so a
# loaded host slows the run without turning startup into a race
# (the disease-survival fixture lifecycle test it replaces is gone with
# its fixture).
stress: prepare-routine-test-tmp
	@test -s "$(ROUTINE_TEST_ARCHIVE)" && test -x "$(SPEC_RUN_BIN)" || { \
	  echo "stress needs the prepared dev build at $(ROUTINE_TEST_ARCHIVE) and $(SPEC_RUN_BIN); run: make prepare-test" >&2; \
	  exit 2; }
	$(MAKE) sync-python-dev
	@set -euo pipefail; \
	repeat="$${BIOMCP_STRESS_REPEAT:-3}"; \
	scale="$${BIOMCP_TEST_TIMEOUT_SCALE:-6}"; \
	for round in $$(seq 1 "$$repeat"); do \
	  echo "=== stress round $$round/$$repeat (two CPUs, 4 workers, timeout scale $$scale) ==="; \
	  BIOMCP_TEST_TIMEOUT_SCALE="$$scale" taskset -c 0,1 tools/run-offline -- cargo nextest run --archive-file "$(ROUTINE_TEST_ARCHIVE)" -j 4 \
	    -E 'test(subprocess_lease_defers_old_generation_cleanup_until_reader_exits) | test(subprocess_lease_child_exits_on_parent_end_of_input) | test(cancelling_stalled_headers_and_streamed_body_drops_request_and_store_work) | test(cancelled_publication_settles_and_the_previous_generation_survives)'; \
	  BIOMCP_TEST_TIMEOUT_SCALE="$$scale" taskset -c 0,1 tools/run-offline -- env TMPDIR="$(TMPDIR)" BIOMCP_BIN="$(SPEC_RUN_BIN)" uv run --no-sync pytest tests/golden -n 4 --dist loadfile --basetemp "$(PYTEST_BASETEMP)"; \
	done
