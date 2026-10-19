---
priority: 2
---

# 2274 — Resolve common disease names past the first page

Status: OPEN.

Milestone: 0.9.4

## Outcome

The exact-term disease resolver settles common names such as melanoma even when MyDisease returns more hits than one page holds, so disease-dependent features such as the article disease filter (2267) work for common cancers.

## Evidence

2267's MyDisease capture for "melanoma" returns 90 hits with 50 supplied, and the resolver refuses completeness, so Semantic Scholar rows become unavailable for one of the most common cancer queries.

## Keeps

Keep every ambiguity safeguard (MM, MF, HD and the 2017 refusals). Bound the extra paging or use an exact-term query so a common name costs at most a small fixed number of requests within the deadline.

## Proof

A replay test resolves melanoma from the 90-hit capture; the existing ambiguous-abbreviation refusals still refuse; the request count stays within the stated bound.
