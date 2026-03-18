.PHONY: build test lint check run clean spec spec-pr validate-skills test-contracts

# Volatile live-network spec headings. These headings fan out across article
# search backends or have repeated timeout history in GitHub Actions, so they
# run in the smoke workflow rather than the PR-blocking spec gate.
#
# PR gate: repo-local checks plus live-backed headings that have been stable
# within the current CI timeout budget.
# Smoke lane: `search article`, `gene articles`, `variant articles`,
# `disease articles`, or any new heading with repeated provider-latency timeouts.
# To move a heading into the smoke lane, add its exact pytest markdown node ID
# below (file path + heading text after `::`).
SPEC_PR_DESELECT_ARGS = \
	--deselect "spec/02-gene.md::Gene to Articles" \
	--deselect "spec/03-variant.md::Variant to Articles" \
	--deselect "spec/06-article.md::Searching by Gene" \
	--deselect "spec/06-article.md::Searching by Keyword" \
	--deselect "spec/06-article.md::Article Full Text Saved Markdown" \
	--deselect "spec/06-article.md::Sort Behavior" \
	--deselect "spec/07-disease.md::Disease to Articles" \
	--deselect "spec/12-search-positionals.md::GWAS Positional Query"

build:
	cargo build --release

test:
	cargo test

test-contracts:
	cargo build --release --locked
	uv sync --extra dev
	uv run pytest tests/ -v --mcp-cmd "./target/release/biomcp serve"
	uv run mkdocs build --strict

lint:
	./bin/lint

check: lint test

run:
	cargo run --

clean:
	cargo clean

spec:
	XDG_CACHE_HOME="$(CURDIR)/.cache" PATH="$(CURDIR)/target/release:$(PATH)" \
		uv run --extra dev sh -c 'PATH="$(CURDIR)/target/release:$$PATH" pytest spec/ --mustmatch-lang bash --mustmatch-timeout 60 -v'

spec-pr:
	XDG_CACHE_HOME="$(CURDIR)/.cache" PATH="$(CURDIR)/target/release:$(PATH)" \
		uv run --extra dev sh -c 'PATH="$(CURDIR)/target/release:$$PATH" pytest spec/ --mustmatch-lang bash --mustmatch-timeout 60 -v $(SPEC_PR_DESELECT_ARGS)'

validate-skills:
	XDG_CACHE_HOME="$(CURDIR)/.cache" PATH="$(CURDIR)/target/release:$(PATH)" \
		uv run --extra dev sh -c 'PATH="$(CURDIR)/target/release:$$PATH" ./scripts/validate-skills.sh'
