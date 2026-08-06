## Code Path Trace

`biomcp get variant` reaches `src/entities/variant/get.rs::resolve_base_with_hit`, which constructs `MyVariantClient`, selects a direct `/variant/{id}` lookup or `/query` for rsID/gene-protein input, then transforms `MyVariantHit` into the public result. `add_cancerhotspots` in the same module constructs `CancerHotspotsClient`, calls `by_gene`, reduces rows through `recurrence_for_change`, and records data/empty/unavailable section outcomes. `src/entities/variant/structure.rs::cancerhotspots` uses the same client and reducer for the structure response.

`MyVariantClient::{query_plan,search_plan,get_plan}` in `src/sources/myvariant.rs` create `RequestPlan`s; `query_with_fields`, `search`, and `get` execute them, bound the response, and deserialize with the production types. `CancerHotspotsClient::by_gene_plan` creates `api/hotspots/single/byGene/{encoded-gene}` and `decode_by_gene_response` deserializes the provider array. The live canaries are `spec/entity/variant-myvariant-live.md` and `spec/entity/variant-hotspots.md`; `scripts/run-specs.sh::SPEC_LIVE_PATHS` routes both exclusively to `make verify`.

## Constraints

`AGENTS.md` requires observable contracts in `spec/*.md`, routine execution through `make spec`, and source internals/error paths in native tests. The live-conversion target requires Tier 2 consumed request plans, Tier 3 production decoding of receipt-backed real bytes, and fixture-backed CLI rendering before deleting a live assertion. `testdata/sources/capture-receipts.json` currently marks all MyVariant and CancerHotspots fixtures `pending_verification`, so none may support Tier 3 conversion.

Existing MyVariant Tier 2 tests cover request construction and filter normalization in `src/sources/myvariant/tests/construction.rs`; existing Tier 3 tests parse unreceipted `search_braf.json` and `get_braf_v600e.json` in `parsing.rs`. Existing CancerHotspots Tier 2 covers encoded by-gene request construction, and Tier 3 parses unreceipted BRAF rows and recurrence in `src/sources/cancerhotspots/tests/{construction,parsing}.rs`. The existing fixture server seam is the source-specific `BIOMCP_*_BASE` environment setting; runner-owned setup scripts, not markdown, own server lifecycle.

## Prior Art

`architecture/technical/live-spec-conversion-target.md` names these exact two live pages as convert targets and requires BRAF/MYD88, empty, recurrence, consequence, and filter captures. Ticket 662 converted ClinGen live contracts with dated receipt-backed captures. `spec/fixtures/setup-variant-identity-spec-fixture.sh` is the local HTTP-server pattern: it owns an ephemeral process, exports `BIOMCP_MYVARIANT_BASE`, and validates requested paths/query semantics. Commit `aa689aef` shows the current conversion direction: replace unstable/empty live proof with deterministic, non-empty, source-faithful assertions.

## Hard Parts & Risks

The main risk is inventing provider fields or response shape. On 2026-08-05, real provider probes observed: `GET https://myvariant.info/v1/variant/chr7:g.140453136A%3ET?fields=_id,dbnsfp.genename,dbnsfp.hgvsp,dbnsfp.bayesdel.add_af.score,dbnsfp.bayesdel.no_af.score,cadd.consequence` returned `_id` `chr7:g.140453136A>T`, `dbnsfp.genename` as an array containing `BRAF`, `dbnsfp.hgvsp` containing `p.Val600Glu` and `p.V600E`, and BayesDel scores `0.399079`/`0.335473`. The observed BRAF missense `/query` request returned `total: 4976` and non-empty `hits` whose gene is an array and CADD consequence is `NON_SYNONYMOUS`; no assertion will pin count/order. The BRAF REVEL presence request returned non-empty hits with numeric `dbnsfp.revel.score`.

Real CancerHotspots `GET https://www.cancerhotspots.org/api/hotspots/single/byGene/BRAF` returned a `V600` row with `tumorCount: 897`, `transcriptId: ENST00000288602`, and `variantAminoAcid.E: 833`. The MYD88 endpoint returned `L265`, `tumorCount: 37`, `transcriptId: ENST00000396334`, and `variantAminoAcid.P: 37`. These values are appropriate only after recording raw responses and their receipt hash. Empty behavior must be proved by an observed captured `[]`, paired with a non-empty decoded case, never by a live empty assertion. Fixture routes must reject unexpected requests so CLI-to-API construction is genuinely exercised.

## Scope: Required vs Deferred

Required: create consumed MyVariant get/search/filter and CancerHotspots join plan proof where missing; record receipt-backed raw MyVariant consequence/filter and CancerHotspots BRAF/MYD88/empty responses; make Tier 3 decoder/orchestration tests consume those bytes; provide runner-owned fixture routing for local CLI presentation proof; migrate the two live documents out of `SPEC_LIVE_PATHS` only after this proof exists. The fixture-backed specs should assert stable public fields and non-empty data, not live totals or provider ordering.

Deferred: changes to filters, thresholds, output schema, unrelated variant sources, redesigning request-plan infrastructure, and provider availability smoke coverage. The implementation must not alter production behavior to suit a capture.

## Test Coverage

New/strengthened Tier 2 native tests belong with `src/sources/myvariant/tests/construction.rs` (consequence and `--has/--missing` plans as consumed by CLI) and `src/sources/cancerhotspots/tests/construction.rs` (BRAF/MYD88 by-gene plans). Tier 3 belongs in the corresponding parsing tests and must use the receipt-backed filenames. Orchestration coverage belongs in `src/entities/variant/get/tests.rs` and structure tests as appropriate, proving matching BRAF/MYD88 output and captured empty/missing-field behavior. A runner-owned source fixture must serve only the captured bytes and capture/validate the outbound MyVariant and CancerHotspots requests. The BDD replacement belongs in `spec/entity/variant.md`, which is already routine, while the two source-specific live pages become explanatory coverage records and are removed from `SPEC_LIVE_PATHS`.

## Spec Coverage

`spec/entity/variant-myvariant-live.md` currently proves BayesDel, GERP/consequence/review/field filters through live provider data. `spec/entity/variant-hotspots.md` currently proves BRAF/MYD88 recurrence and the broader structure join live. They are verify-only and therefore do not give routine deterministic proof. `spec/entity/variant.md` already documents deterministic source contracts and runs in the check lane, but has no fixture-backed MyVariant filter/consequence or CancerHotspots recurrence assertion. The new spec assertions will be improved green tests: the ticket explicitly converts/strengthens already-shipped coverage, manual live probes show behavior works, and intentionally making runtime behavior red would be false/out of scope. They will catch the realistic regression of a CLI request not reaching the captured source route or public JSON losing decoded consequence/recurrence fields.
