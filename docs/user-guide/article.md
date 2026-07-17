# Article

Use article commands for literature retrieval by disease, gene, drug, and identifier.

## Typical article workflow

1. search a topic,
2. choose an identifier,
3. retrieve default summary,
4. request indexing, full text, or annotations only when needed.

## Search articles

By gene and disease:

```bash
biomcp search article -g BRAF -d melanoma --limit 5
```

By keyword:

```bash
biomcp search article -k "immunotherapy resistance" --limit 5
```

By author:

```bash
biomcp search article -a "Williams LS" --limit 5
```

Author filtering is an authorship constraint. The default `--source all` route
limits candidate search to backends with native author fields: Europe PMC and,
when the other selected filters are compatible, PubMed. Filters such as
`--open-access` or `--no-preprints` can narrow the plan further to Europe PMC.
You can select `--source europepmc` or `--source pubmed` directly; PubTator3,
Semantic Scholar, and LitSense2 reject `--author` instead of treating the name
as free text.

Tune keyword-bearing relevance:

```bash
biomcp search article -k "Hirschsprung disease ganglion cells" --ranking-mode hybrid --weight-semantic 0.5 --weight-lexical 0.2 --limit 5
```

By date:

```bash
biomcp search article -g BRAF --since 2024-01-01 --limit 5
```

By year range:

```bash
biomcp search article -k "BRAF melanoma" --year-min 2000 --year-max 2013 --limit 5
```

Exclude preprints when supported by source metadata:

```bash
biomcp search article -g BRAF --since 2024-01-01 --no-preprints --limit 5
```

## Query formulation

Turn a natural-language literature question into two parts:

- Put a known gene, disease, or drug in `-g/--gene`, `-d/--disease`, or `--drug`.
- Put mechanisms, phenotypes, outcomes, datasets, and other provider-neutral free-text concepts in `-k/--keyword`.
- Put a known author in `-a/--author` and a known journal in `--journal`; do not put PubMed or Europe PMC field grammar in `-k/--keyword`.
- If the question is asking which gene, disease, or drug fits the evidence and you do not know the entity yet, do not guess a typed flag. Start with keyword-only article search or run `biomcp discover "<question>"` first.
- Question-format article terms are acceptable: PubMed ESearch cleans bounded filler words from unfielded gene, disease, drug, and keyword terms provider-locally, while query echoes and non-PubMed sources keep the original wording.
- Use `--type review` for synthesis questions, list-style questions, and dataset surveys.

Keyword-only searches can also return exact entity suggestions. When the whole
keyword exactly matches a gene, drug, or disease vocabulary label or alias,
BioMCP can add a typed `get gene`, `get drug`, or `get disease` follow-up in
`See also`, `_meta.next_commands`, and JSON `_meta.suggestions[]`. The
structured suggestion object includes `command`, `reason`, and `sections`.
Multi-concept phrases such as `BRAF V600E` or `lung cancer immunotherapy` do
not get direct entity suggestions, and searches that already use `-g`, `-d`,
or `--drug` suppress the exact suggestion.

For agent loops, `--session <token>` lets JSON article search compare the
current keyword with the previous successful article keyword search for the
same local token. The token is not a secret; use a short non-identifying label
such as `lit-review-1`. When post-stopword term overlap is at least 60%,
BioMCP can add JSON-only `_meta.suggestions[]` fallbacks after exact entity
suggestions: prior `article batch`, `discover`, and a date-narrowed retry when
available. Session baselines expire after 10 minutes. Markdown output is
unchanged.

Known anchor only:

```bash
biomcp search article -g BRAF --limit 5
```

Known anchor plus mechanism or process:

```bash
biomcp search article -g TP53 -k "apoptosis gene regulation" --limit 5
```

Unknown-entity disease-identification question:

```bash
biomcp search article -k '"cafe-au-lait spots" neurofibromas disease' --type review --limit 5
```

Known drug plus mechanism:

```bash
biomcp search article --drug amiodarone -k "photosensitivity mechanism" --limit 5
```

Dataset or method question:

```bash
biomcp search article -k "TCGA mutation analysis dataset" --type review --limit 5
```

### Multi-source federation

