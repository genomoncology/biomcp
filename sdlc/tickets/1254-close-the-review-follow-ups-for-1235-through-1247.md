# Close the review follow-ups for 1235 through 1247

From sdlc/issues/2026-09-24-review-follow-ups-for-1235-through-1247.md.
Every item below is a should-fix from the independent reviews; none
blocks another ticket.

## Items

1. 1241 synonyms end to end: a `merge_mychem_hits` test with more
   than three DrugBank synonyms asserting "acetylsalicylic acid"
   reaches `ddinter_synonyms` (`src/transform/drug.rs:530`) and that
   `interactions.rs:110` passes it on. Also take synonyms from the
   chosen anchor hit only — pooled synonyms from a combination
   product can name a real interaction partner and the `(true, true)`
   skip at `interactions.rs:286` drops that row silently.
2. 1241 freshness through the real path:
   `cached_index_freshness_ignores_later_file_replacement`
   (`src/sources/ddinter/tests/construction.rs:25-50`) passes a fixed
   timestamp twice, so it passes even if `ready()` re-reads file
   times. Go through `cached_index_for_root` or `ready()`, rewrite
   the files, assert the label stays Stale.
3. 1241 error markers: the arm at `src/error.rs:483` catches every
   DDInter error, so a sync download failure leaks upstream body text
   ("bundle could not be read: ... HTTP 503: <upstream body>"). Give
   read and parse errors their own marker, match only those, and test
   both arms. Also test the covered-zero-rows wording
   (`src/render/provenance.rs:280-287`).
4. 1239 cleanup classification: `open_directory_at`
   (`src/sources/gencc/store.rs:679`) and the owner and mode check
   (`:811-817`) return `Unavailable` for wrong mode, wrong owner, and
   not-a-directory, so those generations are never pruned and cleanup
   warns on every publish (`:568`). Route them through
   `store_error_for_errno`, make a deliberate mismatch `Invalid`, and
   add a test with a 0755 generation directory. Also decide and
   record whether EACCES, ESTALE, and EAGAIN mapping to prune is
   intended.
5. 1244 licensing guard: the date check calls `print`
   (`tests/test_source_licensing_docs_contract.py:204-220`), which
   pytest hides for passing tests. Use `warnings.warn`, and fail the
   release gate when a `reviewed_on` date is past 365 days.
6. 1244 ctgov duplicates: the three `2026-09-13-raw-ctgov-total-*`
   issue files were never merged although the hygiene record says all
   items landed. Merge them and correct the record.
7. 1247 test honesty: `...joins_cleanup_and_releases_locks` claims
   more than the product does (cleanup runs detached; the refresh
   lock releases first). Rename the test or wait for cleanup before
   releasing, and record which.
8. 1236 health handshake: `tests/tls_ca_bundle_contract.rs:263`
   exercises the orphan client only. Add a test-only address override
   to `health_http_client` and a real handshake test.
9. Pending-review check: tickets 1239's line still says pending
   (1239, 1240, 1241, 1245 were corrected in their tickets' merges;
   verify every ticket with a record file has verdicts). Add a check
   that fails when a ticket with a `sdlc/records/` file still says
   "Code review: pending".
10. 1235 minors as recorded: run `rmcp_client_contract.rs:1495` with
    `--release` in a standing gate; share `panic_payload_message`
    between `src/mcp/shell.rs:1428` and `src/cli/outcome.rs:596` and
    unit-test it; route the locks at `src/entities/drug/get.rs:549,567`
    and `src/cache/clear.rs:158,168` through `recover_poison`.
11. 1236 minors: `tests/test_provider_network_policy.py:140` counts
    comment text and skips `src/sources/mod.rs`; build the real
    shared, ORCID, and CSPEC clients with a valid bundle and assert
    one parse; make `stdio_bad_fallback_starts_and_warns_once` issue
    two tool calls and assert one warning; add a loader case for a
    real file named `ca-\xff.pem`; replace the `unreachable!` at
    `src/sources/ca_bundle.rs:106`.
12. 1237 minors: with label and safety both requested the ordinary
    warning text prints twice under two Warnings headings
    (`templates/drug.md.j2:32`, `src/render/markdown/drug_regulatory.rs:348`);
    reuse `dailymed_setid_url` in `src/entities/drug/label.rs:40-48`;
    set the drug tests inventory floor to 954.

## Order

Work top to bottom; items 1-4 are one PR-sized batch (the DDInter and
GenCC follow-ups), items 5-9 a second (hygiene and guards), items
10-12 a third (minors). Each batch gets its own review and gate.

## Review

- Design review: this ticket is the design; deviations recorded here
- Item 1 done: the fold takes synonyms from the chosen anchor hit only
  (the first hit with a best name); `merge_mychem_hits` tests prove
  the >3-synonym passthrough into `ddinter_synonyms` and that a pooled
  combination-product synonym (`dipyridamole`) no longer widens the
  identity; the new `ddinter_identity_for_anchor` seam
  (`src/entities/drug/interactions.rs`) is the real call-site helper
  and `aggregate_rows` tests prove the partner row survives.
- Item 2 done: `cached_index_freshness_ignores_later_file_replacement`
  was vacuous and is replaced by two tests that drive
  `cached_index_for_root` with real bundle CSVs and pin both
  directions with `File::set_modified` (stale load stays stale after
  fresh replacement; fresh load stays fresh after the files age).
- Item 3 done: bundle read/parse errors carry
  `DDINTER_BUNDLE_READ_MARKER` and only those render as "bundle could
  be read" text; the sync HTTP failure keeps the generic API line and
  no longer embeds the upstream body excerpt (the HTML content-type
  check also drops its excerpt); tests pin both arms and the
  covered-zero-rows wording.
- Item 4 done: `validate_directory_owner_mode` and `open_directory_at`
  classify through the errno taxonomy, so a deliberate mismatch (0755
  mode, wrong owner, not-a-directory) is `Invalid` and those
  generations prune; `remove_generation_if_unleased` tolerates
  `Invalid` from its checks and still removes; the 0755-generation
  test proves the prune end to end through a real publish cycle.
  EACCES/ESTALE/EAGAIN decision: they map to `Unavailable` (retain),
  because they are environmental — permission changes, NFS stale
  handles, transient contention — and pruning a healthy generation
  over them would be destructive; the errno unit tests record it.
- Items 5-12: not started; next batch.
- Code review (batches 2-3): pending (batch 1 accepted below)
- Code review (batch 1): ACCEPT with three report-only P2s
  (unused parameter dropped; local-read line recorded; floor
  attestation confirmed) 2026-09-25
- Verification (batch 1): yellow gate at d8475987 lint/test/spec OK;
  see `sdlc/records/1254-close-the-review-follow-ups-for-1235-through-1247.md`
