---
base: 38b9411e
head: cc0ea39c
---

Merged `origin/main` at `43edc43c` into `biodata/biomcp-1.0` as one merge commit, `749d67fd`. A second merge, `cc0ea39c`, brought in the concurrent docs-only commit `153b1016` for tickets 2002 and 2003. Main is untouched.

## Conflicts

- `.github/workflows/ci.yml`: took main's unpinned apt installs for bubblewrap, apparmor, and ripgrep. Kept the 1.0 `NODE_VERSION` and Node setup steps. The `on:` block is unchanged: pull requests plus pushes to `main`.
- `architecture/technical/overview.md`: kept the 1.0 development candidate versions (`1.0.0-dev.1` and `1.0.0.dev1`). Took main's text for the single release workflow and container publication.
- `docs/blog/we-deleted-35-tools.md`: took main's text without hand-copied counts. It names the current `1.0.0-dev.1` development build as the build CI measures.
- `docs/getting-started/claude-desktop.md` and `docs/reference/mcp-server.md`: took main's text. The budget line names the current `1.0.0-dev.1` development build. The 1.0 side's copied 0.9.0-dev.5 counts are gone because main's audit rejects copied counts.
- `docs/reference/release-process.md`: took main's runbook for the current `release.yml`. The version section names Cargo `1.0.0-dev.1` and Python `1.0.0.dev1`. The quality ratchet checks this against the package metadata.
- `src/cache/migration.rs`: kept the 1.0 deletion of the deadline tests. Main's timing fix touched one of the deleted tests, so it has nothing left to apply to.
- `tests/test_docs_changelog_refresh.py`: took main's overview assertions and its new single-workflow test. Kept the 1.0 assertion `public metadata remains at 0.9.0`.
- `tests/test_documentation_consistency_audit_contract.py`: took main's rule against copied catalog counts. The 1.0 build name replaces main's `0.9.0 released build`.
- `tests/test_source_package_boundary.py`: kept the 1.0 side in full: the BioData pin, the boundary checker, and the reviewed package sets. Main's only change was a package-count bump, and the 1.0 test has no such constant.
- `tools/rust-source-size-inventory.json`: rebuilt from the 1.0 inventory. Added main's growth as named increases: tickets 1221, 1223, 1230, and 1232 in `drug/get.rs`, `error.rs`, `mcp/shell.rs`, `render/json.rs`, `render/provenance.rs`, and `sources/mod.rs`. Took main's three new entries. `tools/update-rust-source-size-inventory` reports no change.

## Changes outside the conflicts

- `tests/test_upstream_planning_analysis_docs.py`: the 1.0 side asserted phrases from its old release page. Main's page words the same facts differently. The assertions now use main's phrases: `tag` (required), five shipped targets, the protected `pypi` environment, and the Homebrew formula update.
- `tests/test_source_package_boundary.py`: the exact package count went from 1,396 to 1,400. Main added four packaged files: `scripts/check-docs-live-revision.py`, `tests/test_docs_live_revision_gate.py`, `src/sources/ca_bundle.rs`, and `tests/tls_ca_bundle_contract.rs`.
- `Cargo.lock`: git merged it cleanly. `cargo update --workspace --offline` changed nothing, and `cargo metadata --locked --offline` passes.

## Checks

- `Cargo.toml` pins `biodata` at `c9938b99bd091ab4bf6da8b909ed239826e5ab6d` (unchanged). `[profile.release]` says `panic = "unwind"`.
- `release.yml` is main's version. It still triggers only on `release: published` or `workflow_dispatch`.
- `git merge-base --is-ancestor origin/main HEAD` succeeds at `cc0ea39c`.
- `python3 tools/check-biodata-boundary.py` passed.
- `tools/check-biodata-1.0 --already-isolated` passed on Blink under BioData's `sdlc/scripts/run-offline`, after `cargo build --locked --offline --no-default-features --bin biomcp`: 119 Rust tests and 152 Python tests passed, with 1 skipped.
- `cargo nextest run --test tls_ca_bundle_contract` passed 8 of 8. `cargo nextest run -E 'test(extract_inline_label) | test(extract_label_boxed_warning)'` passed 10 of 10. `cargo nextest run --release -E 'test(worker_panic)'` passed 1 of 1 (`worker_panic_on_execute_thread_surfaces_as_an_error_result`).
- Yellow ran from a clean detached worktree at `cc0ea39c` while otherwise idle. `make lint` passed. `make test` passed: 3,703 Rust tests and 1,260 Python tests, with 4 skipped. `make spec` passed.

## Open

The hosted workflow requires the branch tip. Hand the commit that adds this record to the migration manager for BioData's hosted verification dispatch. That commit changes only this record.
