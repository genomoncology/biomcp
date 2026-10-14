# 2081 — Delete every test that is not one of the four kinds

Status: OPEN.

Milestone: 0.9.3

## Outcome

Every Rust and Python test left in the routine suite is a behavior test through a real entry point, an edge-case table, a contract check or a regression test that failed before its fix, and no routine test waits on the wall clock.

## Evidence

- Ian ruled on 2026-10-14 that tests are deleted boldly and come back when a real need shows up ([decision](../decisions/2026-10-14-delete-tests-boldly-and-split-the-suites.md)). This rewrite replaces the bar that combined coverage stays within a few points of 84% and the wait for 2079.
- The 2069 baseline: 4,205 Rust tests and 1,251 Python tests; 62 tests over one second (`sdlc/records/2069-0-9-3-baseline.md`). The lib test unit takes 1.7 times as long to compile as the lib (`sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, Build). 75 files are covered 90% or more from outside; 34 files are never reached from outside (same design, Coverage, step 7).
- 63 test-only items (about 450 lines) and 58 unread fields exist only for construction tests (`sdlc/designs/2026-10-13-0-9-3-dependencies-and-dead-code.md`, Dead code).
- PM decision 2026-10-14: `search all`, `update` and `uninstall` stay, as the 2026-10-13 amendment ruled. Their unit tests go in the last batch, after 2079 gives `search all` golden cases and an outside-in test covers `update --check`, `update` and `uninstall` against a staged local release, including the refusal to touch a package-managed install.

- Starts from: the per-file outside-in coverage method in `sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, the dead-code list in `sdlc/designs/2026-10-13-0-9-3-dependencies-and-dead-code.md`, and the four kinds in the decision.
- Keeps: behavior tests through `biomcp` commands and the MCP server, edge-case tables for parsers and resolvers, contract checks on JSON and Markdown output, the MCP schema and releases, and every regression test that pins a refusal or a fixed wrong answer.
- Changes: claims `CHANGELOG.md`. Each batch names its files on this ticket before its first push, and avoids files an open product ticket claims, so the parallel lane never waits on it.
  - Work in small batches, one module or file group each, starting with the 75 files the outside-in suite already covers at 90% or more. Each batch deletes every test that is not one of the four kinds and the test-only items and unread fields its deleted tests used.
  - Where a kept behavior has no outside-in test, write that test in the same batch before deleting the inside one. Do not add outside-in tests for behavior nobody needs; delete the code instead.
  - Give each slow test that stays an injected clock or a shorter deadline.
  - Each batch lands on its own: `pm ticket progress 2081 --landed SHA --next TEXT`. Run `pm ticket land` after the last batch.
  - One Internal changelog bullet, with the first batch.
- Proof: per batch, the Rust and Python test counts, the lib test compile time and the routine test run time before and after, and outside-in and combined coverage reported, not gated; the first batch also records the share of test code, so the milestone's test-code target can be set; at the end, no routine test takes more than a few seconds in `cargo nextest run --no-default-features` or `pytest --durations=0`.
- Defers: enforcing the per-test limit in the runners (2095); the `search all` golden cases (2079).
