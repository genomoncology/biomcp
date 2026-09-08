---
flow: quickfix
priority: 10
---

# Make the managed-state parent-mode proof deterministic

## Goal

The Unix managed-state permission contract passes when the operating system creates temporary directories with a private mode. The contract still proves that BioMCP narrows only the cache paths it owns and rejects hard-linked managed files.

## Current behavior

On current main, `cache_open_narrows_only_managed_paths_and_rejects_hardlinks` fails before it reaches the hard-link proof because `tempfile::tempdir()` creates its parent with mode `0700` on this machine. The final assertion assumes the parent began with another mode. Production cache behavior is not implicated.

## Required behavior

Make the test establish and verify its own parent-mode precondition before invoking the real BioMCP cache command. The command must narrow the managed root, directory, and file while leaving the parent unchanged. Preserve the hard-link rejection proof.

## Observable success

The exact test fails on current main and passes after the repair on a system where temporary directories begin with mode `0700`. The standard BioMCP gates pass without a production-code change.

## Boundaries

Do not change cache behavior, security policy, lifecycle scripts, unrelated permission tests, or quality ceilings.
