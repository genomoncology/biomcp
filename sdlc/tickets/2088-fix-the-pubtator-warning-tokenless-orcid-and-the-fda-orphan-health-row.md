---
priority: 2
---

# 2088 — Fix the PubTator warning, tokenless ORCID and the FDA orphan health row

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 5caba37, accept

Reviews: revision 20b5ef5, accept

Reviews: revision 955d8d5, accept

Reviews: revision 0fef3ba, accept

Reviews: revision f916765, accept

Reviews: revision 273703b, accept

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

Integration review accepted 955d8d524 against main 17f4db007: the accepted product code, captures and tests are unchanged, preserving health deduplication, typed identity, warning policy, audits and build profiles. Reuse the 69-Rust/77-Python provider proof 1792247602292-74303 and header regression 1792241788131-11817. The next candidate carries accepted 2120 coverage isolation, 2158 joined-entry validation and 2164 stable metadata; their merges required no implementation conflict. Each retains its review. One combined landing gate and an actual-main full check remain required.

Combined candidate gate 1792257271484-20082 stopped at compiler lint before tests: the probe_server helper in src/cli/health/tests/http.rs returns a tuple whose nested shared counters trigger type_complexity with warnings denied. This is a test-helper declaration fix, with no intended behavior change or lint suppression. Retain its assertions and verify the affected health tests plus all-target compiler lint. Main remains qualified at 17f4db007.

The fresh reviewer corrected the gate-only Clippy type_complexity finding by naming the existing health probe fixture fields in a local ProbeServer struct. Behavior and assertions remain unchanged. Job 1792257701978-9535 passed formatting, all-target routine Clippy with warnings denied, and all 76 selected health tests on the published candidate plus that exact source patch. The reviewer accepts pushed revision 2b79d92a4399db36edd9513b65d4394281c005d4. The shared candidate gate remains required.
