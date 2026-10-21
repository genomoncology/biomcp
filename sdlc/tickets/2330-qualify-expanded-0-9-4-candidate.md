---
priority: 1
---

# 2330 — Qualify the expanded 0.9.4 candidate

Status: OPEN.

Milestone: 0.9.4

Depends on: 2324
Depends on: 2326
Depends on: 2327
Depends on: 2328
Depends on: 2329
Depends on: 2331
Depends on: 2332
Depends on: 2333
Depends on: 2334
Depends on: 2335
Depends on: 2336
Depends on: 2337
Depends on: 2338
Depends on: 2339
Depends on: 2340
Depends on: 2341
Depends on: 2342

## Outcome

After the required fixes land and pass review, qualify the exact new 0.9.4 source with the full local main gate, fresh-clone Linux offline qualification, authorized Semantic Scholar live check and complete untagged release rehearsal. Hand the explicit qualified and rehearsed commit to the PM for Ian. Ian alone publishes.

- Proof: the new source passes the required final local main and fresh Linux checks and every required job in one complete untagged rehearsal, including native installed-wheel checks, both containers and rehearsal-complete. Retained raw evidence and the fresh completion review name the actual revisions and limits. No earlier source pass substitutes for these new runs.

Changes: `sdlc/tickets/2330-qualify-expanded-0-9-4-candidate.md`, `sdlc/records/2330-expanded-release-admission.md`.

## Evidence

- PM m2994 relays Ian's 2026-10-21 decision to include 2324, 2327, 2328 and 2329 before publishing 0.9.4. PM m3064 subsequently adds 2326 and 2333; the earlier exclusion of 2326 is superseded.
- The earlier source cdb9f4a1bbb37195b512c42351638a3cffe0870d and its successful rehearsal 39593499234 remain historical evidence. They are no longer the release candidate, and their local qualification reuse approvals do not carry over to product changes.
- Follow docs/reference/release-process.md. Keep each product ticket's reproduction, review and gate proof in its owning record. Land the toolchain/gate change separately from product changes; batch accepted noncolliding product changes. Run the final main check once on the final source, not after every landing.
- Preserve historical measurements at their measured sources; this ticket makes no unsupported speed claim. Under PM m3064, 2326 must pass correctness, deterministic concurrent-process proof, security composition, the full routine and fresh review. The final paired gaps meet the accepted target. Ian's later m3121 ruling attributes the historical slowdown to system load and closes that limit; step 5 remains deferred unless required for correctness. Its final paired measurements use both 0.9.3 and the corrected branch on the M5.
- No tag, public release, package upload, deployment or announcement is authorized. The final handoff names the exact source rather than an advanced administrative HEAD.
- PM m3019 authorizes a read-only live check of exactly five Semantic Scholar recordings using the inherited S2_API_KEY, paced at one second. No agent inspects, prints, copies, records or stores credentials; logs and fixtures contain no credentials or authorization headers. KEGG stays uncalled. After actual success, remove the Semantic Scholar unverified limit from the release handoff. Ticket 2331 supplies the narrowly supported checker path before this admission.
- PM m3026 adds 2332 to 0.9.4 before admission: keep credentials out of cache metadata and logs, retire legacy stores and clean BioMCP-owned caches on the authorized machines. The live check waits for that reviewed fix.
- PM m3064 also requires 2333 to unify the release workflow inventories without losing their existing protections. All required changes precede final qualification and the full rehearsal.
- PM m3073 adds the four research-study fixes: 2334 frameshift parsing, 2335 reversed help, 2336 bounded literature retries and 2337 capped-route recovery. Sequence 2337 with 2327's mechanism. The exact study commands and minimized permitted replies come through the PM's sanitized handoff. Include all four before final admission.

Resolved tool interruption: On 2026-10-21 the live pm database reported schema 14, while installed revision c8527e1 and the clean pm main checkout support schema 12. Job, inbox and mail commands all refuse this mismatch, including the coordinator's attempted escalation through `pm send biomcp`. Ask Jarvis and the pm owner to restore a compatible reader without losing existing job evidence. No agent changed the database or installed tooling. Remaining product sources and reviewed fixture repairs are preserved on their ticket branches; new gates and landings await compatible pm.

- PM m3178 reports that unfinished pm code ran on the live database during the 13:44–13:54 interruption. Jarvis restored compatibility without losing mail or job rows. The coordinator independently reran inbox, daily, next and job-list reads successfully, recorded the pending reviewed fixture acceptance, and resumed only the refused landing/full-gate operations. Previously passed focused evidence remains valid.

- Final Linux recipe review found that the earlier prepared command-final.sh supplied only development-profile feature output proof and omitted release-profile specifications. Its retained SHA256 b496079787599ae41926e7c56d0d5bb9eb6905d2294dfd1c613563c7ad0086d8 remains historical preparation, not a qualifying run. The corrected command-final-release.sh retains fresh-clone isolation and full suites, invokes the declared full-feature-check with FULL_FEATURE_PROFILE=release, records the optimized artifact, then runs declared release-profile specifications. Its SHA256 is 4160458ac4e97af8fa7be3e421b893ed086cb7b9e10b1dd4bed995f287f6e5df. Both recipes live under Yellow data/biomcp/release-evidence/0.9.4/qualification-2330. The direct main-gate remains a separate unchanged declared job command; the full rehearsal supplies its actual workflow qualification and platform proof.

- PM replies to m3211 and m3212 authorize four fresh-review corrections on tickets/pm-artifacts at bfcbfc2f5: 2338 narrows the local-guide promise to embedded skills and the documentation site and removes the unused packaging path; 2339 fully parses installed-smoke JSON; 2340 replaces Windows managed-directory DACLs exactly; 2341 repairs broad variant pagination if the isolated probe confirms the counterexample. All four precede final admission. Preserve completed review scope, and bind final qualification and rehearsal only after their product changes land. The low long-suite documentation mismatch has no currently omitted marked product cases and is not a separate product blocker.

- The canonical fresh project review is recorded in sdlc/records/2330-expanded-release-admission.md, pinned to integration 3c8aee87a69c5745412e46ef706b9f11c7c209cc. It holds admission for the four PM-authorized corrections. Actual isolated Gator j2686 confirms pagination repetition; the other findings retain their source/platform proof limits.

- PM m3223 retains confirmed pagination correction 2341 in 0.9.4 and assigns removal of the unsupported workflow long-suite claim to 2339 as a text correction only. Legacy pm housekeeping is outside this product release and admission.

- PM m3262 approves the documented hand-merge landing path for 2338–2341 because the legacy batch command classifies every script/workflow path as a gate definition. One isolated candidate carries four reviewed no-ff ticket merges; one direct make landing-gate tests its exact source; publication follows the actual pass; standard pm ticket land N --merge records each carried merge and cleanup. No tool edit, receipt copying, check exemption or gate skipping is permitted. Fresh reviewers read the changed smoke/workflow paths; final main qualification and full rehearsal remain mandatory.

- PM m3274 adds 2342 to 0.9.4 before admission: feature coverage reads LLVM JSON regions rather than LCOV lines, exposes unrun closures sharing executed lines, and resets the region baseline once. This admission uses that actual region floor. Collect the baseline after the currently accepted product batch lands so the one reset covers its source; final qualification and rehearsal follow 2342.
