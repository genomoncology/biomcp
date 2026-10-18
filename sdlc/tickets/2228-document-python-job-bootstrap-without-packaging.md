---
priority: 3
---

# 2228 — Document Python job setup without packaging

Status: OPEN.

Milestone: 0.9.4

## Outcome

The repository instructions give one existing Python setup and invocation path that runs focused checks without building an editable Rust package or assuming shell-workflow bootstrap tools contain Ruff.

- Changes: `AGENTS.md` and this ticket. Document the existing commands; add no framework, installer, test inventory or mechanical guard.
- Evidence: separate lanes repeatedly called ordinary uv run after no-project synchronization, triggering an unnecessary maturin editable package build. The2220review attempt1792354064746-74203 and2227interrupted process group81250 required explicit stops before useful test proof.2079and2227also assumed tools/bootstrap-lint-tools installed Ruff and stopped after otherwise successful checks because that executable path does not exist.
- Proof: compare the documented recipe with Makefile sync-python-dev and test-contracts-prepared, and the actual shell-workflow bootstrap tool scope. Use the existing prepared Python environment or `make sync-python-dev` followed by `uv run --no-sync`; use the configured Ruff command from PATH. Verify command availability without compiling or rerunning already-passing suites. No new tests are needed for this instruction-only change.

## Keeps

Keep tool installation, dependency versions and gate behavior unchanged. Preserve focused checks and avoid editable project installation for the docs/contract harness. Public instructions name only BioMCP commands and repository paths.
