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

Revised after the first design review (four P1 corrections folded in;
the reviewer verified every item against current code).

1. The licensing staleness assertion
   (`test_source_review_dates_are_not_stale`) becomes a warning printed
   with the offending dates. Recorded decision: freshness of the
   `reviewed_on` dates is deliberately not enforced mechanically — the
   repo carries no scheduled workflow and this one assertion does not
   justify one; the warning keeps the signal visible in test logs. This
   supersedes the 1224 record's "age-checks reviewed_on at 365 days"
   sentence.
2. The migration deadline test (`src/cache/migration.rs`): a flag set
   after the operation's yield is asserted after `io.await`, pinning
   the settle contract — the falsifiable part. Real disk non-touch
   coverage already lives in
   `epoch_cleanup_stops_mutating_after_a_mid_traversal_deadline`; the
   untouched/marker checks are deleted as vacuous, and the record says
   so. The file sits at its exact 1086-line inventory pin: the edit is
   line-neutral or the inventory gains a ticket-1244 authorization.
3. The article spec fixture test asserts the spec-runner summary's
   indented `  page N (exit N)` form from `record_spec_failure`, a
   unique target instead of a substring of the line above.
4. The cellosaurus redundant `!url.contains("dr")` check is dropped.
5. The documentation audit: the hand-copied-catalog ban learns the
   blog's count form ("21,701 UTF-8 bytes and 5,599 tokens") but stays
   scoped to the ban's existing catalog pages (not all current
   markdown — the count form appears verbatim in open sdlc issue and
   ticket files) — the blog's historical snapshot citations remain
   allowed by design; the change bans current-build hand-copies on
   those catalog pages.
6. clingen-cspec fixture flake hardening: both ~5-second loops in
   `spec/fixtures/setup-clingen-cspec-spec-fixture.sh` (pid-file and
   readiness) move to ~30 seconds, and on readiness failure the setup
   script prints `server.log`; the driver
   (`spec/fixtures/run-clingen-cspec-fixture.sh`) prints the report
   JSON in its exit trap before the work-dir cleanup. The issue's other
   two suspected causes (the 180-second block limit, the retry breaking
   the exact request-log check) stay deliberately out of scope,
   recorded.
7. CI: the version-record step prints `bwrap --version`,
   `apparmor_parser --version`, and `rg --version`; the apt install
   gains `-y`; `tests/test_offline_gate_contract.py`'s apt pin is
   updated in the same change.
8. Architecture overview: names the seven advertised tools; the
   exact-versions sentence keeps its claim only for the still-pinned
   tools (nextest, deny, ruff, mustmatch, protoc) and states the four
   apt packages are unpinned. Ownership of the architecture facts stays
   in records (naming an owner ticket in the overview would itself go
   stale); the 1244 record carries the correction.
9. Residual dispositions (recorded in the ticket when implemented):
   1219's M5 credential-helper hang is machine-local; the M5 DDInter
   leg is record 1235's residual; 1221's client-per-call gap is
   superseded by 1236 (parse-once across all builders; non-UTF-8 and
   mixed-bundle tests landed with 1231/1236), its unreadable-bundle
   privileged-runner sub-residual remains open pending 0.9.1, and
   GitHub issue #250 stays open until 0.9.1 ships; 1222's upload path
   and 1225's args plumbing will be exercised at the 0.9.1 release
   run; 1224's Cloudflare lag is accepted; 1225's panic-abort residual
   is stale since 1230 set unwind; 1226's fail-branch narrowing was
   accepted by 1234's design; 1229's step-level-if weakness is
   documented in the 1229 record, with 1234's gate-neutering mutation
   tests as the newer partial mitigation. The 1222 record/ticket
   filename mismatch is renamed to match here (the ticket file's
   slug wins), updating the ticket's textual reference to the record
   filename at the same time.

## Acceptance

- No test in the suite fails on a future date without a code change.
- The fixture hardening and CI prints land; the overview facts match
  the code; every named residual is filed or gated with a pointer.

## Review

- Design review: REJECT once (four P1s: staleness enforcement had no
  scheduled lane to move to, the migration flag item misstated what is
  falsifiable, the audit extension would turn the suite red without a
  blog exemption, and the apt pin needed a paired test update);
  revised above, re-review pending
- Code review: pending
