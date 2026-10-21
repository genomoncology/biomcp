# Route the last four XML sources through the depth cap

From the 2026-09-27 review file (Product, first bullet). Ticket 1243
put a pre-parse nesting cap in `parse_external_xml` and proved the
bomb against it, but four upstream sources still call
`roxmltree::Document::parse` directly, so deep input on those paths
can still overflow the parser stack. Ian confirmed this himself.

## Problem

- `src/sources/vaers.rs:329` — direct `Document::parse`
- `src/sources/hpa.rs:434` — direct `Document::parse`
- `src/sources/medlineplus.rs:185` — direct `Document::parse`
- `src/sources/pmc_oa.rs:227` — direct `Document::parse`
- `src/sources/vaers/tests/mod.rs:22` — a test helper parses a
  request fixture directly; route it too or scope the guard so the
  choice is deliberate, not accidental.

## Fix

1. Route all four production sites through
   `crate::xml::parse_external_xml`. Preserve each site's current
   error mapping quality: the depth-cap rejection
   (`DepthLimitExceeded`) must surface as the same error type the
   site already uses (`BioMcpError::Api` wording or the site's
   existing degrade arm), never as a panic or a generic loss.
2. Add a repository guard test that fails on any direct
   `Document::parse` outside `src/xml.rs`. Structural, not a phrase
   ban: scan tracked `.rs` files under `src/` for the call form
   (allowing the one inside `src/xml.rs`), so renames, aliases, and
   new files are all covered. A test of the checker itself must
   plant a direct call in a scratch file and see the checker catch
   it, and must show the checker ignoring a comment that merely
   mentions the call.

## Review

- Design review: n/a (the ticket fixed findings Ian confirmed
  himself; the fix shape was his direction)
- Code review: ACCEPT with notes 2026-09-27 — the P1 (record the
  revert red run) is discharged in the record; the P2s (guard
  evasion spellings, the single-alias test, the archived probe
  outside src/) are recorded for ticket 1258's hardening class
- Verification: yellow gate at 8bf9884a — lint, test, spec, and
  stress all OK, zero failed lines in the clean log (one earlier
  gate log was corrupted by two overlapping script instances and
  was discarded; the clean re-run is the evidence)

## Acceptance

- The guard test passes on the fixed tree and fails when any one of
  the four sites is reverted (prove one red run per revert in the
  commit message or the record; one representative revert proof is
  enough if the four are mechanically identical).
- The checker's own test proves it catches a planted direct call in
  a file it did not know about.
- Each of the four sources has a depth-bomb rejection test in its
  own test module (a >64-deep fixture through the source's parse
  entry point, asserting the honest error, not a panic).
- Yellow gate green (lint, test, spec, stress).

## Out of scope

- Changing the cap value or the scan algorithm (1243 territory,
  already recorded).
