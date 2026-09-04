---
flow: build
priority: 8
---

# Return variant literature within a total work budget

## Goal

One variant-literature request returns useful completed work before an agent runner must terminate the process. On 2026-09-04, `biomcp --json variant articles "ODC1 c.1342A>T" --limit 10` produced no JSON before both 55-second and 130-second process limits expired. The annotation-only strategy completed in 21.26 seconds during the original diagnosis. The reproduction and code evidence came from `sdlc/issues/2026-09-04-variant-article-union-can-exceed-two-minutes.md` in commit `f8ff2a78`.

## Desired functionality

The default variant-literature request enforces a documented total work budget of 60 seconds. It returns completed results when another source or route cannot finish in that budget. The response states that coverage is incomplete and names unfinished source routes. A request that yields no usable result exits with an actionable failure. Explicit diagnostic strategies retain their current purpose.

## Success criteria

- A deterministic slow-source reproduction completes within the total work budget.
- Completed article results survive when another source or route reaches the budget.
- Human-readable, JSON, and MCP responses state incomplete coverage and identify unfinished routes.
- A response with at least one usable result exits successfully and does not claim complete coverage.
- A response with no usable result exits unsuccessfully with an actionable explanation.
- Explicit annotation and lexical strategies remain independently usable.
- Existing logical work caps and provider rate limits remain enforced.

## Boundaries

This ticket bounds the complete variant-literature request. It does not promise that every provider finishes, remove existing work limits, weaken identity checks, or redesign article relevance.
