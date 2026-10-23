# Variant article acquisition, ranking and paging

## Outcome

For a gene and protein change, the default `variant articles` result contains the literal PubMed query's acquired PMIDs and every acquired component result. It ranks papers from their recorded evidence and pages one immutable ordered pool. Slow providers cannot consume another provider's work allowance. Every response names unfinished sources. This design covers 2373 through 2377 and implements the shape requested in PM message m4157.

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

Resolve the request under the existing identity rules, then start all applicable source workers together. Each provider owns its deadline and work allowance. Requests within one provider share its pacing and allowance. One provider cannot consume another's reservation. Preserve a finite invocation ceiling and the existing concurrency and resource limits. Divide the approved invocation work among providers before starting them instead of admitting providers in route order. The split and provider deadline values must live once in configuration or owning code. Removing a per-query result cut does not authorize unlimited HTTP work.

The literal PubMed worker sends the original gene and protein-change words through ESearch with relevance sort. Extend the internal ESearch parameter type to express sort while preserving existing callers' defaults. This worker acquires the entire PMID list through provider paging. It creates membership seeds before optional ESummary or abstract hydration. Missing summaries leave PMID seeds available with missing metadata status. They do not erase membership or manufacture identity evidence. The recorded 192-ID response requires no new live capture.

The plain PubMed query runs independently of strict Title/Abstract and alias expansion queries. It is an additional default-union source. It cannot be removed by an identity-dependent free-text fallback condition. PubTator and LitVar keep their applicable typed entity requirements. Empty or failed PubTator discovery cannot become a successful shared annotation result. Shared acquisition may be reused only when its query, identity and result coverage satisfy both consumers.

A worker retains every successfully acquired identifier before attempting more work. When its deadline, provider ceiling or allowance ends, it returns that membership and an unfinished outcome with the existing reason codes. An ESearch count beyond the provider's retrievable range remains explicitly incomplete. Neither the current provider ceiling nor finite budgets permit a universal promise to retrieve an arbitrarily large PubMed result. The 192-ID regression must retrieve all 192 under the unchanged resource envelope.

## Merge membership without loss

The pool is the set union of acquired component membership and the literal PubMed PMID set. Two rows with the same PMID produce one candidate with merged provenance. Two distinct nonempty PMIDs remain distinct even when a secondary identifier agrees. A row without a PMID may join a matching identified paper through existing secondary identifiers, provided the merge cannot bridge two distinct PMIDs. Missing metadata never decides membership.

Record each finding provider, route, query and native position using existing provenance. Repeated aliases and strict plus annotation routes from one provider do not count as independent agreement. Preserve received rows when subsequent work fails. Source status describes acquisition completeness separately from the number of stored candidates and the number on this page.

Union and its component strategies use the same component query definitions and source configuration. A standalone component may spend its own unused allowance differently, so strict cross-invocation superset claims require matching acquired source outcomes. The replay contract compares union and components against the same recorded source streams. If a component receives more IDs because union exhausts an allowance, union must report the missing work; it cannot claim a complete superset.

## Rank from carried evidence

Use one deterministic formula over the snapshot's captured metadata, validated annotations and provenance. The proposed score is:

`score = 8*T + 4*A + 8*C + 2*min(I, 4)`

- `T` is one when the existing lexical matcher finds the requested gene and a supported requested variant spelling in the title; otherwise it is zero.
- `A` is one when the existing matcher finds that evidence in the abstract, or the paper carries existing validated PubTator or LitVar variant linkage; otherwise it is zero.
- `C` is one when the existing compatible MyVariant record supplies the PMID through its CIViC curated citation field; otherwise it is zero. A generic citation or a provider's guessed variant identity does not qualify.
- `I` counts distinct finding-source families. Count a provider once across all its routes and aliases. Count PubTator and LitVar together where they share annotation origin. Credit CIViC rather than its MyVariant transport. This measures retrieval agreement, not independent clinical confirmation.

Break equal scores by reciprocal native rank summed once per source family, then by normalized stable identifier. Reuse the existing reciprocal-rank denominator. No PMID, paper title, publication year or manually assigned landmark label enters the algorithm. The score gives curated variant citations enough weight to compete with title matches while giving a direct title match more weight than abstract evidence alone.

Text relevance and identity confirmation remain separate. Preserve 2139's typed Gene/Variant linkage, expected-paper checks, contradictions and missing-typed-Gene reason. Preserve 2345's transcript and assembly context and unconfirmed equivalence behavior. No new gene-token fallback is allowed. Optional verification may annotate the page, but cannot rerank the immutable snapshot. A confirmed-only snapshot is separate, uses existing confirmation rules, and reports unfinished verification instead of claiming its verified subset covers all candidates.

The recorded AKT1 E17K replay must put 33123537 and 17611497 on the first 50-row page and match or exceed the recorded PubMed first-ten variant-evidence count. These are fixture expectations, not algorithm inputs. The recorded evidence must support each ranking input. If this formula cannot meet those expectations, return the discrepancy for PM consideration with the actual evidence; do not adjust a paper-specific exception or invent a clinical classification.

## Cache an immutable query snapshot

A snapshot contains the merged candidate membership, ranking inputs, final ordered identifiers, source outcomes, creation and expiry times, and the versions of acquisition, identity and ranking rules. It preserves unverified reasons and provenance. Use one atomic SQLite publication so readers see a complete old or new record. Reuse the cache's size accounting, cleanup, private storage and failure handling. A failed cache write must not erase the response's acquired rows.

The lookup identity contains the normalized requested identity and literal query, strategy, verification and confirmed-only options, source enablement, provider endpoint configuration, alias and genome configuration, and query/verifier/ranking versions. Offset and page size do not alter pool identity. The page token binds the snapshot identifier, query identity, ordering version and next offset. The returned page limit only controls slicing. Reject a token with conflicting request options.

