# 2013 — Incorporate accepted main maintenance into 1.0

Status: COMPLETE.

Milestone: 1.0
Component: backend
Owner: Root.
Authority signature: Root owns routine incorporation under Ian's October 6 direction. Fresh DESIGN review ACCEPT at 985ac7e80bfff0a7d4a2193cd43532f59eae6ddd released the bounded build. Fresh CODE review ACCEPT at 3a836563af98b06fbdc2fe53daef8e8bf1b3ceff. Root accepts the recorded inherited findings and authorizes routine landing.

## Outcome

Incorporate accepted main revision 3ad939f047c04a6bc4d659d963483729b52dfd7b into the dedicated 1.0 line. Carry working Europe PMC pagination, whole article abstracts, cleaner JATS and PMC HTML, and honest WHO export parsing, sync reporting and drug-search degradation through the existing CLI and MCP surfaces. Preserve the adopted shared providers and the exact BioData dependency. Root reviews this design before assigning the merge build.

## Evidence

- Starts from: dedicated revision c66bea4217898956b5183698e2fd165e786a59fe and BioData 33b8ff13f1f445eadbfa10fb433852f5b77b4a1d. Main was fetched October 6 and remains 3ad939f047c04a6bc4d659d963483729b52dfd7b. The common ancestor is 6d525ac17fcd1bf0ba6248cd0fb4175f563e70be. Accepted donor tickets [1294](1294-return-whole-abstracts-and-cleaner-full-text.md), [1298](1298-page-europepmc-with-cursormark.md) and [1304](1304-track-the-who-pq-exports-real-header-set.md) own the maintenance behavior. Their donor completion updates arrive with main.
- Keeps: [2011](2011-adopt-shared-myvariant-hit.md) complete MyVariant source encoding and stored integer versus float categories; [2012](2012-adopt-shared-cached-variant-evidence.md) immutable CGI and CIViC projections, independent source presence, normalized selection, uncapped publication collection, actual hydration, section policy, citations and private errors. Keep shared PubTator and Europe PMC detail admission, acquired bytes, identity binding, strict refusal, source masks, budgets, deadline handling and provenance. Keep the existing dependency float configuration and exact pin.
- Changes: adopt the donor behavior described below through the migrated 1.0 owners. Reconcile only the seven predicted conflicts and the directly affected moved callers, tests and existing registrations. Keep one active release declaration for 1.0 targeting biodata/biomcp-1.0 and the backend component. Keep the global ticket allocation. The obsolete number-range configuration is already removed. Replace AGENTS.md's remaining branch-specific numbering paragraph with Ian's global allocation ruling; replace its stale hosted-verification instruction with the authorized M5 focused verification.
- Proof: reuse the donor's accepted experiments 437 and 434 for abstract and full-text defects, experiment 439 for the ignored page parameter, and the recorded October 6 WHO exports and receipts. [2012's completion record](../records/2012-adopt-cached-variant-evidence.md) establishes the retained shared-cache baseline; [2011's record](../records/2011-adopt-shared-myvariant-hit.md) establishes complete-hit and numeric behavior. Run the finite affected owners below after the reviewed merge build. DESIGN preparation makes no product or gate-pass claim.
- Defers: other migrations, final response ownership, new providers, new acquisition, live checks, source capture jobs, release, publication and unrelated gate repairs. Keep the donor's documented sort-tie risk, merged-cell table limitation and JATS byline gap. No new checker, inventory, gate or CI work belongs to this ticket.

## Changed behavior

Europe PMC starts at cursorMark=* and follows nextCursorMark. Offset walks and discards rows within the existing fetch bound. Fifty available distinct rows need two requests at page size 25. Fetching stops at the requested limit, an empty page, missing or repeated cursor, hitCount exhaustion or the existing fetch cap. Deduplication, sort, merge order, request accounting and real fetched-page warnings retain their owners. DOI, PMID, PMCID and retraction rescue requests use the first cursor.

Article detail and detail batches retain whole abstracts. Search snippets keep their 240-byte bound. JATS inserts spaces at qualifying inline element boundaries, labels PMID and PMCID reference identifiers, and renders ragged tables as raw rows. Source prose stays faithful. PMC HTML retains title and byline and drops viewer furniture and scholar lookup links while keeping canonical citation links. Full-text source order, sections, coverage and citation provenance remain governed by the existing detail path.

WHO parsing trims header cells before comparing case-insensitively, accepts current recorded medicine headers and the existing applicant aliases, and removes the obsolete required column. Sync attempts every export and reports refreshed and failed files. Missing required files produce a nonzero outcome; cached usable files retain the donor's degraded reporting behavior. Region-less drug search degrades when WHO is absent; explicit WHO requests retain their failure. Regional selection, stale windows and body limits keep their existing behavior.

## Merge choices for fresh review

A Git merge-tree preview of the exact revisions reports these seven conflicts. It writes no checkout merge or product commit. The preview is evidence for this plan, not an implementation.

| Conflict | Resolution choice |
| --- | --- |
| src/cli/system/mod.rs | Keep 1.0 PreparedBatch and preflight_batch. Add donor who_sync module and export handle_who from it. Retain the other dispatch exports and batch admission. |
| src/entities/article/backends.rs | Keep plain_page_result! around variant_article_request and the dedicated failure handling. Carry cursor state, nextCursorMark capture, fetched-row accounting, walk-and-discard offset and donor termination into that wrapper. Update the rescue request to *. Preserve provider units, deadlines and candidate provenance. |
| src/sources/europepmc.rs | Keep shared detail transport, doi_query and search_by_doi_with_query. Change its legacy DOI first request to *. Retain response.request and EuropePmcLegacyRequest beside next_cursor_mark. Preserve strict optional request echoes in detail.rs and the adopted detail request plan. Update all existing search callers and response literals to the cursor signature. |
| src/transform/article/federation.rs | Keep shared publication mapping and the test-only retained_from_pubtator_document alias. Apply donor whole-abstract behavior to the moved assembler in src/transform/article/pubtator.rs for both adopted and legacy detail. Keep the donor Europe PMC metadata changes and the existing adopted detail owner. Remove the moved assembler's truncate_abstract dependency before accepting the donor helper removal. Do not resurrect main's local authoritative PubTator detail converter. |
| tests/test_source_package_boundary.py | Keep the 1.0 BioData boundary contract, exact revision and required area/member checks. Register the donor article-text-fidelity page in its existing required-member set. Main's 1399 fixed package count and zero-coupling prohibition belong to the other release line and do not replace this contract. |
| tools/rust-source-size-inventory.json | Keep 1.0 allowances and prior removals. Reconcile error.rs using its retained 1109 floor and only the donor's 41 added lines: preview gives 1371 baseline and 262 delta. Retain the donor WHO entry's 1229 baseline and 112 delta. Confirm exact counts after conflict resolution and formatting. Existing 2012 get.rs excess remains separately recorded; do not hide it by adding headroom. |
| tools/zero-coupling-historical.json | Preserve the dedicated deletion. The donor receipt digest update belongs to main's retired 0.9 zero-coupling checker. Carry the merged capture-receipts.json and recorded public fixtures through the existing 1.0 boundary contract. |

The two sides also touch dispatch.rs, drug/search.rs, error.rs, article transforms, fixture routing and receipts without reported conflicts. Retain both reviewed paths when resolving adjacent hunks. Preserve fixture cursor canonicalization and the original captured-body digests. Do not replace the 1.0 source/detail layer with a complete donor file.

The accepted 0.9 metadata is donor history. Main has no new delta in AGENTS.md, sdlc/pm.json, sdlc/planning/milestones.md, Cargo.toml or Cargo.lock since this common ancestor. Keep the dedicated declarations and pin. The new baseline is the reviewed merge candidate with both c66bea4217898956b5183698e2fd165e786a59fe and 3ad939f047c04a6bc4d659d963483729b52dfd7b as ancestors; its exact SHA belongs in Root's build record.

## Finite proof and existing owners

| Behavior | Existing owner and bounded action after ticket acceptance |
| --- | --- |
| Cursor paging and offset | Donor spec/entity/article.md Europe PMC cursor fixture owns 50 rows in two requests, hitCount exhaustion, missing-cursor stop and offset 25. Keep its fixture lifecycle registrations and run its relevant scenarios. Existing src/sources/europepmc/tests/construction.rs owns the request shape. Retain the updated canary routing test without a live canary run. |
| Whole abstracts and clean full text | Donor spec/entity/article-text-fidelity.md owns recorded whole abstracts, JATS boundaries and references, PMC title and furniture, and authored ragged rows. Run it through the actual merged binary. Existing entities::article::detail::pubtator_surfaces and europepmc_surfaces own CLI, ordered detail batches and raw/typed MCP. Adjust only stale truncation expectations. If adopted PubTator long-abstract channels lack a claim, add one ticket-specific outside-in case using the existing recorded abstract; do not copy the donor parser matrix. |
| Shared Europe PMC detail | Existing sources::europepmc::tests::detail owners cover exact plans, acquired bytes, admission, optional echoes, guarded legacy DOI and terminal refusals. Existing entities::article::detail::europepmc_surfaces owners cover identity, hint reuse, long abstract, provenance and channels. Run the affected existing scope. |
| WHO sync and degradation | Existing cli::system::tests::who_sync::who_sync_reports_refreshed_and_failed_files_instead_of_a_bare_success_claim and entities::drug::search::tests::who::all_region_search_degrades_when_who_pq_data_is_absent_but_explicit_who_fails own the user behavior. Existing sources::who_pq::tests::sync replays recorded exports and required-file failure; parsing owners cover trimmed headers and applicant aliases. Run these existing owners with recorded local fixtures. |
| Shared cached evidence and channels | Existing entities::variant::get::cached_evidence_transport_tests::{default_cached_sections_keep_cli_and_mcp_channels,requested_cgi_keeps_cli_and_mcp_channels} own the six CLI/raw MCP/typed MCP operations per case. Retain these two owners. |
| Hydration, citations and provenance | Existing entities::article::variant_search::cached_evidence_tests::{cached_source_presence_plans_only_missing_hydration,citations_beyond_cached_display_cap_reach_article_route} and hit_adoption_tests::{complete_hit_normalization_selects_confirmed_citations_in_both_orders,opaque_civic_presence_controls_hydration_and_citation_provenance} own the affected seam. Run them without duplicating producer semantics. |
| Stored numbers and encoding | Existing sources::myvariant::tests::hit_adoption::{callable_hits_keep_stored_numeric_precision_and_independent_depth,shared_hit_keeps_original_fragments_and_normalizes_product_once} own numeric precision and complete encoding. Preserve these two checks and the pin. |
| Package and fixture contract | Existing tests/test_source_package_boundary.py, tests/test_article_spec_fixture_lifecycle.py and tests/test_variant_article_live_canary.py own package inclusion and offline fixture routing. Run their affected scope. Existing whitespace and source-size checks verify only the changed boundary. |

Root owns the existing focused migration verification and records the exact candidate, exact pin and inherited findings. Use tools/check-biodata-1.0 and its existing focused selection when qualifying the candidate. Add only new affected owner selections if required; maintain its existing count contract. Extend no proof beyond an unresolved merge risk or the existing required gate. No broad donor re-audit, new inventory, repeated producer matrix or hosted job is authorized by this DESIGN assignment.

## Allocation and next action

PM reserved refs/pm/2013 through SSH from an isolated allocation clone whose local main equals freshly fetched origin/main. The ticket retains 2013. The design branch starts at the exact dedicated base and changes only this ticket. The inherited configuration declares only active 1.0 and has no branch-owned number range. Existing allocation clones and their untracked 2011/2012 originals remain untouched.

Root must obtain a fresh read-only ticket review of this pushed design. After ACCEPT, Root signs the review decision and assigns the bounded merge build, fresh code review, verification, record and landing. Any newer donor tip needs a named baseline amendment before incorporation. The designer does not merge main, edit product code or assign nested work.

## Review correction

Fresh ticket review identified the stale AGENTS.md numbering paragraph. Root replaces it with the final global sequence, PM allocation, sole active1.0 declaration and backend component. The adjacent instruction still called for hosted verification; Root replaces it with the already authorized local focused checks. This amendment implements existing rulings and changes no product behavior or release declaration. Original product merge choices and scoped checks remain unchanged.

## Build handoff

The exact accepted donor is merged through normal Git history. The candidate also merges dedicated metadata revision 4817445c93c29551e6a66d3a29d897b5aebfe5f2 and retains the completed 2012 cleanup record. Both source histories remain ancestors. The BioData pin and active release declarations remain unchanged. The build record names checked revisions, measured costs and inherited failures. Ticket status stays OPEN pending fresh CODE review and Root landing.

## Acceptance and landing

Verification (carried ticket): ACCEPT 2026-10-09 — carried from the 1.0 branch with its landing history intact. Fresh CODE ACCEPT at 3a836563af98b06fbdc2fe53daef8e8bf1b3ceff. Root accepts the documented inherited Darwin, Linux isolation, Clippy and source-size findings for this bounded incorporation.

Landed: 3a836563af98b06fbdc2fe53daef8e8bf1b3ceff. This reviewed candidate retains the qualified runtime from 510077760477781ab9b24611656e394984264753. The subsequent Land 2013 commit records completion and acceptance. See the [existing build record](../records/2013-incorporate-main-maintenance-build.md).

## What the build taught us

The moved detail owners needed updated donor callers and whole-abstract expectations. The existing finite behavior checks cover the incorporation. Darwin replay qualifies the runtime behavior but does not qualify the inherited Linux fixture supervisor. Retain the inherited findings and use the recorded results for landing.
