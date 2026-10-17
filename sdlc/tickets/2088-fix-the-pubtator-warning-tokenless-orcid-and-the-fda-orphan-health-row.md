---
priority: 2
---

# 2088 — Fix the PubTator warning, tokenless ORCID and the FDA orphan health row

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 5caba37, accept

Reviews: revision 20b5ef5, accept

Reviews: revision 955d8d5, accept

## Outcome

Coordinator implementation brief: preserve the adopted fixes and the recorded decisions. Successful article fallback suppresses warnings; total fallback failure still warns. Public ORCID requests omit Authorization unless a token is configured. FDA redirects remain blocked and are named in health output. Current-main job `1792241031845-74738` on `078ce756d` overlaid only existing regression tests and exited 100: both the FDA redirect-row case and ordinary successful-fallback case failed; the latter showed an unwanted WARN. Add the missing variant-fallback and total-failure cases to the existing behavior table, preserving the declared claims.

Article searches warn only when every metadata fallback fails, public author reads work without an ORCID token, and FDA health reports scripted-client redirects without following them.

Depends on: 2139
Depends on: 2159

- Starts from: the existing implementation branch and the approved product decisions recorded for this ticket.
- Keeps: warnings when all metadata sources fail, configured-token author reads, existing source facts and the refusal to bypass the FDA block.
- Changes: claims `src/entities/article/enrichment.rs` `src/entities/article/enrichment/tests.rs` `src/sources/orcid.rs` `src/sources/orcid/tests.rs` `src/cli/health/mod.rs` `src/cli/health/http.rs` `src/cli/health/runner.rs` `src/cli/health/tests/**` `src/sources/fda_orphan.rs` `src/cli/list/literature.rs` `docs/reference/data-sources.md` `tests/test_author_surface_docs.py` `tests/test_provider_network_policy.py` `tests/health_cli_structure.rs` `tests/test_source_package_boundary.py` `testdata/sources/orcid/**` `testdata/sources/capture-receipts.json` `tools/zero-coupling-historical.json` `CHANGELOG.md`.
- Proof: recorded replies cover ordinary and variant searches with working fallback and empty warning output, empty PubTator replies covered by Europe PMC, warnings when both fail, tokenless author/papers requests with no Authorization header, configured-token requests, and a 302 FDA response with the named blocked message. Each required correction is demonstrated against current main before code changes. Run focused checks and the routine merged-candidate gate through build jobs.
- Defers: bypassing the FDA block.

PM decision: 2026-10-14, retained 2026-10-17. Successful fallback logs at debug; only total failure warns. Send Bearer only when a token is set and otherwise use the public author API. Name FDA's scripted-client redirect without trying to evade it. Reuse the latest pushed implementation, not a stale local tip; serialize the capture/inventory edits after 2139.
