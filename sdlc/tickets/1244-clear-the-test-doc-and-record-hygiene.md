# Clear the test, doc, and record hygiene

Split from ticket 1238. Source issue:
`sdlc/issues/2026-09-23-test-doc-and-record-hygiene.md`.

## Problem

Concrete hygiene faults: the licensing docs contract goes red on a date
(`reviewed_on` + 365 days) with no code change; the migration test's
`untouched` marker cannot fail as written; the article spec fixture
test asserts a substring of the line above it; the cellosaurus test
repeats a proven fact; the documentation audit misses the blog's count
form; the clingen-cspec spec fixture flakes under load (5-second
readiness wait, tight 180-second block, strict request-log check);
"record exact tool versions" omits bwrap/apparmor_parser/rg and the apt
install omits `-y`; the architecture overview names one MCP tool where
seven ship, says CI installs exact versions after 1220 unpinned them,
and its fact ownership moved from 1222's record to 1227's; the open
residuals from records 1219, 1221, 1222, 1224, 1225, 1226, and 1229
live only in prose.

## Design

1. Move the licensing expiry check to a warning (or scheduled check)
   so `make test` cannot go red on a calendar date; keep the freshness
   pressure in CI where it belongs.
2. Fix the three named test weaknesses and the audit pattern.
3. clingen-cspec fixture: 30-second readiness wait, print server.log
   and the report JSON on failure.
4. CI version record: print bwrap, apparmor_parser, and rg versions;
   add `apt-get install -y`.
5. Architecture overview: seven tools, no exact-version claim, fact
   ownership named in the 1227 record's terms.
6. File or gate each open record residual; record the disposition of
   each in this ticket.

## Acceptance

- No test in the suite fails on a future date without a code change.
- The fixture hardening and CI prints land; the overview facts match
  the code; every named residual is filed or gated with a pointer.

## Review

- Design review: pending
- Code review: pending
