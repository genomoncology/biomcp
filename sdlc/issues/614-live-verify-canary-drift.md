# Refresh unrelated live verify canaries that currently fail upstream

Severity: should-fix.

This is live-verify-lane work. `make verify` is deliberately not a
gate rung here (`sdlc/planning/verify-lane.md`), so this cannot fail
a flight and should not outrank gate-lane work when it is triaged.

Carried over from March, where it was raised against ticket 614
on 2026-07-27 and left open. The text
below is as filed.
## Summary

The live verify matrix is not fully green despite the ticket's ClinGen LDH probe
passing. Four unrelated public-upstream canaries failed during verification.

## Detail

`make verify` and the post-repair shared live runner failed these existing checks:

- `spec/entity/article-assets-live.md`: PMID 20516115 no longer reports both named
  supplements in an acceptable provider-labelled outcome.
- `spec/entity/article-graph-live.md`: Semantic Scholar recommendation requests
  for PMID 23450558 and PMID 22663011 exited nonzero.
- `spec/entity/clingen-cspec-live.md`: the specified ATM CSpec resource no longer
  returned the required capture-provenance fields.
- `spec/entity/variant-hotspots.md`: the BRAF V600E structure-context request
  exited nonzero.

The new `spec/entity/clingen-ldh-live.md` passed in the same runner after its
omission was repaired, so these failures are not caused by the LDH change.

## Suggested action

Investigate each current provider response, then either repair the client or
retarget the live probe to a still-supported stable resource. Preserve the
live lane for genuinely mutable upstream behavior; add deterministic fixture
or request-contract coverage under `test`/`spec` when a client-side regression
is found.

## Superseded — 2026-07-27

Ian's standing direction resolves the "repair or retarget" question above: a
live canary may be replaced by unit tests provided those tests fully cover the
CLI-to-API-call transition and the parsing of a locally captured response.
Only "does the provider still serve this today" is dropped, and that was never
BioMCP behaviour.

The conversion plan and per-canary priority order live in
`2026-07-27-convert-remaining-live-canaries-to-unit-coverage.md`. That issue
owns the remaining work; this one records the original observation.

`spec/entity/article-indexing-live.md` is already retired with offline
replacement (ticket 614, merged `89cf6f11`). Three remain:
`clingen-cspec-live.md` (must be **converted, not deleted** — zero unit tests
today), `article-assets-live.md`, `article-graph-live.md`.

`spec/entity/variant-hotspots.md` is listed above but is **not** covered by the
conversion issue and still needs its own look.
