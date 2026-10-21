---
flow: quickfix
priority: 9
---

# Make global CLI modifiers truthful

The adversarial dev.3 CLI review found two related global-option contract defects. `--no-cache` is accepted with the `cache` family even though `cache stats` and `cache clean` can create managed state and `cache clear --yes` deletes it, contradicting the documented promise that the flag neither reads nor writes managed request state. Separately, JSON help and version requests exit successfully but are serialized as `invalid_argument` error objects because every Clap result is sent through the parse-error renderer.

Reject `--no-cache` with every cache subcommand before cache configuration, path resolution, prompting, or filesystem access, in either flag position. Use the normal exit-2 human or JSON invalid-argument contract, and stop advertising `--no-cache` on cache-family help while retaining `--json`. Keep `--no-cache` behavior for provider commands unchanged.

Treat Clap `DisplayHelp` and `DisplayVersion` as successful output under JSON mode. Root, `help`, short-form, and nested help must emit one object with exactly `kind: "help"` and the sanitized rendered help in `content`, on stdout with exit 0. JSON `--version` must emit the exact same `version`, `git_revision`, and `build_timestamp` object as `biomcp --json version`. Human help/version and genuine JSON parse errors remain unchanged.

Authorize implementation in `src/cli/shared.rs`, `src/cli/outcome.rs`, `src/cli/system/dispatch.rs`, and the owning CLI facade/help code. Authorize contract updates in `docs/user-guide/cli-reference.md`, `src/cli/tests/facade/help.rs`, `src/cli/tests/facade/cache.rs`, `tests/test_no_cache_managed_state.py`, and a new focused process test if needed. Acceptance must cover every cache subcommand in both flag positions with no filesystem mutation, human/JSON stream and exit parity, root and nested JSON help in long/short forms, exact JSON version parity, and a genuine malformed JSON command that remains an exit-2 error.

Focused tests, `make lint`, `make test`, `make spec`, and `git diff --check` must pass. No version change, queue action, tag, staging, signing, promotion, or publication belongs in this ticket.
