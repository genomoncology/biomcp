# 2044 — Clear the 2038 code-lane review findings

Status: OPEN.

Milestone: 0.9.2

## Outcome

The 2038 code-lane changes on main meet Ian's terminal-only WHO decision on every surface, keep every explicit filter on every hint path, carry honest per-file size records, and landed through green CI with a recorded review of the landed head.

## Evidence

Filed 2026-10-09 from the independent review of `origin/tickets/landcheck-2038-code` at `336d11eb5` against main `ec8a72d0d`. The lane then landed on main as `d592c469c` and `336d11eb5`, followed by `a2ee01764`, `0246c876c` and `cd257afe8` to fix CI. The shared day rule, the WHO card degrade, the JSON projection and the source registry entries meet their findings, and tests catch regressions in each.

- **MCP still returns the local WHO path.** MCP errors become `format!("Error: {err}")` (`src/mcp/shell.rs:620`), which uses Display. The Display arm at `src/error.rs:831-837` writes the reason and the suggestion, and the suggestion names the data folder. `search drug X --region who` (`src/entities/drug/search.rs:726`, which degrades only for the all-regions search) and the WHO structured search (`search.rs:808`) both reach MCP. Found by reading the code; no test covers it.
- **Main moved before CI was green again.** Run 37970933940 on `d592c469c` failed `make lint`, and run 37975607066 on `336d11eb5` failed four contract tests. Both commits then reached main, and three fixing commits followed directly on main. 2038 finding 12 names this pattern.
- **The four failures were local.** All four reproduce on the build machine in minutes. The builder's offline command skips them because both test files are marked `needs_binary`. The lane spent three CI cycles of about 20 minutes each finding them, and waited on each with a blind `sleep 1700`.
- **The package cap was raised, not the test tooling excluded.** `a2ee01764` raises `MAX_PACKAGE_FILES` in `tests/test_source_package_boundary.py` to admit `.config/nextest.toml`. That file is test tooling and belongs in `Cargo.toml` `exclude`.
- **The size inventory was hand-edited and lost history.** `336d11eb5` rewrote `tools/rust-source-size-inventory.json` with an inline script instead of `tools/update-rust-source-size-inventory`, and was committed with `--no-verify`.
  - All 8 entries lost their earlier tickets' reasons and removal conditions; for example `src/transform/drug.rs` lost "Split the merge tests into a dedicated tests submodule" and `src/error.rs` lost the error-projection condition.
  - `src/transform/variant.rs` shrank from 1661 to 1659 lines. The updater would lower its floor; the edit kept floor 1335 and stamped a 2038 growth reason on it.
  - The shared reason names hint threading in `src/cli/variant/`, which the inventory does not track.
  - The removal condition "the WHO terminal-only rule and the shared day rule are retired" names permanent rules, so it can never trigger.
  - `src/render/json.rs` grew 34 lines, all of it one test inside a production file, and `src/render/markdown/drug/tests.rs` grew 46 lines of tests.
  - The updater stops at the second grown file, so several growths cannot pass in one run. That limit needs a fix in the updater, not a bypass.
- **Hint wiring is untested, and two flags still drop.** Making the refused-path Markdown hint at `src/cli/variant/dispatch.rs:490-494` pass no filters keeps all 3,974 library tests green; the new test calls `gene_first_working_form` directly (`tests/unit/cli/variant.rs:621-625`). When the gene is refused, `--hgvsp` and `--consequence` still vanish: the `Refused` note does not keep them (`src/cli/variant/query.rs:165-185`) and the hint passes `None` for hgvsp (`dispatch.rs:446, 493`). Example: `search variant "FOO melanoma" --hgvsp V600E`.
- **The name guard is half closed.** The secret-driven scan now runs in its own job, but no `PM_FORBIDDEN_NAMES` secret exists, so CI warns and passes. When the secret is set, the scan meets the real names for the first time and may go red on existing text.
- **No review covers the landed head.** The 2038 ticket cites an ACCEPT on `e4f10fad3` stored only at `/tmp/review-2038-code.md`. The hand-resolved merge `d592c469c`, the inventory edit and the three CI fixes are unreviewed.
- **The ticket text breaks house rules.** The new 2038 sections are hard-wrapped and add internal machine wording.
- **The sibling branch is stale.** `origin/tickets/union-2038-code` (`fbb27693b`) pasted the whole 2038 body into a new section and reset the size inventory to main's.
- **One checker quirk.** `tools/check-zero-coupling.py` looks for the local name declaration beside the script, not under `--root`, so the new test patches around it.

- Starts from: ticket 2038 and main `cd257afe8`.
- Keeps: the shared day rule, the WHO card degrade, the JSON projection, the per-test kill budget, the source registry entries and their tests.
- Changes: turn a `BioMcpError` into its public projection in MCP's error branch, with an MCP test for the WHO region search; exclude `.config/` from the source package and restore the file cap; rebuild the inventory entries one file at a time with each file's own reasons and a removal condition naming the split that would shrink it, lower the `src/transform/variant.rs` floor, and move the new tests out of `src/render/json.rs`; let the updater approve several named files in one run; carry `--hgvsp` and `--consequence` in the refused note and add a dispatch-level hint test; run the name check by hand against the real names before anyone creates the secret; read the local declaration under `--root`; unwrap the 2038 text; delete `origin/tickets/union-2038-code` once Ian confirms.
- Proof: an MCP test that fails on `cd257afe8` when WHO data is missing; the source package test passing at 1408 files; `tools/update-rust-source-size-inventory` reproducing the inventory; a dispatch test that fails when the refused-path hint drops filters; the offline suite including the `needs_binary` contract tests run on the build machine before each push; a recorded review of the final head.
- Defers: nothing.
