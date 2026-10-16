# 2127 — Bring back the spec checks 2080 dropped

Status: OPEN.

Milestone: 0.9.3

## Outcome

Every claim the 35 spec pages 2080 removed from the routine list used to check runs again before release, either as a routine spec page on the replay server or as a named surviving test, and the release gate runs them.

## Evidence

Found by the 0.9.3 adversarial review on 2026-10-16, against 2080 (landed 6ad795db1).

- Before 2080, `scripts/run-specs.sh` listed 37 routine pages (6ad795db1^1, lines 13-51). Now it lists `cli-contract-ratchet.md`, `build-profile.md` and `tests/golden/spec`, which holds four tests (gene, variant, two search-all). `make spec`, `spec-pr`, `spec-contracts`, CI (`ci.yml:251`) and `release-gate` all read that list.
- The article, disease, drug, gene, trial, variant, pgx, section-outcomes and mcp pages run nowhere; their executable blocks were deleted and the prose kept (for example `spec/surface/mcp.md:242-306`).
- The end-to-end check that raw MCP returns the same diagnostic provenance as the CLI is gone; only unit tests remain.
- 2080's handoff names a page that does not exist ("clingen-cspot") and leaves out `spec/surface/trial-retirement.md`, which still exists and is unlisted.

- Starts from: 6ad795db1^1's routine list, `scripts/run-specs.sh`, `tests/golden/`, the 2079 replay server.
- Decisions:
  - For each of the 35 pages, write one line in this ticket: re-listed on the replay server, or covered by a named surviving test, or deleted with the reason. A page claim with neither keeps its page.
  - Re-listed pages run in the routine spec lane within 2080's time target; a page that cannot meet it runs in the release gate instead and says so.
  - 2111 keeps the fixture-consumer moves; this ticket owns the spec list.
- Changes: claims `scripts/run-specs.sh` `spec/**` `tests/golden/**` `testdata/replay/**` `CHANGELOG.md`.
- Proof: the 35-line table in this ticket; `make spec` and the release gate run every re-listed page green on the replay server; the spec lane time recorded against 2080's target.
- Order: before release; after 2079's replay server (landed).
