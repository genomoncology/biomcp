# 2324 — Run the landing gate on any build machine

Status: OPEN.

Milestone: 0.9.4

## Outcome

BioMCP's lint passes under -D warnings on current stable Rust, and every build machine and release job uses one declared toolchain. Replace the four deprecated Atomic fetch_update calls with their supported form, keeping behavior. Declare the toolchain in the repo so a machine's rustup update cannot fail lint without a code change. Declare every host tool the gate needs, such as cargo-deny, mustmatch and the system python3 used outside uv, with their versions, and have the gate name all missing tools in its first step.

- Proof: make lint passes on the newest stable rustc and on the declared toolchain; the release workflow and pm jobs build with the declared toolchain; on a machine missing a declared tool, the gate fails first and lists every missing tool.

Changes: `.github/actions/lane-setup/action.yml`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `AGENTS.md`, `CHANGELOG.md`, `Cargo.toml`, `Makefile`, `rust-toolchain.toml`, `src/entities/article/variant_search.rs`, `tests/test_dependency_advisory_gate.py`, `tests/test_gate_preflight.py`, `tests/feature-links.json`, `tests/test_ci_workflow_contract.py`, `tests/test_upstream_planning_analysis_docs.py`, `tests/test_feature_coverage.py`, `tests/test_routine_cargo_feature_contract.py`, `tests/test_release_check_counterparts.py`, `tests/test_release_workflow_provenance.py`, `tools/gate-tools`, `tools/rust-toolchain-version`.

## Evidence

- The Gator early-signal gate j1104 for 2323 on 2026-10-21 failed lint: rustc 1.99.1 rejects four existing Atomic fetch_update uses as deprecated under -D warnings. Cargo.toml declares rust-version 1.89, and the repo has no rust-toolchain file.
- Jarvis, 2026-10-21: Gator has 1.95.0, the M5's version, beside its default 1.99.1. Pin 1.95.0 first, then fix the deprecation and raise the pin.
- The Gator rerun j1188 with Rust 1.95.0 passed Clippy, licenses and advisories, then failed spec lint because mustmatch was absent. Host tools failed one at a time across two runs.
- Ian moved this into 0.9.4 on 2026-10-21; 0.9.4 requalifies and rehearses after it lands.
