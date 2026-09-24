# Render source failures honestly

Split from ticket 1238. Source issue:
`sdlc/issues/2026-09-23-source-failures-render-as-empty-or-complete-results.md`.

## Problem

Several paths turn a source failure into clean-looking output: trial
eligibility keeps a trial whose detail fetch failed and still reports an
exact count; `search all` silently drops the recruiting filter when its
source fails; a CIViC schema change reads as "no evidence"; when Open
Targets fails, the fallback source's genes appear under the Open
Targets heading; offline, http-cache serves stale entries of any age
with no flag. Variant, gene, and drug safety pages already report
per-source status; these paths do not.

## Design

1. Trial eligibility: a failed detail fetch or missing criteria text
   marks the trial's eligibility as unavailable and the count as
   partial.
2. `search all`: a recruiting-filter failure reports in the output's
   source status instead of dropping the filter.
3. CIViC: a schema mismatch surfaces as a source failure, not as an
   empty evidence list.
4. Enrichment: fallback data is labeled with its real source.
5. Stale cache entries are labeled with their age once past expiry.
6. Add a check that fails when a renderer drops source status for a
   failing source, so new paths cannot regress silently.

## Acceptance

- Each listed path shows the failure and marks counts partial.
- The anti-regression check exists and covers the listed renderers.

## Review

- Design review: pending
- Code review: pending
