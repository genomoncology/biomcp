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

Article assets are a separate on-demand surface. `get article <id> assets`
fans in PMC OA, Europe PMC supplementary files, JATS XML, PMC HTML, and eligible
Figshare discoveries for a validated PMCID. Provider-relative JATS/HTML links
become stable BioMCP handles only after bounded, allowlisted retrieval. A linked
binary received as HTML or XHTML is rejected; PMC proof-of-work responses remain
named `pmc_proof_of_work` coverage with no asset key or handle. Europe PMC ZIPs are validated under compressed, per-member,
expanded-total, count, and normalized-name bounds entirely in memory. Figshare
uses the same collection resolver as raw-byte retrieval: it starts from the
linked record, adds same-paper sibling records found by DOI/title, filters out
wrong-paper candidates, and keeps handles as BioMCP commands. The command
`get article <id> asset <asset-key>` uses the same merged resolver as the manifest and returns
the selected asset bytes without conversion. A successful manifest makes an unknown asset key a
true asset miss; without a winner, any source failure produces
`source_unavailable`, while all-healthy absence produces `not_found`.

PMC OA resolves a versioned S3 metadata object, then downloads only its
declared XML and media objects. Each object is bounded at 8 MiB; media is
bounded to 256 objects and 64 MiB aggregate payload. Unsafe provider paths and
over-limit results cross the provider boundary only as sanitized source
unavailability.

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

Each eligible rung is structurally classified before the ladder is folded. JATS
qualifies only when the direct article `body` contains a nonblank supported body
block; front/title/abstract-only, back-reference, root-float, and supplement-only
documents are partial non-winners. PMC HTML selects `main article`, then
`article`, then `main`; abstract-token containers and labelled chrome, metadata,
byline, affiliation, keyword, permission, and reference regions cannot establish
body coverage. Neither format uses byte, line, word, or section-count thresholds.

A documented not-found/no-content response is a healthy absence. Initialization,
identity, authentication, throttling, transport, timeout, 5xx, body-limit,
malformed or unsupported content, decode, conversion, worker, and
empty-conversion errors are failures. Partial content is healthy: its abstract
fills an absent or blank base abstract, but it does not create a saved path,
provider credit, a data outcome, or a full-text quality signal. A later body
winner always wins; without a winner, a later healthy absence cannot erase an
earlier failure.

BioMCP does not enforce article-level reuse licenses at runtime. Users must
review provider terms and the returned article license context before reusing or
redistributing downloaded full text, saved Markdown, or PDFs. JSON fulltext
manifests report `reuse.license_present`, a trimmed `reuse.license` when known,
and a warning when license/reuse status is unknown. The durable terms inventory
lives in `docs/reference/source-licensing.md`.

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
- `full_text_coverage`: additive JSON emitted only for requested full text. Its
  `coverage` is `full_text`, `abstract_only`, `metadata_only`, `none`, or
  `unavailable`, and `attempts` records every eligible content rung in order.
  Each attempt has a stable provider, `source_kind` (`jats_xml`, `pmc_html`, or
  `pdf`), structural coverage, `data`/`empty`/`unavailable` outcome,
  `hit`/`miss`/`bypass` cache state, and a closed bounded reason. Attempts never
  contain response content, raw errors, identifiers/URLs, parser details,
  credentials, signed queries, or local paths.
- `full_text_manifest`: additive JSON-only manifest emitted when a source wins.
  It includes:
  - `source_kind`: normalized artifact family (`jats_xml`, `pmc_html`, `pdf`).
  - `provider.label` and `provider.source`: same stable labels as the winning
    `full_text_source`.
  - `source_identifier`: the concrete PMCID, PMID, package/PDF URL, or other
    source identifier used by the winner.
  - `quality`: booleans for sections, tables, references, non-empty fulltext
    signal, and fulltext entity annotations. Current HTML/PDF paths do not
    claim section/table/reference or entity-annotation structure.
  - `reuse`: known license state, optional license text, and an unknown-license
    warning when BioMCP has no article/PDF license fact.
  - `provenance`: available open-access/retraction facts, package URL when
    available, and `pdf_fallback_used` for explicit PDF winners.

Markdown prints `Saved to:` and does not inline full text or manifest prose in
the article card. When OA package assets are available but not inlined, Markdown
points to `biomcp --json get article <id> assets`. JSON fulltext responses add a
structured `not_included` summary for package figure images, supplementary
files, and complex tables plus asset retrieval next commands. It is deliberately
package-only: JATS- or HTML-linked-only assets are available through the explicit
`assets` and `asset` surfaces, not counted or acquired for an ordinary fulltext
summary. Non-PMC Figshare assets likewise stay on the explicit asset surface;
they are not parsed into full text or treated as the `fulltext --pdf`
article-body fallback.
Every Article owns `section_outcomes.fulltext`. A base card records
`not_requested`; a requested ladder completes it once as `data`, `empty`, or
`unavailable`. Structural coverage and source health are independent: the best
partial observation survives a later failure, while any failure still makes the
section outcome unavailable unless a later body wins. With no partial, healthy
exhaustion is `none`/`empty` and failed exhaustion is
`unavailable`/`unavailable`. JSON `_meta.section_sources` projects the
entity-owned outcome: `data` retains its winning provider and `empty` retains
healthy consulted content providers; `unavailable` has no successful sources,
and `not_requested` is omitted. The compatible `full_text_path`,
`full_text_source`, `full_text_manifest`, and `full_text_note` fields agree with
the same outcome.

