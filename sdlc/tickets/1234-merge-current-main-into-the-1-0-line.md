---
flow: build
priority: 1
deps: []
---

# 1234: Merge current main into the 1.0 line

## Outcome

The 1.0 line contains main's tip and keeps every 1.0 behavior, including its BioData dependency. New 1.0 work, starting with ticket 1235, builds on main's crash, TLS, and label fixes.

## Current Facts

Measured on 2026-09-23 with the 1.0 line at `f99229e8` and main at `43edc43c`:

- `git merge-base HEAD origin/main` is `3b1d6452`.
- `git rev-list --count 3b1d6452..origin/main` is 102. `git rev-list --count 3b1d6452..HEAD` is 96.
- Main's side carries tickets 1219 through 1232 and the filed, unimplemented ticket 1233. Among them: the MCP server survives a panicking tool call (1230), every TLS client honors an operator CA bundle (1221, 1231), FDA labels show boxed and legacy warnings (1232), and container, CI, and release hardening (1219, 1220, 1222, 1226, 1229).
- 28 files changed on both sides since the merge base, including `src/mcp/shell.rs`, `src/sources/mod.rs`, `src/error.rs`, `src/cache/migration.rs`, `Cargo.toml`, `Cargo.lock`, and `AGENTS.md`.
- The `src/cache/migration.rs` conflict is test code only. The 1.0 side deleted about 200 test lines. Main changed 6 lines to remove a flaky timing assumption.
- Main switched `[profile.release]` from `panic = "abort"` to `panic = "unwind"` for ticket 1230. The 1.0 `Cargo.toml` still says `abort` (`Cargo.toml:155`). Git merges this hunk without a conflict, so a wrong result would pass unnoticed.
- Both lines trigger `.github/workflows/ci.yml` only on pull requests and on pushes to `main`. `release.yml` runs only on a published release or a manual dispatch.
- `git merge-tree --write-tree --name-only HEAD origin/main` reports 11 files with text conflicts: `.github/workflows/ci.yml`, `architecture/technical/overview.md`, `docs/blog/we-deleted-35-tools.md`, `docs/getting-started/claude-desktop.md`, `docs/reference/mcp-server.md`, `docs/reference/release-process.md`, `src/cache/migration.rs`, `tests/test_docs_changelog_refresh.py`, `tests/test_documentation_consistency_audit_contract.py`, `tests/test_source_package_boundary.py`, `tools/rust-source-size-inventory.json`.
- Main removed BioData before the merge base (record 1183), so main's side deletes no BioData wiring in this merge. The 1.0 `Cargo.toml` pins `biodata` by git revision and lists the BioData boundary tools and tests in its package includes.

## Scope

- One merge commit of `origin/main` into `biodata/biomcp-1.0`. No rebase and no squash.
- Resolve each conflict so both sides' behavior survives. The risky conflicts are `tests/test_source_package_boundary.py`, `tests/test_documentation_consistency_audit_contract.py`, and `tests/test_docs_changelog_refresh.py`. Every BioData guard in them survives the resolution.
- Confirm the merged `[profile.release]` says `panic = "unwind"`.
- Regenerate `Cargo.lock` and `tools/rust-source-size-inventory.json` rather than hand-merge them.
- Keep main's release and container workflow changes.

## Exclusions

- No new behavior and no refactor beyond what a conflict forces.
- No change to main.
- No BioData revision bump.

## Acceptance

1. `git merge-base --is-ancestor origin/main HEAD` succeeds at the pushed SHA.
2. `Cargo.toml` still depends on `biodata` at the same revision, and `tools/check-biodata-boundary.py` passes.
3. Main's new tests pass on the 1.0 line: `cargo nextest run --test tls_ca_bundle_contract`, `cargo nextest run --release -E 'test(worker_panic)'` (includes `worker_panic_on_execute_thread_surfaces_as_an_error_result`), and `cargo nextest run -E 'test(extract_inline_label) | test(extract_label_boxed_warning)'`.
4. The merged `Cargo.toml` has `panic = "unwind"` under `[profile.release]`.
5. The `on:` block of `.github/workflows/ci.yml` is unchanged: pull requests plus `push: branches: [main]` only. No workflow runs automatically on this branch, as `AGENTS.md` requires. `release.yml` still triggers only on a published release or a manual dispatch.
6. `make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA.
7. `tools/check-biodata-1.0` passes, and the SHA is handed to the migration manager for BioData's hosted verification, as `AGENTS.md` requires.
8. The record lists each conflicting file and how it was resolved.

## Dependencies

None.

## Complexity

- Contract score: 1 (no new behavior, but two lines' contracts must both hold)
- State and timing score: 1 (the panic strategy auto-merges and changes runtime behavior)
- Reach score: 2 (28 shared files across runtime, CI, docs, and tests)
- Proof score: 2 (full gates plus the BioData focused runner)
- Cost of error score: 2 (a wrong resolution can silently drop a fix or the BioData dependency)
- Total: 8
- Minimum level floor: none
- Final level: 3
- Reasons: broad reach, silent auto-merged panic strategy, BioData guards inside conflicting tests
- Selected model: claude-opus

## Review

- Design review: accepted with edits (2026-09-23). Dropped the moot BioData-wins rule, named the BioData guard conflicts, added workflow trigger and panic strategy checks, corrected the cache migration fact, named the test commands.
- Code review: pending. Completion record: `sdlc/records/1234-merge-current-main-into-the-1-0-line.md`.
