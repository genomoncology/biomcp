# 2014 — Adopt the shared direct ClinVar record

Status: OPEN.
Draft classification: pending Root.

Milestone: 1.0
Component: backend
Owner: Root.
Prepared and signed: delegated GPT-6.1 SOL consumer ticket preparer, October 6, 2026.
Phase: ticket preparation only. Root classification and subsequent BUILD release remain pending.
Risk: High. Shared direct and cached records must preserve separate admission, source attribution, section outcomes and output compatibility.

## Outcome

Carry the accepted BioData 0710 complete direct ClinVar graph through the actual BioMCP 1.0 ClinvarClient and requested variant section. Store the shared record directly, delegate cached conversion, and retire the five local authoritative structs, local XML projection algorithm and cached fallback reader after actual caller proof. Preserve existing CLI, native callable, typed MCP and raw MCP answers in JSON and Markdown.

This serves the ideal-state 1.0 backend move: BioData owns pure source parsing and values; BioMCP owns retrieval, request policy, section assembly, headline policy and rendering. This is one complete source and section object. Whole variant response ownership remains deferred.

## Evidence

- Starts from: dedicated consumer `b70aa501ade1a3ad52fdf78b4067a8cdb50b0ed8`, with locked BioData `33b8ff13f1f445eadbfa10fb433852f5b77b4a1d`. [2012 cached adoption](../records/2012-adopt-cached-variant-evidence.md) and [2013 maintenance incorporation](../records/2013-incorporate-main-maintenance-build.md) are complete. The accepted paired [BioData 0710 ticket](https://github.com/genomoncology/biodata/blob/2d96b0059702c09a8938672cc12dc9025c3dd89a/sdlc/tickets/0710-design-direct-clinvar-record.md), [design](https://github.com/genomoncology/biodata/blob/2d96b0059702c09a8938672cc12dc9025c3dd89a/sdlc/planning/direct-clinvar-record-0710/design.md), [cases](https://github.com/genomoncology/biodata/blob/2d96b0059702c09a8938672cc12dc9025c3dd89a/sdlc/planning/direct-clinvar-record-0710/cases.md) and [file claims](https://github.com/genomoncology/biodata/blob/2d96b0059702c09a8938672cc12dc9025c3dd89a/sdlc/planning/direct-clinvar-record-0710/file-claims.md) define the unchanged contract. The October 6 narrow repair confirmation accepts DESIGN `2d96b0059702c09a8938672cc12dc9025c3dd89a`. It supplies contract review evidence, not Root acceptance of this consumer draft. Prior producer records 0667/0669 and experiments 674/703 establish package availability, real execution and independent expected literals as distinct proof. Existing consumer records 1154/1243 and the retained XML fixture establish direct behavior and pre-parse depth protection. No new experiment is needed.
- Keeps: successful direct empty or absent records without cached fallback; cached fallback only on direct failure; unavailable without usable cache; missing numeric ID inapplicable without fetching; default unrequested null with no direct call; all-selection behavior; NCBI versus MyVariant source provenance and private errors. Preserve supported VCV/RCV/SCV metadata, current noncontributing submissions, row order, assertion-linked conditions, citation/criteria/comment text, classification levels, headline/date/review policy, finite JSON field order, omission/default/null behavior and Markdown distinctions.
- Changes: exact qualified BioData dependency pin, shared immutable record storage, thin optional target Serde bridge, ClinvarClient byte-parser delegation, finite cached conversion delegation and shared getter/borrowed target use by current policies and renderer. Retire displaced local ownership only after replacement owners pass.
- Proof: reuse the repaired paired cases table and existing actual-caller owners below. Add only the missing positive complete-record channel claim in `direct_clinvar_record_transport_tests.rs`. Later BUILD grows that missing behavior red-green through the actual client, records the exact producer/consumer pair and performs focused local verification. Preparation validates only records, config and Git; it runs no product checks.
- Defers: whole Variant/VariantSearchResult ownership, other enrichment families, normalization providers, clinical interpretation, new provider support, raw XML archival, new fixtures or provider acquisition, release and publication. Completed whole-hit and cached evidence migrations stay complete. No broad donor audit, producer semantic matrix, new proof system or new ticket is in scope.

## Prerequisite and authority

Producer pin: PENDING. The existing locked `33b8ff13f1f445eadbfa10fb433852f5b77b4a1d` is the starting pin, not the new direct-record pin. BUILD requires Root acceptance of a qualified exact producer revision exposing the reviewed API and target, plus Root classification of this ticket and explicit BUILD release. Record that exact revision before changing Cargo files. Never infer it from the design SHA, a moving producer branch or a current producer tip.

This ticket does not depend on BioData 0710 COMPLETE. The paired 0710 outcome includes this consumer adoption. A Root-accepted qualified producer pin unlocks consumer BUILD; accepted consumer proof then contributes to paired completion. This avoids a circular completion dependency.

Root owns maintenance reconciliation under BioData ADR 0029 before BUILD and again before CODE review. Ticket 2013 already records the incorporated maintenance baseline `3ad939f047c04a6bc4d659d963483729b52dfd7b`. Classify any later delta before incorporating it. Preparation performs no merge or product edit. Paired DESIGN ACCEPT may supply review evidence while the contract stays unchanged. Root decides this draft's classification and any additional review. No new acceptance is asserted here.

## Proposed BUILD file claims

| File or boundary | Authorized later change |
| --- | --- |
| `Cargo.toml`, `Cargo.lock` | Pin only Root's accepted qualified producer revision. |
| `src/sources/ncbi_efetch.rs::clinvar` | Delegate admitted bytes to `NcbiClinVarVcv::parse_record_bytes`; use its authoritative `MAX_BODY_BYTES` in the existing limited reader. Retain URL/key handling, HTTP plan, status/content-type checks, NoStore, timeout/retry and static failure mapping. Retire local ClinVar XML walkers, budgets, constants and projection tests only after shared public coverage. Preserve unrelated EFetch/JATS behavior. |
| `src/entities/variant/mod.rs` | Store `ClinVarRecordProjection` in `Variant.clinvar`; bridge Option Serde through `as_record_target` and `deserialize_record_target`; delegate fallback to `ClinVarRecordProjection::from_myvariant`; read shared getters in section policy. Retire ClinvarRecord, ClinvarRecordClassification, ClinvarAggregate, ClinvarSubmission and ClinvarCitation, plus indirect_conditions and the semantic indirect_clinvar_record reader. A needed compatibility alias points to the shared type and owns no fields. |
| `src/entities/variant/get.rs` and `get/tests.rs` | Change existing headline getter reads and register the new leaf. Migrate existing setup through shared target decoding. Add no test to the shared tests file. Retire only covered fallback scaffolding. |
| `src/entities/variant/get/direct_clinvar_record_transport_tests.rs` | Add the unique complete direct-record metadata, current noncontributing submission and Markdown behavior through existing TestHttpFixture and ContractHarness and the actual ClinvarClient. |
| `src/mcp/shell/typed_get_tests.rs` | Narrow only assigned overlaps in the existing ClinVar owner after replacement proof. Retain its unique all-selection and inapplicable/no-fetch/provenance assertions. Add no shared-file test. |
| `src/render/markdown/variant.rs` and its existing test callers | Feed the shared borrowed target into the separate template serialization at the clinvar field; a Variant field bridge alone does not cover this caller. Preserve templates and output rules; adapt existing setup only as needed. |
| Existing focused, package, boundary and source/test size registrations | Register only actual new owners, paths and measured counts required by current policy. Preserve count contracts and inherited findings; add no unrelated headroom or warning suppression. |
| This ticket and one `sdlc/records/2014-*` completion record | Record accepted pair, actual results, retirement and deferred gaps at completion. |

Product `src/xml.rs` and its tests remain because unrelated actual callers use them. The shared producer owns direct XML admission and resource/property evidence. This consumer does not implement a second guard or copy the producer's matrix. Retain existing source fixture bytes and capture receipts unchanged. Keep cached headline condition counting separate from cached record conversion. Direct XML limits never narrow already admitted cached projections.

## Behavior and test ownership

| Given and when | Then and owner |
| --- | --- |
| Default, direct-empty, malformed direct response, usable or absent cache, recursive cached conditions | Existing `get/clinvar_adoption_transport_tests.rs::clinvar_native_cards_keep_conditions_fallback_and_channels` retains native, CLI JSON, typed/raw MCP JSON and raw MCP Markdown outcomes, requests, provenance and privacy. Successful empty never falls back; direct failure alone may degrade. No duplicate state matrix. |
| Current direct VCV with distinct record/RCV/SCV metadata and a current false-contribution SCV reaches the actual client | New `get/direct_clinvar_record_transport_tests.rs` carries independently authored fields through shipped CLI and typed/raw MCP JSON and Markdown. Keep private markers absent. Its additional claims cover the complete record, retained noncontributor and Markdown distinctions. It adds no request-state matrix. |
| Request all or request ClinVar without numeric ID | Existing `mcp::shell::typed_get_tests::clinvar_override_is_consistent_across_request_modes_and_mcp_surfaces` retains unique all-selection and native/typed/raw no-fetch, absent payload, empty section sources and source-free provenance. |
| Direct record changes the existing headline | Existing get/tests.rs owners `headline_follows_record_level_germline_classification_and_names_ncbi`, `record_without_germline_classification_keeps_derived_value_labeled` and `agreeing_record_level_classification_drops_the_cached_copy_note` retain product precedence and derived-label policy using shared fixtures. |
| HTTP transport refuses a response or reads a bounded body | Retain `request_plan_uses_numeric_variation_identity_and_vcv_mode`, `response_requires_success_and_xml_content_type` and existing body-reader ownership. Shared typed parser failures map to the existing private product error. |
| Recorded TP53 G105S input is retrieved | Existing `spec/entity/variant.md` direct/cached assertions and ordinary fixture machinery retain the answer. Reuse `testdata/sources/ncbi_efetch/clinvar_428884_20261003.xml`, SHA256 `1077664822261adf69c1c7b6090f1df4d736c12113476f31a25d366c7689a329`, with its capture receipt. This establishes that retained input, not every provider form. |
| Markdown distinguishes source, status and classification domain | Existing `clinvar_render_tests::clinvar_markdown_keeps_vcv_rcv_and_scv_statuses_and_domains_distinct` and renderer assertions remain until the positive shipped-channel owner covers every displaced claim. |

Retire overlapping assertions only after their named owners pass. In typed_get, default/no-call and fallback outcome/source/provenance move to the existing channel owner; fallback aggregate version/date/submitter values belong to the producer cached-conversion owner; exact/excess criteria admission belongs to its public parser edge table; excessive-criteria degradation belongs to the existing channel owner's direct-error fallback; direct-data outcome and NCBI attribution belong to the new positive leaf. Retain the existing typed_get test and its unique request claims.

The producer public owner must preserve displaced source mapping, identity/status, numeric/Boolean refusal, duplicate-ID, hostile XML, projected-text/preflight and list/text/node claims before local decoder tests retire. Retire `get/tests.rs::indirect_clinvar_fallback_preserves_accession_freshness_and_submitter_count` only after public cached conversion and the existing channel owner preserve every assertion. Keep mod.rs policy cases for states without wider coverage. Remove only redundant constructor/builder scaffolding; preserve each remaining unique policy claim.

## Later proof and finish line

After Root releases BUILD, write the missing actual-caller behavior first and observe a failure for its named reason. Implement the reviewed API integration, run the smallest affected offline selection on M5 and keep one owner per claim. Producer qualification supplies its separate package/API, parser/generated-depth, privacy and finite-target checks. Do not recreate them in the consumer.

Root qualifies the exact consumer revision and locked producer pin with affected transport, section, headline, renderer, CLI/MCP, recorded spec and package/privacy checks. Use the existing focused migration selection and add only the new owner if required. Retain 2012/2013 inherited Darwin isolation, Clippy and source-size findings as evidence; a changed pair still needs its own affected proof. No prior exception automatically accepts a new defect. No whole consumer gate, hosted job, provider request or release qualification is claimed.

Finish after fresh CODE review, qualified exact-pair behavior, verified actual-client delegation, shared record storage and renderer target use, retirement of the five authoritative structs and both displaced projection algorithms, and one completion record. Root owns record acceptance, paired reconciliation and later landing. Preparation stops at the pushed draft and receipt; main, the existing consumer checkout and the producer remain untouched.
