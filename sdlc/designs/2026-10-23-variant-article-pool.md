# Variant article acquisition, ranking and paging

## Outcome

For a gene and protein change, the default `variant articles` result contains every PMID returned by the literal PubMed query and every result returned by its component strategies for the same query and options. It ranks papers from their recorded evidence and pages one immutable ordered pool. Slow providers cannot consume another provider's work allowance. Every response names unfinished sources. This design covers 2373 through 2377 and implements the shape requested in PM message m4157.

The membership guarantee applies before an explicit result filter. `--confirmed-only` still returns only confirmed papers. A candidate's inclusion never establishes its biological identity. A provider limit, timeout or failed acquisition cannot become a claim that BioMCP found all upstream papers.

## Starting facts and limits of the evidence

The source baseline is main `4506cadbfec08b9f95096bfa595d3da3c9d2fcbd`.

- The recorded literal `AKT1 E17K` PubMed ESearch response contains 192 PMIDs with relevance ordering. Research's 35 union invocations contain 1,386 distinct PMIDs and omit 20 of those 192. The raw replies live in research run 0043 under `routes/ncbi/case-005/deep-ncbi-search-1-0000/stdout` and `routes/biomcp/case-005/deep-variant-union-*/stdout`. These public provider replies are the replay input. No curation-team answers enter fixtures.
- The saved traces and the baseline invocation in job j7921 show six PubMed Title/Abstract plans, no literal plain query, and a strict query that receives and retains 25 rows while reporting a cut. Same-word lexical retrieval yields zero and the free-text fallback is inapplicable. The retained traces are bounded. They do not establish whether each missing paper reached an upstream intermediate result.
- The baseline rebuilds and ranks the pool for every offset. Recorded pool counts range from 1,516 to 1,709. The first three recorded pages show no deadline stop and use no identity verification. The outside-in replay in j7926 isolates the paging defect: annotation returns A and B, while two union pages return B and B although both calls retain two deduplicated candidates. Changing route support changes rank before the offset is applied.
- `planned_candidates` requests the first 25 rows of each planned query. Routes share an invocation deadline and run sequentially. A successful result can be discarded when shared admission stops during its request. `rank_candidates` uses confirmation when present, then route support and native positions. Default requests do not populate identity confirmation. Verification of an offset-dependent window can itself change final rank.
- The merge helper can collapse two distinct PMIDs through an equal DOI or PMCID. This threatens an every-PMID contract, but the recorded 20 misses do not establish this cause.

Accepted changes must be integrated before implementation changes these lines. Paging change 2365 at `884719fe780bc902cf8ba8425c305e7c49ee2db7` corrects false `has_more` without making a snapshot. The 2367 changes include parallel planned acquisition and commit `a41f3ed8efdf1a6927019ae58c3c7eb15aed634d`, which retains committed results after admission stops. The 2370 assembly ending at `81c81a5ce2ae20120602898526079c60dd68609a` includes 2367 and alias coverage work. The 2366 assembly ending at `84e5ea312d458c5a83567b4e2e3a0b0e639dba66` includes publication-date work and 2363. These accepted revisions are separate from the baseline observations above. Reuse their repairs and tests instead of recreating them.

## Stages and ownership

Replace the long `search_variant_articles_identity` body with orchestration over five local stages under `src/entities/article/variant_search/`: `sources`, `merge`, `rank`, `pool` and `page`. The existing module retains the public request and response boundary. Keep variant identity verification in its current module. Put snapshot persistence in `src/cache/variant_article_pool.rs` using the existing SQLite store.

The stages exchange an acquired candidate collection and source outcomes. Source outcomes carry successful membership, native ordering, provenance and unfinished work separately. Merge and rank perform no HTTP work. Page slices an already ordered snapshot and never discovers candidates or changes rank. Pool owns snapshot construction, publication and reuse. Visible metadata enrichment may fill missing display fields after slicing, but it cannot change membership or ordering.

Use `variant_article_request` and its existing provider-unit helper as the shared source request path for deadline handling, pacing, retries, typed failures and captured responses. Extend that path where needed for the touched providers. Do not introduce a second transport or move unrelated providers. Delete the unused executable discovery-plan representation and code that only maintains it. Keep debug query and coverage output as a description derived from the work that actually runs. Remove per-query 25-row cuts and their obsolete tests. Keep provider page sizes, hard provider limits and configured resource limits in their owning code.

