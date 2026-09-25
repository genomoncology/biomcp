# Review follow-ups for tickets 1235 through 1247

Filed 2026-09-24 from reviews of the tickets merged 2026-09-23 and 2026-09-24. Tickets 1235, 1236, and 1237 were accepted. Tickets 1239, 1241, 1244, and 1247 were accepted with the should-fix items below. Release gates, wheels, MCP schemas, and test waits have their own issue files dated 2026-09-24.

## Should-fix

- 1241: nothing tests the synonym path end to end. `src/sources/ddinter/tests/parsing.rs:106-156` exercises only `with_aliases`. Add a `merge_mychem_hits` test with more than three DrugBank synonyms asserting "acetylsalicylic acid" reaches `ddinter_synonyms` (`src/transform/drug.rs:530`) and `interactions.rs:110` passes it on.
- 1241: `cached_index_freshness_ignores_later_file_replacement` (`src/sources/ddinter/tests/construction.rs:25-50`) passes a fixed timestamp twice. It passes even if `ready()` re-reads file times. Go through `cached_index_for_root` or `ready()`, rewrite the files, and assert the label stays Stale.
- 1241: the new arm at `src/error.rs:483` catches every DDInter error. A sync download failure now reads "bundle could not be read: ... HTTP 503: <upstream body>". Give read and parse errors their own marker, match only those, and test both arms.
- 1241: take synonyms from the chosen anchor hit only. Pooled synonyms from a combination product can name a real interaction partner, and the `(true, true)` skip at `interactions.rs:286` then drops that row silently.
- 1241: the real-bundle checks live only in prose. File an open issue with the exact command, the file sizes to compare against `DDINTER_MAX_BODY_BYTES`, and the aspirin combination-product check, so Ian can run it on the Mac.
- 1239: `open_directory_at` (`src/sources/gencc/store.rs:679`) and the owner and mode check (`:811-817`) return `Unavailable` for wrong mode, wrong owner, and not-a-directory. Those generations are now never pruned, and cleanup warns on every publish (`:568`). Route them through `store_error_for_errno`, make a deliberate mismatch `Invalid`, and add a test with a 0755 generation directory.
- 1247: `...joins_cleanup_and_releases_locks` claims more than the product does. Cleanup runs detached and the refresh lock releases first. Rename the test or wait for cleanup before releasing, and record which.
- 1244: the licensing date guard now calls `print` (`tests/test_source_licensing_docs_contract.py:204-220`). Pytest hides the output of passing tests, so the guard is gone. Use `warnings.warn`, and fail the release gate when a `reviewed_on` date is past 365 days.
- 1244: the three `2026-09-13-raw-ctgov-total-*` issue files were not merged, yet the hygiene issue says all items landed. Merge them and correct the record.
- 1244: three residual dispositions hang on "the 0.9.1 release run" with nothing that fails if the release skips them: 1221's leftover check, the 1222 upload, and the 1225 args. List them in the release-prep ticket as checklist items.
- 1236: the health client has no real handshake test. `tests/tls_ca_bundle_contract.rs:263` exercises the orphan client. Add a test-only address override to `health_http_client` and a handshake test.

## Minor

- 1235: run `tests/rmcp_client_contract.rs:1495` with `--release` in a standing gate, the way `Makefile:109` does for other contract tests. Share `panic_payload_message` between `src/mcp/shell.rs:1428` and `src/cli/outcome.rs:596` and unit-test it. Route the locks at `src/entities/drug/get.rs:549,567` and `src/cache/clear.rs:158,168` through `recover_poison`.
- 1236: `tests/test_provider_network_policy.py:140` counts text, including comments, and skips `src/sources/mod.rs` (two builds at 898 and 981). The parse-once test uses `configure` directly with a broken file; build the real shared, ORCID, and CSPEC clients with a valid bundle and assert one parse. `stdio_bad_fallback_starts_and_warns_once` makes no tool calls and sleeps 300 ms; make two calls and assert one warning. Add a loader case for a real file named `ca-\xff.pem`. Replace `unreachable!` at `src/sources/ca_bundle.rs:106`.
- 1237: with label and safety both requested, the ordinary warning text prints twice under two "Warnings" headings (`templates/drug.md.j2:32`, `src/render/markdown/drug_regulatory.rs:348`). Reuse `dailymed_setid_url` in `src/entities/drug/label.rs:40-48`. Set the `src/render/markdown/drug/tests.rs` inventory floor to 954.
- 1241: test the "covered, zero rows" wording (`src/render/provenance.rs:280-287`).
- 1239: EACCES, ESTALE, and EAGAIN map to prune. Decide and record whether that is intended.
- Tickets 1239, 1240, 1241, and 1245 still say "Code review: pending" after acceptance. Add a check that fails when a ticket with a record file still says pending.
- cf22293c replaced the other repository's name with "BD". That is invented shorthand. Write "another repository's gate", or fix the coupling gate if naming that public repository is allowed.
