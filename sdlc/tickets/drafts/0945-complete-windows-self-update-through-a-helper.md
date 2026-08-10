---
flow: build
priority: 6
deps: ["0917"]
---
# Complete Windows self-update through a verified helper

Held as a draft until the scheduled-update user contract is deliberately
approved. Ticket 0917 makes Windows self-update fail closed in the meantime;
the verified standalone installer remains the supported upgrade path.

## Proposed transaction

The running process downloads, verifies, and smokes a unique staged new
executable in the destination directory. It writes a pending receipt containing
the old and new SHA-256 values, target path identity, random transaction nonce,
and its process identity. It then launches the staged new executable through a
hidden helper mode with an inherited handle to the parent and exits with a
truthful `scheduled`, not `completed`, result.

The helper waits for the exact parent handle to signal exit, reopens the target
without following a link, and revalidates the target identity, old checksum,
pending receipt, nonce, staged checksum, and staged version. It uses the native
Windows replace primitive to retain a unique backup until the new target is
visible and validated, then finalizes the receipt. Any mismatch or replacement
failure preserves/restores the old binary and writes a typed failed transaction
state. The next normal invocation reconciles and reports pending, completed, or
failed state before starting another update.

## Promotion requirements

- Approve that the initiating CLI reports `scheduled` because it cannot observe
  work performed only after its own exit.
- Prove every transaction boundary on Windows CI with injected open, lock,
  replace, receipt, helper-launch, parent-exit, validation, rollback, and reboot
  interruption failures.
- Prove one helper owns one nonce, never follows a user-controlled path, and
  cleans only its own staging/backup files.
- Keep package-managed and unreceipted installations fail closed under 0916.
- Restate Windows update documentation only when this ticket is promoted.

The src line ceiling may rise by at most 320 lines.
