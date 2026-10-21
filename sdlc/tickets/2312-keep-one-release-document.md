---
priority: 2
---

# 2312 — Keep one release document

Status: OPEN.

Milestone: 0.9.4

Reviews: revision ea5563b, accept

Reviews: revision ec16b3f, accept

## Outcome

The release process lives in one place, `docs/reference/release-process.md`. `sdlc/release-checklist.md` and the release section of `AGENTS.md` shrink to a pointer. The process drops the fresh review of records-only commits and the full rehearsal after a records-only change. A release ticket points to the document; it does not copy and tick the checklist.

- Proof: One file holds the release steps; grep finds no second copy of them; tests that read the old paths read the new one.

## Evidence

The release steps are described in three places, and the checklist demands review of records-only commits and repeated rehearsals; 0.9.1 needed six rounds of go requests. From Ian's process assessment of 2026-10-20.

- Changes: `docs/reference/release-process.md`, `sdlc/release-checklist.md`, `AGENTS.md`, `CHANGELOG.md`.
