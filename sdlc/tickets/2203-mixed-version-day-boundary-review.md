---
priority: 3
---

# 2203 — Review mixed-version use across the daily clean

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision b645f49, accept

Reviews: revision 3e701aa, accept

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

PM compatibility ruling supersedes that initial defect classification: v0.9.2 writes may leave the cache above the current size limit until daily maintenance. Keep fast paths without a per-operation scan; 2223 documents the boundary without a code repair. Authorized follow-up job `2203/1792356717813-28154` passed in 1.008 seconds on the same verified executable identities. A fresh finite thirteen-command pass was necessary because the original stopped provider's ephemeral port was embedded in its cache keys. Both production daily cleans ran after aging only the owned startup marker 25 hours, rewrote their startup markers, and ended with exactly `v2 34892\n` against 34,892 actual content bytes, below the 50,000-byte limit. Intermediate mixed-version overages are recorded separately. Both cleans began with already exact, in-budget accounting following ordinary eviction, so this pass proves two successful controlled daily-clean invocations but does not prove the clean itself restores a stale, over-budget cache. No actual 48 hours elapsed or expiry-clock/long-lived-process behavior was claimed. Original evidence remains unchanged; follow-up evidence is in owned experiment `2077/followup`. A preceding setup-only job `2203/1792356699681-26929` failed before executing any product command. No rebuild, new download or product change occurred in the follow-up.

Targeted recovery proof: the coordinator authorized one additional copied-cache challenge to settle that remaining gap. Job `2203/1792356877113-39081` passed in 0.646 seconds with one current CLI command. An unchanged copy of the original failing cache started with 52,332 physical bytes and stale `v2 34890\n`. Before any current command, only its startup marker was aged 25 hours. The probe rebound the original loopback origin with a guard that rejected refill attempts and queried the newest cached `BRAF1current1`. Production daily cleanup rewrote its startup marker, returned physical usage to 34,892 and saved exactly `v2 34892\n`; the CLI returned the BRAF answer with exit 0 and no stderr. Zero provider requests and a surviving-content hash subset exclude a later write repairing the state first. The original cache manifest remained unchanged. Thus the two controlled daily cleans and the separate stale/over-budget recovery challenge all passed within the PM's accepted compatibility boundary. No actual 48-hour, expiry-clock, concurrency, or long-lived-process claim is made. Raw challenge evidence remains owned by 2203 through fresh review.
- 2026-10-18 next: Targeted original-overage daily-recovery proof independently accepted at b645f4969091dc2fc5dc0c7cfef965473ea7ea39; all commands and limits recorded. Verified four finished job results/no process groups and removed only owned raw experiment2077. Ready to close with accepted documentation batch after2077 solo qualification.
