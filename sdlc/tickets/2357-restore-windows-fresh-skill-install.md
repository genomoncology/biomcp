# 2357 — Restore fresh Windows skill installation

Status: OPEN.

Milestone: 0.9.4

Priority: 1

Changes: src/cli/skill/atomic_swap.rs, src/cli/skill/assets.rs, src/cli/skill/install.rs, src/cli/skill/tests/install.rs, tests/skill_install_contract.rs, tests/feature-links.json, .github/workflows/ci.yml, tests/test_ci_workflow_contract.py, CHANGELOG.md

Reviews: revision 8ced436b6ff0a6ee3a0d5a0e7e18553509e3f5b1, accept

## Outcome

Restore the published fresh skill-install contract on Windows without overwriting an existing destination. Keep complete sibling staging, same-filesystem publication, failure cleanup and preservation of unrelated user files. Preserve the installed-wheel smoke assertion. Do not introduce copy/delete fallback, weaken atomic publication, change the public layout or claim forced repair works; forced repair is separately tracked in 2358.

## Evidence

- Actual native Windows RED: untagged release rehearsal 39712462971 at b4c4c675855ce433bbb16362a47003295a0519ed, job119626255006. The installed wheel passed rendering and other smoke commands, then skill install exited2: Atomic skill installation is unsupported on this platform. scripts/installed-wheel-smoke.sh correctly expects exit zero and matching guide content.
- Read-only review found unconditional published install promises in docs/getting-started/skills.md and CLI help, and prior installed-wheel obligations in2323,2338,2339. Complete sibling staging reaches rename_absent, whose implementation rejects Windows. No ruling authorizes withdrawing Windows installation.
- Failed run remains failed:31jobs passed,one failed,10skipped. All five archives and wheels and the other four native wheel smokes passed; containers and final rehearsal completion were not reached. All eight publisher/live jobs skipped. Raw proof and exact Windows wheel are retained on Yellow release-evidence/0.9.4/rehearsal-2330/39712462971, manifest SHA256 f50d75a00416e2fe00913dae4d2c61a8a93d72673c60d22b201c2a9d5170ee9e.

## Proof

- Restate the outcome and reuse the actual native Windows failure with exact commands and output before editing. Name the minimal files and failing outside-in case.
- Use a documented native same-filesystem directory publication operation that refuses an existing destination. Research official platform documentation; do not infer platform guarantees.
- Prove fresh installation, matching rendered guide and current status; preserve modified guide and unrelated files on repeat installation; preserve a destination created before commit and remove abandoned staging on failure. Keep the actual executable boundary.
- Run affected checks through pm job on approved build machines. Identify the supported route to native Windows proof; local cross-platform tests must not be relabeled as Windows execution. Get fresh review and the declared landing gate. Native verification and release admission remain open until actually executed.
