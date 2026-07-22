# Variant Queries

Variant workflows need to balance exact identity with search-time normalization.
These canaries keep the stable column contracts, normalization rules, and
opt-in clinical sections without depending on brittle row counts.

## Deterministic Source Contracts

Ticket 376 moves routine variant-source proof from live/cache-backed MyVariant
and normalization-service canaries to source-local request-plan and
fixture-backed contracts. Any irreducible public availability check belongs in
an explicit release/live-smoke lane; routine specs must instead prove MyVariant
search/get request shape, identifier normalization, and Mutalyzer/
VariantValidator status mapping locally.

## Deterministic Renderer Envelope Contracts

Ticket 377 moves routine variant renderer/envelope proof into fixture-result
contracts. The deterministic tests should cover variant search JSON
`_meta.next_commands`, markdown related anchors, and normalization JSON/markdown
per-service status, warnings, and genomic-description rendering without live
MyVariant, Mutalyzer, or VariantValidator calls.

Ticket 456 keeps the default variant card cheap while making CIViC actionability
discoverable: a pure renderer test proves the cached-CIViC pointer, the bare
fallback pointer, and the CIViC-section currency caveat without a live source
call.

```bash
grep -h -F \
  -e 'Therapeutic evidence: 1 CIViC predictive item(s) / 0 assertion(s)' \
  -e 'Therapeutic evidence: see `get variant \"chr1:g.101A>T\" civic`' \
  -e 'Caveat: CIViC evidence may lag current standard of care' \
  ../../src/render/markdown/variant/tests.rs ../../templates/variant.md.j2 \
  | mustmatch like 'Therapeutic evidence: 1 CIViC predictive item(s) / 0 assertion(s)
Therapeutic evidence: see `get variant \"chr1:g.101A>T\" civic`
Caveat: CIViC evidence may lag current standard of care'
```

## Finite score thresholds

<!-- mustmatch-lint: skip -->

GERP and CADD thresholds must be finite numbers. Non-finite values are rejected
as invalid arguments instead of being sent upstream and misreported as a
confident empty result.

| str:flag | str:value | str:label |
|---|---|---|
| --gerp-min | NaN | GERP NaN |
| --gerp-min | +inf | GERP positive infinity |
| --gerp-min | -inf | GERP negative infinity |
| --gerp-min | 1e309 | GERP overflow |
| --min-cadd | NaN | CADD NaN |
| --min-cadd | +inf | CADD positive infinity |
| --min-cadd | -inf | CADD negative infinity |
| --min-cadd | 1e309 | CADD overflow |

```bash run id=non-finite-threshold exit=2 each_row="Finite score thresholds"
biomcp --json search variant --gene BRAF {{flag}}={{value}} --limit 1
```

```json expect=non-finite-threshold contains each_row="Finite score thresholds"
{
  "error": {
    "code": "invalid_argument"
  }
}
```

## Variant filter vocabularies

<!-- mustmatch-lint: skip -->

Consequence, review-status, and field-presence filters use documented vocabularies. Unknown
values are rejected locally instead of being sent upstream and reported as a
successful empty search.

| str:flag | str:value | str:label |
|---|---|---|
| --consequence | protein_altering_variant | unsupported consequence |
| --consequence | missense_variant* | malformed consequence |
| --consequence | '' | empty consequence |
| --review-status | bogus | unknown review status |
| --review-status | 2* | malformed review status |
| --review-status | '' | empty review status |
| --has | not_a_real_field_zzz | unknown required field |
| --has | revel:* | malformed required field |
| --has | '' | empty required field |
| --missing | not_a_real_field_zzz | unknown missing field |
| --missing | revel:* | malformed missing field |
| --missing | '' | empty missing field |

```bash run id=invalid-variant-filter exit=2 each_row="Variant filter vocabularies"
biomcp --json search variant --gene BRAF {{flag}} {{value}} --limit 1
```

```json expect=invalid-variant-filter contains each_row="Variant filter vocabularies"
{
  "error": {
    "code": "invalid_argument"
  }
}
```

## Coordinate Genome-Build Context

<!-- mustmatch-lint: skip -->

Variant and gene coordinate strings are source-derived genomic positions, so
consumer-facing output must say which genome build those coordinates use rather
than emitting a bare chromosome/start/end string. The deterministic renderer
contract covers the markdown and JSON envelopes without depending on live
MyVariant or MyGene responses.

## Gene-Scoped Variant Search

Gene-first search should still return the canonical variant identity columns and
preserve the BRAF V600E row as a recognizable anchor.

## Search Table Contract

The JSON path should keep the same follow-up shape so agents can pivot into the
default card without scraping markdown helper text.

## Protein-Filter Narrowing

Long-form protein filters should normalize to the same compact spelling that the
short-form query uses, rather than leaking a second variant identifier shape.

## Strict exact variant identity

Exact protein search keeps the supplied identity separate from its normalized
alias and checks the source's returned identity before including a row. Here the
healthy fixture offers only BRCA1 residue 16, so a request for residue 1783 is
explicitly unresolved instead of being relabeled as a match.

```bash
biomcp --json search variant -g BRCA1 --hgvsp p.Met1783Ile --limit 5 \
  | jq '{requested_gene: .requested_variant.gene, supplied_protein: .requested_variant.protein_change, normalized_proteins: .resolution.normalized_aliases.protein_changes, status: .resolution.status, exhaustive: .resolution.exhaustive, retained: (.results | length), filtered_total: .pagination.total, has_more: .pagination.has_more}' \
  | mustmatch like '{"requested_gene":"BRCA1","supplied_protein":"p.Met1783Ile","normalized_proteins":["M1783I"],"status":"unresolved","exhaustive":true,"retained":0,"filtered_total":0,"has_more":false}'
