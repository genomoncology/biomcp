# Article Fulltext Markdown Target State

This document captures the target architecture for adopting the validated
JATS/HTML/PDF-to-Markdown extraction spike into BioMCP's article fulltext path.
It records the current problems from ticket 250's survey, the intended Rust
module boundaries, the fallback policy, and the incremental build path. The
current implementation remains JATS-only; this file describes the target state.

## Current Problems

The survey identified six root causes in the current article fulltext
architecture:

1. The spike's HTML converter uses `html2md`, which is GPL-3.0+ and cannot ship
   in the MIT-licensed BioMCP binary.
2. The spike's JATS path duplicates production logic. Production
   `src/transform/article/jats.rs` plus `jats/tests.rs` is already the stronger
   implementation and must remain the canonical JATS renderer.
3. `src/entities/article/detail.rs` only resolves XML. There is no HTML
   acquisition path today, even though the spike proved the PMC article page and
   other open HTML pages can be converted to agent-readable Markdown.
4. `src/entities/article/detail.rs` never fetches PDF bytes. The current code
   surfaces `semantic_scholar.open_access_pdf.url` in the card, but never uses
   it for fulltext extraction.
5. `templates/article.md.j2` hardcodes `## Full Text (PMC OA)`, so the rendered
   section label is wrong for Europe PMC MED XML today and would be wrong for
   any future HTML or PDF path.
6. `src/entities/article/detail.rs::fulltext_cache_key()` only keys by article
   identifier and one global `FULLTEXT_CACHE_VERSION`, so JATS/HTML/PDF output
   would collide once multiple extraction families exist.

## Target Module Boundaries

### Keep Production JATS as the Canonical XML Renderer

`src/transform/article/jats.rs` stays in place and remains the production JATS
renderer. The target does **not** import the spike's `extract_jats_markdown()`
implementation or add a second JATS crate.

- Keep `transform::article::extract_text_from_xml(&str) -> String` as the
  canonical JATS/XML-to-Markdown contract.
- Continue using `tokio::task::spawn_blocking` from the article fulltext path
  when rendering large XML payloads.
- Keep the existing `transform/article/jats/tests.rs` as the baseline JATS
  regression contract, and keep the active saved-artifact/fulltext canary in
  `spec/entity/article.md`.

This preserves the reference rendering and `xref` handling that production
already does better than the spike.

### Extract Fulltext Resolution out of `detail.rs`

The XML/HTML/PDF acquisition and fallback policy should move out of
`src/entities/article/detail.rs` into a dedicated article-local module:

- Create `src/entities/article/fulltext.rs`.
- Move the current XML waterfall and saved-file handling into
  `fulltext::resolve_fulltext(...)`.
- Keep `src/entities/article/detail.rs::get()` responsible for article identity
  resolution and Semantic Scholar enrichment, but delegate fulltext work to the
  new module.

The target data structures are:

```rust
pub struct ArticleGetOptions {
    pub allow_pdf: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleFulltextKind {
    JatsXml,
    Html,
    Pdf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleFulltextSource {
    pub kind: ArticleFulltextKind,
    pub label: String,
    pub source: String,
}
```

`src/entities/article/mod.rs::Article` gains:

- `full_text_source: Option<ArticleFulltextSource>`

`src/entities/article/fulltext.rs` owns:

- XML waterfall order and source labeling
- HTML fallback policy
- PDF fallback policy
- source-aware cache-key generation
- bounded download helpers for HTML/PDF payloads

No other entity module should know the JATS/HTML/PDF ordering.

### XML Fallbacks Remain First and Truthful

The existing XML sources remain the first-choice path because they are the most
structured:

1. `EuropePmcClient::get_full_text_xml("PMC", pmcid)`
2. `NcbiEfetchClient::get_full_text_xml(pmcid)`
3. `PmcOaClient::get_full_text_xml(pmcid)`
4. `EuropePmcClient::get_full_text_xml("MED", pmid)`

The target changes are architectural, not source-replacement:

- Keep these existing source clients unchanged.
- Return an `ArticleFulltextSource` with the saved file so the renderer can
  label the result truthfully (`Europe PMC XML`, `NCBI EFetch PMC XML`,
  `PMC OA Archive XML`, `Europe PMC MED XML`).
- Keep JATS XML as the default and fastest-success path for `get article ...
  fulltext`.

### Add a Permissive HTML Transform Module

HTML conversion belongs in a new transform module:

- Create `src/transform/article/html.rs`.
- Re-export it from `src/transform/article.rs` as
  `extract_text_from_html(html: &str, base_url: &str) -> Result<String, BioMcpError>`.
- Use `readability-rust` for content extraction and a permissive HTML-to-Markdown
  crate (`htmd` is the leading candidate from the survey) instead of `html2md`.

Spike-to-production mapping:

| Spike function | Rust target |
|---|---|
| `extract_html_markdown(html, base_url)` | `transform::article::extract_text_from_html(html, base_url)` |
| `ProbeReport` | stays in the experiment only; not a production entity type |
| HTML metrics/scoring helpers | test-only helpers if needed; not part of the runtime article API |

