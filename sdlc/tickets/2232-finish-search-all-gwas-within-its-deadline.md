---
priority: 2
---

# 2232 — Finish search all's GWAS query within its deadline

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 156c356, accept

Reviews: revision f3c131f, accept

## Outcome

- Changes: `sdlc/records/2232-search-all-gwas.md` and this ticket during the study phase; no product edits are authorized before the exact source claim is declared.

`biomcp search all` for a common cancer gene returns GWAS results within the GWAS deadline, or reports a named GWAS outage only when the GWAS service is actually slow.

## Evidence

The 0.9.3 release measurement (`sdlc/records/2069-*`) found `gwas search timed out after 12s` in every search-all prefetch and measured run, in both v0.9.2 and v0.9.3. Search all therefore always returns without GWAS and always takes about twelve seconds. The single GWAS command answers from its day-cache in 0.04 seconds, so the query search all sends may differ from the one the GWAS command sends.

## Keeps

Keep search all's other sources, its overall deadline and its day-cache. Do not raise the deadline to hide a slow query.

## Proof

Preserve the approved bounded rsID intersection and both 50-row requests. A successful empty intersection is valid for the recorded inputs. A named timeout is valid when the service genuinely exceeds the unchanged deadline. Retain the observed live request timings and a replay using those bodies and the explicitly observed latency. Before closing, show a second search all within the day reuses completed GWAS day-cache bodies without another provider wait, using an existing test if it proves this exact workflow. If reuse fails, that behavior-preserving defect is the implementation outcome. No nonempty result or narrower candidate set is required.

PM decision: 2026-10-19. Correct the proof as above. Keep the intersection and both 50-row requests. Close by the evidence record if the second-call day-cache reuse check passes; otherwise fix that demonstrated reuse defect.

## Development brief

Study the actual current search-all and standalone GWAS requests before changing code. Preserve all other sources, overall deadline and day-cache; do not increase the timeout. Reproduce with recorded replies and one live opt-in run as authorized by the outcome. Name exact request differences and any demonstrated defect, then report proposed files and outside-in failing proof before the coordinator declares a source claim. Start from current main 096c11fc416ddc18ee3b05814a6d0b9fede63648; PM commit 4da16647b changes no product code. Clinical semantics require a clinical reviewer, not a developer judgment.

## Progress

- 2026-10-19 started

The clinical reviewer confirmed the existing semantic approvals in record 0928 and ticket 2089. The observed pages do not establish absence of an association in the entire Catalog, and no clinical rule requires a nonempty result for this bounded BRAF-plus-melanoma query. No retrieval-semantic change is authorized.
