---
flow: quickfix
priority: 8
---

# Rank an exact gene symbol before alias matches

## Goal

An exact gene-symbol search returns that gene first and builds its first follow-up command from the same result. On 2026-09-04, `biomcp --json search gene ODC1 --limit 5` returned `SLC25A21` before `ODC1` and suggested `biomcp get gene SLC25A21`. The reproduction and code evidence came from `sdlc/issues/2026-09-04-exact-gene-symbol-does-not-rank-first.md` in commit `f8ff2a78`.

## Desired functionality

BioMCP gives an exact case-insensitive canonical symbol match precedence over alias-only matches. Alias searches continue to find the canonical genes. Free-text gene searches retain provider relevance. Every output surface uses the same result order, and pagination remains stable after the promotion.

## Success criteria

- `biomcp search gene ODC1` returns `ODC1` first.
- The first suggested detail command opens `ODC1`.
- A query through a recognized legacy alias still returns its canonical gene.
- Human-readable, JSON, and MCP results use the same order.
- Continued pages contain no duplicate or missing results because of exact-match ranking.
- Free-text gene-name searches keep their existing relevance behavior.

## Boundaries

This ticket changes ranking for exact gene-symbol queries. It does not change gene detail records, merge genes, alter provider data, or redesign general free-text relevance.
