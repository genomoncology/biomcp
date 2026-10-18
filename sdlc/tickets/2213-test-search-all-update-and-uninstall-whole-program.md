---
priority: 2
---

# 2213 — Test search all, update and uninstall end to end

Status: OPEN.

Milestone: 0.9.4

## Outcome

`search all`, `update` and `uninstall` each have whole-program tests that run the shipped binary.

- Evidence: the 0.9.3 checklist found no whole-program tests for these three commands; 2081 kept them and asked for coverage through a staged local release before deleting their owning tests.
- Changes: outside-in tests over recorded replies for `search all`, and a staged local release for `update --check`, `update` and `uninstall`; `CHANGELOG.md` only if behavior changes.
- Proof: each test fails when its command is broken on purpose.

## Progress

- 2026-10-18 started
