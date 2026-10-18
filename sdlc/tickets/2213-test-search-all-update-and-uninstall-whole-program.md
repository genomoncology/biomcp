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

PM decision: approve the split. Reuse existing whole-program search-all JSON/Markdown and counts-only golden tests instead of duplicating them; implement real-binary uninstall first. Do not add a production release-source override or TLS interception framework. For update, add a live opt-in test using the published v0.9.2 binary in a temporary install prefix, run update --check and update against the real GitHub release, and verify current published version and receipt. Keep that test outside routine/offline gates and run it explicitly at release qualification; file a design prerequisite only if this proves unworkable. Initial exact implementation claim: `tests/golden/spec/test_install_lifecycle.py`, reusing existing replay.command_env/BINARY/receipt conventions; update exact live-test claim before editing it. Check uninstall success, missing/invalid receipt refusal, and preservation of an unrelated sibling. Use a meaningful bounded mutation such as retaining the owned receipt to show the test fails. Isolate HOME/config/cache/prefix; never update or uninstall the host installation. Build jobs wait for the first2191 idle baseline. Root records partial accepted batches with pm progress and keeps this ticket open until its complete outcome is met.
