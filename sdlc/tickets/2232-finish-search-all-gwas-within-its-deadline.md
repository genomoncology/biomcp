---
priority: 2
---

# 2232 — Finish search all's GWAS query within its deadline

Status: OPEN.

Milestone: 0.9.4

## Outcome

`biomcp search all` for a common cancer gene returns GWAS results within the GWAS deadline, or reports a named GWAS outage only when the GWAS service is actually slow.

## Evidence

The 0.9.3 release measurement (`sdlc/records/2069-*`) found `gwas search timed out after 12s` in every search-all prefetch and measured run, in both v0.9.2 and v0.9.3. Search all therefore always returns without GWAS and always takes about twelve seconds. The single GWAS command answers from its day-cache in 0.04 seconds, so the query search all sends may differ from the one the GWAS command sends.

## Keeps

Keep search all's other sources, its overall deadline and its day-cache. Do not raise the deadline to hide a slow query.

## Proof

Reproduce the timeout on current main with recorded provider replies and with one live opt-in run. Show which request search all sends and why it exceeds the deadline. After the fix, a replay test shows search all returning GWAS rows for the measured gene, and one live run returns in well under twelve seconds with GWAS present.
