# 2126 — article search deadline: build record

Built: 2026-10-16. Branch `ticket/2126-article-search-deadline`, tip 76ec214d6 (on main 12166589f). Cause and evidence recorded in the ticket before the fix commit (6018098e5): the test's spawned server inherited the default user cache, and its start-up secure walk of that cache spent the article-search deadline. The fix (76ec214d6) pins a fresh per-test cache root, the same isolation 2103 gave the siblings, and removes the 600 s budget override, returning the test to the product's 60 s deadline. No runner limit changed. No product file changed — the walk itself is 2133's claim — so there is no changelog bullet.

Gates, macOS build host via pm job at 76ec214d6:

- Alone, product deadline, ambient default cache present: PASS in 1.333 s (job 1792181736991-62560).
- Beside a full closing suite in the same worktree (suite run with `--no-fail-fast` in the background, host load 18.39 at measurement): PASS in 1.258 s (job 1792181975495-98406).
- `cargo fmt --check` clean; clippy `--all-targets` stops on three pre-existing lints at main (`src/mcp/shell.rs`, `src/sources/opencitations.rs`, `src/entities/article/ranking/tests.rs`) under rustc 1.95.0 (job 1792181763837-66683).
- Full archived suite on the macOS build host: 4234/4277 pass. All 43 failures reproduce identically at main tip 12166589f on the same host (jobs 1792181879245-85598, 1792182056811-14499, 1792182073581-16595): gencc and ca_bundle fault-injection tests and every spawned-binary integration test fail on macOS from an archive run. Standing those up is 2137's scope.
- The `make lint`, `make test`, and `make spec` targets themselves do not run on the macOS build host yet: `tools/bootstrap-lint-tools` has no Darwin arm64 archive and `tools/run-offline` refuses macOS. Also 2137's scope.

CI at the exact tip 76ec214d6 (run 38960602485): build, rust-tests (full Linux suite), spec-lane, python-lane, repository-contracts, changes, release-checks, generated-sources all green. Lint fails on the zip symlink advisory that also fails main's own run 38959091109 at 12166589f; ticket 2143's lane owns that upgrade. Landing therefore waits on a green main lint (2143), not on this branch.

Linux check host: the lane's first pass queued one diagnostic job (`job-runs/biomcp/2126/1792180014383-1693189`, ad89df69b); its three arms are the cause evidence in the ticket. The one beside-a-suite check the coordinator approved ran at 04b4365fa (`job-runs/biomcp/2126/1792182421046-1724950`): ambient 206 MB default cache present, the test passes in 0.297 s under the product deadline beside a full closing suite that itself finishes green, 4277/4277. `pm job run --machine <linux-host>` from the local repo refuses (machines.json names only macOS build host for biomcp), so the runner was invoked on the Linux check host over ssh; recorded in the ticket.
