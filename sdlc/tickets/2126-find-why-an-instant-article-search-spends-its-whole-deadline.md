# 2126 — Find why an instant article search spends its whole deadline

Status: OPEN.

Milestone: 0.9.3

## Outcome

The article-query contract test that 2102 bounded at 600 s answers in a few seconds under the product's own 60 s deadline, because the cause of the wait is found and fixed, and its budget returns to the product deadline.

## Evidence

Found by the 0.9.3 adversarial review on 2026-10-16, against 2102 (landed 16f0a1320) and 2119.

- The test sends one Semantic Scholar search, limit 1, to a loopback fixture that answers at once, and took 60.198 s on a quiet host: the full 60 s article-search deadline.
- 2102 raised the budget tenfold to 600 s (`tests/rmcp_client_contract.rs:1179-1188`) without finding why the search uses the whole deadline. 2119 then proposed raising nextest's 240 s limit for it too.
- 2103 found the shared cause for the sibling tests, child processes waiting on the user's default cache, and gave them fresh cache folders; this test never got one.
- A search that runs to its deadline against an instant provider is a product failure a clinician would feel.

- Starts from: the test above, 2103's fix, the article search fan-out and its per-source deadlines.
- Decisions:
  - Find the waiting step first and record it in this ticket. If it is test isolation, give the test the same fresh cache 2103 gave its siblings. If it is the product, fix the product path so an instant provider answers instantly.
  - Restore the test's budget to the product deadline; do not raise any test runner limit.
- Changes: claims `tests/rmcp_client_contract.rs` and the source files the cause names; the lane adds them to this ticket before its first push. `CHANGELOG.md` if the product changes.
- Proof: the test passes under its product deadline in a few seconds, alone and beside a second closing suite on the Linux gate host; the cause and its evidence are recorded here.
- Order: replaces 2119, which is withdrawn.

## Cause, recorded before the fix (2026-10-16, developer)

The waiting step is the spawned server's own start-up walk of the HTTP cache, not a provider round-trip. Shared-client construction (`build_http_client_with_config_and_manager`, `src/sources/mod.rs`) runs `secure_managed_tree` recursively over `cache_root/http` on every process start, and `secure_entry` consults the variant/article deadline at every entry, so the walk runs until the article-search deadline is spent and the search dies on deadline without the fixture being asked. This test never pinned `BIOMCP_CACHE_DIR`, so its child inherited the default user cache; 2102's 600 s override only let the walk finish first (~60 s on the quiet host) and hid it. The day-due start-up clean is not the step: the diagnostic ran with a clean marker 6.6 h old, so the clean was skipped and the walk alone spent the time.

Diagnostic on the Linux check host (Yellow), ad89df69b, job-run directory `job-runs/biomcp/2126/1792180014383-1693189` (queued by this lane's first pass, 2026-10-16):

- Arm 1, ambient default cache (206 MB, 20,940 files under `http/`, clean marker fresh), test as-is with its 600 s override: the child never converged and nextest's 240 s slow-timeout killed it — 0 passed, 1 timed out.
- Arm 2, fresh cache root via `BIOMCP_CACHE_DIR`: the whole test passes in 0.44 s; `start-up cache clean skipped` is the only cache debug output.
- Arm 3, `strace -f -e trace=openat`: 30,932 opens in the process tree, 10,138 of them under `http/content-v2/sha256/` — the walk opens every shard directory of the content store.

So hypothesis one holds: test isolation. The fix gives this test the fresh cache root 2103 gave its siblings and removes the 600 s deadline override, returning the test to the product's 60 s deadline; no runner limit changes. The walk itself is a product start-up cost ticket 2133 already claims (`src/cache/**`, stop every command walking the whole HTTP cache), so this ticket changes and claims only `tests/rmcp_client_contract.rs`. Proof to come: the test alone, then beside a second closing suite, each under the product deadline.