Article search fans out to PubTator3, Europe PMC, and PubMed by default when
the filter set is compatible. Known gene, disease, drug, and keyword queries
participate in that route. Semantic Scholar can still join the same query when
the filter set is compatible. An author filter is intentionally narrower: the
default route limits candidate search to Europe PMC and compatible PubMed,
whose native author fields make every returned candidate an authorship match
rather than a lexical match.
Semantic Scholar and LitSense2 are also available as explicit single-source
routes with `--source semanticscholar` and
`--source litsense2`. BioMCP merges duplicates across PMID,
PMCID, and DOI where possible. `S2_API_KEY` upgrades the Semantic
Scholar leg to authenticated requests at 1 req/sec; without it, BioMCP uses
the shared unauthenticated pool at 1 req/2sec. Search results are still
deduplicated by PMID when BioMCP can resolve one.

Default `--sort relevance` is mode-aware:

- Keyword-bearing queries default to `--ranking-mode hybrid`, using
  `0.4*semantic + 0.3*lexical + 0.2*citations + 0.1*position` with the
  LitSense2-derived semantic signal.
- Entity-only queries default to `--ranking-mode lexical`, preserving the
  existing calibrated PubMed rescue plus lexical directness comparator.
- `--ranking-mode semantic` sorts the LitSense2-derived semantic signal first
  and falls back to the lexical comparator for deterministic ties.
- Rows without LitSense2 provenance contribute `ranking.semantic_score = 0`
  in semantic-aware ranking modes.
- `--weight-semantic`, `--weight-lexical`, `--weight-citations`, and
  `--weight-position` retune the hybrid formula.

Markdown preserves the merged rank order. JSON search rows are compact by
default: available PMID/PMCID/DOI/arXiv/Semantic Scholar identifiers, title,
journal, date, citation count, primary source, and tri-state retraction state.
Use `biomcp --json search article -g BRAF --limit 5 --full` to restore detailed
rows with `matched_sources`, `ranking`, `first_index_date`, influential counts,
scores, and abstract snippets.

`--sort date` replaces relevance ranking rather than refining it. Compact JSON,
`--full` JSON, and Markdown all emit an in-band warning when date sort is used.

Use `--source <all, pubtator, europepmc, pubmed, semanticscholar, litsense2>`
to select one backend or keep the default federated search.
BioMCP caps each federated source's contribution after deduplication and before
ranking. Default: 40% of `--limit` on federated pools with at least three
surviving primary sources. Rows count against their primary source after
deduplication. Use `--max-per-source <N>` to override that cap, use
`--max-per-source 0` for the default cap explicitly, and set it equal to
`--limit` to disable capping.
Default article search excludes confirmed retractions unless you pass
`--include-retracted`. Sources that do not expose retraction metadata still
participate in the search, and compact and `--full` JSON search rows keep the
tri-state contract: `"is_retracted": true`, `false`, or `null`.
`--type`, `--open-access`, and `--no-preprints` are backend-compatibility
constraints rather than universal filters across every article source.
`--type` on `--source all` uses Europe PMC + PubMed when `--open-access` and
`--no-preprints` are both absent. If you add `--open-access` or
`--no-preprints`, PubMed becomes ineligible and BioMCP surfaces the Europe
PMC-only note in markdown, JSON, and debug-plan output instead of silently
pretending the filter applies across every source.

To search a single backend:

```bash
biomcp search article -g BRAF --source pubtator --limit 5
biomcp search article -g BRAF --source europepmc --limit 5
biomcp search article -g BRAF --source pubmed --limit 5
```

To force a tighter federated balance:

```bash
biomcp search article -k "Kartagener syndrome ciliopathy" --limit 50 --max-per-source 10
```

## Get an article

Supported IDs are PMID (digits only), PMCID (e.g., PMC9984800), and DOI
(e.g., 10.1056/NEJMoa1203421). Publisher PIIs (e.g., `S1535610826000103`) are not
indexed by PubMed or Europe PMC and cannot be resolved.

```bash
biomcp get article 22663011
```

Detail returns every author name supplied by the selected metadata source in
source order. JSON includes `authors`, `author_count` (the number returned),
`author_completeness` (`complete`, `source_limited`, or `unavailable`), and
`author_source` (`pubtator` or `europepmc`). PubTator's structured author list is
`complete`; Europe PMC's display string is `source_limited`, so BioMCP does not
claim it reconstructs a ground-truth author list. Markdown prints the same names
and status without inserting an ellipsis or a synthetic author.

