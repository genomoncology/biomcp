# Replace text-scanning checks with compiler tools

Ian approved this move to 1.0 on 2026-09-29 (recorded in the review file 2026-09-29-review-of-the-enrichr-cache-and-records-round.md and main commit 5bb12367). The homemade checks (the wait ratchet, the stdio guard) scan source text for patterns; each review finds a new spelling that slips past, and the patch cycle has no natural end.

## Scope

- Clippy's disallowed-methods understands aliases and imports; it can replace most of the wait ratchet and the stdio guard, with the enforcement inside the compiler instead of beside it.
- actionlint or zizmor covers the workflow-file checks (BASH_ENV, step pinning, needs graphs) with maintained parsers.

## Owner and trigger

Owner: the developer agent on Ian's queue. Trigger: the 1.0 feature-track start.
