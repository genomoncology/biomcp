# Diagnostic synonym provenance fails under the provider-contract fixture

Disposition (2026-10-20): Closed under the recorded resolution and its landed revision b36ad664f. The original evidence and later dispositions remain below.


Status: resolved.

Opened as: open

## Finding

Ticket 2020's lane (2026-10-07) found the spec-contracts lane failing
one pre-existing case on both the changed and the pristine page, so it
predates this branch and is not caused by any open lane:

- `spec/surface/mcp.md` — "Diagnostic Synonym Provenance Reaches Raw
  MCP": the GTR read returns 0 results under the provider fixture, so
  the case fails.

The make spec-contracts lane is 181 passed, 1 failed. CI never runs
spec-contracts, and the yellow `make spec` lane does not exercise this
provider-contract case, so nothing gated has caught it. The failure
was first observed in the 2020 worktree; it reproduces with the
pre-change page swapped in.

## Triage questions

1. When did it last pass — bisect the fixture and the GTR capture.
2. Whether the GTR capture under `spec/fixtures/provider-contract/`
   is stale (GTR changed shape) or the fixture lost the route.
3. Whether the diagnostic synonym path (GTR) shares the 1304-style
   header-drift failure class.

## Disposition

Queue after 2022 and 2023 unless the bisect shows user-visible wrong
answers; then it jumps the queue. The 0.9.2 bar does not gate on it
(CI never ran this lane), but the final report should name it.

## Closure in2242

Resolved in2242 under PM ruling: live study found MyDisease returned zero exact matches for Bachmann-Bupp while GTR used an expanded condition name. This does not prove a GTR parser fault. Restore authored deterministic synonym coverage and the raw MCP checks; empty diagnostic JSON/Markdown now names missing synonym resolution. No new fallback provider or claim that the upstream reply supplied expansion.

## Release dispositions (2026-10-19)

- Diagnostic synonym coverage: FIXED by 2242 at b36ad664f. Authored deterministic synonym controls cover the raw MCP surface, and empty diagnostic output names missing synonym resolution.
- Proposed GTR parser diagnosis: NOT ESTABLISHED. The live study found zero exact MyDisease matches and an expanded GTR condition name; it did not demonstrate a GTR parser bug.
