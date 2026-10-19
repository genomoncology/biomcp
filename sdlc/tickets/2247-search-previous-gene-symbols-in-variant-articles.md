---
priority: 2
---

# 2247 — Search previous gene symbols in variant articles

Status: OPEN.

Milestone: 0.9.4

## Outcome

Variant article search also queries a gene's previous official symbols and well-recorded aliases, labeled with where each came from, or states plainly that it searched only the current symbol.

## Evidence

Reproduced on 0.9.3: the ABCC8 R1437Q search never uses the older name SUR1. Europe PMC `SUR1 AND "R1437Q"` returns six PMIDs, including free PMC papers 10685980 and 9312191, against three for `ABCC8 AND "R1437Q"`.

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
