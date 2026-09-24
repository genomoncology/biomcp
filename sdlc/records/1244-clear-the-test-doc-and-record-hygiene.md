---
base: cd18cefa
head: e4e27a8e
---

Cleared the test, doc, and record hygiene backlog, from the review
issue of the same name.

The licensing staleness check no longer fails `make test` on a calendar
date: it prints a warning naming the stale sources, with the
no-mechanical-enforcement decision recorded in the test (the repo
carries no scheduled workflow, and one date assertion does not justify
one). The migration deadline test asserts what is actually falsifiable
— a flag set after the operation's yield, checked after the deadline
fires — replacing disk checks that could never fail because the settle
path never touches disk; the comment names the real-disk test. The
article spec test now asserts the runner's two-space-indented summary
line instead of a substring of the line above it; the cellosaurus test
drops a redundant substring check already implied by the exact-URL
assertion. The documentation audit bans the blog's byte/token count
form on the two catalog pages while leaving the blog's historical
snapshot citations allowed, with the exemption commented.

The clingen-cspec fixture hardens against gate-host load: both
five-second waits (pid file and readiness) move to thirty seconds, the
readiness timeout prints `server.log` instead of only server death
doing so, and the driver's exit trap prints the report JSON to stderr
before removing the work directory. The issue's other two suspected
causes (the 180-second block limit, the retry breaking the exact
request-log check) stay out of scope, recorded. CI's version record
prints bwrap, apparmor_parser, and rg; the apt install gains `-y` with
the offline-gate pin updated in the same change. The architecture
overview names the seven advertised tools and states that the build
tools are pinned while the four apt packages are not. The 1222 record
file was renamed to its ticket's slug with its textual reference
updated.

Every open residual from records 1219, 1221, 1222, 1224, 1225, 1226,
and 1229 has a recorded disposition in the ticket, including the stale
panic-abort residual (1230 set unwind) and the corrections the review
caught (the M5 DDInter leg belongs to 1235's record; the step-level-if
weakness is documented in the 1229 record).

Evidence: design REJECT with four P1s, revised, re-review ACCEPT; code
review REJECT once (a dead temp-dir binding the full-feature lint would
fail on), fixed and verified ACCEPT with the diff enumerated; local
contract suites green; the two article tests that need a built binary
fail identically on the clean tree (verified) and pass on the gate
host; yellow gate at e4e27a8e — lint, test, and spec OK (the first run
needed one formatter collapse of the flag binding).

Residuals: the clingen-cspec 180-second block limit and retry/request-
log items remain open in the issue file; the freshness warning is
deliberately unenforced.
