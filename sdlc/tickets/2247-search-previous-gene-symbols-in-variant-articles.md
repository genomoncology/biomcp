---
priority: 2
---

# 2247 — Search previous gene symbols in variant articles

Status: OPEN.

Milestone: 0.9.4

## Outcome

PM ruling: For Boolean-capable Europe PMC and PubMed queries, group the requested symbol and every recorded MyGene alias in one correctly escaped OR clause joined with the variant term. Split only as provider query-length limits require. Keep existing caps for providers without Boolean support, using longest spelling first and source order for ties, and disclose their omissions. Preserve typed Gene and same-DNA identity rules, invocation deadline and provider accounting. No gene-specific exception or silent alias exclusion.

Variant article search queries the unambiguous queried gene's recorded MyGene aliases with their provenance, including source-provided former spellings without relabeling them as official previous symbols. It discloses current-spelling-only only when those MyGene facts are absent, unavailable or disabled.

- Changes: `src/entities/article/search.rs`, `src/entities/article/backends/provider_unit.rs`, `src/entities/gene.rs` `src/entities/article/variant_search.rs` `src/cli/variant/articles.rs` `tests/test_variant_article_gene_aliases.py` `tests/fixtures/article/gene-aliases/authored.json` `CHANGELOG.md` `sdlc/tickets/2247-search-previous-gene-symbols-in-variant-articles.md` `tests/test_variant_article_alias_coverage.py` `tests/test_variant_article_transcript_aliases.py`.


## Evidence

Reproduced on 0.9.3: the ABCC8 R1437Q search never uses the recorded alias SUR1. Europe PMC `SUR1 AND "R1437Q"` returns six PMIDs, including free PMC papers 10685980 and 9312191, against three for `ABCC8 AND "R1437Q"`.

## Keeps

Take alternative symbols only from a recorded source such as HGNC previous symbols, never a guess. Bound the number of extra queries within the deadline. Keep result order rules.

## Proof

A replay test shows the SUR1 query sent with its provenance and the extra papers returned, or the restriction stated in the output.

## Coordinator study brief

Restate the approved outcome, reproduce on current main before source edits, and report the exact command/output, proposed precise source/test paths and outside-in failing case. Reuse qualified unchanged current-main CLI when possible; all execution goes through the configured build job. Preserve exact raw source responses and invocation in the next numbered three-digit experiment on the executing host, with OWNER biomcp/2247. No live rerun if an existing deterministic replay already reproduces the same failure; any authored controls must be explicitly identified, not called captured source. Read source redistribution standards before publishing new captures. No product or test claim is granted yet. No source edits, WIP pushes, child agents, mail or status changes during this study. Stop after bounded reproduction and proposal; do not widen the ticket.

Depends on: 2246

## Bounded study and dependency queue

Study reused completed791replay and retained a separately started duplicate chain in792. All41providerplans usedABCC8; none usedSUR1. HGNC59records SUR/HRINS as previousofficialsymbols andSUR1 as alias, not previousofficialsymbol. No newclinicalidentityrule. PM corrected the implementation outcome: query source-backed aliases and previous symbols using the existing MyGene integration, label that provenance and remain within the invocation deadline; fall back to restriction disclosure only when those MyGene facts are absent or unavailable. Ordinary JSON/Markdown/debug/MCP use the same shared coverage value. No promise of extra paper retrieval. Source paths will be variant_search.rs,cli/variant/articles.rs,newfocusedtest,CHANGELOG; the exact claim is granted only after dependency candidate is adopted.

Implementation waits for the accepted2246candidate because these changes share source files. The coordinator will adopt that candidate; no independent overlapping source writer is authorized.

Keep: experiments/792-biomcp-2247-previous-symbols — Retain cited raw proof until release evidence is explicitly released.

## PM correction and next preparation

