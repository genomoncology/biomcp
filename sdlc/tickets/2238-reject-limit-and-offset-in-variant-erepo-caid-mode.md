---
priority: 2
---

# 2238 — Reject limit and offset in variant ERepo allele mode

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 27091d7, accept

Reviews: revision 5e92900, accept

## Outcome

- Changes: `CHANGELOG.md` `sdlc/issues/2026-09-24-mcp-schema-follow-ups-after-1240.md` `sdlc/tickets/1260-unify-variant-transcript-numbering-in-lookup-and-display.md` `sdlc/tickets/2237-repair-garbled-gtr-trademark-characters.md` `sdlc/tickets/2239-cap-the-drug-trial-alias-cache.md` `sdlc/tickets/2240-use-the-ca-bundle-for-alphagenome.md` `sdlc/tickets/2241-carry-stale-cache-notes-in-get-json.md` `sdlc/tickets/2242-settle-three-suspected-bugs-with-live-runs.md` `sdlc/tickets/2243-fix-the-remaining-test-reliability-leftovers.md` `sdlc/tickets/2244-find-the-single-cpu-gencc-lease-hang.md` `spec/entity/clingen-erepo.md` `spec/fixtures/run-clingen-erepo-fixture.sh` `src/cli/variant/erepo.rs` `src/cli/variant/mod.rs` `src/entities/variant/erepo.rs` `src/entities/variant/mod.rs` `src/mcp/shell.rs` `src/mcp/shell/typed_get_tests.rs` `tests/unit/cli/variant.rs` and this ticket. Carried PM ticket and milestone records are unchanged except this ticket own reviewed proof.

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

## Focused proof

Source 27091d77c956364da72847c95230bc55568ecc14 passed the public CLI/MCP ERepo fixture, targeted parser/schema cases, formatting, all-target Clippy and quality boundaries in job 1792420695659-83007. The trailing ad hoc ShellCheck omitted the standard severity flag; canonical ShellCheck passed in 1792420924670-40576. Published verification 1792420971750-51557 matched all ten product/spec files and patch SHA256 3e181163c04bd3eaff6d36f223c2de41fb0f71738d9f2f69373ec0b9d76a95e8. Private build-host experiment 782-biomcp-2238-erepo-paging retains exact raw proof. Fresh review accepted the source without changes.
