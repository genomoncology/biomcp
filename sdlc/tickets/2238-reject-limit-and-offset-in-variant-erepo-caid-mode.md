---
priority: 2
---

# 2238 — Reject limit and offset in variant ERepo allele mode

Status: OPEN.

Milestone: 0.9.4

## Outcome

The `variant_erepo` MCP tool and its CLI form either honor `limit` and `offset` when looked up by ClinGen allele ID, or refuse them with a clear argument error. They are never silently ignored.

## Evidence

Leftover from 1251 in `sdlc/issues/2026-09-24-mcp-schema-follow-ups-after-1240.md`. The allele-ID branch in `src/mcp/shell.rs` near lines 1420-1470 never reads `input.limit` or `input.offset`; gene mode honors them.

## Keeps

Gene mode is unchanged.

## Proof

A test calls allele-ID mode with `limit` and `offset` and gets either the paged result or the argument error. Mark the issue finding resolved.
