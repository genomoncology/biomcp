# Source API Feasibility — Ticket 369

Date: 2026-05-22  
Team: biomcp  
Ticket: 369 — Score variant-agent source APIs

## Question

Which requested variant-agent source APIs are worth exposing in BioMCP as read-only public service/API proxies, versus leaving to the agent or rejecting, given current BioMCP capabilities and product boundaries?

## Scope and Full-Scale Definition

This exploit did not build runtime connectors because the ticket explicitly marks connector/CLI/MCP behavior changes out of scope and requires no repo code modifications. Full scale for this spike means the complete decision artifact:

- all 21 candidate sources scored against all 9 requested criteria;
- current BioMCP capability/gap evidence by repo path;
- clear BioMCP-vs-agent ownership boundaries;
- auth/terms and operational-risk flags;
- 3–5 ordered follow-up ticket recommendations;
- reproducible JSON results and regression-control probes in persistent paths.

Persistent artifact directory: `~/workspace/planning/biomcp/artifacts/369-score-variant-agent-source-apis/`.

## Current BioMCP Capability / Gap Evidence

### Variant and evidence surfaces

- `src/cli/variant/mod.rs`, `src/cli/variant/dispatch.rs`: public CLI exposes `search variant`, `get variant`, `variant trials`, `variant articles`, and token-gated `variant oncokb`.
- `src/entities/variant/resolution.rs`: exact lookup accepts rsID, genomic HGVS `chrN:g.posRef>Alt`, and gene+protein. Transcript HGVS such as `NM_000248.3:c.135del` is deliberately unsupported today.
- `src/entities/variant/get.rs`, `src/entities/variant/search/mod.rs`, `src/sources/myvariant.rs`: MyVariant.info is the default variant identity/search/annotation layer, including MyVariant-carried ClinVar, gnomAD, COSMIC, CIViC, dbNSFP, and prediction fields.
- `src/sources/gnomad.rs`: direct gnomAD support is gene constraint only, not exact allele lookup.
- `src/sources/clingen.rs`: direct ClinGen support is gene validity/dosage only, not ClinGen Allele Registry / CAid resolution.
- `src/sources/civic.rs`, `src/sources/cbioportal.rs`, `src/sources/oncokb.rs`: selected curated sources already exist as opt-in/direct helpers; OncoKB is token-gated.
- `src/render/provenance.rs`: section-level provenance exists; population is labelled `gnomAD via MyVariant.info`.

### Article, fulltext, and literature surfaces

- `src/cli/article/mod.rs`, `src/cli/article/dispatch.rs`: public CLI exposes article search/get/entities/batch/citations/references/recommendations.
- `src/entities/article/search.rs`, `backends.rs`, `enrichment.rs`: PubTator3, Europe PMC, PubMed, optional Semantic Scholar, and keyword-gated LitSense2 are federated with source status/ranking metadata.
- `src/entities/article/detail.rs`: accepts PMID, PMCID, DOI; DOI resolution currently depends on Europe PMC.
- `src/entities/article/fulltext.rs`: existing full-text ladder uses NCBI ID Converter, Europe PMC PMC XML, NCBI EFetch PMC XML, PMC OA archive XML, Europe PMC MED XML, PMC HTML, and optional Semantic Scholar PDF only with `--pdf`.
- `src/sources/{pubmed,europepmc,pubtator,litsense2,semantic_scholar,ncbi_idconv,ncbi_efetch,pmc_oa}.rs`: current article source clients.
- Gap: no Crossref, OpenAlex, or Unpaywall source client and no structured standalone `article access` status aggregator.

### Capability/list surfaces

- `src/cli/list/{mod.rs,literature.rs,molecular.rs}`: prose command-reference pages; JSON exposes command lists, not service/input capability metadata.
- `src/cli/health/catalog.rs`: source readiness/auth catalog, not a machine-readable supported-input/status capability registry.

## BioMCP vs Agent Boundaries

| Request | Owner/classification | Rationale |
|---|---|---|
| Messy report/entity cleanup | agent-owned | Agents parse prose, decide aliases, and choose retry strategy. |
| Regex/pattern validation | BioMCP input contract | BioMCP should validate supported service input shapes and return guardrail errors. |
| Biological normalization/equivalence | BioMCP only through upstream services | Proxy Mutalyzer/VariantValidator/SPDI/ClinGen and preserve source-specific outputs; do not invent equivalence locally. |
| Source lookup/provenance/status | BioMCP | This is the core read-only public-service proxy job. |
| Local PDF/Word/Markdown search | agent-owned | BioMCP can retrieve allowed metadata/fulltext; agents use local tools for local document search. |
| Classification/oncogenicity/actionability | agent-owned | BioMCP returns source records and provenance, not clinical interpretation. |
| COSMIC direct integration | reject/default-exclude | Planning excludes direct COSMIC because of licensing; allow only indirect aggregator provenance with caution. |

## Source-by-Source Feasibility Matrix

Scores are 1–5 where 5 is best/lower-risk for BioMCP. Overall is the mean of: auth/terms openness, official/stable machine API, accepted input forms, failure/status semantics, provenance quality, rate-limit/operational risk, BioMCP fit, clinical/legal safety, and maintenance safety.

