---
flow: quickfix
priority: 7
deps: ["1017", "1018", "1019"]
---

# Advance the development identity to dev.5

After tickets 1017, 1018, and 1019 are complete, advance the private package identities to Rust `0.9.0-dev.5` and Python `0.9.0.dev5`. Keep the public release identity at 0.8.25 and keep development promotion fail-closed.

Follow the exact private-version boundary established by record 1008: change `Cargo.toml`, only the root `biomcp-cli` version in `Cargo.lock`, `pyproject.toml`, only the root version in `uv.lock`, and the current-private references in `CHANGELOG.md`, `architecture/technical/overview.md`, `docs/reference/release-process.md`, `spec/surface/mcp.md`, `tests/test_version_sync_script.py`, `tests/test_citation_contract.py`, `tests/test_directory_submission_contract.py`, and `tests/test_docs_changelog_refresh.py`. Add a truthful dev.5 changelog entry covering the three fixes in tickets 1017, 1018, and 1019 while preserving dev.2, dev.3, and dev.4 history.

Do not change `manifest.json`, either `server.json` version, `CITATION.cff`, `Formula/biomcp.rb`, release code or schemas, signing policy, or publication workflows. No dependency resolution change is allowed in either lockfile.

Done means Cargo metadata and the Python lock report the canonical dev.5 pair, `uv lock --check --offline` passes, and the committed version-sync check reports `Versions in sync: 0.9.0-dev.5 (Python 0.9.0.dev5; development candidate)`. The focused version/citation/directory/changelog tests and `tests/test_release_candidate.py`, `tests/test_release_promotion.py`, `tests/test_release_notes.py`, `tests/test_release_stage_workflow.py`, and `tests/test_release_workflow_provenance.py` must pass, as must `make spec-contracts`, `make lint`, and `git diff --check`. No tag, candidate staging, signing, promotion, publication, queue action, or public metadata update belongs in this ticket.
