# Test, doc, and record hygiene

Filed 2026-09-23 from an independent review of `v0.9.0..f2549676`.

## Tests

- `tests/test_source_licensing_docs_contract.py:204` fails once a `reviewed_on` date passes 365 days. The oldest is 2026-03-20, so `make test` goes red on every branch around 2027-03-20 with no code change. Move it to a scheduled check or a warning.
- `src/cache/migration.rs:862-892` (ticket 1228): the `untouched` and marker checks cannot fail because the operation never touches disk. Set a flag after the operation's yield and assert it after `io.await`.
- `tests/test_article_spec_fixture_lifecycle.py:813` asserts a substring of the line above it.
- `src/sources/cellosaurus/tests.rs:19` repeats what the exact URL check before it proves.
- `tests/test_documentation_consistency_audit_contract.py`: the hand-copied catalog pattern misses the blog's count form ("21,701 UTF-8 bytes and 5,599 `cl100k_base` tokens").

## clingen-cspec spec flake

Seen once under load during a gate. Likely causes, unconfirmed:

- `setup-clingen-cspec-spec-fixture.sh` waits 5 seconds for its server (`seq 1 50` at 0.1s).
- The driver runs about 25 CLI calls and two `uv run` starts inside a 180-second block limit.
- The exact request-log check breaks if the client retry fires.

Allow about 30 seconds for readiness and print `server.log` and the report JSON on failure.

## CI

"Record exact tool versions" does not print `bwrap`, `apparmor_parser`, or `rg` versions. The apt install omits `-y` and relies on the runner's assume-yes setting. Add both.

## Architecture doc

- `architecture/technical/overview.md:10` says the advertised MCP tool is `biomcp`. The server advertises seven tools.
- `:291` says CI installs exact tool versions. Ticket 1220 unpinned them.
- The 1224 record says 1222 owns the architecture facts. 1227 took them over.

## Records

- The review residuals from 1219, 1221, 1222, 1224, 1225, 1226, and 1229 live only in record prose. File each open one here or gate it.
- `sdlc/issues/` holds three near-duplicate 2026-09-13 files about the raw-ctgov-total abort on the gate host. Merge them.
- The 1222 record and ticket file names differ.
