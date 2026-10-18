---
priority: 3
---

# 2203 — Review mixed-version use across the daily clean

Status: OPEN.

Milestone: 0.9.4

## Outcome

An agent test review alternates v0.9.2 and current BioMCP on one cache with a small size limit across at least two daily cleans and checks the saved byte total and size enforcement.

- Evidence: the 0.9.3 release review could not run past 24 hours of wall time.
- Changes: `sdlc/records/2203-mixed-version-day-boundary-review.md` `sdlc/tickets/2203-mixed-version-day-boundary-review.md`. Review and report only; file any product defect separately.
- Proof: recorded results; file defects as tickets.

## Progress

- 2026-10-18 started

Pre-execution scope: begin from qualified product87d53c6782aaf07dcba4269f7dbd573a5ef00b58 and fixed published v0.9.2, reusing existing cache fixtures and day-boundary mechanisms. Prepare a bounded shared-cache scenario and artifact identities before executing; do not introduce clock frameworks or product hooks. Development builds/tests run only through pm job. No execution until coordinator admission release.

Approved bounded experiment: alternate published v0.9.2 and exact87d53c current binaries on one owned50KBcache, recording content bytes and saved marker after every operation. Age only the existing startup-clean marker25h twice and invoke the real current cleanup, requiring marker refresh, exact post-clean reconciliation and size enforcement. Legacy writes may leave a stale current marker between maintenance runs and must be reported honestly. This controls the due-marker state; it does not claim actual48h elapsed, expiry-clock coverage or long-lived MCP coverage. Use existing retained-cache environment and gene fixture, one finite run, verified artifact identities, no new clock framework/hooks. Current binary copy is verified batch artifact; public old-release download is authorized after coordinator admission.

Experiment result: job `2203/1792355882950-20969` failed its size assertion after 6.597 seconds on fixed current product `87d53c6782aaf07dcba4269f7dbd573a5ef00b58`. Three successful CLI processes used one 50,000-byte cache: current `BRAFseed` left actual/marker 17,440/17,440; published v0.9.2 `BRAF1old1` left 34,882/17,440; current `BRAF1current1` left 52,332/34,890. The final current command exceeded the budget by 2,332 bytes and omitted the old write's 17,442 bytes from its saved total. Per the coordinator's stopping rule, no further command ran and neither planned controlled daily clean occurred. Full identities, commands, observations and limits are in `sdlc/records/2203-mixed-version-day-boundary-review.md`; owned experiment `2077` preserves the failing cache and raw evidence for a separate repair. Product code is unchanged. The review found a defect; it does not establish the ticket's two-clean acceptance criterion.
- 2026-10-18 next: Finite mixed-version review stopped at first real size breach: current command left52332actual bytes against50000budget after uncounted oldwrite, marker34890. Report02d528e5761a3919003222c31814e534998757dc records exact artifacts and limits. Approved priority2 repair2223 now owns the next implementation step; keep2203 OPEN and raw experiment2077 intact. After2223, finish the two controlled daily-clean invocations and prove exact marker reconciliation; neither has run yet.
- 2026-10-18 next: PM chose bounded daily reconciliation for legacyv0.9.2 writes: temporary mixed-version overage is documented; fast paths remain unchanged.2223 is now policy documentation, not a product-code prerequisite. One finite follow-up is released to finish BOTH controlled daily-clean invocations, preserve original failedstate, record every intermediate value, and prove each post-clean exact marker and physical<=50000. No actual48h claim or rebuild. KeepOPEN until this proof is reviewed.