| Source | Area | Current BioMCP | Classification | Score | Auth/terms flag | First-slice implication |
|---|---|---|---:|---:|---|---|
| Mutalyzer | normalization | absent | good BioMCP proxy candidate | 4.44 | low/open | Best first slice for transcript HGVS validation/protein consequence. MITF probe returned normalized cDNA and p.(Asn46ThrfsTer4). |
| VariantValidator | normalization | absent | good BioMCP proxy candidate | 4.78 | low/open | Excellent for transcript-version warnings and genomic loci; ERBB2 and MITF probes returned explicit TranscriptVersionWarning. |
| NCBI Variation/SPDI | normalization | absent | possible but gated | 4.22 | low/open | Useful exact-allele primitive, but requires agent or upstream service to supply sequence accession/position/ref/alt. Probe surfaced explicit reference mismatch warning. |
| ClinGen Allele Registry | normalization | absent for alleles; gene ClinGen exists | possible but gated | 4.11 | low/open | Strong identity/provenance value, but probe returned blank-node @id rather than a stable CAid for MITF transcript HGVS; needs more endpoint/terms review. |
| MyVariant.info | variant annotation/normalization fallback | core get/search variant source | good BioMCP proxy candidate | 4.56 | low/open | Already integrated; best to deepen explicit annotate/provenance/status rather than duplicate. Good fallback for MYD88 and KLHL6; provenance must preserve upstream source licensing. |
| Ensembl VEP REST | annotation/normalization | absent | possible but gated | 4.22 | low/open | Valuable when genomic coordinate is known; MITF RefSeq transcript HGVS probe failed with clear 400, so not the first transcript-normalization slice. |
| gnomAD direct API | population | gene constraint only; variant population via MyVariant | possible but gated | 3.89 | review | High value as exact allele population proxy, but GraphQL contract/versioning and coordinate normalization must be pinned. Probe returned explicit Variant not found for guessed MYD88 allel... |
| MyVariant.info gnomAD fallback | population | variant population section | good BioMCP proxy candidate | 4.44 | low/open | Already ships useful cached gnomAD fields; should remain labeled fallback/annotation, not absence proof. |
| PubMed/PMC/NCBI E-utilities | literature/access | PubMed search/get; EFetch/ID Converter/PMC OA fulltext rungs | good BioMCP proxy candidate | 4.67 | low/open | Already strong. Add access-status wrapper rather than new fulltext machinery. |
| Europe PMC | literature/access | article search/get/fulltext metadata | good BioMCP proxy candidate | 4.89 | low/open | Already integrated and good for metadata/open flags/fulltext XML; extend status fields if needed. |
| PubTator3 | literature/text-mining | article search and annotations | good BioMCP proxy candidate | 4.67 | low/open | Already integrated; expose as service-provenanced annotations/matches, not interpretation. |
| LitSense2 | literature/text-mining | keyword-gated article search | good BioMCP proxy candidate | 4.33 | low/open | Excellent service-provided snippet/match proxy; KLHL6 L65P probe returned PMID 29695787 among 100 sentence hits. |
| Semantic Scholar | literature/graph/access metadata | optional article search enrichment, TLDR, citations, references, recommendations, PDF metadata | possible but gated | 3.89 | gated/high | Already useful but terms/rate limits make it optional. Keep source-status and token guidance; don't make it required for access decisions. |
| Crossref | DOI/conference metadata | absent | good BioMCP proxy candidate | 4.89 | low/open | ASCO DOI probe succeeded and found title/publisher URL where current BioMCP article get failed. Strong first-slice DOI metadata proxy. |
| OpenAlex | DOI/OA/citation metadata | absent | good BioMCP proxy candidate | 4.78 | low/open | ASCO and CCR DOI probes succeeded with OA status closed and landing pages. Good complement to Crossref/Unpaywall. |
| Unpaywall | access status | absent | good BioMCP proxy candidate | 4.56 | review | CCR DOI probe returned closed status and null best OA location. Strong access-status component; requires product email/contact policy. |
| CIViC | curated variant evidence | direct GraphQL sections for variant/gene/drug/disease plus MyVariant cached fields | good BioMCP proxy candidate | 4.44 | low/open | Already integrated and open. Keep opt-in and source-provenanced; agent owns interpretation/evidence weighting. |
| ClinVar | curated variant evidence | indirect via MyVariant | possible but gated | 4.22 | low/open | Public and clinically important, but direct NCBI ClinVar integration is a separate source surface. Current indirect path is useful; direct ClinVar can wait behind normalization/access work. |
| cBioPortal | curated/cohort variant context | variant gene-level mutation summary and study surfaces | possible but gated | 3.67 | review | Already present, but variant-specific frequency by alteration is more study-dependent and terms-sensitive. Keep supplemental, not classification-moving. |
| OncoKB | curated oncogenic/actionability | explicit token-gated variant helper | possible but gated | 3.44 | gated/high | Keep explicit and token-gated. Lead-only metadata mode may be acceptable, but proprietary interpretation/actionability summaries are terms-sensitive. |
| COSMIC | curated somatic variant evidence | indirect MyVariant payload only | reject/default-exclude | 1.78 | gated/high | Direct integration remains excluded by planning docs and license risk. Do not add a connector; preserve indirect-only caution. |