### PMC HTML Page Is the Initial HTML Source

The initial production HTML source should be derived from metadata we already
have:

- When `article.pmcid` is present and all XML sources miss, derive the HTML page
  URL as `https://pmc.ncbi.nlm.nih.gov/articles/{pmcid}/`.
- Fetch that page from `src/entities/article/fulltext.rs` using the shared HTTP
  client conventions from `architecture/technical/source-integration.md`.
- Reject obvious non-HTML responses and bounded-error bodies before invoking the
  HTML renderer.

This keeps HTML acquisition article-local instead of inventing a generic new
source module for one derived article fallback URL.

Non-PMC HTML pages from the spike (bioRxiv preprint page, NIH news release) stay
as extractor-quality fixtures for now. They are not initial production sources
because current article metadata does not carry canonical HTML URLs for them.

### Add an Opt-In PDF Transform Module

PDF conversion belongs in a second transform module:

- Create `src/transform/article/pdf.rs`.
- Re-export it from `src/transform/article.rs` as
  `extract_text_from_pdf(bytes: &[u8], page_limit: usize) -> Result<String, BioMcpError>`.
- The initial production engine is `unpdf`.
- Keep `pdf_oxide` in the spike and evaluation fixtures only until a production
  regression proves `unpdf` is insufficient.

Spike-to-production mapping:

| Spike function/type | Rust target |
|---|---|
| `extract_pdf_markdown(input, PdfEngine, page_limit)` | `transform::article::extract_text_from_pdf(bytes, page_limit)` with `unpdf` only |
| `PdfEngine` | stays in the experiment until BioMCP ships more than one engine |
| PDF metrics/scoring helpers | test-only helpers if needed; not part of the runtime article API |

### PDF Fallback Policy: Explicit `--pdf`, Never Silent by Default

PDF extraction is slower and noisier than XML or HTML, so the target policy is:

- Add `--pdf` as a named modifier on `src/cli/article/mod.rs::ArticleGetArgs`.
- Change the article facade to
  `get(id: &str, sections: &[String], options: ArticleGetOptions)`.
- Only attempt PDF if:
  - `fulltext` was requested,
  - XML and HTML both failed,
  - `options.allow_pdf` is true, and
  - `article.semantic_scholar.open_access_pdf.url` is present.

PDF is therefore an explicit last-resort fallback, not part of the default
`get article <id> fulltext` behavior. This preserves current latency and avoids
quiet regressions on articles that already have structured XML.

The PDF fetch path lives in `src/entities/article/fulltext.rs`, not in
`src/sources/semantic_scholar.rs`, because the URL is arbitrary third-party
content referenced by Semantic Scholar metadata rather than a Semantic Scholar
API endpoint.

### Rendering, Provenance, and Cache Keys Must Be Source-Aware

These surfaces change together:

- `src/render/markdown/article.rs`
- `templates/article.md.j2`
- `src/render/provenance.rs`
- `src/entities/article/detail.rs::fulltext_cache_key()`
- article markdown/root tests that assert on `## Full Text (PMC OA)`

Target contract:

- Render `## Full Text ({{ full_text_source.label }})` when the section exists.
- Publish `_meta.section_sources.fulltext` from the actual fulltext source label,
  not a hardcoded `PMC OA`.
- Key the cache by extraction family:
  `article-fulltext-{FULLTEXT_CACHE_VERSION}:{kind}:{id}`.
- Bump `FULLTEXT_CACHE_VERSION` when the cache schema changes so pre-source-aware
  entries do not collide with the new family-specific layout.

The `Saved to:` contract remains unchanged.

## Runtime Data Flow

1. `src/entities/article/detail.rs::get()` resolves the base article exactly as
   it does today and enriches Semantic Scholar metadata.
2. If the user requested `fulltext`, `get()` calls
   `fulltext::resolve_fulltext(&article, id, options)`.
3. `fulltext::resolve_fulltext()` tries XML in the existing waterfall order.
4. On XML success, it runs `transform::article::extract_text_from_xml()`,
   persists the saved markdown, and sets `article.full_text_source` to the
   resolved XML source.
5. If XML fails and `article.pmcid` exists, it fetches the PMC article HTML page
   and runs `transform::article::extract_text_from_html()`.
6. If XML and HTML fail and `options.allow_pdf` is true, it fetches
   `semantic_scholar.open_access_pdf.url`, runs
   `transform::article::extract_text_from_pdf(bytes, 12)`, persists the output,
   and sets `article.full_text_source` to `Semantic Scholar PDF`.
7. If all eligible paths fail, `article.full_text_note` explains the truthful
   reason: no PMC identity, no HTML page, no PDF URL, or upstream failure.

## Surfaces That Gain Quality

- `get article <id> fulltext` for PMCID-backed articles keeps the current JATS
  path and gains a concrete PMC HTML fallback instead of terminating after XML.
