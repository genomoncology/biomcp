---
flow: quickfix
priority: 10
---

# Make the managed-state parent-mode proof deterministic

The Unix managed-state permission test now establishes its temporary parent at mode `0755` and verifies that precondition before running BioMCP. It then requires the cache command to leave that parent unchanged while narrowing the managed root, directory, and file. The test retains the hard-link rejection proof.

## Evidence

On `origin/main` commit `987ccbf3`, the focused test failed because `tempfile::tempdir()` created the parent at `0700` and the old assertion expected any other mode. After the test-only repair, the focused nextest command and the complete `managed_state_permissions` test binary each passed. `cargo fmt --all -- --check` and `git diff --check` also passed.

## Boundary

The implementation changed only `tests/managed_state_permissions.rs`. Production cache behavior, security policy, lifecycle scripts, and unrelated permission tests remain unchanged.
