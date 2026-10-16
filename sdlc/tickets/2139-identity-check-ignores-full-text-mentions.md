---
flow: build
priority: 3
deps: [2138]
---

# 2139: Count full-text mentions in the variant identity check

Status: OPEN.

Milestone: 0.9.3

## Outcome

`variant articles --verify-identity` confirms a paper when PubTator's full-text annotation finds the variant in it.

## Evidence

Reported 2026-10-16 by a literature probe of 0.9.2.

- `biomcp --json variant articles "SAMHD1 I448T" --verify-identity --limit 50` marks 32371413, 34480199 and 30275001 `identity.status: unverified` with no observations.
- PubTator full-text search finds the variant in each of those three papers.

## Work

- Reproduce on main first.
- Feed PubTator's full-text variant annotations into the identity check as observations, beside the abstract evidence it reads today.

## Acceptance

1. A recorded-reply test confirms the three SAMHD1 I448T papers from full-text annotations; it fails on current main.
2. A paper with no annotation of any kind stays unverified.
3. The changelog carries a bullet for this ticket.