## Scored Decision Summary

Good BioMCP proxy candidates:
- Variant normalization/fallback: VariantValidator, Mutalyzer, MyVariant.info, MyVariant-backed gnomAD fallback.
- Literature/access: PubMed/PMC/NCBI E-utilities, Europe PMC, PubTator3, LitSense2, Crossref, OpenAlex, Unpaywall.
- Curated/open evidence: CIViC.

Possible but gated:
- NCBI Variation/SPDI and ClinGen Allele Registry: valuable identity primitives, but best after input/capability discovery and endpoint-shape review.
- Ensembl VEP REST: useful after genomic coordinates are known; not a first transcript-HGVS normalizer.
- gnomAD direct API: valuable for exact allele population, but must be behind normalized coordinate/build/allele contracts.
- Semantic Scholar: useful optional graph/access metadata; keep explicit source status and rate/auth behavior.
- ClinVar direct, cBioPortal, OncoKB: useful but not first-slice; OncoKB remains token/terms gated and interpretation-sensitive.

Rejected/default-excluded:
- COSMIC direct integration remains out of bounds because of license risk. Keep only indirect provenance from aggregator payloads, clearly labelled.

## Explore/Exploit Probe Results

Regression-control and full-scale JSON files are stored in this artifact directory and in `architecture/experiments/369-score-variant-agent-source-apis/results/`:

- `current_biomcp_baseline.json`
- `external_api_probes.json`
- `source_feasibility_matrix.json`
- `exploit_current_biomcp_baseline.json`
- `exploit_external_api_probes.json`
- `exploit_source_feasibility_matrix.json`
- `exploit_regression_control_summary.json`

Key exploit confirmation:
- Candidate source count remained 21.
- Classification counts remained 12 good BioMCP proxy candidates, 8 possible-but-gated, and 1 reject/default-exclude.
- Boundary classification remained complete.
- Follow-up recommendation count remained 4.
- Current BioMCP CLI regression-control probes all preserved expected success/failure status and were faster than explore in this run.
- External public API probes preserved response status/shape. Several public-service latencies shifted by more than 3%; those are recorded as network-noise carveouts because no implementation code changed and classification/output contracts stayed stable.

## Recommended Follow-Up Tickets

1. **Add service capability discovery for source/input/status metadata.**  
   First slice: `biomcp list services --json` plus selected `service <name> capabilities --json` for existing article/variant sources. Include auth mode, supported/unsupported input forms, example queries, status taxonomy, and provenance label.

2. **Add public variant normalization proxies.**  
   First slice: Mutalyzer + VariantValidator for transcript HGVS with typed statuses: `success`, `invalid_input`, `unsupported_notation`, `not_found`, and `service_error`. Preserve source-specific warnings and do not collapse disagreements into one asserted truth.

3. **Add DOI/conference metadata and article access-status proxies.**  
   First slice: Crossref + OpenAlex DOI lookup for conference abstracts and Unpaywall/OpenAlex access status. Use the ASCO DOI `10.1200/JCO.2018.36.15_suppl.e24316` as canary because current BioMCP misses it while Crossref/OpenAlex find metadata.

4. **Add direct exact-allele population proxy only after normalization exists.**  
   First slice: gnomAD exact variant ID lookup with explicit `found`, `not_found`, `not_queryable`, and `service_error`; keep MyVariant gnomAD fields as labelled fallback/annotation rather than absence proof.

## Contract Numbers for Downstream Work

- Matrix scope: 21 candidate sources, 9 scoring criteria, 6 core boundary rows plus direct COSMIC exclusion, and 4 follow-up tickets.
- Good-candidate priority scores: Europe PMC 4.89, Crossref 4.89, VariantValidator 4.78, OpenAlex 4.78, PubMed/PMC/NCBI E-utilities 4.67, PubTator3 4.67, MyVariant.info 4.56, Unpaywall 4.56, Mutalyzer 4.44, MyVariant gnomAD fallback 4.44, CIViC 4.44, LitSense2 4.33.
- Gate/reject scores: NCBI Variation/SPDI 4.22, Ensembl VEP REST 4.22, ClinVar 4.22, ClinGen Allele Registry 4.11, gnomAD direct API 3.89, Semantic Scholar 3.89, cBioPortal 3.67, OncoKB 3.44, COSMIC 1.78.
- Current BioMCP canaries: transcript HGVS `NM_000248.3:c.135del` remains unsupported by current `get variant`; ASCO DOI remains not found by current `get article`; KLHL6 population remains available through MyVariant-carried gnomAD with `gnomad_af=0`.

## Conclusion

Promote the follow-up set above. The highest-leverage first build is service capability discovery, then public transcript-HGVS normalization, then DOI/access metadata. Direct exact-allele gnomAD should wait until normalized coordinate/build handling exists. Workflow interpretation, local document search, report cleanup, and direct COSMIC integration remain outside BioMCP.