PM ruled that disclosure alone leaves the search defect. Reuse the existing MyGene alias-symbol integration (matching_canonical_alias_symbols in src/entities/gene.rs), not a new HGNCclient. Query recorded gene aliases/previous symbols for the queried gene with MyGene provenance, bounded by the deadline; disclose current-spelling-only only when MyGene has none or is unavailable. SUR1is an HGNC-recorded alias, not an HGNCpreviousofficialsymbol; do not relabel source facts. Preserve typed Gene annotation and same-DNA variant identity after retrieval; no gene-token identity fallback.

Read accepted-in-review2246candidate first to plan integration with per-form coverage. Before any source edit, propose exactadditionalfiles and outside-in failingcase for MyGene alias expansion/provenance and unavailable fallback. Reuse791replay; no repeated fullABCC8livechain. If no MyGeneABCC8record exists, one bounded existing-client/MyGene response capture is allowed, recorded privately792 with exactrequest/date/hash. Implementation still waits for2246acceptance and a declared exactclaim.

## Granted MyGene implementation boundary

After the accepted2246candidate is adopted by the coordinator, implement the six declared product/test paths. Add a small internal existing-client lookup using MyGeneClient::search, mygene_query_term and matching_canonical_alias_symbols; accept only one unambiguous Entrez-backed gene record and retain its alias-field provenance. Do not call aliases previous official symbols. Query the primary requested spelling first, then recorded gene-alias primary spellings before additional transcript/protein forms, within existing overall caps/deadline and identity reservation. Correct the planner boundary so SUR1R1437Q is not rewritten as ABCC8SUR1R1437Q. Keep2246per-form dispositions, and expose actual gene-symbol query coverage/omissions/source through shared JSON/debug/batch/MCP and Markdown values.

No newHGNCclient, typedGeneannotationrelaxation, gene-token identity fallback, same-DNA rule change or paper-recovery promise. MyGene absent/unavailable/disabled gets truthful restriction disclosure, not a broken command. One bounded ABCC8MyGene lookup capture through the existing integration is authorized to establish the actual source aliases (including whether SUR1 is present), retained privately792; do not repeat the article provider chain. Public controls stay explicitly authored with source provenance separated.

Owning outside-in regression: authored unambiguous ABCC8/MyGeneSUR1 response produces an actual correctly formed SUR1R1437Q provider query and source coverage, never ABCC8SUR1R1437Q. Cover absent/unavailable/cap and shared output surfaces, preserve2246codingformproof. Build/test only through configuredbuildjob; privatepatch/hashmanifest/finalblobmatch; noWIPpush, merges, ticket/status/mail or childagents. Exact additional paths require declaration before edits.

The coordinator adopted accepted2246source02c08e8fcfc480c5719a14978e41977628ca7653, including the strict original-key attribution repair. The declared MyGene implementation is now authorized on this dependency base. Every new Python binary-driven module declares the required module-level needs_binary marker.

Owning regression should also return a clearly authored SUR1-only article control when that alias is queried, preserving existing typed-gene/same-DNA identity classification. Do not claim recovery of any real extra paper without its recorded identity proof. The old broad Proof wording is interpreted under the later PM ruling: source-backed query expansion is required when facts exist; restriction is fallback only.

Fixture compatibility claim: extend the existing2246alias-coverage fixture with an authored empty MyGene query/base while preserving all its assertions. The existing transcript-alias fixture may explicitly disable the unrelated MyGene alias lookup, preserving its recorded bytes and all transcript/typed-gene identity assertions. These source-controlled fixtures must work in normal gates; a special job-only environment is insufficient. Report any other exact affected fixture path before editing it.

## Superseded query ordering decision

PM approved queried primary symbol first, followed by MyGene aliases in descending spelling length with source order retained for ties, within the existing query cap. Longer spellings are less ambiguous. Preserve original recorded aliases and disclose omissions; use no gene-specific exception. Typed Gene annotation and same-DNA variant identity remain required.

## Revised query planning decision

The recorded MyGene array proved that longest-first still omitted SUR1 under the combined spelling cap. PM rejected both eleven slots and another example-fitting sort. Boolean grouping removes the cause by covering every recorded gene alias with the same or fewer provider requests. Any observed noise requiring exclusions needs a PM decision; authored query proof does not establish recovered real papers.
