.PHONY: build test lint check check-quality-ratchet release-gate run clean spec spec-pr spec-smoke validate-skills test-contracts install

# Volatile live-network spec headings. These headings fan out across article
# search backends or have repeated timeout history in GitHub Actions, so they
# run in the local smoke lane rather than the PR-blocking spec gate.
#
# PR gate: repo-local checks plus live-backed headings that have been stable
# within the current CI timeout budget.
# Smoke lane: `search article`, `gene articles`, `variant articles`,
# `disease articles`, or any new heading with repeated provider-latency timeouts.
# SPEC_PR_DESELECT_ARGS stores PR-lane deselects. SPEC_SMOKE_ARGS stores
# stable targeted smoke section IDs that `spec-smoke` resolves to current
# mustmatch pytest item IDs at runtime.
# To move a heading into the smoke lane, add its stable section ID to both
# variables.
SPEC_PR_DESELECT_ARGS = \
	--deselect "spec/02-gene.md::Gene to Articles" \
	--deselect "spec/03-variant.md::Variant to Articles" \
	--deselect "spec/06-article.md::Searching by Gene" \
	--deselect "spec/06-article.md::Searching by Keyword" \
	--deselect "spec/06-article.md::Article Search Gene Keyword Pivot" \
	--deselect "spec/06-article.md::Article Search Drug Keyword Pivot" \
	--deselect "spec/06-article.md::First Index Date in Article Search" \
	--deselect "spec/06-article.md::Keyword Search Can Force Lexical Ranking" \
	--deselect "spec/06-article.md::Source-Specific PubTator Search Uses Default Retraction Filter" \
	--deselect "spec/06-article.md::Source-Specific PubMed Search" \
	--deselect "spec/06-article.md::Source-Specific LitSense2 Search" \
	--deselect "spec/06-article.md::Live Article Year Range Search" \
	--deselect "spec/06-article.md::Federated Search Preserves Non-EuropePMC Matches Under Default Retraction Filter" \
	--deselect "spec/06-article.md::Keyword Anchors Tokenize In JSON Ranking Metadata" \
	--deselect "spec/06-article.md::Article Full Text Saved Markdown" \
	--deselect "spec/06-article.md::Large Article Full Text Saved Markdown" \
	--deselect "spec/06-article.md::Optional-Key Get Article Path" \
	--deselect "spec/06-article.md::Article Search JSON Without Semantic Scholar Key" \
	--deselect "spec/06-article.md::Article Query Echo Surfaces Explicit Max-Per-Source Overrides" \
	--deselect "spec/06-article.md::Article Search Discover Keyword Pivot" \
	--deselect "spec/06-article.md::Getting Article Details" \
	--deselect "spec/06-article.md::Article Batch" \
	--deselect "spec/06-article.md::Article Debug Plan" \
	--deselect "spec/06-article.md::Semantic Scholar Citations" \
	--deselect "spec/06-article.md::Semantic Scholar References" \
	--deselect "spec/06-article.md::Semantic Scholar Recommendations (Single Seed)" \
	--deselect "spec/06-article.md::Semantic Scholar Recommendations (Multi Seed)" \
	--deselect "spec/06-article.md::Sort Behavior" \
	--deselect "spec/09-search-all.md::Debug Plan" \
	--deselect "spec/09-search-all.md::Distinct Disease And Keyword Stay Separate" \
	--deselect "spec/07-disease.md::Disease to Articles" \
	--deselect "spec/07-disease.md::Disease Search Discover Fallback" \
	--deselect "spec/07-disease.md::Disease Search Discover Fallback Synonym" \
	--deselect "spec/07-disease.md::Disease Search Discover Fallback for T-PLL" \
	--deselect "spec/12-search-positionals.md::GWAS Positional Query" \
	--deselect "spec/02-gene.md::Gene DisGeNET Associations" \
	--deselect "spec/07-disease.md::Disease DisGeNET Associations" \
	--deselect "spec/17-cross-entity-pivots.md::Gene to Articles" \
	--deselect "spec/17-cross-entity-pivots.md::Variant pivots" \
	--deselect "spec/19-discover.md" \
	--deselect "spec/20-alias-fallback.md" \
	--deselect "spec/06-article.md::Article Fulltext HTML Fallback Saved Markdown" \
	--deselect "spec/06-article.md::Article Fulltext PDF Fallback Is Opt-In" \
	--deselect "spec/18-source-labels.md::Article Fulltext Source Labels" \
	--deselect "spec/26-workflow-ladders.md::Article Follow-up" \
	--deselect "spec/26-workflow-ladders.md::Mechanism Pathway"

