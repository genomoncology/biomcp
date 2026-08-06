# Design — Ticket 664: Captured MyVariant and CancerHotspots Contracts

## Investigation Summary

The live assertions are `spec/entity/variant-myvariant-live.md` and `spec/entity/variant-hotspots.md`; `scripts/run-specs.sh::SPEC_LIVE_PATHS` places both only in `make verify`. The product routes are already split at useful boundaries: `MyVariantClient::{query_plan,search_plan,get_plan}` in `src/sources/myvariant.rs` construct source-local requests, then `query_with_fields`, `search`, and `get` execute and deserialize them. `resolve_base_with_hit` in `src/entities/variant/get.rs` chooses the get/query request from user input, and `add_cancerhotspots` calls `CancerHotspotsClient::by_gene` then `recurrence_for_change`. `src/entities/variant/structure.rs::cancerhotspots` follows that same join path.

The prior Tier 2/Tier 3 tests exist but their fixtures are not admission-eligible: `testdata/sources/capture-receipts.json` marks `myvariant/{get_braf_v600e,search_braf,search_brca1_contradictory_protein}.json` and `cancerhotspots/by_gene_braf.json` `pending_verification`. They cannot replace a live assertion. `architecture/technical/live-spec-conversion-target.md` specifically requires real, dated MyVariant consequence/filter and CancerHotspots BRAF/MYD88/empty captures, receipt metadata, production decoder/orchestration coverage, and fixture-backed CLI proof before live removal.

Provider shape was observed, not assumed, on 2026-08-05. The exact MyVariant BRAF V600E get request recorded `_id` `chr7:g.140453136A>T`; array-valued `dbnsfp.genename` and `dbnsfp.hgvsp`; and numeric BayesDel `add_af.score`/`no_af.score` values. The BRAF consequence/filter query responses recorded non-empty `hits`, array-valued gene identity, scalar CADD consequence `NON_SYNONYMOUS`, and numeric REVEL scores. CancerHotspots BRAF supplied a `V600` row with `tumorCount: 897`, `variantAminoAcid.E: 833`, and transcript `ENST00000288602`; MYD88 supplied `L265`, `tumorCount: 37`, `variantAminoAcid.P: 37`, and transcript `ENST00000396334`. The implementation will retain raw bytes and receipt hashes; it will not pin live totals or row positions.

### Minimal Slice

1. Capture the identified raw provider responses (including an observed empty CancerHotspots response) and register each as `real_and_receipted` with request, UTC capture time, SHA-256, and byte-faithfulness/minimization statement.
2. Extend source-local Tier 2 and production-byte Tier 3 tests to cover the MyVariant get/search/filter and CancerHotspots BRAF/MYD88/empty recurrence paths.
3. Add one runner-owned local source fixture that serves exactly those captures, verifies the expected MyVariant/CancerHotspots request routes, and enables fixture-backed CLI JSON assertions in `spec/entity/variant.md`.
4. Move only these converted assertions out of `SPEC_LIVE_PATHS`; retain the two old pages as explanatory documents or replace them with links to the deterministic contract rather than silently losing their user guidance.

Deferred: filter/threshold/output behavior changes, unrelated variant sources, a generic fixture framework, public availability smoke tests, and broad renderer/structure refactoring.

## Architecture Decisions

- **Captures and admission:** add date-suffixed raw files below `testdata/sources/myvariant/` for BRAF get, consequence query, and field-filter query, and below `testdata/sources/cancerhotspots/` for BRAF, MYD88, and observed-empty by-gene responses. Update `testdata/sources/capture-receipts.json` with their checksums. Existing unreceipted files remain historical parser inputs; do not relabel or reshape them.
- **Tier 2:** extend `src/sources/myvariant/tests/construction.rs` using the existing `params()` helper and `MyVariantClient::search_plan`; assert the CLI-used consequence and field-presence expressions and query parameters. Extend `src/sources/cancerhotspots/tests/construction.rs` with BRAF/MYD88 plans. These tests assert the actual `RequestPlan` consumed by `search`, `get`, and `by_gene`, not reconstructed URLs.
- **Tier 3 and orchestration:** extend `src/sources/myvariant/tests/parsing.rs` and `src/sources/cancerhotspots/tests/parsing.rs` to feed production decoders the new receipt-backed bytes and assert non-empty decoded landmarks, recurrence, and checked absence. Add focused tests in `src/entities/variant/get/tests.rs` and the existing structure test module only where current source-level proof does not demonstrate that the entity maps captured recurrence to its result/outcome. No production behavior change is planned.
- **Fixture CLI contract:** add a source-specific setup/cleanup pair under `spec/fixtures/`, following `setup-variant-identity-spec-fixture.sh`: ephemeral server/process group, dynamic port, stale-owner cleanup, no worktree scratch, and exported `BIOMCP_MYVARIANT_BASE` and `BIOMCP_CANCERHOTSPOTS_BASE`. It will serve raw capture bytes (not hand-authored JSON), reject unexpected paths/query values, and preserve the existing HTTP/client/decoder route. Wire it into `scripts/run-specs.sh` for `spec/entity/variant.md` alongside its routine fixture lifecycle.
- **Shipped docs/spec:** add the literal fixture-backed CLI assertions under `spec/entity/variant.md::Captured MyVariant Filters and Consequences` and `::Captured CancerHotspots Recurrence`. Update the introductory deterministic-contract prose and `scripts/run-specs.sh::SPEC_ROUTINE_PATHS` / `SPEC_LIVE_PATHS` together. Update `spec/entity/variant-myvariant-live.md` and `variant-hotspots.md` so users are directed to the deterministic coverage and no longer encounter stale live-canary instructions. The only new interface is test harness configuration already supported by `env_base`; no CLI flag or user configuration is introduced.