Credentials never enter lookup keys, token material, stored configuration or logs, either directly or through a hash. Provider endpoints must be normalized without credentials. A credential-bearing custom endpoint whose semantics cannot be represented safely disables shared snapshot reuse. A source configuration change cannot reuse verification from the prior configuration. Honor cache bypass, clearing and expiry. Bound snapshot freshness by the existing source-cache freshness policy; do not let infinite HTTP cache mode silently make query snapshots immortal.

## Retries, late arrivals and page binding

A timed-out or unavailable source is retried on the next matching call within its own allowance. Keep successful source outcomes available so the retry does not rerun every source. A cached incomplete response never suppresses that retry. The response names missing sources and distinguishes the snapshot's membership from work acquired by a retry.

A continuation's retry allowance must fit its warm-page budget. Do not attach a full cold-discovery deadline to a cached page, launch detached retry work from the CLI, or call a slow retry a fast cache hit. If a source cannot settle within that allowance, keep its missing status and let the next call retry again. Measure complete and incomplete warm pages separately. Source retry timing must not change the page's membership.

Late rows produce a successor snapshot. They never insert into, rerank or replace the snapshot named by an existing continuation token. New searches can select the successor. Existing continuations keep reading their original ordered pool until it expires or is evicted. They then receive an explicit expired or unavailable continuation error and restart from offset zero. They never silently rebuild at a nonzero offset. Partial snapshots retain `complete=false`; reaching their last page means the stored pool ended, not that all providers completed. Preserve 2365's settled `has_more=false` behavior at that boundary.

PM decision needed: approve an optional CLI page-token argument and matching typed MCP input, using the existing `pagination.next_page_token` output to carry the bound continuation. This is the recommended way to preserve pages while retries publish successors. For compatibility, requests with offsets but no token reuse one active query snapshot during its lifetime. They cannot distinguish concurrent searches or safely continue across a refresh. Document that boundary and refuse a nonzero-offset rebuild when the bound cached pool is missing. Do not claim token-level guarantees for legacy offsets.

PM decision needed: approve the proposed score and source-family definition above. The requested evidence factors and named-paper expectations are approved requirements; these weights and agreement groups are the concrete proposed policy. The implementation starts only after PM consideration of this design and its fresh review.

## Ticket boundaries and build order

The accepted native-gate batch lands independently. The five changes then share this design and one eventual integrated review boundary.

1. 2373 owns literal PubMed membership acquisition, explicit relevance sort and its replay fixture. Its product claim covers `variant_search.rs`, the new `sources` stage and the PubMed parameter implementation. Its outside-in test proves all 192 acquired IDs survive complete paging, including a missing-summary case.
2. 2374 owns pure merge and retained component membership. Its developer can prepare its separate outside-in regression concurrently with 2373. It takes the shared source-file writer slot after 2373 commits. It preserves the accepted 2367 retention fix. Its test proves union contains component membership and reproduces A/B becoming B/B before the integrated paging fix.
3. 2375 owns the ranking stage after membership is settled. It reuses recorded metadata and supported annotations. Its separate replay test checks ordering and the two page-one expectations without naming either PMID in product code.
4. 2376 owns snapshot construction, cache integration, stage orchestration and page binding after the membership and ranking interfaces settle. It replaces the long orchestration body and removes dead plan bookkeeping. Its proof covers late arrival, expiry, configuration separation and explicit filters at the same public boundary as paging.
5. 2377 measures and removes remaining warm-page work after 2376. It cannot build another pool or cache. A warm complete continuation reads its snapshot and enriches only visible missing metadata. A warm incomplete continuation may retry missing providers under their own budgets without rebuilding successful membership. Measure this cost separately so a retry is not disguised as a cache hit.

Use separate test files for independent fixture work. Only one developer writes the route module or shared boundary at a time. Narrow each ticket's claimed files when the stage extraction determines them. Public help, MCP schema, examples and documentation follow the approved page interface. No source collection, benchmark harness or evidence checker is added for this design.

## Outside-in proof and stopping rule

Use the existing CLI or typed MCP entry point against isolated local provider replays and the existing cache isolation helper. Keep one owning regression for each claim:

- Plain PubMed superset: the recorded 192-PMID ESearch list is a subset of default union's pages. Missing metadata and unfinished enrichment retain membership and honest status.
- Union superset: the same recorded component streams yield a union containing each component's acquired IDs and provenance. Distinct PMIDs survive equal secondary identifiers. Explicit confirmed-only filtering remains intentional and named.
- Stable paging to the end: walk the same snapshot twice with varied page sizes, equal counts, no repeated PMID and no gaps. Include the A/B route-support reversal, a late source retry, a conflicting configuration, an expired token and the settled final empty page. Use events for concurrency, without fixed sleeps.
- Evidence ranking: the recorded query satisfies the stated first-page and first-ten expectations. Unsupported identity evidence stays unverified under the existing rules.
- Page-two time: record first and warm continuation time and provider requests through the public command on the configured build machine. The warm complete continuation performs no discovery or pool reranking. The routine regression asserts request reuse and output, without a wall-time failure threshold. The performance measurement compares the same query and cache state before and after; it does not require a new live 35-page walk.

Run affected checks through build jobs, then the existing integrated landing gate once. Do not weaken limits, rewrite expected results to fit a failure, or add a new gate. Reuse the recorded raw replies and the relevant accepted gate evidence. Research rechecks the final qualified candidate as already required by the release tickets. This design makes no claim about per-paper upstream causes that the bounded source traces cannot establish.
