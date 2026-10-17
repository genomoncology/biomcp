---
priority: 1
---

# 2162 — Reconcile the release queue with local qualification

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 751d7ea, accept

## Outcome

Every open 0.9.3 ticket has a current, observable contract, correct prerequisites and priority under the approved local qualification and untagged rehearsal process.

Depends on: 2156

- Starts from: the four independent readiness reviews of nineteen open release tickets against current source and existing implementation branches.
- Keeps: recorded product decisions, existing useful branch implementations, historical evidence, paused work and review independence. A prior acceptance does not cover a rewritten outcome.
- Changes: claims `sdlc/plan.md` `sdlc/tickets/2073-*.md` `sdlc/tickets/2077-*.md` `sdlc/tickets/2079-*.md` `sdlc/tickets/2081-*.md` `sdlc/tickets/2082-*.md` `sdlc/tickets/2088-*.md` `sdlc/tickets/2094-*.md` `sdlc/tickets/2095-*.md` `sdlc/tickets/2111-*.md` `sdlc/tickets/2120-*.md` `sdlc/tickets/2121-*.md` `sdlc/tickets/2122-*.md` `sdlc/tickets/2123-*.md` `sdlc/tickets/2128-*.md` `sdlc/tickets/2129-*.md` `sdlc/tickets/2139-*.md` `sdlc/tickets/2149-*.md` `sdlc/tickets/2151-*.md` `sdlc/tickets/2152-*.md` `sdlc/tickets/2158-*.md` `sdlc/tickets/2159-*.md` `sdlc/tickets/2163-*.md` `CHANGELOG.md`.
- Proof: a fresh record review checks each amended outcome, explicit prerequisite and active file claim against the reviewed source; pm next shows main stability first, research product defects next, release-speed work afterward and final measurement last. No active contract requires automatic branch/main CI, elapsed-time test failure or a cross-host comparison. Existing product outcomes are not silently expanded or narrowed.
- Defers: implementation and runtime proof to each ticket; broad shared fixture identifiers to the next release; publication to Ian.

PM decision: 2026-10-17. Ian approved the revised plan and queue reconciliation. Withdraw 2121 through pm. Both versions are measured anew on the same build machine with the same corrected harness; full untagged rehearsal is required before tagging. Preserve needed coverage and require active outside-in replacements before test deletion. Public instructions contain no private machine names.