## Acquire sources independently

Start the literal PubMed worker immediately after parsing the original gene and protein-change words. Start variant and gene resolution alongside it under separate bounded deadlines and reservations. Slow canonical, alias or gene resolution cannot delay the literal worker's start or consume its allowance. Resolution-dependent source workers start as soon as their required identity is available. Preserve all existing identity rules.

Each provider owns its deadline and work allowance. Corresponding component acquisitions receive the same reserved allowance, query semantics, provider paging and retry policy in standalone strategies and union. A standalone strategy cannot spend another component's unused reservation to acquire results that union is denied. Reserve these component allowances within the approved invocation envelope before starting work. Requests within one provider share its pacing; one provider cannot consume another's reservation. Preserve a finite invocation ceiling and the existing concurrency and resource limits. The reservations and provider deadline values must live once in configuration or owning code. Removing a per-query result cut does not authorize unlimited HTTP work.

The literal PubMed worker sends the original gene and protein-change words through ESearch with relevance sort. Extend the internal ESearch parameter type to express sort while preserving existing callers' defaults. This worker acquires the entire PMID list through provider paging. It creates membership seeds before optional ESummary or abstract hydration. Missing summaries leave PMID seeds available with missing metadata status. They do not erase membership or manufacture identity evidence. The recorded 192-ID response requires no new live capture.

The plain PubMed query runs independently of strict Title/Abstract and alias expansion queries. It is an additional default-union source. It cannot be removed by an identity-dependent free-text fallback condition. PubTator and LitVar keep their applicable typed entity requirements. Empty or failed PubTator discovery cannot become a successful shared annotation result. Shared acquisition may be reused only when its query, identity and result coverage satisfy both consumers.

A worker retains every successfully acquired identifier and its provider continuation before attempting more work. When its deadline or allowance ends, it returns that membership and an unfinished outcome with the existing reason codes. Persist the source's query identity, total, next provider offset or cursor, received membership and unfinished status through the same pool-cache module. The next matching call resumes this acquisition instead of restarting its first page. If the provider does not supply a stable continuation, restart that source's enumeration and union its retained IDs without deleting earlier membership. Keep acquisition resumable toward the complete literal ESearch floor; an incomplete response is visibly partial and never satisfies the completed superset outcome.

An ESearch count beyond the provider's retrievable range is an explicit blocker to the all-PMID outcome. Preserve the partial membership and identify that hard limit; do not silently redefine the contract as acquired IDs or mark it complete. The implementation must either use an already supported complete retrieval path within the approved limits or leave that case blocked for the owning product decision. The recorded 192-ID regression must retrieve all 192 under the unchanged resource envelope.

## Merge membership without loss

The pool is the set union of acquired component membership and the literal PubMed PMID set. Two rows with the same PMID produce one candidate with merged provenance. Two distinct nonempty PMIDs remain distinct even when a secondary identifier agrees. A row without a PMID may join a matching identified paper through existing secondary identifiers, provided the merge cannot bridge two distinct PMIDs. Missing metadata never decides membership.

Record each finding provider, route, query and native position using existing provenance. Repeated aliases and strict plus annotation routes from one provider do not count as independent agreement. Preserve received rows when subsequent work fails. Source status describes acquisition completeness separately from the number of stored candidates and the number on this page.

Union and its component strategies use identical component query definitions, source configuration and reserved allowances. Retained source membership and acquisition progress are keyed by those semantics, so a component's successful acquisition remains available to union. A failed or timed-out union worker cannot overwrite or omit that retained membership. Resume unfinished work for every source on the next matching call. Union is a superset of each component strategy; unequal allowances are a defect, not an exception to that outcome. A visibly partial union remains incomplete until its unfinished acquisitions settle.

## Rank from carried evidence

Use one deterministic lexicographic key over the snapshot's captured metadata, validated annotations and provenance. Compare each factor before considering the next:

`key = (T, A, C, I, R)` descending, then normalized stable identifier ascending.