Default article output can include an optional Semantic Scholar section with
TLDR text, influence counts, and open-access PDF metadata when that paper
resolves in Semantic Scholar. `S2_API_KEY` makes those requests authenticated;
without it, BioMCP uses the shared pool. `search article --source` supports
`all`, `pubtator`, `europepmc`, `pubmed`, `semanticscholar`, and `litsense2`;
Semantic Scholar remains an automatic compatible leg and can also be queried
alone with `--source semanticscholar`.

## Request specific sections

Full text section:

```bash
biomcp get article 22663011 fulltext
```

This uses the default article full-text ladder: XML first, then PMC HTML when
the XML path misses for a PMCID-backed article. It never falls back to PDF.
When full text resolves, BioMCP prints a local `Saved to:` path for cached
Markdown and surfaces the winning source label (`Europe PMC XML`, `PMC HTML`,
etc.) in markdown and JSON provenance. For XML/JATS winners, the saved Markdown
keeps section text, tables, references, figure captions, supplementary-material
metadata, and explicit markers for complex merged-cell tables that are not yet
flattened. JSON fulltext responses also include `full_text_manifest`, an
additive artifact manifest with the normalized source family (`jats_xml`,
`pmc_html`, or `pdf`), provider label/source, concrete source identifier,
quality flags, known license/reuse state, and provenance facts such as
open-access and explicit PDF fallback status. JSON fulltext also reports
`not_included` counts and points to the OA package asset manifest when figure
images, supplementary files, or complex tables are not inlined.

JSON always exposes `section_outcomes.fulltext`: base cards are
`not_requested`, successful retrieval is `data`, an all-healthy ladder with no
winner is `empty`, and a ladder with any failed eligible source and no winner is
`unavailable`. `_meta.section_sources` mirrors that same outcome and provider
list. Markdown uses the same state, so confirmed absence says full text was not
found while incomplete retrieval says it is unavailable.

Article assets:

```bash
biomcp --json get article <id> assets
biomcp get article <id> asset <filename>
```

`get article <id> assets` returns a JSON-only provider-labelled manifest. BioMCP
tries the canonical PMC OA package first, a validated Europe PMC supplementary
ZIP second, and supported Figshare/AACR Figshare metadata last. Figshare
manifests can include same-paper sibling records discovered by DOI/title, while
excluding records that do not match the paper. `get article <id> asset <name>`
returns the selected member as raw bytes with no conversion; downstream tools
parse CSV, XLSX, DOC, PDF, or images. Manifest handles remain BioMCP commands,
not provider download URLs. Europe PMC reuse remains unknown unless article
metadata or a retained PMC OA manifest supplies a license; retained licenses
name PMC OA as their source. A healthy provider ladder with no package returns
`not_found`, while a failed source with no successful fallback returns
`source_unavailable` rather than claiming that assets are absent. This also
holds when an earlier archive provider failed and a healthy Figshare collection
lacks the requested filename. A filename missing from an already selected
successful package remains `not_found`. Figshare supplement PDFs and tables
remain assets, not full-text article substitutes.

Opt in to the final PDF rung only when you want the last-resort open-access PDF
path after XML and PMC HTML both miss:

```bash
biomcp get article 22663011 fulltext --pdf
```

With `--pdf`, BioMCP can use a Semantic Scholar open-access PDF URL from an
explicitly allowed Semantic Scholar/CDN HTTPS origin as the final fallback and
labels the winner as `Semantic Scholar PDF`. A successful lookup with no PDF is
a healthy absence; a failed lookup contributes unavailable state only when PDF
fallback was requested. Other provider-returned origins are rejected before
contact. `--pdf` is only valid with the `fulltext` section;
`biomcp get article 22663011 --pdf` is rejected instead of silently doing
nothing.

Indexing section:

```bash
biomcp get article 22663011 indexing
```

This opt-in section fetches PubMed citation XML and keeps each author's name,
optional ORCID, and source-associated affiliations together. MeSH descriptors
and qualifiers retain their UIs and independent major-topic flags. The explicit
`available` or `unavailable` status distinguishes a complete empty author/MeSH
list from metadata BioMCP could not retrieve. BioMCP accepts PubMed's normal
external `DOCTYPE` without downloading the DTD, while retaining the 8 MiB body
limit and a 100,000-node XML limit. When indexing is unavailable, JSON and
Markdown include a sanitized `failure` code and static message (for example,
`rate_limited`, `parse_error`, or `timeout`) without upstream bodies, URLs,
credentials, or parser details. The base article still succeeds. Ordinary detail,
search, and batch do not make this extra request; `get article <id> all` includes
it.