## JATS Markdown Coverage

JATS parsing, structural classification, abstract extraction, rendering, and
quality facts share one bounded parsed document. `quality.has_fulltext_signal`
is true only when the direct body qualifies. The JATS converter renders section
text, inline body figures, root-level
`floats-group` figures and tables after the body, regular tables, references,
and supplementary-material label/caption/filename metadata. Float rendering
keeps document order and deduplicates root floats by `id` when the same figure
or table was already rendered from the body.

Supplementary-material filenames and links remain display facts for the network-free
converter. The separate asset resolver may fetch recognized JATS/PMC-HTML supplement
links under its URL and byte budgets. Tables
with `rowspan` or `colspan` keep their caption and render an explicit
`*[complex table omitted: N×M, merged cells]*` marker instead of silently
dropping the grid; full span flattening remains out of scope.

## Failure Visibility

A winning source is visible through the Markdown heading label,
`full_text_source`, `full_text_manifest`, and `_meta.section_sources`.

There is no public per-leg trace in Markdown: it never renders per-provider
attempts or errors. JSON exposes only the sanitized typed attempt records above. With no winner, an all-healthy ladder
reports confirmed absence (`empty`) while any failed eligible consultation
reports a bounded `unavailable` outcome and in-band Markdown note. Partial states
instead give one bounded abstract/body or metadata/body statement. Opt-in
Semantic Scholar discovery is part of the attempt fold: no URL is a healthy
absence, discovery/fetch failure is unavailable, and discovery has no effect
without `--pdf`.

Normal PMC HTML reads the cache middleware's trusted `x-cache` HIT/MISS result
after middleware processing; explicit no-cache and pre-lookup failures are
`bypass`. Cached and fresh bodies pass through the same structural classifier.
Saved full-text artifacts use the `v4` namespace, and only a current classified
body can create or reuse a v4 path, so stale v3 partial artifacts cannot win.
Saved-artifact failure remains a returned BioMCP error after a source winner
because local delivery, not provider availability, failed.

## Module Ownership

- `src/entities/article/detail.rs`: base article orchestration, section
  validation, Semantic Scholar enrichment timing, and the `--pdf` precondition.
- `src/entities/article/fulltext.rs`: identity bridge, content ladder,
  eligibility policy, fulltext source labels, cache key, and saved artifact
  assignment.
- `src/entities/article/assets.rs`: merged PMC OA, Europe PMC, recognized linked
  JATS/PMC HTML, and eligible Figshare outcomes shared by manifests and raw byte
  retrieval; final absence/failure classification; retained reuse provenance;
  identity/hash deduplication; typed named coverage; and stable retrieval handles.
- `src/sources/europepmc.rs`, `src/sources/ncbi_efetch.rs`,
  `src/sources/pmc_oa.rs`, `src/sources/pmc_article.rs`, and
  `src/sources/ncbi_idconv.rs`: upstream transport for direct source APIs,
  including bounded Europe PMC ZIP, PMC OA metadata-declared objects, PMC HTML,
  and linked-asset acquisition behind provider URL policy.
- `src/sources/figshare.rs`: Figshare/AACR Figshare URL parsing, article search
  and article API metadata normalization, safe file filtering, and bounded
  file-byte downloads including `202 Accepted` cold-storage staging retries.
- `src/sources/semantic_scholar.rs`: metadata enrichment that may expose
  `openAccessPdf`. Arbitrary PDF byte fetching remains article fulltext
  policy, not a Semantic Scholar source-client method.
- `src/transform/article/jats.rs`, `src/transform/article/html.rs`,
  `src/transform/article/pdf.rs`, and `src/transform/article.rs`: typed
  structural classification, abstract extraction, and winner conversion to
  Markdown.
- `src/render/markdown/article.rs`, `templates/article.md.j2`, and
  `src/render/provenance.rs`: visible Markdown and JSON provenance.
- `src/utils/download.rs`: atomic saved-file persistence.

## Verification

The current contract is covered by:

- Rust article fulltext tests in `src/entities/article/detail/tests.rs`,
  `src/entities/article/fulltext.rs`, `src/render/provenance.rs`, and
  `src/render/markdown/article/tests.rs`.
- The bootstrap canary in `spec/entity/article.md` proves the saved-artifact
  contract, the PMC HTML fallback path, PMC OA, Europe PMC, and Figshare asset
  byte surfaces,
  the named `--pdf` opt-in, and the keyless article-search degradation markers
  that stay in the blocking lane.
- Resolver-order and provenance-label details stay pinned by the focused Rust
  tests above until the follow-on v2 surface rewrites land.