- `T` is one when the existing lexical matcher finds the requested gene and a supported requested variant spelling in the title; otherwise it is zero.
- `A` is one when the existing matcher finds that evidence in the abstract, or the paper carries existing validated PubTator or LitVar variant linkage; otherwise it is zero.
- `C` is one when the existing compatible MyVariant record supplies the PMID through its CIViC curated citation field; otherwise it is zero. A generic citation or a provider's guessed variant identity does not qualify.
- `I` counts distinct finding-source families. Count a provider once across all its routes and aliases. Count PubTator and LitVar together where they share annotation origin. Credit CIViC rather than its MyVariant transport. This measures retrieval agreement, not independent clinical confirmation.
- `R` is reciprocal native rank summed once per source family, using the existing reciprocal-rank denominator.

No combination of later factors can outrank an earlier factor. A title match therefore precedes a paper supported only by abstract, annotation, curation or source agreement. No PMID, paper title, publication year or manually assigned landmark label enters the algorithm. This follows the requested factor priority without introducing a competing weighted policy.

Text relevance and identity confirmation remain separate. Preserve 2139's typed Gene/Variant linkage, expected-paper checks, contradictions and missing-typed-Gene reason. Preserve 2345's transcript and assembly context and unconfirmed equivalence behavior. No new gene-token fallback is allowed. Optional verification may annotate the page, but cannot rerank the immutable snapshot. A confirmed-only snapshot is separate, uses existing confirmation rules, and reports unfinished verification instead of claiming its verified subset covers all candidates.

The recorded AKT1 E17K replay must put 33123537 and 17611497 on the first 50-row page and match or exceed the recorded PubMed first-ten variant-evidence count. These are fixture expectations, not algorithm inputs. The recorded evidence must support each ranking input. If this priority cannot meet those expectations, the ranking outcome remains unmet. Return the discrepancy for PM consideration with the actual evidence; do not switch to weights, adjust a paper-specific exception or invent a clinical classification.

## Cache an immutable query snapshot

A snapshot contains the merged candidate membership, ranking inputs, final ordered identifiers, source outcomes, creation and expiry times, and the versions of acquisition, identity and ranking rules. It preserves unverified reasons and provenance. Use one atomic SQLite publication so readers see a complete old or new record. Reuse the cache's size accounting, cleanup, private storage and failure handling. A failed cache write must not erase the response's acquired rows.

The lookup identity contains the normalized requested identity and literal query, strategy, verification and confirmed-only options, source enablement, provider endpoint configuration, alias and genome configuration, and query/verifier/ranking versions. Offset and page size do not alter pool identity. The page token binds the snapshot identifier, query identity, ordering version and next offset. The returned page limit only controls slicing. Reject a token with conflicting request options.

Credentials never enter lookup keys, token material, stored configuration or logs, either directly or through a hash. Provider endpoints must be normalized without credentials. A credential-bearing custom endpoint whose semantics cannot be represented safely disables shared snapshot reuse. A source configuration change cannot reuse verification from the prior configuration. Honor cache bypass, clearing and expiry. Bound snapshot freshness by the existing source-cache freshness policy; do not let infinite HTTP cache mode silently make query snapshots immortal.

## Retries, late arrivals and page binding

A timed-out, unavailable or allowance-limited source resumes on the next matching call within its own reservation. Keep successful source outcomes and incomplete enumeration progress available so the retry does not rerun every source or repeatedly lose the same tail. A cached incomplete response never suppresses that retry. The response names missing sources and distinguishes the snapshot's membership from work acquired by a retry.

A continuation's retry allowance must fit its warm-page budget. Do not attach a full cold-discovery deadline to a cached page, launch detached retry work from the CLI, or call a slow retry a fast cache hit. If a source cannot settle within that allowance, keep its missing status and let the next call retry again. Measure complete and incomplete warm pages separately. Source retry timing must not change the page's membership.

Late rows produce a successor snapshot. They never insert into, rerank or replace the snapshot named by an existing continuation token. New searches can select the successor. Existing continuations keep reading their original ordered pool until it expires or is evicted. They then receive an explicit expired or unavailable continuation error and restart from offset zero. They never silently rebuild at a nonzero offset. Partial snapshots retain `complete=false`; reaching their last page means the stored pool ended, not that all providers completed. Preserve 2365's settled `has_more=false` behavior at that boundary.

