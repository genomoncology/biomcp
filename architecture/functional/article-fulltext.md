# Article Fulltext Architecture

This document defines the current article fulltext contract for BioMCP. For
user-facing commands and examples, see `docs/user-guide/article.md`. For
provider terms and article-level reuse constraints, see
`docs/reference/source-licensing.md`.

## Current Surface

The public entry point is `get article <id> fulltext`. The PDF rung is available
only through explicit opt-in with `get article <id> fulltext --pdf`. The section
accepts the same article identifiers as the base article card: PMID, PMCID, and
DOI.

Full text is saved as a local Markdown artifact. BioMCP prints a source-labeled
fulltext heading and `Saved to:` path, but it does not inline the full article
body in the article card.

## Identity Bridge and Resolver Order

NCBI ID Converter is an identity bridge, not a content resolver. It runs only
when the base article has no PMCID and the article has a PMID or DOI that can be
bridged to a PMCID. PMCID-dependent content rungs then use the original or
bridged PMCID.

The current shipped order is:

1. NCBI ID Converter identity bridge when PMCID is missing.
2. Europe PMC PMC XML.
3. NCBI EFetch PMC XML.
4. PMC OA Archive XML.
5. Europe PMC MED XML.
6. PMC HTML.
7. Semantic Scholar PDF, only when `--pdf` is present.

The stable display labels are `Europe PMC XML`, `NCBI EFetch PMC XML`,
`PMC OA Archive XML`, `Europe PMC MED XML`, `PMC HTML`, and
`Semantic Scholar PDF`.

## Eligibility, Format, and License Gates

Runtime eligibility is separate from license and reuse guidance:

- NCBI ID Converter runs only when PMCID is missing and PMID or DOI is present.
- Europe PMC PMC XML, NCBI EFetch PMC XML, PMC OA Archive XML, and PMC HTML are
  PMCID-dependent.
- Europe PMC MED XML requires a PMID.
- PDF requires `fulltext`, `--pdf`, successful Semantic Scholar enrichment, and
  a non-empty `semantic_scholar.open_access_pdf.url`.
- XML/JATS is accepted from XML content sources.
- PMC HTML accepts `text/html` or `application/xhtml+xml`.
- PDF accepts `application/pdf` or a `%PDF-` body signature.

Unsupported content types, missing upstream records, oversized HTML/PDF bodies,
conversion failures, and empty converted output are resolver misses. They do
not become user-facing hard errors while a later eligible rung can still win.

BioMCP does not enforce article-level reuse licenses at runtime. Users must
review provider terms and the returned article license context before reusing or
redistributing downloaded full text, saved Markdown, or PDFs. The durable terms
inventory lives in `docs/reference/source-licensing.md`.

## Saved Artifact Contract

The stable output fields are:

- `full_text_path`: saved Markdown path.
- `full_text_note`: final user-visible miss or error note when no source wins.
- `full_text_source.kind`: serialized as `jats_xml`, `html`, or `pdf`.
- `full_text_source.label`: display label, one of `Europe PMC XML`,
  `NCBI EFetch PMC XML`, `PMC OA Archive XML`, `Europe PMC MED XML`,
  `PMC HTML`, or `Semantic Scholar PDF`.
- `full_text_source.source`: JSON provenance source, one of `Europe PMC`,
  `NCBI EFetch`, `PMC OA`, `PMC`, or `Semantic Scholar`.

Markdown prints `Saved to:` and does not inline full text in the article card.
JSON `_meta.section_sources` includes a `fulltext` row only when
`full_text_source` exists. Note-only misses do not publish a `fulltext`
provenance row.

## Failure Visibility

A winning source is visible through the Markdown heading label,
`full_text_source`, and `_meta.section_sources`.

There is no public per-leg trace in Markdown or JSON. XML API errors are
recorded internally and may collapse into `Full text not available: API error`
when no later eligible source wins. HTML and PDF fetch, conversion, or
content-type failures are misses. Semantic Scholar enrichment failure is
swallowed as a warning before fulltext resolution; with `--pdf`, that can make
PDF ineligible without a PDF-specific public note.

## Module Ownership

- `src/entities/article/detail.rs`: base article orchestration, section
  validation, Semantic Scholar enrichment timing, and the `--pdf` precondition.
- `src/entities/article/fulltext.rs`: identity bridge, content ladder,
  eligibility policy, fulltext source labels, cache key, and saved artifact
  assignment.
- `src/sources/europepmc.rs`, `src/sources/ncbi_efetch.rs`,
  `src/sources/pmc_oa.rs`, and `src/sources/ncbi_idconv.rs`: upstream
  transport for direct source APIs.
- `src/sources/semantic_scholar.rs`: metadata enrichment that may expose
  `openAccessPdf`. Arbitrary PDF byte fetching remains article fulltext
  policy, not a Semantic Scholar source-client method.
- `src/transform/article/jats.rs`, `src/transform/article/html.rs`,
  `src/transform/article/pdf.rs`, and `src/transform/article.rs`: source
  payload conversion to Markdown.
- `src/render/markdown/article.rs`, `templates/article.md.j2`, and
  `src/render/provenance.rs`: visible Markdown and JSON provenance.
- `src/utils/download.rs`: atomic saved-file persistence.

## Verification

The current contract is covered by:

- Rust article fulltext tests in `src/entities/article/detail/tests.rs`,
  `src/entities/article/fulltext.rs`, `src/render/provenance.rs`, and
  `src/render/markdown/article/tests.rs`.
- The bootstrap canary in `spec/entity/article.md` proves the saved-artifact
  contract, the PMC HTML fallback path, the named `--pdf` opt-in, and the
  keyless article-search degradation markers that stay in the blocking lane.
- Resolver-order and provenance-label details stay pinned by the focused Rust
  tests above until the follow-on v2 surface rewrites land.
