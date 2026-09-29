---
base: 43edc43c
head: 086f40e5
---

Made the release check the tag, the changelog, and the wheel behavior, from
the two issues filed 2026-09-23.

A `version-check` job runs first on both triggers. It checks out the tag
ref and fails unless `${TAG#v}` equals the committed `Cargo.toml` and
`pyproject.toml` versions — the check reads the tag ref, so a
`container_only` backfill dispatch on an older tag compares the versions
committed at that tag and passes. `build`, `pypi-build`, and `docs-live`
need it, so every publish path (PyPI, Homebrew, GHCR, the docs gate) is
covered. The same job runs `scripts/check-changelog-coverage.py`, which
lists the tickets in every commit between the previous release and this
tag and fails when the CHANGELOG Unreleased section does not name one; a
tag with no previous release passes, and an absent Unreleased section
fails.

The wheel smoke gained two legs from the issue #282 follow-up: the
not-found `drug adverse-events` fallback (must print the FAERS fallback
text without crashing) and JSON-mode commands (`get drug ... regulatory
-j` and `search trial -j`), which must exit 0, print an object, and never
mention a missing skill asset — the exact failure a debug-profile wheel
shows on machines other than the build runner. The CHANGELOG Unreleased
section now names every user-visible change since 0.9.0: the wheel fix,
the CA bundle, PharmagoDB drug response, HPA grouped cell lines, ChEMBL
cell-line records, and the MCP object-root schemas. The runbook describes
the new job, the smoke legs, and the dispatch semantics.

Process deviation, recorded: the subagent harness broke mid-ticket (the
installed pi-subagents package references bootstrap files it does not
ship), so the design verification was performed by the orchestrator
against the same checklist posed to reviewers, with file:line evidence in
the ticket. An independent review of this diff is queued before the 0.9.1
tag once the harness is repaired.

Evidence: yellow at 379dc571 — `make lint` OK, `make spec` OK, Rust suite
3,780/3,780 in both `make test` runs; the Python lane failed only the two
package-boundary pins, fixed by the exact-count bump to 1,348 (ticket
1233's two files), verified 40/40 at the final head 086f40e5 together
with the provenance, coverage, and publication contracts. The coverage
helper has six fake-gh tests: covered, missing, no-previous-release,
newer-releases-ignored, gh failure, absent Unreleased section.

Residuals: the JSON legs depend on live network sources in CI like the
existing legs; a source outage fails the smoke rather than skipping it,
which is the intended fail-closed direction. The GenCC subprocess-lease
test flaked once in this gate's first run (second full-suite occurrence
this week, passing in isolation both times) and is filed separately.
