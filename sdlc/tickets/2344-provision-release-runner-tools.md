# 2344 — Provision the declared release-runner tools

Status: OPEN.

Milestone: 0.9.4

## Outcome

The shared release-qualification setup provisions the already declared host tools before preflight runs. Linux qualification has working ripgrep and bubblewrap, including the existing kernel-isolated offline test path. Keep preflight, tool versions, advisory policy and offline failure behavior intact.

- Changes: `.github/actions/lane-setup/action.yml`, `tests/test_ci_workflow_contract.py`, `tests/test_gate_preflight.py`, `sdlc/tickets/2344-provision-release-runner-tools.md`.
- Proof: reproduce the setup omission against the existing declared preflight, verify the affected setup/preflight contracts through a build job, and require actual lint and offline qualification to pass in the full untagged rehearsal. Inspect all declared prerequisites together to prevent a second missing-tool cycle. No silent skips or fallback.

## Evidence

- Coordinator restatement: make the release runner satisfy its existing tool contract before checking it, without changing that contract or product behavior.
- Corrected full rehearsal 39686850300 on published b49a5096fa2a9d2d77517e65ab26b62fc74ee97a passed version and changelog checks, then qualification/lint job 119524809613 failed gate-preflight before lint: make lint invokes bash tools/gate-tools, which reports missing rg --version and bwrap --version and exits 2. The shared release build and repository-contract jobs passed. Other already-running lanes retain their actual results; this attempt is not a complete rehearsal pass.
- The original bootstrap/CI setup developer owns the bounded class diagnosis and repair. The qualified runtime and package inputs remain held. Any qualification reuse must state exact changed setup/test files and run the real corrected release runner; a passing local machine with preinstalled tools does not prove GitHub provisioning.
