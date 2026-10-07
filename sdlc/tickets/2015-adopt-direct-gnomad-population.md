# 2015 — Adopt the complete direct gnomAD population graph

Status: OPEN.

Milestone: 1.0
Component: backend
Owner: Root.
Prepared and signed: delegated Codex design owner, October 7, 2026.
Phase: CODE review ready. Root released BUILD after paired DESIGN ACCEPT, accepted producer 0711 and maintenance landing.
Risk: Response compatibility. Source-derived frequencies and decoded product targets have different construction rules.

## Outcome

Move the complete direct gnomAD v4 population record into BioData and carry its shared exome, genome, ancestry and FAF values through actual variant retrieval, CLI and MCP JSON and Markdown. Remove the four local source structs and their frequency projection after replacement proof. This completes one enrichment boundary in the 1.0 backend adoption. BioMCP retains the Variant and VariantSearchResult envelopes, execution, coordinate admission, section outcomes and display policy.

The [design and exact file claims](../planning/direct-gnomad-population-2015/design.md) define the proposed paired change. This ticket authorizes design preparation only. Root coordinates the BioData producer ticket, maintenance reconciliation, independent review and subsequent build release. No implementation authority follows from the remote reservation.

## Evidence

- Starts from: BioData main b0fc4ec3ec64b649ca8fe55d0c0584fc7c5972e5 and dedicated BioMCP d6aa002f2b5ea02f83aee7adab87216c44e911b5, locked BioData dbbaa1aedbd16c5e51e022197792dc35bb8132bc, version 0.0.37. [2011](../records/2011-adopt-shared-myvariant-hit.md), [2012](../records/2012-adopt-cached-variant-evidence.md) and [2014](../records/2014-adopt-direct-clinvar-record.md) complete the source hit, cached evidence and direct ClinVar boundaries. The assigned current delivery plan and consumer chart identify remaining response and enrichment adoption. Actual callers are GnomadClient::variant_population, get.rs::add_population/population_result and markdown/variant.rs. Experiment lessons in producer records 0667/0669 and subsequent source-adoption designs require independent expected literals and actual caller proof; no new experiment or copied corpus is needed.
- Keeps: direct exome/genome separation; overall and ancestry ac/an, homozygote and hemizygote counts; filters, FAF95 and raw ancestry IDs; null and empty distinctions; zero-denominator null frequencies and count-derived finite frequencies; coordinate and response-ID admission; GraphQL not-found/error policy; default no-fetch and requested/all section behavior; dataset/release/caveat, dbSNP-assisted provenance and private section failures; JSON field order and Markdown compact/details policy. Preserve existing cached MyVariant AF and search behavior.
- Changes: one immutable complete source graph and finite borrowed population target in BioData; original streaming decoder delegation from the existing HTTP path; shared nested storage in GnomadPopulationResult; shared getters in source policy and rendering; retirement of GnomadVariantPopulation, GnomadSequencingPopulation, GnomadFaf95, GnomadAncestryPopulation and the local count-frequency mapper. Thin compatibility aliases may own no fields. Cargo pins Root's accepted qualified producer only after review.
- Proof: one small public producer owner covers the complete projection, source/target distinction, nearest invalid shapes, bounds and sanitized failures. One new product channel owner covers the missing actual direct-data path through native, CLI and typed/raw MCP JSON and Markdown plus the changed failure/no-call claims. Retain existing coordinate recovery, section-null and renderer policy owners. Run focused offline checks against the reviewed pair. No live network, hosted CI, release gate or donor-wide audit belongs to this ticket. The design records observed historical build/test costs and requires measured costs at BUILD.
- Defers: whole Variant/VariantSearchResult ownership, gene constraint, cached MyVariant source adoption already completed, other enrichment providers, normalization, structure, new provider acquisition, clinical interpretation, publication and release. Root owns any newly fetched maintenance delta and separate producer allocation. This design performs no deletion, worker dispatch, nested agent or CI.

## Review prerequisites

PM allocated global 2015 with the sole active milestone 1.0. Component backend identifies ownership. The dedicated target remains biodata/biomcp-1.0. Fresh review must assess both proposed library and consumer boundaries and the source-versus-target frequency distinction.

During allocation, PM refused its hard-coded local-main freshness guard. Fetch observed origin/main 37631c3574703755bf742e1e8459bc0d77522220. Local maintenance main had no unique commits and no active checkout; its reference was fast-forwarded for PM allocation. No maintenance files or dedicated product bytes changed. HTTPS reservation pushes failed authentication; command-local SSH transport then reserved refs/pm/2015. No global configuration changed and no CI was dispatched. Root must classify and incorporate the new maintenance delta under ADR 0029 before BUILD, and refresh the reviewed scope if it affects these callers. This draft claims only the inspected d6aa002f behavior.

Root's next action is fresh read-only DESIGN review. No builder has started. Keep this draft local for that review; pushing a ticket branch can trigger repository CI and this assignment prohibits CI.

## Accepted producer and BUILD release

Root adopted fresh paired DESIGN ACCEPT after the borrowed ancestry-view correction. BioData0711 passed fresh CODE review at9169bfe1, with unchanged runtimeef4a5bbe and lint/test/spec passes. Accepted main25f0c7fd3cee947230231aae0e3ed3aa960ecd46 is version0.0.38. Current consumer target9ada31a9 includes reviewed maintenance37631c35. This release authorizes the exact reviewed consumer file claims, outside-in checks and four local model/count-mapper retirement.

## Consumer build candidate

The assigned builder implemented shared source decoding, nested product storage and borrowed target encoding on this ticket branch. [The build record](../records/2015-build-direct-gnomad-population.md) names the precise files, red evidence, measured focused checks, narrow retirement and inherited findings. The four additional contract files were named to Root before editing their revision/version/count constants. Fresh consumer CODE review remains pending. Root owns landing and ticket completion.
