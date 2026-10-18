---
priority: 2
---

# 2213 — Test search all, update and uninstall end to end

Status: OPEN.

Milestone: 0.9.4

Reviews: revision ab501ee, accept

## Outcome

`search all`, `update` and `uninstall` each have whole-program tests that run the shipped binary.

- Evidence: the 0.9.3 checklist found no whole-program tests for these three commands; 2081 kept them and asked for coverage through a staged local release before deleting their owning tests.
- Changes: `tests/golden/spec/test_install_lifecycle.py` `tests/golden/live/test_install_update.py`. Reuse existing recorded search-all cases and add isolated uninstall tests plus the approved published-version live opt-in update test; no production behavior change.
- Proof: each test fails when its command is broken on purpose.

## Progress

- 2026-10-18 started

PM decision: approve the split. Reuse existing whole-program search-all JSON/Markdown and counts-only golden tests instead of duplicating them; implement real-binary uninstall first. Do not add a production release-source override or TLS interception framework. For update, add a live opt-in test using the published v0.9.2 binary in a temporary install prefix, run update --check and update against the real GitHub release, and verify current published version and receipt. Keep that test outside routine/offline gates and run it explicitly at release qualification; file a design prerequisite only if this proves unworkable. Initial exact implementation claim: `tests/golden/spec/test_install_lifecycle.py`, reusing existing replay.command_env/BINARY/receipt conventions; update exact live-test claim before editing it. Check uninstall success, missing/invalid receipt refusal, and preservation of an unrelated sibling. Use a meaningful bounded mutation such as retaining the owned receipt to show the test fails. Isolate HOME/config/cache/prefix; never update or uninstall the host installation. Build jobs wait for the first2191 idle baseline. Root records partial accepted batches with pm progress and keeps this ticket open until its complete outcome is met.

Coordinator claim addition before live-test edits: `tests/golden/live/test_install_update.py`, using the existing isolated replay environment/receipt conventions and an explicit module-level skip unless BIOMCP_TEST_LIVE_UPDATE=1. This test is never part of the offline release_gate marker. Run explicitly during release qualification through a build job. Start with an official published v0.9.2 archive/checksum installed only in a temporary prefix, then check availability without changes, perform the real GitHub-backed update and compare the resulting version, receipt and binary hash with current published metadata. No source override, TLS interception, host installation change or new replay framework. Use the actual supported check-only CLI syntax and report any discrepancy before implementing the test. Existing uninstall test claim stays unchanged.
- 2026-10-18 next: Ready branch ab501ee323e82c0fe6ee7b21f704e466494582d0 accepted by fresh review. Actual mutation RED2213/1792350654892-10372 caught exactly3 runtime assertions (retained uninstall receipt and corrupted normal/counts-only search-all JSON count). Its purported GREEN was discarded because the checking harness reused the mutated supplied binary; restored source and BIOMCP_BIN handling were corrected. GREEN2213/1792350776746-44007 passed66.180s: rebuilt actual restored CLI,10passed,1live opt-in skipped,4unselected get cases, Ruff and binary-selection contract/diff passed. Explicit live job2213/1792350860407-62318 passed8.143s: official published0.9.2 updated to published0.9.3 in isolated temporary installation, check-only preservation and final version/receipt/hash/sibling/stage checks passed. This positive live proof covers published releases, not unshipped updater mutations. Only two approved test modules changed; developer/reviewer stopped. Join accepted landing batch after the ORCID test-expectation correction.