PM decision needed: approve an optional CLI page-token argument and matching typed MCP input, using the existing `pagination.next_page_token` output to carry the bound continuation. This is the recommended way to preserve pages while retries publish successors. Legacy offset calls must keep one active query snapshot unchanged during its lifetime, including when a retry produces a successor. They cannot silently switch to that successor. A missing or expired active pool refuses a nonzero-offset rebuild. The PM must settle how a caller explicitly starts a successor session and how legacy offsets coexist with concurrent sessions. This interface decision stays open; implementation cannot treat an ambiguous continuation as permission to weaken fixed paging.

The implementation starts only after PM consideration of this design and its fresh review.

## Ticket boundaries and build order

The accepted native-gate batch lands independently. The five changes then share this design and one eventual integrated review boundary.

1. 2373 owns literal PubMed membership acquisition, explicit relevance sort and its replay fixture. Its product claim covers `variant_search.rs`, the new `sources` stage and the PubMed parameter implementation. Its outside-in test proves all 192 PubMed IDs survive complete paging, including a missing-summary case and interrupted acquisition that resumes to completion.
2. 2374 owns pure merge and retained component membership. Its developer can prepare its separate outside-in regression concurrently with 2373. It takes the shared source-file writer slot after 2373 commits. It preserves the accepted 2367 retention fix. Its test proves union contains component membership and reproduces A/B becoming B/B before the integrated paging fix.
3. 2375 owns the ranking stage after membership is settled. It reuses recorded metadata and supported annotations. Its separate replay test checks ordering and the two page-one expectations without naming either PMID in product code.
4. 2376 owns snapshot construction, cache integration, stage orchestration and page binding after the membership and ranking interfaces settle. It replaces the long orchestration body and removes dead plan bookkeeping. Its proof covers late arrival, expiry, configuration separation and explicit filters at the same public boundary as paging.
5. 2377 measures and removes remaining warm-page work after 2376. It cannot build another pool or cache. A warm complete continuation reads its snapshot and enriches only visible missing metadata. A warm incomplete continuation may retry missing providers under their own budgets without rebuilding successful membership. Measure this cost separately so a retry is not disguised as a cache hit.

Use separate test files for independent fixture work. Only one developer writes the route module or shared boundary at a time. Narrow each ticket's claimed files when the stage extraction determines them. Public help, MCP schema, examples and documentation follow the approved page interface. No source collection, benchmark harness or evidence checker is added for this design.

## Outside-in proof and stopping rule

Use the existing CLI or typed MCP entry point against isolated local provider replays and the existing cache isolation helper. Keep one owning regression for each claim:

- Plain PubMed superset: the entire recorded 192-PMID ESearch list is a subset of default union's pages. Hold resolution unresolved while the literal worker receives IDs. Interrupt enumeration, resume it on the next call, and require all 192 IDs before accepting completeness. Missing metadata and unfinished enrichment retain membership and honest status. A provider hard limit reports an explicit unmet floor.
- Union superset: run standalone components and union through the public boundary with their equal reservations and the recorded provider streams. Union contains every component result and provenance after completion. An interrupted component resumes without losing retained membership. Distinct PMIDs survive equal secondary identifiers. Explicit confirmed-only filtering remains intentional and named.
- Stable paging to the end: the existing offset interface walks one fixed active pool twice with varied page sizes, equal counts, no repeated PMID and no gaps, even while late sources settle. The approved continuation-token interface must prove the same whole-pool invariant across successor publication and concurrent sessions. Include the A/B route-support reversal, a late source retry, a conflicting configuration, an expired continuation and the settled final empty page. Expiry must fail explicitly instead of producing a different pool. Use events for concurrency, without fixed sleeps. Neither partial acquisition nor an unresolved interface decision counts as passing the full approved fixed-paging guarantee.
- Evidence ranking: the recorded query satisfies the stated first-page and first-ten expectations. Unsupported identity evidence stays unverified under the existing rules.
- Page-two time: record first and warm continuation time and provider requests through the public command on the configured build machine. The warm complete continuation performs no discovery or pool reranking. The routine regression asserts request reuse and output, without a wall-time failure threshold. The performance measurement compares the same query and cache state before and after; it does not require a new live 35-page walk.

Run affected checks through build jobs, then the existing integrated landing gate once. Do not weaken limits, rewrite expected results to fit a failure, or add a new gate. Reuse the recorded raw replies and the relevant accepted gate evidence. Research rechecks the final qualified candidate as already required by the release tickets. This design makes no claim about per-paper upstream causes that the bounded source traces cannot establish.
