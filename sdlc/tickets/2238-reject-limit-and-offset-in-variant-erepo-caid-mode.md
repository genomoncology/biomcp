---
priority: 2
---

# 2238 — Reject limit and offset in variant ERepo allele mode

Status: OPEN.

Milestone: 0.9.4

## Outcome

- Changes: `src/cli/variant/mod.rs` `src/cli/variant/erepo.rs` `src/mcp/shell.rs` `src/entities/variant/erepo.rs` `src/entities/variant/mod.rs` `src/mcp/shell/typed_get_tests.rs` `tests/unit/cli/variant.rs` `spec/fixtures/run-clingen-erepo-fixture.sh` `spec/entity/clingen-erepo.md` `CHANGELOG.md` and this ticket.

The `variant_erepo` MCP tool and its CLI form either honor `limit` and `offset` when looked up by ClinGen allele ID, or refuse them with a clear argument error. They are never silently ignored.

## Evidence

Leftover from 1251 in `sdlc/issues/2026-09-24-mcp-schema-follow-ups-after-1240.md`. The allele-ID branch in `src/mcp/shell.rs` near lines 1420-1470 never reads `input.limit` or `input.offset`; gene mode honors them.

## Keeps

Gene mode is unchanged.

## Proof

A test calls allele-ID mode with `limit` and `offset` and gets either the paged result or the argument error. Mark the issue finding resolved.

## Coordinator development brief

Refuse any explicitly supplied paging options in allele-ID mode, including explicit default values. Preserve omitted allele paging and unchanged gene defaults. Use optional typed paging and shared entity validation; remove misleading MCP schema defaults while retaining gene default descriptions. Baseline2238/1792420441910-44821 on239d7adc4 runs disabled-source allele mode with --limit1 --offset5 and receives source_unavailable rather than the argument error; the external assertion fails. The existing CLI/MCP fixture owns regression proof. Update parser expectations without adding duplicate behavior tests.

All builds and tests run through the configured build job. Stage private test/source patches on a published base without pushing WIP; retain patch and per-file hashes with raw evidence, then verify the final published source matches. Push source once ready for review. The developer may resolve the named issue findings as required by the outcome, but does not edit ticket status or send mail.
