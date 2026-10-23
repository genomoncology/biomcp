# 2371 — Serialize mutations of a standalone installation

Status: OPEN.

Milestone: 0.9.4

Priority: 1

- Changes: `install.sh`, `docs/install.sh`, `src/cli/install.rs`, `src/cli/update.rs`, `src/cli/system/dispatch.rs`, `tests/test_public_installer_checksum.py`, `tests/golden/spec/test_install_lifecycle.py`, `CHANGELOG.md`.

## Outcome

Concurrent installation, update, uninstall and pending recovery cannot publish an executable with another writer's ownership receipt or recover a live writer's transaction.

## Starts from

Fresh release review confirmed the defect by tracing main `01eccd84ff53cc97c684906d95c730610d51feae`. The shell installers publish pending receipt, executable and final receipt separately. Rust update revalidates before its mutations, pending recovery can act during another writer's transaction, and uninstall reads ownership before unlinking. Interleaved writers can leave one executable with a different checksum receipt, after which ownership validation refuses maintenance. The structural cause is shared state without common exclusion.

## Keeps

Preserve ownership checks, checksum verification, package-managed refusals, supported installation paths, repair messages and the existing Windows forced-repair ruling. Do not remove another live writer's lock or weaken verification. Keep any new coordination local to the installation being changed.

## Design

One fresh design review chooses the smallest shared exclusion mechanism before implementation. Every mutating entry point must hold it through recovery, validation, executable mutation and final receipt publication. Define interruption and abandoned-owner behavior without treating elapsed time alone as proof that an owner died. Correct the concrete file claim when the mechanism is chosen.

- Proof: Extend the existing outside-in installation lifecycle cases with controlled concurrent writers at the real installer and CLI entry points. The regression must fail on current main by observing the mismatched ownership or unsafe pending recovery, then pass with a consistent executable/receipt and safe loser behavior. Preserve interruption, update, uninstall and existing Windows refusal cases. Use event synchronization rather than timing assertions, and run affected tests through build jobs.
