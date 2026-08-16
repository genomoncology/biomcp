---
flow: build
priority: 10
deps: ["1018"]
---

# Authorize raw MCP commands from the parsed command

The blind dev.4 review found that raw MCP authorization reads fixed positions in the untyped argument list while execution parses the same list with Clap, where a global flag may sit anywhere. Inserting `--json` or `--no-cache` before the subcommand shifts every position and the guards inspect the wrong words. `variant --json articles --input Cargo.toml` is admitted and opens a server-local file, and `get --json article <id> asset` is admitted and performs a CLI-only binary download. The same defect silently disables the full-text redaction guard, which is supposed to fail closed when article metadata is missing or unparseable, and it makes the `study` and `skill` guards reject commands that should be allowed. Only `--json`/`-j` and `--no-cache` are global today, so every affected command is reachable through both transports.

Decide what a raw MCP command may do from the parsed command structure rather than from token positions, and execute that same parsed value so authorization and execution cannot disagree. The allowlist must be exhaustive over the command enum with no catch-all arm, so that adding a subcommand later fails to compile until someone decides whether MCP may run it; this default-deny property is the durable half of the fix and is part of done. Rejection message selection and the full-text redaction guard must come from the same parsed value, not from a second inspection of the raw tokens. A command that does not parse is not an admitted command: reject it before any execution, file access, stdin read, or provider contact, and keep today's rejection wording so no local path is disclosed. Treat `--input -` exactly like `--input <path>` — a stdin read is the same refusal.

Done means every currently blocked command stays blocked and every currently allowed command stays allowed regardless of where a global flag appears, through real stdio and real HTTP MCP sessions, and that the session remains usable after a rejection. The refusal must happen before the file, stdin, or provider is touched. Existing command-length limits, shell-syntax handling, typed-tool behavior, error-message shape, and safe read-only routes are unchanged.

Authorize implementation in `src/mcp/shell.rs`, `src/mcp/shell/` and the CLI execution entry point that raw MCP calls, which today accepts an argument vector and re-parses it; giving it a parsed-command entry point is expected and in scope. Authorize restating tests in `src/mcp/shell.rs`, specifically `binary_downloads_are_rejected_but_manifests_remain_allowed`, `mcp_allowlist_blocks_mutating_commands`, `raw_local_input_rejection_covers_every_spelling_and_variant_route`, `raw_mcp_local_input_inventory_matches_the_cli_surface`, `mcp_full_text_path_redaction_is_field_driven_for_text_and_json`, `cache_family_rejection_message_mentions_local_path_disclosure`, and `generic_mcp_rejection_message_stays_read_only_for_mutating_commands`; and in `tests/rmcp_client_contract.rs` and `tests/test_mcp_http_surface.py`.

Focused tests, `make lint`, `make test`, `make spec`, and `git diff --check` must pass. No version change, queue action, tag, staging, signing, promotion, or publication belongs in this ticket.