```

The same source response does contain residue 16. Asking for that identity keeps
its source row and records the source alias that proved the match, rather than
dropping every exact result indiscriminately.

```bash
biomcp --json search variant -g BRCA1 --hgvsp p.Met16Ile --limit 5 \
  | jq '{supplied_protein: .requested_variant.protein_change, normalized_proteins: .resolution.normalized_aliases.protein_changes, status: .resolution.status, exhaustive: .resolution.exhaustive, retained: (.results | length), matched_alias: .results[0].matched_alias, source_has_supplied_alias: (.results[0].source_identity.protein_changes | index("p.Met16Ile") != null), source_has_short_alias: (.results[0].source_identity.protein_changes | index("p.M16I") != null), filtered_total: .pagination.total, has_more: .pagination.has_more}' \
  | mustmatch like '{"supplied_protein":"p.Met16Ile","normalized_proteins":["M16I"],"status":"resolved","exhaustive":true,"retained":1,"matched_alias":"p.Met16Ile","source_has_supplied_alias":true,"source_has_short_alias":true,"filtered_total":1,"has_more":false}'
```

## Residue-Alias Search

Residue aliases should stay on the typed variant path instead of falling
through to free-text or disease-style fallback behavior.

## Clinical Significance

ClinVar remains an opt-in deepen path. The section should keep the human heading
and a compact JSON disease anchor without bloating the default card.

## Population Frequency

Population frequency also stays opt-in. The markdown and JSON views should keep
the same compact gnomAD frequency story.

## Variant Follow-Ups

The default card should still advertise typed follow-ups for downstream trial
and article pivots even when those surfaces are covered elsewhere.

## Structure Helper Discoverability

The structure helper is an opt-in variant pivot for residue, domain, PDB,
AlphaFold, and hotspot context. It should be visible in help and structured
command listings before users try a live source join.

```bash
../../tools/biomcp-ci variant structure --help | mustmatch like 'biomcp variant structure "BRAF V600E"
residue
domain
PDB
AlphaFold
Cancerhotspots'
```

```bash
../../tools/biomcp-ci --json list variant | jq -r '.commands[]' | mustmatch like 'variant structure <variant>'
```

## Variant Structure Blog Walkthrough

The public blog should teach the shipped variant-structure workflow as a real
BRAF V600E command sequence, link readers to the reference how-to, and be wired
into the MkDocs Blog nav.

```bash
grep -h -F \
  -e 'blog/variant-structure-in-commands.md' \
  -e '**TL;DR:**' \
  -e 'biomcp get variant "BRAF V600E"' \
  -e 'biomcp variant structure "BRAF V600E"' \
  -e '../how-to/annotate-variant-structure.md' \
  -e 'InterPro' \
  -e 'AlphaFold' \
  -e 'Cancerhotspots' \
  -e 'biomcp variant articles "BRAF V600E"' \
  -e '## Try it' \
  ../../mkdocs.yml ../../docs/blog/variant-structure-in-commands.md | mustmatch like 'blog/variant-structure-in-commands.md
