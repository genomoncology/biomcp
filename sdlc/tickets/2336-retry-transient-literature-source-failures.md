# 2336 — Recover from transient literature source failures

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 9fa4ff2c8b607d11381940248ea6874b079e2c1b, accept

Reviews: revision 910f9b9ff36a3dc85baeccea320c5616c9962265, accept

Reviews: revision 474a07136c26b55ea0ec80306a593d52ceb1c0bd, accept

## Outcome

A literature search survives a transient Europe PMC, Semantic Scholar or PubTator3 failure where it can. After an HTTP 503, 502, 429 or timeout, BioMCP retries once with backoff, honoring Retry-After within the command's deadline. If the retry also fails, it returns every result the other routes found and names the failed route, its error class and that a retry ran. Dated, newer-paper searches get the same handling. No retry follows a 4xx other than 429, and provider pacing rules stay in force.

Restated outcome: reserve time for one real retry of an eligible literature request, preserve other sources' results, and report its actual route, failure class and admitted retry without changing provider pacing or unrelated retry behavior.

- Changes: `src/error.rs` (preserve classified provider warnings), `src/sources/mod.rs`, `src/sources/literature_retry.rs`, `src/sources/semantic_scholar.rs`, `src/entities/article/search.rs` (source-leg failure handling only), `src/entities/article/backends.rs` (partial-page and Semantic Scholar error reporting only), `src/entities/article/variant_search.rs` (error event and source-status reporting only), `tests/test_literature_transient_retries.py`, `tests/feature-links.json`, `tools/check-source-registry.py` (classify shared retry helper), `docs/user-guide/variant.md`, `CHANGELOG.md`, `src/entities/article/backends/provider_unit.rs`, `src/entities/article/enrichment.rs`, `src/entities/article/enrichment/visible_metadata.rs`, `src/entities/article/search/deadline.rs`, `src/entities/article/search/tests/deadline.rs`, `src/entities/article/detail.rs`, `tests/test_source_licensing_docs_contract.py`.

- Proof: outside-in replay tests with a Europe PMC 503 then success return complete results; 503 twice returns partial results naming Europe PMC with retry_attempted; a timeout on a dated search behaves the same; a 400 is not retried; the total time stays within the command deadline.

## Evidence

- On 2026-10-21, research ran five fresh agents with BioMCP 0.9.4 (commit cdb9f4a1b) on literature searches for five hereditary cancer variants, with no answer key. Exact commands and output are in research's protected store under runs/0010/five-v1.
- Item 3: Europe PMC returned HTTP 503 or timed out in four of five cases, including every dated search. Semantic Scholar and PubTator3 timed out once each. Each search came back incomplete.
- src/sources/europepmc.rs already attaches retry source context (lines 87-173). Start by recording what it retries today and why these four cases still ended incomplete.
- Ian asked on 2026-10-21 to clear every known issue in 0.9.4.

- PM decision m3080 preserves unauthenticated Semantic Scholar shared-pool 429 fast refusal with the `S2_API_KEY` hint. Keyed requests and other eligible transient failures use the single retry policy. This exception overrides the general 429 wording above; a refusal after an earlier admitted retry still reports that retry honestly.
- Baseline M5 j2110 showed that `SourceContext::retry` was advice: the old shared transport already allowed three retries. Its 503/502/429-then-success cases passed, while the failure disclosure lacked the required class/retry metadata. The new literature policy replaces that retry layer for eligible article requests; it does not stack another retry on it. It reserves attempt time within both source-leg and invocation deadlines, honors integer and HTTP-date Retry-After floors, and returns partial results when the floor cannot fit. Historical study outputs have no per-attempt trace, so this does not assert their original send counts.
- Fresh review reproduced admitted-retry cancellation loss, overflowing Retry-After, shared-pool refusal evidence loss and batch-enrichment projection loss (j2231 and j2248). Repairs retain the observer at the actual provider-leg boundary, preserve typed sanitized failures through projections, and retain the actual HTTP status including 422 and the existing PubTator 400/404 recovery behavior. Existing metadata-schema assertions are unchanged.
- Full-gate failures remain evidence: j2185 exposed helper registration, j2200 invocation-deadline selection, j2252 the large-future stack overflow, and j2279 HTTP compatibility and harness integration. The stack repair boxes the backend future before nested scopes and retains normal stack limits. The helper inventory test now consumes the existing authoritative registry classification instead of copying it. Focused j2362 passed all 84 affected Python cases and 13 Rust cases with Clippy, Rustfmt and Ruff.
- Fresh review accepts published runtime `2026f74c5c4f62f877c5100b7853c803ed2ed540`. Direct M5 `make landing-gate` job j2374 passed exit 0 at that exact revision, including routine Rust/Python, strict documentation and specifications. The subsequent ticket-only record/claim update changes no runtime source. Build copies touched M5 and Gator; final raw job receipts belong in the landing evidence archive before cleanup.
- Land this ticket alone because no other ticket is accepted. The proposed dependent batch partner needs an authored-fixture correction and a fresh full routine gate, so it cannot join this accepted landing.
- The first integrated landing candidate `10300c4d31bc4e2f84d145adc04932fcae2160f8` failed M5 job `1792604161008-59444`: the invocation-deadline fixture held only Europe PMC, whose bounded attempts could finish before the invocation deadline. A preserved PubTator HTTP 400 could therefore win while time remained. The reviewed test-only repair also holds PubMed esearch and asserts that request occurred, preserving the strict deadline error and wall-bound assertions. Focused job j2548 passed all 12 deadline/cancellation cases, Rustfmt and all-target Clippy. Fresh review accepted repaired revision `2cf8792569c4f329d700cc74c9bdc396e57c2bff`; the actual landing gate must pass before publication.
