---
priority: 2
---

# 2202 — Review cache behavior on a full disk

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 5c3ea58, accept

Reviews: revision 1e65533, accept

## Outcome

An agent test review fills a small dedicated filesystem during cache writes and eviction and checks requests still answer, no temp files leak and the size total stays right.

- Evidence: the 0.9.3 release review only simulated low disk with the free-disk setting; a real ENOSPC mid-write was not exercised.
- Changes: `sdlc/records/2202-full-disk-cache-review.md` `sdlc/tickets/2202-full-disk-cache-review.md`. Record the bounded actual-filesystem review; product repairs are separate tickets.
- Proof: recorded results; file defects as tickets.

## Progress

- 2026-10-18 started

Coordinator pre-execution scope: this is a read-only product review with a disposable experiment, not a cache implementation change. Claim `sdlc/records/2202-full-disk-cache-review.md` and this ticket only. Use the ordinary-user disk-image tools on the build host to attach one fixed32MiB filesystem beneath the next numbered experiment folder, OWNER biomcp/2202. Fill only that filesystem, require actual errno28 during a cacheable write, and check the provider answer survives, staging files do not leak, and accounting recovers after removing filler and successful cleanup. Retain the already-approved conservative overcount bound until maintenance; do not invent immediate exactness. No sudo, host disk filling, production cache, or configuration changes. Detach only the owned image in a trap; if ordinary-user attach is unavailable, stop and report capability rather than simulating ENOSPC with disk-floor settings. Reuse qualified2195/2206 behavior checks; report concrete defects for separate tickets, do not fix product code here. Execute through a build job after the initial2191 idle baseline; retain raw evidence until report acceptance, then clean it.
