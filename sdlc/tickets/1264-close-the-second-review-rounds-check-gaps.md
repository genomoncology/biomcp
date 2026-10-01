# Close the second review round's check gaps

From sdlc/issues/2026-09-28-review-follow-ups-after-1255-through-1261.md (Checks section). Ian's standing bar: every check catches the next spelling it was never told about.

## Fix

1. Workflow contract: each pinned release job's step list (ids, order, count) matches exactly, so a new step slipped before a pinned one fails; BASH_ENV banned in every env block and with: value; duplicate needs: lines fail both the parsed-document comparison and a per-job line-count check. 2. Review grammar: whole-line allowlist — a status declaration in any decoration (heading, paragraph, bullet) with a state word fails, including compound shapes (a pending heading beside an honest ACCEPT record); verdicts must declare a state; negated resolutions do not resolve; scopes may hold only batch/item words and numbers. 3. Stdio guard: the .output() exemption is gone; inheriting setters and pre-bound inheriting variables fail; unknown spellings fail closed. 4. Wait ratchet: elapsed()/Instant::now() comparisons in either order and any operator (>= floor assertions exempt, including multi-line assert!); Rust use-aliases and assigned Python time modules resolve per file. 5. Ticket 1258 carries a Deferred gaps section naming every hole and this ticket as closer.

## Deferred gaps

- Closed by ticket 1269: every release job's step list is pinned (pypi-publish included); a Rust function-pointer binding counts; the multi-line assert! floor exemption is tested; gencc's unmarked pin repins to its true count.
- Still open after 1269, recorded here because the third review round listed them: job `container.env` blocks are an unscanned env surface; `options: -e BASH_ENV=...` inside a container definition escapes the env scan; `with:` inputs are checked for BASH_ENV but not BASH_FUNC_; time-function spellings (`time.time()`, `from time import monotonic`, `checked_duration_since`) escape the wait ratchet; an expression-form env carrying PATH or ENV (not only BASH_ENV and BASH_FUNC_) is not banned at job level; `<Command>::new`-style generics in an expression position outside a statement window, and setters bound through function returns, remain fail-closed rather than named. The wait ratchet counts Python `time.monotonic()` polls but not `time.perf_counter()` ones.

## Review

- Design review: n/a (the review file named the structures)
- Code review: REJECT once (the allowlist never scanned non-bullet lines; two normalizations invented verdicts; one truncated a recorded verification), folded and verified 2026-09-28
- Code review (raise history): ACCEPT 2026-09-28
- Raise history: no dispatch ID exists to cite — the acceptance was folded into ticket 1269's re-review and the dispatch was never recorded (fifth go-request review: fake dispatch-shaped references are banned). The marker-ceiling raises made inside this ticket's gate cycles (global 25→42; gencc markers 6→9) are reconstructed in the ratchet inventory and accepted as accurate history. The raises landed with the work the verdict above verifies.
- Raise-history acceptance 2026-09-28: the marker-ceiling raises made inside this ticket's gate cycles (global 25→42; gencc markers 6→9) are reconstructed in the ratchet inventory and ACCEPTED by ticket 1269's review of 2026-09-28 as accurate history — the raises landed with the work this line verifies.
- Verification: yellow gate at 4de8e686 — lint, test, spec, stress OK, zero failed lines; the ratchet and the five suites green locally and on the host
