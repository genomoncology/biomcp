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
- `git merge-tree --write-tree --name-only HEAD origin/main` reports 11 files with text conflicts: `.github/workflows/ci.yml`, `architecture/technical/overview.md`, `docs/blog/we-deleted-35-tools.md`, `docs/getting-started/claude-desktop.md`, `docs/reference/mcp-server.md`, `docs/reference/release-process.md`, `src/cache/migration.rs`, `tests/test_docs_changelog_refresh.py`, `tests/test_documentation_consistency_audit_contract.py`, `tests/test_source_package_boundary.py`, `tools/rust-source-size-inventory.json`.
- Main removed BioData on purpose (record 1183). The 1.0 `Cargo.toml` pins `biodata` by git revision and lists the BioData boundary tools and tests in its package includes.

## Scope

- One merge commit of `origin/main` into `biodata/biomcp-1.0`. No rebase and no squash.
- Resolve each conflict so both sides' behavior survives. Where main deleted BioData wiring, the 1.0 side wins.
- Regenerate `Cargo.lock` and `tools/rust-source-size-inventory.json` rather than hand-merge them.
- Keep main's release and container workflow changes. Main's release workflow runs on a published release or a manual dispatch, so a push to this branch does not trigger it.

## Exclusions

- No new behavior and no refactor beyond what a conflict forces.
- No change to main.
- No BioData revision bump.

## Acceptance

1. `git merge-base --is-ancestor origin/main HEAD` succeeds at the pushed SHA.
2. `Cargo.toml` still depends on `biodata` at the same revision, and `tools/check-biodata-boundary.py` passes.
3. Main's new tests pass on the 1.0 line: panic survival, CA bundle fallback, and FDA label warnings.
4. `make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA.
5. `tools/check-biodata-1.0` passes, and the SHA is handed to the migration manager for BioData's hosted verification, as `AGENTS.md` requires.

## Dependencies

None.

## Complexity

- Contract score: 1 (no new behavior, but two lines' contracts must both hold)
- State and timing score: 1 (a cache migration file conflicts)
- Reach score: 2 (28 shared files across runtime, CI, docs, and tests)
- Proof score: 2 (full gates plus the BioData focused runner)
- Cost of error score: 2 (a wrong resolution can silently drop a fix or the BioData dependency)
- Total: 8
- Minimum level floor: level 3 (cache migration is shared durable state)
- Final level: 3
- Reasons: broad reach, cache migration conflict, two dependency contracts to keep
- Selected model: claude-opus

## Review

- Design review: pending
- Code review: pending
