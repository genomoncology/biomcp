---
flow: quickfix
priority: 8
---

# Advance the development identity to dev.3

After tickets 1004 through 1007 are complete, advance the private package identities to Rust `0.9.0-dev.3` and Python `0.9.0.dev3`. Keep the public release identity at 0.8.25 and keep development promotion fail-closed.

The exact mutable files and values are:

- `Cargo.toml`: `0.9.0-dev.3`.
- `Cargo.lock`: only the root `biomcp-cli` package version becomes `0.9.0-dev.3`; no dependency resolution changes.
- `pyproject.toml`: `0.9.0.dev3`.
- `uv.lock`: only the root `biomcp-cli` package version becomes `0.9.0.dev3`; no dependency resolution changes.
- `CHANGELOG.md`: add a new Unreleased dev.3 entry while preserving the historical dev.2 entry.
- `architecture/technical/overview.md`, `docs/reference/release-process.md`, and `spec/surface/mcp.md`: identify the current private pair as dev.3 without changing public-release claims.
- `tests/test_version_sync_script.py`, `tests/test_citation_contract.py`, `tests/test_directory_submission_contract.py`, and `tests/test_docs_changelog_refresh.py`: advance only assertions that describe the current private pair.

Do not change `manifest.json`, either `server.json` version, `CITATION.cff`, `Formula/biomcp.rb`, release code or schemas, signing policy, or publication workflows. The Homebrew file is a `__VERSION__` release template and must remain byte-for-byte unchanged. The latest reachable `v0.8.25` tag, public manifests, and citation continue to carry the public 0.8.25 identity; there is no separate committed published-release record to rewrite.

Done means Cargo metadata and the Python lock report the exact canonical dev.3 pair, only the two root lock entries change, and `uv lock --check --offline` passes. After the isolated version change is committed, `scripts/check-version-sync.sh` must print exactly `Versions in sync: 0.9.0-dev.3 (Python 0.9.0.dev3; development candidate)`. The version-sync, citation, directory, changelog/docs, candidate, promotion, release-notes, and release-workflow suites must pass, as must `make spec-contracts`, `make lint`, and `git diff --check`. Existing tests must continue to prove that development candidates are rejected by promotion and publication before side effects.

No tag, candidate staging, signing, promotion, publication, or public metadata update is part of this ticket.