**TL;DR:**
biomcp get variant "BRAF V600E"
biomcp variant structure "BRAF V600E"
../how-to/annotate-variant-structure.md
InterPro
AlphaFold
Cancerhotspots
biomcp variant articles "BRAF V600E"
## Try it'
```

## Variant Article Entity Recall

The default union remains honest when strict resolution finds no allele and
BioMCP uses labeled best-effort text. This healthy fixture serves the MYD88
paper only for the non-exact fallback path; exact annotation behavior is shown
with the diagnostic strategy below.

```bash
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. myd88 | mustmatch like '## MYD88 S219C fallback
best-effort free-text fallback
24534189'
```

```bash
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. myd88-json | mustmatch like 'JSON fallback path preserved
24534189'
```

## Variant Article Routes Are Unioned Before Pagination

<!-- mustmatch-lint: skip -->

The default literature strategy preserves every compatible annotation entity,
resolved protein/coding/genomic alias, and source-backed citation before it
deduplicates, ranks, and applies the public limit. A paper reached by two routes
remains one row with associated provenance, and offset pages retain global rank.

```bash run id=variant-article-union exit=0
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. union-json
```

```json expect=variant-article-union contains
{
  "strategy": "union",
  "requested_gene": "BRAF",
  "supplied_protein": "p.V600E",
  "resolution": "resolved",
  "complete": true,
  "pmids": ["6010001", "6010002", "6010003", "6010004", "6010005", "6010006", "6010007", "6010008"],
  "all_rows_keep_requested_variant": true,
  "alias_matches": [
    {"pmid": "6010002", "matched_aliases": ["BRAF p.Val600Glu"]},
    {"pmid": "6010005", "matched_aliases": ["BRAF c.1799T>A"]},
    {"pmid": "6010006", "matched_aliases": ["chr7:g.140453136A>T"]}
  ],
  "shared_provenance": [
    {"route": "exact_lexical", "source": "pubtator", "matched_alias": "BRAF p.V600E"},
    {"route": "pubtator_variant", "source": "pubtator", "matched_alias": "BRAF p.V600E"}
  ],
  "citation_provenance": [
    {"route": "source_citation", "source": "civic", "matched_alias": "BRAF p.V600E"}
  ],
  "pubmed_provenance": [
    {"route": "exact_lexical", "source": "pubmed", "matched_alias": "BRAF p.Val600Glu"}
  ],
  "annotation_pmids": ["6010001", "6010003", "6010007"],
  "page_matches_full_slice": true,
  "page_ranks": [3, 4],
  "pagination": {"offset": 2, "limit": 2, "returned": 2, "total": 8, "has_more": true},
  "truncated": true,
  "source_status": [
    {"route": "exact_lexical", "source": "pubmed", "status": "ok"},
    {"route": "pubtator_variant", "source": "pubtator", "status": "ok"},
    {"route": "source_citation", "source": "myvariant", "status": "ok"}
  ]
}
```

## Verified variant-article identity does not promote retrieval aliases

<!-- mustmatch-lint: skip -->

Retrieval aliases explain why a paper was found; they are not observations from
that paper. With identity verification enabled, the local fixture preserves
alias-only collisions as unverified, contradictory, or conflicting, and emits
the captured evidence needed to audit a confirmed result. `--confirmed-only`
filters that verified pool before ranking and pagination, so earlier collisions
cannot hide the confirmed paper.

```bash run id=variant-article-identity-verification exit=0
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. identity-verification-json
```

```json expect=variant-article-identity-verification contains
{
  "normal_statuses": [
    {"pmid": "6010001", "status": "confirmed"},
    {"pmid": "6010002", "status": "unverified"},
    {"pmid": "6010005", "status": "contradictory"},
    {"pmid": "6010006", "status": "conflicting"}
  ],
  "alias_only_candidates_never_confirmed": true,
  "confirmed_observation_is_auditable": true,
  "confirmed_only_keeps_the_confirmed_result": true,
  "confirmed_only_excludes_nonconfirmations": true,
  "debug_plan_records_verification_artifact": true
}
```

## Pagination Limits Metadata Enrichment

<!-- mustmatch-lint: skip -->

Variant-article pagination selects the visible ranked page before optional
metadata enrichment. A small page does not spend source lookups enriching a
source-citation candidate that will not be returned.

```bash run id=variant-article-visible-enrichment exit=0
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. page-enrichment-json
```

```json expect=variant-article-visible-enrichment contains
{
  "visible_pmids": ["6010003"],
  "hidden_candidate_enriched": false
}
```

## Strategy Modes Isolate Diagnostic Routes

<!-- mustmatch-lint: skip -->

Omitting `--strategy` is the dependable union behavior. The annotation and
lexical modes are diagnostic views: each returns only candidates acquired by
that route, while source-backed citations remain part of union.

```bash run id=variant-article-strategies exit=0
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. strategies-json
```

```json expect=variant-article-strategies contains
{
  "omitted_equals_union": true,
  "annotation_pmids": ["6010001", "6010003", "6010007"],
  "lexical_pmids": ["6010002", "6010003", "6010005", "6010006", "6010008"],
  "union_pmids": ["6010001", "6010002", "6010003", "6010004", "6010005", "6010006", "6010007", "6010008"]
}
```

## Unresolved Fallback Does Not Claim Exact Provenance

<!-- mustmatch-lint: skip -->

When strict identity resolution is healthy but finds no resolved allele,
best-effort text can still help discovery. Such a row is explicitly non-exact
and carries no matched exact alias.

```bash run id=variant-article-unresolved exit=0
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. unresolved-json
```

```json expect=variant-article-unresolved contains
{
  "resolution": "unresolved",
  "complete": true,
  "pmid": "24534189",
  "row_requested_gene": "MYD88",
  "routes": ["best_effort_free_text"],
  "matched_aliases": [],
  "has_exact_claim": false
}
```

## Healthy Empty Variant Literature Keeps Its Envelope

<!-- mustmatch-lint: skip -->

A healthy annotation miss is different from a provider failure. JSON keeps the
empty collection, resolution, source status, completeness, and pagination facts
so callers do not have to infer state from missing keys.

```bash run id=variant-article-empty exit=0
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. healthy-empty-json
```

```json expect=variant-article-empty contains
{
  "strategy": "annotation",
  "resolution": "unresolved",
  "results": [],
  "complete": true,
  "truncated": false,
  "pagination": {
    "offset": 0,
    "limit": 3,
    "returned": 0,
    "total": 0,
    "has_more": false,
    "next_page_token": null
  },
  "source_status": [
    {"route": "pubtator_variant", "source": "pubtator", "status": "ok"}
  ]
}
```

## Caller-supplied RefSeq identities remain exact when MyVariant has no record

<!-- mustmatch-lint: skip -->

A complete versioned RefSeq chromosome identity and explicit assembly is an exact
caller assertion even when MyVariant has no matching row. Both decomposed fields
and genomic HGVS canonicalize to the same public identity shape; exact literature
routes retain the literal transcript, coding, and genomic aliases without
inventing chromosome coordinates or provider confirmation.

```bash run id=variant-article-refseq-not-found exit=0
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. refseq-not-found-json
```

```json expect=variant-article-refseq-not-found contains
{
  "items": [
    {
      "request_id": "atm-grch38",
      "requested_variant": {
        "gene": "ATM",
        "transcript": "NM_000051.4",
        "coding_change": "c.1066-6T>G",
        "genomic_accession": "NC_000011.10",
        "genome_build": "GRCh38",
        "position": 108248927,
        "reference": "T",
        "alternate": "G"
      },
      "resolution": {
        "status": "resolved",
        "basis": "caller_supplied",
        "exhaustive": true,
        "provider_validation": {
          "source": "myvariant",
          "status": "not_found",
          "matched_alias": null,
          "contradictory_field": null
        }
      },
      "complete": true,
      "truncated": false,
      "source_citation": {
        "status": "skipped",
        "detail": "no compatible MyVariant record"
      },
      "literal_exact_aliases": [
        "ATM c.1066-6T>G",
        "NC_000011.10:g.108248927T>G",
        "NM_000051.4:c.1066-6T>G"
      ],
      "only_literal_exact_aliases": true,
      "literal_exact_route_queries": [
        "ATM c.1066-6T>G",
        "NC_000011.10:g.108248927T>G",
        "NM_000051.4:c.1066-6T>G"
      ],
      "only_literal_route_queries": true,
      "literal_route_source_provenance": true
    },
    {
      "request_id": "palb2-grch38",
      "requested_variant": {
        "gene": "PALB2",
        "transcript": "NM_024675.4",
        "coding_change": "c.3350+5G>A",
        "genomic_accession": "NC_000016.10",
        "genome_build": "GRCh38",
        "position": 23607859,
        "reference": "C",
        "alternate": "T"
      },
      "resolution": {
        "status": "resolved",
        "basis": "caller_supplied",
        "exhaustive": true,
        "provider_validation": {
          "source": "myvariant",
          "status": "not_found",
          "matched_alias": null,
          "contradictory_field": null
        }
      },
      "complete": true,
      "truncated": false,
      "source_citation": {
        "status": "skipped",
        "detail": "no compatible MyVariant record"
      },
      "literal_exact_aliases": [
        "NC_000016.10:g.23607859C>T",
        "NM_024675.4:c.3350+5G>A",
        "PALB2 c.3350+5G>A"
      ],
      "only_literal_exact_aliases": true,
      "literal_exact_route_queries": [
        "NC_000016.10:g.23607859C>T",
        "NM_024675.4:c.3350+5G>A",
        "PALB2 c.3350+5G>A"
      ],
      "only_literal_route_queries": true,
      "literal_route_source_provenance": true
    }
  ],
  "encoding_equivalence": {
    "same_requested_variant": true,
    "expected_normalized_aliases": true,
    "same_normalized_aliases": true,
    "same_route_queries": true,
    "same_public_behavior": true
  }
}
```

## Batch Variant Literature Is Ordered and Compact

<!-- mustmatch-lint: skip -->

A structured input file replaces caller-authored alias query matrices when several
exact variants need literature triage. The response keeps request order and each
sibling's resolution state while returning shortlist facts rather than hydrated
article cards. Its next commands can be parsed directly for article triage and
detail retrieval.

```bash run id=variant-article-batch exit=0
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. batch-compact-json
```

```json expect=variant-article-batch contains
{
  "request_ids": ["braf-v600e", "myd88-s219c"],
  "requested_genes": ["BRAF", "MYD88"],
  "sibling_arrays_retained": true,
  "resolutions": [
    {"request_id": "braf-v600e", "status": "resolved"},
    {"request_id": "myd88-s219c", "status": "unresolved"}
  ],
  "match_reasons": {
    "braf_all_exact": true,
    "myd88_all_best_effort": true
  },
  "route_claims": {
    "braf_has_exact": true,
    "myd88_only_fallback": true
  },
  "aggregate": {"complete": true, "truncated": true},
  "item_state_present": true,
  "compact_rows": true,
  "followups": {
    "parseable": true,
    "article_batch": true,
    "article_detail": true,
    "fulltext": true,
    "assets": true,
    "citations": true
  }
}
```

## Variant Article Route Plans Are Opt In

<!-- mustmatch-lint: skip -->

Request a route plan only in JSON when aliases, provider work, ranking, or a
truncated acquisition needs explanation. Ordinary output stays compact. A single
request and every item in a batch expose the same typed route facts, while the
batch adds its fixed item-worker and request-budget summary.

```bash run id=variant-article-plan exit=0
bash ../fixtures/run-variant-article-entity-fixture.sh ../.. debug-plan-json
```

```json expect=variant-article-plan contains
{
  "ordinary_omits_plan": {"single": true, "batch": true},
  "single": {
    "aliases_present": true,
    "required_routes": {
      "annotation": true,
      "lexical": true,
      "source_citation": true
    },
    "shape_complete": true
  },
  "batch": {
    "item_concurrency_limit": 2,
    "items_planned": 2,
    "request_budget_consistent": true,
    "every_item_has_plan": true
  }
}
```

## ID Normalization

Exact variant lookup should normalize equivalent identifiers back to the same
canonical record instead of splitting the user into parallel identities.

## Transcript HGVS Normalization Proxies

Transcript HGVS strings are not exact MyVariant IDs, but agents often already
have a source-shaped transcript candidate from a report or another database. The
normalization proxy keeps that input separate from each upstream service's
returned notation and warnings.

## ERBB2 Transcript HGVS Canary

The proxy must handle transcript strings with substitution notation and shell
metacharacters such as `>` without losing source warnings or conflating service
outputs.

## Unsupported Normalization Inputs

BioMCP should not guess transcripts or convert gene-protein shorthand into a
transcript HGVS query. Unsupported input gets a typed guardrail so an agent can
choose a better source-shaped string.

```bash
set +e
out="$(../../tools/biomcp-ci --json variant normalize all 'BRAF V600E' 2>&1)"
rc=$?
set -e
test "$rc" -ne 0
mustmatch like 'unsupported_notation
BRAF V600E
transcript HGVS' <<<"$out"
```

## Normalization Command Discoverability

The explicit proxy command should be visible from help and structured list
output so agents can find it without trying hidden `get variant` rewrites.

```bash
../../tools/biomcp-ci variant normalize --help | mustmatch like 'all, mutalyzer, or variantvalidator
NM_000248.3:c.135del'
../../tools/biomcp-ci --json list variant | jq -e '.commands | any(. == "variant normalize <service> <transcript_hgvs>")' >/dev/null
```