Annotation section:

```bash
biomcp get article 22663011 annotations
```

Semantic Scholar TLDR section:

```bash
biomcp get article 22663011 tldr
```

## Helper commands

```bash
biomcp article entities 22663011   # extract annotated entities via PubTator
biomcp article batch 22663011 24200969          # compact multi-article summary cards
biomcp article citations 22663011 --limit 3         # Semantic Scholar citation graph
biomcp article references 22663011 --limit 3        # Semantic Scholar reference graph
biomcp article recommendations 22663011 --limit 3   # Semantic Scholar related papers
```

`article batch` works without `S2_API_KEY` and returns a bare JSON array in
request order. Each compact card echoes the original `requested_id`, keeps its
resolved PMID/PMCID/DOI fields, and carries the same full `authors`, returned
`author_count`, `author_completeness`, and `author_source` contract as detail.
Markdown cards show the source-ordered names and an explicit status, including
when no author list was supplied. When Semantic Scholar data is available, the
batch helper can add optional TLDR and citation metadata without changing
authorship. `S2_API_KEY` makes that enrichment authenticated and more reliable.
Use `article batch` as the default follow-up after `search article` when you
already have several shortlisted PMIDs or DOIs.

The Semantic Scholar graph helpers also work without `S2_API_KEY`, but they use
the shared pool and can fail fast on HTTP 429 with guidance to set the key for
a dedicated rate limit. Citations usually work broadly; references and
recommendations can be sparse or empty for paywalled papers because of
publisher elision in the Semantic Scholar graph. In JSON, citation/reference
responses always carry `edges`, and recommendation responses always carry
`recommendations`, including `[]` on successful emptiness and parsed structured
errors. On errors, the accompanying `error` and nonzero exit status remain
mandatory, so an empty array must not be interpreted as a biomedical negative.

## Caching behavior

Downloaded content is stored in the BioMCP cache directory.
This avoids repeated large payload downloads during iterative workflows.

## JSON mode

```bash
biomcp --json get article 22663011
biomcp --json search article -g BRAF --limit 3
biomcp --json search article -k "Oncotype DX review" --session lit-review-1 --limit 5
biomcp --json article batch 22663011 24200969
```

JSON article responses include `_meta.next_commands` and `_meta.section_sources`,
so article workflows can promote the next likely pivots and preserve section
provenance without scraping markdown. For `get article <id> fulltext --json`,
`full_text_manifest.quality` reports whether saved Markdown has sections,
tables, references, fulltext signal, and fulltext entity annotations;
`full_text_manifest.reuse` separates known licenses from an explicit unknown
license/reuse warning; and `full_text_manifest.provenance` carries available
open-access, retraction, package, and PDF-fallback facts. JSON `search article`
responses also echo `query`, `sort`, `semantic_scholar_enabled`, and row-level ranking/provenance
metadata. In relevance mode, ranking metadata now includes the effective mode
plus normalized lexical, citation, and position components; semantic-aware
rows expose `ranking.semantic_score` as the LitSense2-derived signal and use
`0` when LitSense2 did not match. Hybrid rows also include the composite
score. Keyword-only article searches with an exact gene, drug, or disease
label/alias match may include `_meta.suggestions[]` objects with `command`,
`reason`, and `sections`; same-session keyword loop-breaker suggestions include
`command` and `reason` and omit `sections`. `_meta.next_commands` remains the
executable string command list. JSON `article batch` responses are a bare array of compact cards
so callers can map results back to the original input order; this compatibility
shape is intentionally not wrapped in an object.

## Practical tips

- Start with narrow `--limit` values.
- Add a disease term when gene-only search is too broad.
- Use section requests to avoid oversized responses.
- Use `biomcp get article <id> indexing` for PubMed author-affiliation and MeSH indexing metadata.
- Use `biomcp get article <id> tldr` when you want only the optional Semantic Scholar section.

## Related guides

- [Gene](gene.md)
- [Trial](trial.md)
- [How to find articles](../how-to/find-articles.md)
