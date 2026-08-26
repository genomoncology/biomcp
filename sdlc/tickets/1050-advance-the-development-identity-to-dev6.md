---
flow: build
priority: 26
deps: ["1021", "1047", "1048", "1049", "1056"]
---

# Advance the development identity to dev.6

After tickets 1021, 1047, 1048, and 1049 are complete, advance the private
package identities to Rust `0.9.0-dev.6` and Python `0.9.0.dev6`. Keep the
public release identity at 0.8.25 and keep development promotion fail-closed.

Follow the exact private-version boundary established by record 1008: change
`Cargo.toml`, only the root `biomcp-cli` version in `Cargo.lock`,
`pyproject.toml`, only the root version in `uv.lock`, and the
current-private references in `CHANGELOG.md`,
`architecture/technical/overview.md`, `docs/reference/release-process.md`,
`spec/surface/mcp.md`, `tests/test_version_sync_script.py`,
`tests/test_citation_contract.py`, `tests/test_directory_submission_contract.py`,
and `tests/test_docs_changelog_refresh.py`. Add a truthful dev.6 changelog
entry covering the batch — the documentation toolchain pin and silenced
Material banner (1021), the Markdown row and block assertion binding (1047),
the PMC3040717 stored-fixture proof-of-work routes (1048), the supported
test lane declaration (1049), and the indel ID round-trip fix (1056) — while
preserving dev.2 through dev.5 history.

Ticket 1021 changes `pyproject.toml` and `uv.lock` dependency bounds; this
ticket changes the version fields only. Land 1021 first (the `deps` above
already enforce it) so the two lock edits do not collide.

Do not change `manifest.json`, either `server.json` version, `CITATION.cff`,
`Formula/biomcp.rb`, release code or schemas, signing policy, or publication
workflows. Beyond 1021's own bounds, no dependency resolution change is
allowed in either lockfile.

Done means Cargo metadata and the Python lock report the canonical dev.6
pair, `uv lock --check --offline` passes, and the committed version-sync
check reports `Versions in sync: 0.9.0-dev.6 (Python 0.9.0.dev6; development
candidate)`. The focused version/citation/directory/changelog tests and
`tests/test_release_candidate.py`, `tests/test_release_promotion.py`,
`tests/test_release_notes.py`, `tests/test_release_stage_workflow.py`, and
`tests/test_release_workflow_provenance.py` must pass, as must
`make spec-contracts`, `make lint`, and `git diff --check`. No tag, candidate
staging, signing, promotion, publication, queue action, or public metadata
update belongs in this ticket.
