# Review follow-ups for tickets 1242 through 1254

Filed 2026-09-26 from an independent read-only review of main at `9908ca6a`. Five fresh reviewers covered 1246, 1248, 1249, 1250, 1251, 1252, 1242 batch 1 and 1254 batch 1. Nothing here blocks. The core fixes are real. The items below are gaps the tickets claimed to close, or claims without evidence.

## Paperwork and process

- Landed tickets still carry stale status lines. `1239:90`, `1241:95` and `1247:44` say "Code review: pending". `1252:134` says "Design review: pending", but the record shows the design was accepted. `1254:112` says "Code review: pending" next to its batch 1 accept. `1249` records one code review reject and no re-review accept. Build the pending-review check from 1254 first. It would have caught all six.
- Twenty-five merged worktrees stay registered. Decision `2026-09-26-landing-agent-removes-its-worktree.md` makes the landing agent remove its worktree and branch. The claim that only `workspace sweep` removes them is wrong. Landed-ticket build folders filled a disk on 2026-09-26.
- Merge `7206240b` committed conflict markers. `a20af5a1` removed them. Run a conflict-marker check before any merge commit.
- The 1251 merge dropped the Resolved section of the schema follow-ups issue after a script failed mid-merge. Stop multi-step git scripts on the first failure (`set -e`).
- Ticket 1253 says fifteen changelog bullets are missing. A v0.9.1 dry run of the gate reports 24: 1220, 1222, 1223, 1224, 1226 through 1237, 1239, 1241, 1242, 1244, 1247, 1248, 1250 and 1254. It still lists 1240 and 1246, which now have bullets.
- `spec-contracts omits two provider fixtures` is untriaged. The issue names the fix: start the provider-contract and variant-identity fixtures in the `spec-contracts` branch of `scripts/run-specs.sh`.

## 1246, icacls stdio

- `tests/test_source_child_stdio_guard.py:93` checks stdout only. Removing `.stdin(Stdio::null())` from the icacls call still passes. The acceptance item says removing the null calls must fail the guard.
- The guard pattern at `:23` misses `process::Command::new` and `use std::process::Command as Cmd`. It stops scanning at the first `#[cfg(test)]` (`:30`), so production code after a mid-file test module goes unchecked.
- `sdlc/records/1246-*.md:30` claims the Windows contract fails on the pre-fix code. No run shows it. Push a scratch branch with the fix reverted and record the failing `windows-contracts` run.

## 1249, glibc floor

- `scripts/check-wheel-glibc-floor.py:55` scans `.so` and `biomcp` members only. The `biomcp-cli` launcher binary in the wheel is never scanned.

## 1250, changelog gate and workflow contract

- `_assert_pipeline_contract` in `tests/test_release_workflow_provenance.py` compares `continue-on-error` against boolean `True`. `continue-on-error: ${{ true }}` on the smoke step or the wheel-smoke job stays green.
- These mutations also stay green: `|| exit 0` on the tag lookup, `set +e` or `trap 'exit 0' EXIT` in the smoke script, `if: ${{ false && true }}` on the docs-live step, docs-live timeout `exit 1` changed to `exit 0`, and `shell: bash {0}` on the smoke step.
- The `expected_ifs` check near line 66 matches substrings. `if: github.event_name == 'push' && !cancelled()` on pypi-publish passes and would publish after a failed wheel-smoke.
- `test_pipeline_mutations_break_the_contract` accepts any `AssertionError`. The record claims each mutation flips a specific assertion. Assert the expected message.
- `scripts/check-changelog-coverage.py:111-113` passes `- Tickets 1226, 1227` and `- see 1226` as described bullets.

## 1251, flat schemas

- ADR `0002-flat-mcp-tool-schemas-for-function-calling.md:36` says the merged root is never narrower. The code comment at `src/mcp/shell.rs:316-323` admits adverse-event `sections` inherits `uniqueItems:true`. Correct the ADR.
- `merge_property` in `src/mcp/shell.rs` keeps the first branch's value on an unhandled clash: a non-string/array type clash, a scalar clash such as `maximum`, or an enum against free text. Panic on any clash it does not handle, so the drift tripwire fails.
- The tripwire at `src/mcp/shell.rs:1876-1880` builds its expected value with the function under test.
- The CHANGELOG 1240 bullet says argument errors now come back as isError. Missing fields, wrong types and erepo unknown fields still return -32602 from rmcp. The 1251 bullet omits that erepo now rejects unknown fields and that a wrong-type `limit` or `offset` now errors.
- Gemini acceptance of `type: ["string","array"]`, a typeless `enum` and `additionalProperties` is claimed and not shown.

## 1252 and 1248, signal waits

- Text still says "one CPU" where the lane pins two: `Makefile:139`, `1252:106`, `1248:30,37`, `CHANGELOG.md:99`, `tests/test_stress_lane_contract.py:1`. `CHANGELOG.md:101` says the scale factor stretches every watchdog. It covers helper-built watchdogs only.
- 1248 closed on green runs. Nobody identified the original failing assertion or reproduced it. If the decoy process was being killed, that is a product bug. Reopen it as a watched issue or record the reproduction.
- `tools/check-test-wait-ratchet.py:86` passes any line containing `watchdog:` with no cap. Its patterns at `:38-43` miss `from time import sleep`, `asyncio.sleep`, `elapsed() >` deadline checks and a bare Rust `sleep(` after `use std::thread::sleep`.
- `make stress` does not run in CI (`.github/workflows/ci.yml:86-88`). It runs only when someone includes it in a yellow gate.
- `2026-09-25-single-cpu-affinity-deadlocks-the-handshake-child.md` has no severity or owner. It says the child holds the pipe's write end while the parent reads end-of-stream (lines 23-25). Both cannot be true, so the diagnosis needs another look. A one-CPU runner or container would fail both lease tests every time.
- `tests/test_stress_lane_contract.py:71` catches `taskset -c 0` only.

## 1242 batch 1, trial partial counts

- `src/entities/trial/mod.rs:832` calls the partial count a floor. Unchecked trials are kept, so the true count can only be lower. The label should say the count may be too high.
- The note at `src/entities/trial/search/ctgov.rs:282` reads poorly ("detail-verified", "trial(s)", doubled brackets) and names two of the three keep reasons.
- No test drives `verify_detail_filters` through the three keep paths. The claim "through all three keep paths" is unproven.
- `src/cli/trial/dispatch.rs:269` drops the partial note from trial search JSON. Only `--count-only` JSON reports it. Ticket 1242 marks item 1 done and does not list this.
- `ctgov.rs:264` applies the age filter after counting unchecked trials. A trial the age filter drops can still mark the count partial.

## 1254 batch 1, DDInter and GenCC

- `src/sources/ddinter.rs:471` labels an HTML download reply as an unreadable bundle and echoes the content-type header. It is a download failure.
- `src/transform/drug.rs:519` now takes brand names from the anchor record only. The ticket does not record this change.
- GenCC tests cover the wrong-mode case only. Wrong owner and not-a-directory have no test.
- Ticket 1254 omits the 1244 release-prep checklist items from the source issue.
