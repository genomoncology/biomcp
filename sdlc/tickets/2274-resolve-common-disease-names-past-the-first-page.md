---
priority: 2
---

# 2274 — Resolve common disease names past the first page

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 7562c27, accept

Reviews: revision 7562c27, accept

## Outcome

Changes: `CHANGELOG.md`, `src/entities/disease/resolution.rs`, `src/entities/disease/resolution/paging.rs`, `src/entities/disease/resolution/paging_tests.rs`, `src/entities/disease/resolution/tests.rs`, `testdata/disease-resolution/melanoma-page-one.json`, `testdata/disease-resolution/melanoma-page-two.json`.

The exact-term disease resolver settles common names such as melanoma even when MyDisease returns more hits than one page holds, so disease-dependent features such as the article disease filter (2267) work for common cancers.

## Evidence

2267's MyDisease capture for "melanoma" returns 90 hits with 50 supplied, and the resolver refuses completeness, so Semantic Scholar rows become unavailable for one of the most common cancer queries.

## Keeps

Keep every ambiguity safeguard (MM, MF, HD and the 2017 refusals). Bound the extra paging or use an exact-term query so a common name costs at most a small fixed number of requests within the deadline.

## Proof

A replay test resolves melanoma from the 90-hit capture; the existing ambiguous-abbreviation refusals still refuse; the request count stays within the stated bound.

## Implementation evidence

Fresh developer completed 7562c27e21ed31211b3ad35d8c326c8a3f505506. Red job 2274/1792438683863-48711 reproduced the incomplete first-page refusal. Green 2274/1792438791281-86968 passed the affected 71 tests, formatting, warnings-denied all-target Clippy and quality checks. Fresh reviewer accepted that exact revision without changes, verified the retained patch, and did not repeat unchanged proof.

Paging uses at most four complete pages of 50 candidates and one detail request within one shared deadline. Refuse excessive or changing totals, short pages, duplicate IDs, incomplete replies and ambiguity. Existing transport retry policy remains; the logical-call bound does not claim a smaller transport-attempt count. Public melanoma replies contain 90 hits across two pages. Other bounded count probes establish counts only, not canonical resolution. Combined article CLI proof and joint landing with 2267 remain required.

Keep: experiments/2088-biomcp-2274-disease-pages — retain actual provider pages, red and green proof, checked patch and checksums.
