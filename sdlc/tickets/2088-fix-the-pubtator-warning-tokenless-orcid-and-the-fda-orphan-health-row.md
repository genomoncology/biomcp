# 2088 — Fix the PubTator warning, tokenless ORCID and the FDA orphan health row

Status: OPEN.

Milestone: 0.9.3

## Outcome

Article search writes no warning when its fallback works, `get author orcid:0000-0002-4299-3018` and `author papers` work without `ORCID_ACCESS_TOKEN`, and the FDA orphan health row names the redirect it got.

## Evidence

Split from 2068 on 2026-10-14; found by the post-release install and product test of v0.9.2 and rechecked against current main.

- Every article search writes "WARN biomcp_cli::error: External provider operation failed provider=\"PubTator 3\" operation=\"visible article metadata fallback\"" to stderr. `warn_external_failure` is called at `src/entities/article/enrichment.rs:405-409` and 336-340; the Europe PMC fallback in `src/entities/article/detail.rs:334-420` runs only on HTTP 400 or 404. That an empty 200 reply from PubTator skips the fallback is inferred, not yet run; the developer confirms the trigger first.
- The ORCID client already uses `pub.orcid.org` (`src/sources/orcid.rs:15`) but refuses without a token (`src/sources/orcid.rs:59-91` and 243-246) and always sends a Bearer header (line 294). `pub.orcid.org/v3.0/0000-0002-4299-3018/record` returned 200 with no token on 2026-10-14. GitHub #289's user relies on this path.
- `biomcp health` shows FDA Orphan Drug Designations as temporarily unavailable. Redirects are off (`src/sources/fda_orphan.rs:516` and 673) and any 3xx becomes "unexpected HTTP status" (lines 448 and 684). The live page answers 302 to an abuse-detection page for scripted clients.
- PM decision 2026-10-14: a fallback that returns rows logs at debug level, and only a fallback that also fails warns. ORCID calls send the Bearer header only when a token is set, and the public API serves every call otherwise. The FDA health row reports "blocked: the FDA site redirects scripted clients" for a 3xx, and BioMCP does not try to get past the block.

- Starts from: the lines above; the health catalog row (`src/cli/health/catalog.rs:174-177`) and its error mapping (`src/cli/health/runner.rs:153-178`).
- Keeps: the warning when every article source fails; ORCID's member API when a token is set.
- Changes: claims `src/entities/article/enrichment.rs` `src/sources/orcid.rs` `src/sources/orcid/tests.rs` `src/cli/health/http.rs` `src/cli/health/catalog.rs` `src/cli/health/runner.rs` `src/cli/health/tests/**` `src/sources/fda_orphan.rs` `src/cli/list/literature.rs` `docs/reference/data-sources.md` `tests/test_author_surface_docs.py` `tests/test_provider_network_policy.py` `testdata/sources/orcid/**` `CHANGELOG.md`.
  - One Fixes changelog bullet naming all three.
- Proof: a recorded-reply test of an article search whose PubTator reply is the trigger, with empty stderr; recorded tokenless ORCID replies for `get author` and `author papers`, with no Authorization header sent; a health test with a recorded 302 showing the redirect message. Each fails on current main. Routine suite.
- Defers: getting past the FDA block itself.
- Order: after 2065, whose `src/cli/health/mod.rs` and `src/cli/health/local.rs` sit beside this ticket's health files.