- PMCID-backed Europe PMC and PubTator article cards gain truthful fulltext
  provenance labels instead of the hardcoded PMC OA label.
- Articles with `semantic_scholar.open_access_pdf.url` gain an explicit
  last-resort PDF recovery path behind `--pdf`.

Surfaces that should **not** change:

- `article citations`, `article references`, and `article recommendations`
- article search ranking, search federation, and section parsing other than the
  named `--pdf` modifier
- the existing XML source clients themselves

## Regression Fixtures and Contract

The target migration should keep explicit regression families for each extractor:

- **JATS:** existing production unit tests in `src/transform/article/jats/tests.rs`
  plus the active article canary in `spec/entity/article.md` for the user-visible
  saved-artifact/fulltext contract.
- **HTML:** commit a focused copy of the spike's PMC article page fixture
  (`pmc_article_page.html`) plus the two non-production extractor-quality
  fixtures (`biorxiv_preprint_page.html`, `nih_news_release.html`) under
  `tests/fixtures/article/fulltext/html/`.
- **PDF:** commit focused copies of the spike's three PDF quality cases under
  `tests/fixtures/article/fulltext/pdf/`:
  - `pmc_oa_article_pdf.pdf`
  - `dailymed_keytruda_label.pdf`
  - `cdc_sti_guideline.pdf`

PDF regression contract:

- The runtime article path only needs the article-grade `pmc_oa_article_pdf.pdf`
  fixture.
- The other two PDFs remain extraction-quality guards so the converter does not
  silently regress on common biomedical PDF shapes.
- The initial production page limit remains 12 pages, matching the spike.

## Dependency and License Position

The target dependency posture is:

| Crate | Survey verdict | Production target |
|---|---|---|
| `unpdf` | Accept | ship in the PDF fallback ticket |
| `pdf_oxide` | Accept | keep out of the first production path; revisit only if `unpdf` fails production fixtures |
| `readability-rust` | Accept | ship in the HTML fallback ticket |
| `html2md` | Reject (GPL-3.0+) | do not ship |
| `htmd` | leading permissive replacement candidate | validate with `cargo deny` before adding |

The repo root should gain `deny.toml` with the BioMCP allowlist:

- Apache-2.0
- MIT
- BSD-2-Clause
- BSD-3-Clause
- ISC
- MPL-2.0

The foundation build ticket should run `cargo deny check licenses` in the repo
lint gate so future extractor dependencies cannot bypass the allowlist.

## Invariants

- Production JATS stays canonical; the spike JATS implementation is not adopted.
- XML remains the default `fulltext` path.
- HTML is an internal fallback after XML, not a new user-facing section.
- PDF is opt-in via `--pdf` and never part of the default fulltext path.
- Every saved fulltext file records truthful source provenance through
  `Article.full_text_source`.
- Cache keys are extraction-family-specific.
- Each intermediate ticket must pass `make check`.

## Build Ticket Decomposition

### Ticket A: Add Article Fulltext Resolver Boundary and License Gate

Foundation only. Create `src/entities/article/fulltext.rs`, move the current XML
waterfall and save/cache logic there, add `ArticleFulltextSource` to the article
model, make the template/provenance/cache surfaces source-aware, and add the
repo `deny.toml` plus `cargo deny check licenses` gate.

Proof:

- `get article ... fulltext` still works for existing XML-backed articles.
- rendered fulltext labels become truthful to the actual XML source.
- `make check` passes with the new license allowlist gate.

### Ticket B: Add Opt-In PDF Fallback for Article Fulltext

Depends on Ticket A. Add `--pdf`, `ArticleGetOptions`, the bounded PDF fetch
helper, `src/transform/article/pdf.rs`, and `unpdf`-backed rendering. Wire PDF
only after XML/HTML failure and only when the flag is set.

Proof:

- default `get article <id> fulltext` remains XML/HTML-only.
- `get article <id> fulltext --pdf` can recover from an XML miss when a PDF URL
  exists.
- `make check` passes with PDF fixtures and source-aware cache tests.

### Ticket C: Add PMC HTML Fallback with a Permissive HTML-to-Markdown Converter

Depends on Ticket B so the fulltext resolver, source-aware model, and cache
contract are already stable. Add `src/transform/article/html.rs`,
`readability-rust`, the audited HTML-to-Markdown replacement, and the PMCID →
PMC article-page HTML fallback between XML and PDF.

Proof:

- PMCID-backed articles without usable XML can still produce saved Markdown from
  the PMC HTML page.
- the non-production HTML fixtures stay green as converter-quality guards.
- `make check` passes, and the updated `spec/entity/article.md` keeps the
  user-facing `Saved to:` contract stable.

## Open Decisions Captured

- The target chooses `--pdf` as the only PDF activator; there is no silent PDF
  fallback on the default path.
- The target chooses PMCID-derived PMC article pages as the first production HTML
  source rather than trying to generalize to arbitrary publisher pages.
- The target chooses `unpdf` as the first shipped PDF engine and keeps
  multi-engine arbitration out of scope for this migration.
