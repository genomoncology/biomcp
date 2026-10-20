---
flow: quickfix
priority: 10
---

# Reject batch source outside trials

`batch --source` is a trial-only option. Trial batches still default to `ctgov` and accept the supported explicit sources; every other entity rejects an explicitly supplied source before constructing provider work. CLI and raw MCP must agree.

Red-green coverage belongs in `src/cli/system/tests.rs` and the existing batch CLI/MCP process contracts; source defaults and applicability assertions may be restated.