## Quality Analysis

- **Reuse** — reuse `RequestPlan`, source production decoders, `recurrence_for_change`, `env_base`, and the variant-identity runner fixture lifecycle; new fixture scripts are needed because they serve two source-specific capture sets and validate their request semantics.
- **Duplication** — searched `src/sources/{myvariant,cancerhotspots}`, `src/entities/variant`, `spec/fixtures`, and receipt registry; retain source-local tests rather than duplicate request construction in a generic test helper.
- **Simplicity** — one local fixture server and raw captures; no runtime API, no parser rewrite, no generic capture abstraction.
- **Separation of concerns** — source tests own plans/decoding, entity tests own outcome/orchestration, fixture scripts own HTTP lifecycle, and mustmatch owns only public CLI documentation/assertions.
- **Performance** — runtime hot paths remain unchanged; fixture serves local raw bytes once per CLI command and avoids public network/cache calls. No text/binary conversion of captures beyond existing HTTP delivery.
- **Data fidelity** — the downstream consumer is `spec/entity/variant.md`; assertions verify public JSON fields fed by production decoding. Receipt hashes and raw response bytes prevent silent fixture reshaping; paired non-empty and empty captures distinguish actual no-data from decoding failure.
- **Security** — captures contain public responses; receipts omit secrets/signed URLs. Fixture binds loopback on a dynamic port, validates only fixed request shapes, and uses runner-owned cleanup; no user input goes to shell/path construction.
- **Scope discipline** — only MyVariant and CancerHotspots assertions/captures/fixture wiring change; all other sources and live pages are deferred.

## Acceptance Criteria

1. Every converted MyVariant/CancerHotspots assertion has a consumed Tier 2 plan, receipt-backed raw Tier 3 decoder/orchestration proof, and fixture-backed CLI proof where it presents output.
2. Tests prove MyVariant filter normalization and decoded consequence/filter landmarks, and CancerHotspots BRAF/MYD88 recurrence plus captured empty/missing-field handling.
3. The fixture checks CLI outbound source routes and serves only recorded bytes; routine proof never calls either public provider.
4. Only the converted live pages leave `SPEC_LIVE_PATHS`; their deterministic replacement executes through `make spec`.
5. `make lint`, `make test`, and `make spec` pass after implementation.

## Proof Matrix

| Location | Behavior assertion | Class | Lane | Red command | Expected observation | Docs/help/examples and final green gate |
|---|---|---|---|---|---|---|
| `spec/entity/variant.md` | A user’s BRAF consequence search and REVEL-present filter reach the captured MyVariant query routes and return at least one decoded BRAF result with the requested consequence/filter data rather than a successful empty response. | semantic | check | `make spec` | green improved test | Updates the same page and retires the equivalent live MyVariant prose; `make spec` green. |
| `spec/entity/variant.md` | A user’s BRAF V600E and MYD88 L265P card exposes source-labelled recurrence and matched transcript from captured rows, while a captured no-match renders checked null recurrence rather than invented counts. | semantic | check | `make spec` | green improved test | Updates the same page and retires equivalent live Hotspots prose; `make spec` green. |
| `src/sources/myvariant/tests/construction.rs` and `src/sources/myvariant/tests/parsing.rs` | The consumed get/search/filter plans request the observed fields and production decoding preserves observed array/scalar consequence/filter shape from receipted bytes. | semantic | unit | `make test` | green improved test | No user docs beyond the CLI spec; `make test` green. |
| `src/sources/cancerhotspots/tests/construction.rs`, `src/sources/cancerhotspots/tests/parsing.rs`, `src/entities/variant/get/tests.rs`, and `src/entities/variant/structure.rs` tests | The by-gene plan is consumed and production decoding/orchestration returns BRAF/MYD88 recurrence or checked absence for the recorded response. | semantic | unit | `make test` | green improved test | No user docs beyond the CLI spec; `make test` green. |

The check entries are improved-green eligible: the ticket explicitly replaces weak/live proof; investigation identified the current code/spec gap; direct live probes demonstrate current behavior; each assertion would fail on a real regression (wrong route, dropped decoded fields, or fabricated recurrence); making them red would require false runtime behavior or out-of-scope degradation. Existing live pages are verify-only and cannot provide this deterministic proof. Unit entries are also green ratchets, not the internal-no-observable-surface exception: their public behavior is covered by the two fixture-backed mustmatch entries.

## Improved Green Test Eligibility

All five conditions hold for the two `lane: check` entries. (1) Ticket 664 explicitly converts assertions to stronger deterministic captured-response proof. (2) `MyVariantClient`/`CancerHotspotsClient`, their source tests, `variant.md`, and the two live pages identify the code and coverage gap. (3) recorded provider requests showed the current runtime’s expected shapes. (4) the commands assert non-empty decoded result/recurrence fields and request routing, catching silent source-query or rendering regressions. (5) red behavior would mean deliberately breaking shipped working commands or inventing a failure; neither is ticket scope. The native unit entries strengthen evidence of the same existing paths and likewise remain green ratchets after raw captures replace pending fixtures.

## Landmine Review

- **Happy path:** the mustmatch commands are successful BRAF/MYD88 user workflows; no credential removal, mock failure, or synthetic error path is documented.
- **Real services:** no service is mocked in Markdown. The routine fixture is an allowed local deterministic replay of recorded provider bytes, with HTTP lifecycle outside Markdown. Public service availability stays optional verify evidence.
- **Observed provider shape:** every asserted provider field/value comes from the 2026-08-05 requests documented above. Captures retain raw bytes and receipts. Assertions avoid volatile totals/order and never rely on an empty collection alone; checked empty output is paired with non-empty BRAF/MYD88 cases.
