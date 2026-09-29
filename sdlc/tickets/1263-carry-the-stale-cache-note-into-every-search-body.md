# Carry the stale-cache note into every search body

From sdlc/issues/2026-09-28-review-follow-ups-after-1255-through-1261.md
(Product section): three JSON outputs dropped the note, a spawned
fetch lost it, and two small comment/branch faults rode along.

## Fix

1. Article search JSON and search-all JSON carry the note through
   the `_meta.notes` channel (article via its meta builder;
   search-all via a `json_body` helper that inserts `_meta` only
   when notes exist). End-to-end tests drive the real CLI entry
   against an axum fixture with a one-second freshness window: the
   stale run carries the sentence, the fresh run does not.
2. GWAS search JSON cannot carry the note: its client sends every
   request with `CacheMode::NoStore` (a deliberate bypass after
   cache decode failures), so no stale serve exists. Recorded in
   the get-JSON notes issue with the code citation; the impossible
   test was removed rather than forced.
3. Spawned fetches inherit the note scope: the ClinGen prefetch
   (the only production fetch spawn — the audit is recorded in the
   code) takes a handle to the command's collector before spawning
   and re-scopes its future onto it; a spawned stale serve reaches
   the parent's drain, proven by a unit test.
4. The DDInter content-type comment now says the header comes from
   upstream.
5. The plain MyDisease label branch is removed: no producer can
   write a bare MyDisease gene source (the seed path always writes
   "DisGeNET (via MyDisease.info)"), recorded in place.

## Deferred gaps

- A future bare `tokio::spawn(fetch())` loses notes silently (the
  None-arm scopes onto an undrained collector); the audit comment
  is the current guard — a ratchet rule is the follow-up shape.

## Review

- Design review: n/a (the review file named the fixes)
- Code review: REJECT once (a meta literal missing the new field
  would not compile; the tests were not hermetic), folded 2026-09-28
- Verification: yellow gate at 064cac91 — lint, test, spec, stress
  OK, zero failed lines (six cycles: the compile literal, the
  string-typed fixture id, the GWAS NoStore discovery, fmt, clippy,
  the boundary count and the dead-code exception)
