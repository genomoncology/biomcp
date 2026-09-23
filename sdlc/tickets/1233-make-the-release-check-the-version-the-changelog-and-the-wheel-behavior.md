# Make the release check the version, the changelog, and the wheel behavior

Filed from `sdlc/issues/2026-09-23-release-does-not-check-the-tag-against-the-committed-version.md` and `sdlc/issues/2026-09-23-changelog-unreleased-misses-user-visible-changes.md`. Both block 0.9.1.

## Problem

The Release workflow never compares the tag with the committed versions.
Main is at `0.9.1-dev.1` (Cargo) and `0.9.1.dev1` (Python); a tag pushed
before the version-bump commit would publish dev artifacts to PyPI, GHCR,
and Homebrew with no gate failing. The CHANGELOG Unreleased section is also
missing every user-visible change since 0.9.0 except the container fix and
the cell-line base entry, and the wheel smoke runs only Markdown-mode
commands, which is exactly the hole a reporter on issue #282 walked through
with `-j`.

## Design

1. Add a `version-check` job to `.github/workflows/release.yml` that fails
   unless `${TAG#v}` equals the `version` in `Cargo.toml` and the matching
   Python version in `pyproject.toml`, with `bash` and explicit diffs in the
   failure message. Make `build`, `pypi-build`, and `docs-live` need it.
   The check reads the tag ref, so a `container_only` backfill dispatch on
   an older tag still passes.
2. Extend the wheel smoke with two legs: `drug adverse-events` on a name
   not in FAERS (expects the not-found fallback text and no crash), and
   JSON-mode runs (at least `get drug aspirin regulatory -j` plus one
   `search`) that must exit 0, print output starting with `{`, and never
   contain `skill asset`.
3. Fill the CHANGELOG Unreleased section: the wheel stack-overflow fix
   (1225, GitHub #282, naming the not-found overflow path and the JSON-mode
   skill-asset failure that share the debug-profile cause), the CA bundle
   (1221, GitHub #250), `get cell-line <acc> drug_response` and
   `get drug <name> cell_lines` (bccd2871), `gene cell-lines <symbol>
   --group` (6d8fd435), `get cell-line <acc> chembl` (fd6a100c), and the
   MCP `search` and `get` top-level `"type":"object"` schemas (1223,
   e559cae2).
4. Add a changelog coverage check (a `scripts/` helper in the style of
   `check-docs-live-revision.py`) that fails a release when the Unreleased
   section names no ticket merged since the previous tag, with fake-git
   tests. Wire it into the `version-check` job.
5. Update `docs/reference/release-process.md` for the new job and smoke
   legs.

## Acceptance

- Provenance mutation tests: removing the `version-check` job, the
  `needs:` edge, either new smoke leg, or the coverage check fails a test.
- Fake-git coverage tests pass for the covered, empty-Unreleased, and
  no-previous-tag cases.
- The version check demonstrably fails for a tag/`0.9.1-dev.1` mismatch
  (test or workflow-level evidence).
- Full yellow gate at the head SHA: `make lint`, `make test`, `make spec`.
- Both issue files gain Resolved sections; a record lands in
  `sdlc/records/`.

## Review

- Design review: self-verified by the orchestrator 2026-09-23 against the
  reviewer checklist (job graph, version shapes, smoke fit, helper style,
  missing entries, runbook paragraphs) because the subagent harness broke
  mid-ticket; an independent review is queued before the 0.9.1 tag
- Code review: same deviation; enforcement is mechanical (provenance
  mutation pins, fake-gh coverage tests, exact package-boundary count)
- Verification: yellow at 379dc571 lint/spec OK and Rust 3,780/3,780
  twice; boundary pins fixed at 086f40e5, 40/40; see
  `sdlc/records/1233-make-the-release-check-the-version-the-changelog-and-the-wheel-behavior.md`
