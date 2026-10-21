---
priority: 1
---

# 2258 — Admit the 0.9.4 release candidate

Status: OPEN.

Milestone: 0.9.4

Depends on: 2257, 2244, 2111, 2262, 2276, 2277, 2278, 2279, 2280, 2281, 2282, 2283, 2285, 2233, 2234, 2286, 2287, 2288, 2289, 2290, 2291, 2236, 2284, 2292, 2293, 2294, 2295, 2296, 2297, 2298, 2299, 2300, 2301, 2302, 2303, 2304, 2305, 2307, 2308, 2309, 2310, 2311, 2312, 2313, 2315, 2316, 2317, 2318, 2319, 2320, 2272, 2306, 2321, 2322, 2323

Reviews: revision 1b8e728, accept

Reviews: revision fffcf09, accept

Reviews: revision 5db2cfd, accept

Reviews: revision 07e81e6, accept

Reviews: revision 851911d, accept

Reviews: revision f4dd2b6, accept

## Outcome

Main holds a stable 0.9.4 candidate that meets `docs/reference/release-process.md`, passes the full Linux fresh-clone offline qualification and one complete untagged release rehearsal, and has a PM go request naming the exact commit. Ian alone authorizes the tag. This ticket creates no tag or publication.

- Proof: The final source passes the full local main check, fresh-clone Linux qualification and full untagged release rehearsal; the go request names their exact revisions and jobs.

Changes: `sdlc/records/2258-release-admission.md`, `sdlc/release-checklist.md`, `docs/reference/release-process.md`, `CHANGELOG.md`, `AGENTS.md`, `sdlc/tickets/**`, `sdlc/plan.md`, `sdlc/planning/**`, `sdlc/archive/planning/**`, `sdlc/decisions/**`, `sdlc/records/**`, `sdlc/issues/2026-09-09-clingen-car-rate-limiting-degrades-batch-identity-verification.md`.

## Evidence

Historical fixture and live-replay evidence has been transferred and hash-verified in the permanent overflow store. The admission record names the canonical copies; the original experiment builds may be removed when this ticket closes.

The release admission record names the exact candidate, final Linux job, successful full untagged release workflow, and unexercised publication operations. Historical measurements stay bound to their measured source under the PM ruling; no measurement refresh or final-source speed claim is required. Disabled sources stay explicitly unverified. The execution trail lives in pm job logs and the owning retained raw evidence.

Final gate:

Source `cdb9f4a1bbb37195b512c42351638a3cffe0870d` passed required M5 candidate gate `2323/1792582004700-5442` and complete untagged rehearsal [39593499234](https://github.com/genomoncology/biomcp/actions/runs/39593499234), including native Windows and `rehearsal-complete`. The completion reviewer issued technical GO. Local main `2258/j637` and fresh Linux `2258/j643` remain bound to `a4918fa3e298b8cc48eb900fb720ea041868850c` and are reused only under the approved reviewed path boundary. The admission record retains every binding, failed attempt and publication limit. Administrative closure changes only SDLC; Ian alone publishes the explicit rehearsed source.

## Progress

- 2026-10-20 landed 368a8ac8f5074d9f78bae5ee5a9b0ff232313572; next: Land remaining accepted release work, complete final qualification and rehearse without publication
- 2026-10-20 next: Qualify frozen main b91c4d46bb33741e76065dbe81e3b406d0e1596c with the final main gate and fresh Linux offline run, then complete one full untagged rehearsal and send the PM go request; no tag or publication
- 2026-10-20 next: Resolve2322 after failed fresh Linux j402 on b91c4d46b, then qualify the repaired final source and complete the full untagged rehearsal; no tag or publication
- 2026-10-20 next: Qualify repaired final main a4918fa3e298b8cc48eb900fb720ea041868850c with the main gate and fresh Linux run, then complete full untagged rehearsal and hand the exact commit to PM for Ian; no publication
- 2026-10-20 next: Local qualification passes on a4918fa3e298b8cc48eb900fb720ea041868850c (main j637, fresh Linux j643); monitor full untagged rehearsal39540105855, then record proof and hand the exact commit to PM for Ian; no publication
- 2026-10-21 landed 13eb246d8a566ead86bb1fbe2c6ac76dc45c0a28; next: Complete full untagged rehearsal on the corrected candidate and hand the qualified commit to PM for Ian; no publication
- 2026-10-21 next: Passed candidate gate and approved metadata-only correction landed13eb246d8a566ead86bb1fbe2c6ac76dc45c0a28; monitor full untagged rehearsal39541833383, then land final proof and hand the exact commit to PM for Ian; no publication
- 2026-10-21 next: Resolve2323 after failed Windows installed-wheel cache smoke in rehearsal39541833383, then complete a successful full untagged rehearsal and hand the exact qualified commit to PM for Ian; no publication
- 2026-10-21 next: 2323 landed cdb9f4a1bbb37195b512c42351638a3cffe0870d after the required M5 gate passed; monitor full untagged rehearsal39593499234, then land final proof and hand the exact qualified commit to PM for Ian; no publication