SPEC_SMOKE_ARGS = \
	"spec/06-article.md::Getting Article Details" \
	"spec/06-article.md::Article Batch" \
	"spec/06-article.md::Article Query Echo Surfaces Explicit Max-Per-Source Overrides" \
	"spec/06-article.md::Article Search Gene Keyword Pivot" \
	"spec/06-article.md::Article Search Drug Keyword Pivot" \
	"spec/06-article.md::Article Search Discover Keyword Pivot" \
	"spec/09-search-all.md::Debug Plan" \
	"spec/09-search-all.md::Distinct Disease And Keyword Stay Separate" \
	"spec/17-cross-entity-pivots.md::Gene to Articles" \
	"spec/17-cross-entity-pivots.md::Variant pivots"

SPEC_SERIAL_FILES = spec/05-drug.md spec/13-study.md spec/21-cross-entity-see-also.md
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
	XDG_CACHE_HOME="$(CURDIR)/.cache" PATH="$(CURDIR)/target/release:$(PATH)" BIOMCP_BIN="$(CURDIR)/target/release/biomcp" RUST_LOG=error \
		uv run --extra dev sh -c 'PATH="$(CURDIR)/target/release:$$PATH" BIOMCP_BIN="$(CURDIR)/target/release/biomcp" pytest spec/ --mustmatch-lang bash --mustmatch-timeout 120 -v $(SPEC_XDIST_ARGS) --ignore spec/05-drug.md --ignore spec/13-study.md --ignore spec/21-cross-entity-see-also.md'
	XDG_CACHE_HOME="$(CURDIR)/.cache" PATH="$(CURDIR)/target/release:$(PATH)" BIOMCP_BIN="$(CURDIR)/target/release/biomcp" RUST_LOG=error \
		uv run --extra dev sh -c 'PATH="$(CURDIR)/target/release:$$PATH" BIOMCP_BIN="$(CURDIR)/target/release/biomcp" pytest $(SPEC_SERIAL_FILES) --mustmatch-lang bash --mustmatch-timeout 120 -v'

spec-pr:
	XDG_CACHE_HOME="$(CURDIR)/.cache" PATH="$(CURDIR)/target/release:$(PATH)" BIOMCP_BIN="$(CURDIR)/target/release/biomcp" RUST_LOG=error \
		uv run --extra dev sh -c 'PATH="$(CURDIR)/target/release:$$PATH" BIOMCP_BIN="$(CURDIR)/target/release/biomcp" pytest spec/ --mustmatch-lang bash --mustmatch-timeout 180 -v $(SPEC_XDIST_ARGS) $(SPEC_PR_DESELECT_ARGS) --ignore spec/05-drug.md --ignore spec/13-study.md --ignore spec/21-cross-entity-see-also.md'
	XDG_CACHE_HOME="$(CURDIR)/.cache" PATH="$(CURDIR)/target/release:$(PATH)" BIOMCP_BIN="$(CURDIR)/target/release/biomcp" RUST_LOG=error \
		uv run --extra dev sh -c 'PATH="$(CURDIR)/target/release:$$PATH" BIOMCP_BIN="$(CURDIR)/target/release/biomcp" pytest $(SPEC_SERIAL_FILES) --mustmatch-lang bash --mustmatch-timeout 180 -v'

spec-smoke:
	XDG_CACHE_HOME="$(CURDIR)/.cache" PATH="$(CURDIR)/target/release:$(PATH)" BIOMCP_BIN="$(CURDIR)/target/release/biomcp" RUST_LOG=error \
		uv run --extra dev bash -lc 'set -euo pipefail; export PATH="$(CURDIR)/target/release:$$PATH"; export BIOMCP_BIN="$(CURDIR)/target/release/biomcp"; resolved_args="$$(python tools/spec_smoke_args.py --root-dir "$(CURDIR)" --makefile "$(CURDIR)/Makefile" --makefile-variable SPEC_SMOKE_ARGS)"; if [[ -z "$$resolved_args" ]]; then echo "SPEC_SMOKE_ARGS resolved to no pytest targets" >&2; exit 1; fi; mapfile -t spec_smoke_args <<< "$$resolved_args"; pytest "$${spec_smoke_args[@]}" --mustmatch-lang bash --mustmatch-timeout 120 -v'

validate-skills:
	XDG_CACHE_HOME="$(CURDIR)/.cache" PATH="$(CURDIR)/target/release:$(PATH)" \
		uv run --extra dev sh -c 'PATH="$(CURDIR)/target/release:$$PATH" ./scripts/validate-skills.sh'
