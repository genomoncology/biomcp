---
flow: build
priority: 3
deps: []
---

# 2005: Let the pre-commit hook skip Rust off the dev box

## Outcome

`scripts/pre-commit` runs its non-Rust checks and skips `cargo fmt` and `cargo clippy` on its own, on the development machine, without `--no-verify`. It prints which path it took. Rust builds still run only on the yellow build host, from pushed SHAs.

## Current Facts

- `scripts/pre-commit` runs `cargo fmt --check` and `cargo clippy` for any change that touches a non-documentation path (`scripts/pre-commit`, final block). It skips them today only for a documentation-only commit.
- Rust builds are banned on the development machine. They run only on the yellow build host, against exact pushed SHAs (`CLAUDE.md`, "Where things are").
- Agents commit with `--no-verify` to get past this, which also skips `pre-commit-reject-march-artifacts.sh` and `tools/check-tracked-text`, checks that have nothing to do with Rust.

## Scope

- `scripts/pre-commit` checks an environment variable, `BIOMCP_SKIP_RUST_PRECOMMIT`, or a host check, for the yellow build host's identity, before running `cargo fmt` or `cargo clippy`. When either says to skip, it runs the march-artifact check and `tools/check-tracked-text` as it does today, prints that it skipped the Rust checks, and exits 0.
- When neither says to skip, the hook runs `cargo fmt --check` and `cargo clippy` as it does today.
- Document the variable and the host check in `AGENTS.md` next to the existing Rust-build-location note.

## Exclusions

No change to what runs on the yellow build host. No change to the documentation-only skip path already in the hook.

## Acceptance

1. A test or script exercises the skip path directly (variable set, or host check matching the dev box) and shows the hook printing that it skipped the Rust checks, with the march-artifact and tracked-text checks still running.
2. A test or script exercises the full path (variable unset, host check matching the yellow build host) and shows `cargo fmt` and `cargo clippy` still invoked.
3. `AGENTS.md` names the variable and the host check.

`make lint` and `make test` pass on the gate host at the pushed SHA.

## Dependencies

None.

## Complexity

- Contract score: 1 (one conditional branch in one script)
- State and timing score: 0
- Reach score: 1 (pre-commit hook only)
- Proof score: 1 (two pinned paths)
- Cost of error score: 1 (a wrong skip condition would let a broken Rust commit through, or force a banned local build)
- Total: 4
- Minimum level floor: none
- Final level: 1
- Reasons: a scoped conditional in one existing script
- Selected model: claude-sonnet

## Review

- Design review: pending
