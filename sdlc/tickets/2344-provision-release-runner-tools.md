# 2344 — Provision the declared release-runner tools

Status: OPEN.

Milestone: 0.9.4

## Outcome

The shared release-qualification setup provisions the already declared host tools before preflight runs. Linux qualification has working ripgrep and bubblewrap, including the existing kernel-isolated offline test path. Keep preflight, tool versions, advisory policy and offline failure behavior intact.

- Changes: `.github/workflows/ci.yml`, `.github/actions/lane-setup/action.yml`, `tests/test_ci_workflow_contract.py`, `tests/test_offline_gate_contract.py`, `tests/test_gate_preflight.py`, `CHANGELOG.md`, `sdlc/tickets/2344-provision-release-runner-tools.md`.
- Proof: reproduce the setup omission against the existing declared preflight, verify the affected setup/preflight contracts through a build job, and require actual lint and offline qualification to pass in the full untagged rehearsal. Inspect all declared prerequisites together to prevent a second missing-tool cycle. No silent skips or fallback.

## Evidence

- Coordinator restatement: make the release runner satisfy its existing tool contract before checking it, without changing that contract or product behavior.
- PM reply to m3473 confirms this as the next 0.9.4 repair, requires pinned declared tools and a complete prerequisite inventory across Linux, Mac and Windows release roles, and preserves preflight/offline checks. The actual omission is ci.yml lint's disabled offline setup, not a missing installer: enable its existing offline input. The same rehearsal's stress lane passed the unchanged package installation, scoped AppArmor profile, restricted-userns assertion and actual namespace isolation. Pin the two missing declared utilities to their tested Ubuntu package versions and retain the existing profile checks; the newer PM pin ruling supersedes the old unpinned test assertion.
- The all-prerequisite audit also finds lint lacks nextest:true even though preflight requires it. Its prior success came from a restored Cargo-bin cache; a cache hit is not provisioning. Enable the existing pinned/checksummed nextest installer as part of this same class. Inventory native artifact-job subsets separately; they do not call the full Linux preflight and must not receive blanket Linux-only prerequisites.
- Corrected full rehearsal 39686850300 on published b49a5096fa2a9d2d77517e65ab26b62fc74ee97a passed version and changelog checks, then qualification/lint job 119524809613 failed gate-preflight before lint: make lint invokes bash tools/gate-tools, which reports missing rg --version and bwrap --version and exits 2. The shared release build and repository-contract jobs passed. Other already-running lanes retain their actual results; this attempt is not a complete rehearsal pass.
- The original bootstrap/CI setup developer owns the bounded class diagnosis and repair. The qualified runtime and package inputs remain held. Any qualification reuse must state exact changed setup/test files and run the real corrected release runner; a passing local machine with preinstalled tools does not prove GitHub provisioning.
