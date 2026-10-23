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

Design review accepted one atomic directory lock in the canonical installation parent, shared by the shell installers and Rust mutators. Hold it through pending recovery, ownership validation, executable mutation and receipt completion. An existing entry fails closed; a losing writer never removes it. Rust guard cleanup and shell traps release only a lock this invocation created, after foreground mutators finish. Preserve clean interruption handling. An untrappable crash leaves the lock: diagnostics require stopping all mutators before manual removal. Do not infer abandonment from elapsed time or steal by PID. Reuse existing filesystem helpers and dependencies; correct the concrete claim before edits.

Reviewed decision: automatic abandoned-lock recovery is not required. Safe manual recovery after quiescence is preferable to a new lease protocol or private installer command. Preserve existing pending-receipt recovery once the lock has been safely released.

- Proof: Extend the existing outside-in installation lifecycle cases with controlled concurrent writers at the real installer and CLI entry points. The regression must fail on current main by observing the mismatched ownership or unsafe pending recovery, then pass with a consistent executable/receipt and safe loser behavior. Preserve interruption, update, uninstall and existing Windows refusal cases. Use event synchronization rather than timing assertions, and run affected tests through build jobs.

Production update uses its fixed GitHub endpoint; its loopback override exists only in Rust tests. Cover update participation through that existing Rust boundary and state this proof limit. Do not add a public provider override solely for tests.
