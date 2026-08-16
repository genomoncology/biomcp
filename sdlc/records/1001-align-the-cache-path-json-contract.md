---
flow: quickfix
priority: 8
---

# Align the cache path JSON contract

`cache path` intentionally returns typed JSON when `--json` is supplied, but help, documentation, and executable specs still claim it is always plain text. Preserve runtime behavior and make every public contract require plain output without `--json` and `{kind:"cache_path",path:...}` with it in either global flag position.

Restatements are authorized in `src/cli/tests/facade/cache.rs`, `spec/surface/cli.md`, `docs/user-guide/cli-reference.md`, `src/cli/list_reference.md`, and the documentation consistency contracts.
