# Close the third review round's check gaps

From sdlc/issues/2026-09-28-review-follow-ups-after-1263-and-1264.md (Checks section). Ian's standing bar holds — catch the next spelling, prove each with a mutation the check was not built from — and his 2026-09-28 direction adds one: no ceiling or line-count pin rises without a reviewer-accepted reason.

## Fix

1. Workflow contract: expression-form env blocks carrying BASH_ENV
   fail; BASH_FUNC_*%, PATH, and ENV at workflow level fail (job and
   step PATH stays legal where the workflow legitimately uses it);
   pypi-publish and every remaining release job's step list is
   pinned exactly, so a new first step anywhere in the release path
   fails.
2. Grammar: the state vocabulary for review verdicts becomes an
   allowlist — the four state tokens and the recorded null forms
   only; the spellings "not done", "not yet passed", and "skipped"
   fail as declared states, and the recorded null form stays legal
   only in its exact shape; the negation list gains two more
   spellings ("never fixed", "to be folded"); the kind vocabulary is
   an allowlist (design, code, verification), so an unknown kind
   declaring status fails whatever follows; table rows shaped like a
   status row are status lines.
3. Stdio guard: a second inherit-setter for stderr after a null
   one fails; setters inside comments or string literals are
   ignored (parsed, not stripped — no false positives on prose);
   `<Command>::new(x)` turbofish and `let mk = Command::new; mk(x)`
   bindings fail; an inherit through a reborrow fails; only
   `#[cfg(test)]` marks test code — a cfg whose feature name
   contains "test" does not.
4. Wait ratchet: braced aliases (`use std::thread::{sleep as nap}`),
   `import os, time as t`, `let e = start.elapsed(); if e < d`,
   `Instant::now().duration_since(s) < d`, and Python
   `while time.monotonic() < deadline` polls all count;
   `watchdog(` in a comment no longer exempts a line — only the
   scaled-helper call form (`test_support::watchdog(`,
   `::watchdog(`) does.
5. Ceiling discipline: the wait inventory records every raise with
   a reason and the ticket whose review accepted it; the ratchet
   fails a raise lacking an accepted review reference. The gencc
   unmarked pin repins to its true count.
6. Ticket 1264's Deferred gaps section lists the expression-env
   hole and every bypass above with its state.

## Review

- Design review: n/a (the review file names the structures)
- Code review: ACCEPT 2026-09-28 — an earlier REJECT was folded and re-reviewed; this acceptance activates the 1269-cited raise records (prior findings: folded-raise acceptance unenforced, the bullet_status term dropped, 1202's verdict invented)
- Verification: yellow gate at 346068be; merged in the 535747d9 chain
