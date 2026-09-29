---
base: fe90d348
head: b0930b51
---

Shrank the large futures and shared the runtime, from the large-
futures issue.

Measured: the top-level dispatch future fell from 225,312 bytes to
2,240, and the run fallthrough from 227,856 to 2,656 — roughly a
hundred-fold reduction, with probe tests pinning ceilings of 4,096
and 8,192 and the measured history recorded in their doc comment.
`with_no_cache` returns the scoped future instead of awaiting it in
place (the doubling gone), the dispatch arms and handler awaits are
boxed, and the four-way discover join is boxed with all-to-completion
semantics unchanged.

The runtime is shared where it should be: MCP tool calls drive on
the server's long-lived runtime from the same dedicated 8 MiB
execute thread (ticket 1225's pin unchanged), so pooled connections
and timers survive between calls; CLI one-shots keep a per-call
runtime dropped with `shutdown_background`, so a blocking
eviction-style task never delays the reply — proven by a test with a
two-second sleeper against a sub-second reply. The safety invariant
is written where it matters: cache puts are awaited inline and only
eviction is backgrounded, so nothing is lost under
`shutdown_background` + `process::exit`. Blocking-thread stacks are
4 MiB on both runtimes (headroom; the depth cap is the control).

XML documents nested past 64 elements deep are rejected before
parsing, with the depth counted in the pre-parse byte scan — a
recorded deviation, because the acceptance test itself disproved the
design's post-parse walk: roxmltree's parser recurses per open tag,
so the 100k-deep bomb overflows inside `Document::parse`. The scan
counts opens, closes, and self-closing tags exactly (comments,
CDATA, and processing instructions are skipped wholesale), and the
cap sits at seven times the deepest tracked fixture. The worker
drive split into its own module under the 700-line CLI cap.

Evidence: design REJECT once (the runtime item named the wrong seam —
the per-call construction serves the MCP path too; probe mechanism
misstated; two issue items dropped silently), folded and re-reviewed
ACCEPT; code review ACCEPT with five P2s (comment ranges, the
boundary comment, a counter test bypassing the real MCP entry —
noted, the production wiring verified — a contradictory test comment,
and the unpinned two-second sleep, marked) all folded; yellow gate at
b0930b51 — lint, test, spec, and stress all OK after one cycle that
caught the CLI line cap.

Residuals: the blocking-stack raise is proven by construction and
the depth-cap test, not a live stack-exhaustion run; POST retries and
TRIAL_ALIAS_CACHE growth stay deferred as recorded in the ticket.
