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

## Outcome

After the required fixes land and pass review, qualify the exact new 0.9.4 source with the full local main gate, fresh-clone Linux offline qualification, authorized Semantic Scholar live check and complete untagged release rehearsal. Hand the explicit qualified and rehearsed commit to the PM for Ian. Ian alone publishes.

- Proof: the new source passes the required final local main and fresh Linux checks and every required job in one complete untagged rehearsal, including native installed-wheel checks, both containers and rehearsal-complete. Retained raw evidence and the fresh completion review name the actual revisions and limits. No earlier source pass substitutes for these new runs.

Changes: `sdlc/tickets/2330-qualify-expanded-0-9-4-candidate.md`, `sdlc/records/2330-expanded-release-admission.md`.

## Evidence

- PM m2994 relays Ian's 2026-10-21 decision to include 2324, 2327, 2328 and 2329 before publishing 0.9.4. PM m3064 subsequently adds 2326 and 2333; the earlier exclusion of 2326 is superseded.
- The earlier source cdb9f4a1bbb37195b512c42351638a3cffe0870d and its successful rehearsal 39593499234 remain historical evidence. They are no longer the release candidate, and their local qualification reuse approvals do not carry over to product changes.
- Follow docs/reference/release-process.md. Keep each product ticket's reproduction, review and gate proof in its owning record. Land the toolchain/gate change separately from product changes; batch accepted noncolliding product changes. Run the final main check once on the final source, not after every landing.
- Preserve historical measurements at their measured sources; this ticket makes no unsupported speed claim. Under PM m3064, 2326 must pass correctness, deterministic concurrent-process proof, security composition, the full routine and fresh review. Record the remaining GWAS gap as a known limit if it exceeds 0.5 ms after one attribution pass; step 5 remains deferred unless required for correctness. Its final paired measurements use both 0.9.3 and the corrected branch on the M5.
- No tag, public release, package upload, deployment or announcement is authorized. The final handoff names the exact source rather than an advanced administrative HEAD.
- PM m3019 authorizes a read-only live check of exactly five Semantic Scholar recordings using the inherited S2_API_KEY, paced at one second. No agent inspects, prints, copies, records or stores credentials; logs and fixtures contain no credentials or authorization headers. KEGG stays uncalled. After actual success, remove the Semantic Scholar unverified limit from the release handoff. Ticket 2331 supplies the narrowly supported checker path before this admission.
- PM m3026 adds 2332 to 0.9.4 before admission: keep credentials out of cache metadata and logs, retire legacy stores and clean BioMCP-owned caches on the authorized machines. The live check waits for that reviewed fix.
- PM m3064 also requires 2333 to unify the release workflow inventories without losing their existing protections. All required changes precede final qualification and the full rehearsal.
