# Source failures render as empty or complete results

Filed 2026-09-23 from an independent review of `v0.9.0..f2549676`.

## Symptom

Each case below turns a source failure into output that looks clean:

- Trial eligibility filters keep a trial whose detail fetch failed or that has no criteria text, and still report an exact count (`src/entities/trial/search/eligibility.rs:321-351`, `src/entities/trial/search/ctgov.rs:999`).
- `search all` drops the recruiting filter when it fails (`src/cli/search_all/dispatch.rs:263-299`).
- A CIViC response schema change reads as "no evidence" (`src/sources/civic.rs:213-228`).
- When Open Targets fails, the fallback source's genes appear under "Genes (Open Targets)" (`src/entities/disease/enrichment.rs:575`).
- Offline, http-cache serves stale entries of any age with no flag.

Variant, gene, and drug safety pages already report per-source status. These paths do not.

## Fix

- Report each failure in the output's source status and mark counts as partial.
- Label fallback data with its real source.
- Add a check that fails when a renderer drops source status, so new paths cannot regress.

## Resolved

Ticket 1242 (both batches; see
`sdlc/records/1242-render-source-failures-honestly.md`): trial
partial counts with the may-be-too-high direction and the
three-reason note in markdown, count JSON, and search JSON;
search-all notes a dropped recruiting filter and stops reporting
exact counts; the CIViC discriminator turns schema breaks into
source failures; enrichment fallback genes carry their real source
in heading and provenance; stale cache serves log their age from the
verified seam; the SOURCE_STATE_ROWS walk guards the renderers. The
default-card owns=true mislabel path is recorded as the next pass.
