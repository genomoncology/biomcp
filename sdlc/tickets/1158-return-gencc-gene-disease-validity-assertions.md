---
flow: build
priority: 7
---

# Return GenCC gene-disease validity assertions

## Goal

A gene request can retrieve public, submission-level Gene Curation Coalition
(GenCC) gene-disease validity assertions without changing the source-specific
ClinGen result. On 2026-09-04, BioMCP returned a healthy empty ClinGen section
for ODC1 even though GenCC publishes public ODC1 assertions. An absence from
ClinGen is not an absence from other public curation groups.

`biomcp get gene ODC1 gencc` is the new direct surface. `clingen` remains the
existing ClinGen-only section. Neither source is collapsed into a consensus or
a strongest-classification claim.

## Current provider facts

The evidence needed to implement this ticket is recorded here because its
historical issue is not present on current `main`:

- GenCC's official download page is `https://thegencc.org/download`. On
  2026-09-05 it documented the recommended new-format CSV endpoint as
  `https://thegencc.org/download/action/submissions-export-csv?format=new`, a
  weekly publication cadence, `ETag`/`If-None-Match` and
  `Last-Modified`/`If-Modified-Since` conditional requests, quota-free `304`
  responses and `HEAD` requests, and a 20-successful-downloads-per-IP daily
  quota. It says polling more than once per day has no benefit and that a query
  API is not yet available.
- The same official page says the data are CC0 1.0, requests attribution to
  GenCC and contributing sources, excludes restricted OMIM data from the
  downloadable product, and warns against direct diagnostic or medical
  decision-making use. `https://thegencc.org/terms` repeats the CC0,
  attribution, and clinical-use boundaries.
- A quota-exempt `HEAD` on 2026-09-05 returned `200`, `Content-Type: text/csv`,
  `Content-Length: 26825294`, ETag
  `"eef7e2a136ebf84effde839603b84777"`, and Last-Modified
  `Sun, 30 Aug 2026 06:00:29 GMT`. The response's current rate-limit headers are
  not treated as a stronger promise than the official 20-download limit.
- One new-format CSV capture from that endpoint on 2026-09-05 was 26,825,294
  bytes, 46,548 data rows, and SHA-256
  `e6b8834da0d156430e10795d20077e6b9b8c6c25f5e8b896c312654de11c41a2`.
  Its exact 31-column header was:

  ```text
  sgc_id,version_number,gene_curie,gene_symbol,disease_curie,disease_title,disease_original_curie,disease_original_title,classification_curie,classification_title,moi_curie,moi_title,submitter_curie,submitter_title,submitted_as_hgnc_id,submitted_as_hgnc_symbol,submitted_as_disease_id,submitted_as_disease_name,submitted_as_moi_id,submitted_as_moi_name,submitted_as_submitter_id,submitted_as_submitter_name,submitted_as_classification_id,submitted_as_classification_name,submitted_as_date,submitted_as_public_report_url,submitted_as_notes,submitted_as_pmids,submitted_as_assertion_criteria_url,submitted_as_submission_id,submitted_run_date
  ```

- That capture contains three ODC1 (`HGNC:8109`) submissions for
  `MONDO:0033642`, all Strong (`GENCC:100002`) and autosomal dominant
  (`HP:0000006`). `SGC-113621` version 1 is from Labcorp Genetics (formerly
  Invitae), cites PMIDs 30239107 and 30475435, and is visible at
  `https://thegencc.org/submissions/SGC-113621.1`. The other submitters are
  PanelApp Australia and G2P. This proves why submissions remain separate.
- The official statistics page, `https://thegencc.org/statistics`, publishes
  the nine standardized classifications represented by the closed mapping
  below. The 2026-09-05 capture contained all except Animal Model Only. The
  GenCC paper (`https://pmc.ncbi.nlm.nih.gov/articles/PMC7613247/`) describes
  submission-level gene, disease, classification, inheritance, submitter,
  report, criteria, PMID, and date fields and GenCC's identifier validation.

The implementation adds a minimized, receipt-backed new-format provider
fixture rather than depending on these observations or a live request in tests.

## Public result contract

When `gencc` is requested, `Gene.gencc` is always present, even when the source
is unavailable. It has this additive shape; nullable fields serialize as
explicit JSON `null`, and arrays serialize as arrays rather than being omitted:

```json
{
  "gencc": {
    "assertions": [],
    "total_matching_assertions": 0,
    "truncated": false,
    "status": {
      "freshness": "fresh",
      "result": "empty",
      "operation": "local_query",
      "checked_at": "2026-09-05T22:51:21Z",
      "retrieved_at": "2026-09-05T22:51:21Z",
      "attempted_at": "2026-09-05T22:51:21Z",
      "etag": "\"eef7e2a136ebf84effde839603b84777\"",
      "last_modified": "Sun, 30 Aug 2026 06:00:29 GMT",
      "upstream_version": null,
      "message": null
    }
  }
}
```

`status.freshness` is the closed enum `fresh`, `stale`, or `unavailable`.
`status.result` is the closed enum `data`, `empty`, or `unknown`.
`status.operation` is the closed enum `local_query`, `initial_download`,
`conditional_refresh`, or `identity_match`. Timestamps are RFC 3339 UTC.
`checked_at` is the last successful `200` or `304`; `retrieved_at` is the
successful `200` whose body produced the active generation; `attempted_at` is
the last completed refresh attempt. They are null when no event exists. ETag is
preserved as a valid HTTP entity tag, including weakness and quotes;
Last-Modified is the validated HTTP-date. `upstream_version` contains a
nonblank provider dataset version only if GenCC supplies a documented version
response header. It is null rather than synthesized from validators, row
versions, or dates.

`message` is null for fresh data/empty. Stale results use exactly `GenCC refresh
failed; results come from the last validated dataset.` Unavailable acquisition
uses exactly `GenCC data is unavailable; no GenCC absence can be concluded.` A
failed identity match uses exactly `GenCC gene identity is inconclusive; no
GenCC absence can be concluded.` Upstream bodies, URLs, local paths, parser
details, and lock errors never enter public messages.

Each `assertions` element has exactly this public shape:

```json
{
  "id": "SGC-113621.1",
  "sgc_id": "SGC-113621",
  "version": 1,
  "gene": {"id": "HGNC:8109", "label": "ODC1"},
  "disease": {
    "id": "MONDO:0033642",
    "label": "neurodevelopmental disorder with alopecia and brain abnormalities"
  },
  "classification": {
    "id": "GENCC:100002",
    "label": "Strong",
    "code": "strong"
  },
  "mode_of_inheritance": {
    "id": "HP:0000006",
    "label": "Autosomal dominant"
  },
  "submitter": {
    "id": "GENCC:000106",
    "label": "Labcorp Genetics (formerly Invitae)"
  },
  "evaluated_date": "2021-08-04",
  "submitted_date": "2023-11-30",
  "source_record_url": "https://thegencc.org/submissions/SGC-113621.1",
  "public_report_url": null,
  "assertion_criteria_url": "https://view.publitas.com/invitae/invitaeposter_nsgc2019_curatingthehumangenome/page/1",
  "publications": [
    {"pmid": "30239107", "url": "https://pubmed.ncbi.nlm.nih.gov/30239107/"},
    {"pmid": "30475435", "url": "https://pubmed.ncbi.nlm.nih.gov/30475435/"}
  ]
}
```

The classification mapping is exact and closed:

| CURIE | provider label | public `code` |
|---|---|---|
| `GENCC:100001` | `Definitive` | `definitive` |
| `GENCC:100002` | `Strong` | `strong` |
| `GENCC:100003` | `Moderate` | `moderate` |
| `GENCC:100004` | `Limited` | `limited` |
| `GENCC:100005` | `Disputed Evidence` | `disputed_evidence` |
| `GENCC:100006` | `Refuted Evidence` | `refuted_evidence` |
| `GENCC:100007` | `Animal Model Only` | `animal_model_only` |
| `GENCC:100008` | `No Known Disease Relationship` | `no_known_disease_relationship` |
| `GENCC:100009` | `Supportive` | `supportive` |

An unknown CURIE, a known CURIE paired with another label, or a known label
paired with another CURIE invalidates the candidate generation. The public
contract never ranks these values and never computes consensus.

Required row values are `sgc_id`, positive `version_number`, `gene_curie`,
`gene_symbol`, `disease_curie`, `disease_title`, the classification pair, the
mode-of-inheritance pair, and the submitter pair. IDs match exact namespaces:
`SGC-[1-9][0-9]*`, `HGNC:[1-9][0-9]*`, `MONDO:[0-9]{7}`,
`GENCC:[0-9]{6}`, and `HP:[0-9]{7}`. Required labels are trimmed, nonblank,
control-free strings. `evaluated_date` is the date component of a valid
`submitted_as_date` in `YYYY-MM-DD`, GenCC's space-separated timestamp, or RFC
3339 form; `submitted_date` is the date from a valid `submitted_run_date`. A
blank optional date becomes null; a nonblank malformed date invalidates the row
and therefore the generation.

Only absolute `http` or `https` public-report and criteria URLs without user
information and no more than 2,048 bytes are exposed; a blank, oversized, or
malformed optional URL becomes null and is not fetched. Each assertion has
exactly the three link slots in its schema (source record, public report, and
assertion criteria), of which only the source record is required. The
source-record URL is constructed from validated SGC identity and version.
PMIDs accept comma-separated decimal identifiers with an optional
`PMID:` prefix and Unicode or ASCII whitespace; they are canonicalized to
digits, de-duplicated in first-seen order, and receive the fixed PubMed URL. A
malformed nonblank PMID token invalidates the row. `disease_original_*`, every
other `submitted_as_*` field, free-text notes, and original OMIM
labels/identifiers are never placed in the normalized cache or public output.

Byte-equivalent duplicate rows for one `(sgc_id, version)` become one row;
conflicting duplicates invalidate the generation. When several versions of an
SGC ID exist, only its greatest positive version is current. This never combines
different SGC IDs, even when their other fields match. Current assertions are
ordered by evaluated date descending (null last), submitted date descending
(null last), submitter label case-insensitively then bytewise, disease ID,
inheritance ID, SGC ID, and version descending. A response returns at most 100
assertions after de-duplication/version selection; `total_matching_assertions`
is the pre-cap count and `truncated` is true exactly above 100.

## Identity matching

GenCC matching uses the canonical identity already resolved for the Gene card,
not the caller's spelling or aliases. Extend the private MyGene fetch projection
and response to retain its HGNC identifier without changing a top-level public
`Gene` field.

- With one canonical symbol and one valid HGNC CURIE, a row matches only when
  both `gene_symbol` (ASCII case-insensitive after trimming) and `gene_curie`
  match. A row where only one side matches is an identity conflict, not data or
  a healthy empty.
- With a canonical symbol but no HGNC ID, symbol lookup is conclusive only when
  every current GenCC row with that symbol has the same valid `gene_curie`.
  That ID is used. Zero symbol rows is a conclusive healthy empty; two HGNC IDs
  for one symbol is an identity conflict.
- A missing/invalid resolved symbol, multiple resolved MyGene HGNC values, or
  disagreement between resolved HGNC and the GenCC symbol index yields
  `unavailable/unknown/identity_match`, no assertions, and no absence claim. If
  the base MyGene lookup fails, existing Gene-card failure remains authoritative
  and no GenCC refresh/query starts.

Tests cover case, alias input resolved to the canonical symbol, matching symbol
with wrong HGNC, matching HGNC with wrong symbol, duplicate identities, and
failed base identity.

## Dataset lifecycle and concurrency

The GenCC store lives at `BIOMCP_GENCC_DIR` when set, otherwise below the
platform BioMCP data directory. It is a dedicated durable source dataset, not
an ordinary HTTP-response cache entry. Directories/lock files are current-user
private (`0700`/`0600` on Unix and the existing ACL equivalent on Windows).
Existing private-path helpers reject symlinks/reparse points, non-regular files,
unexpected hard links, and paths escaping the root. Metadata has no absolute
local paths.

One validated generation contains only the normalized bounded index and a
manifest. The manifest records schema version, body and index SHA-256, row and
assertion counts, timestamps, validators, nullable upstream version, and
endpoint identity. Raw CSV exists only as a private temporary file while it is
bounded, hashed, and parsed; it is removed after publication/failure and never
becomes queryable state. Thus original/notes/OMIM-only columns are not
redistributed through BioMCP state.

Freshness is exactly 604,800 seconds (7 x 24 hours) from `checked_at`; equality
is refresh-due. A wall clock earlier than a stored timestamp is refresh-due,
not indefinitely fresh. Failed ordinary refresh records `attempted_at` and
suppresses another automatic attempt for 86,400 seconds, consistent with the
official once-daily polling guidance. The generation remains stale then. An
injected clock tests one second before, exactly at, and one second after both
boundaries and clock rollback.

- First use with no valid generation takes the refresh lock, rechecks disk,
  performs one unconditional `GET`, validates completely, publishes, and
  queries. Failure is unavailable; partial data is not retained.
- A generation younger than 604,800 seconds is queried with no HTTP request.
- A due generation sends one conditional `GET`, preferring `If-None-Match` and
  also sending `If-Modified-Since` when both validators exist. A `304` is valid
  only for a conditional request backed by a valid generation and no meaningful
  body. It atomically updates `checked_at`/`attempted_at` while retaining
  `retrieved_at` and the index. A valid `200` publishes a new generation. Any
  other response, invalid header/body/schema, timeout, or publication failure
  keeps the old generation and makes this lookup stale.
- `biomcp gencc sync` always takes the lock and performs the same conditional
  revalidation even when fresh. `304` and valid `200` succeed. Refresh failure
  exits nonzero even if prior data remains; it never destroys that generation.
  Explicit sync has a 120-second deadline.
- Global `--no-cache` still bypasses HTTP-response/session caches. It does not
  ignore, delete, or force-refresh this durable dataset; callers use
  `biomcp gencc sync`. Configuration/CLI docs name this exception.
- `biomcp health --api GenCC` makes one quota-exempt `HEAD` of the exact
  new-format endpoint under the existing health deadline and labels the
  affected surface `gene gencc section`. Success requires expected origin, CSV
  content type, ETag, and Last-Modified. Health never downloads, publishes, or
  changes timestamps. Normal all-provider health includes the descriptor.

Refresh uses a process-local async mutex and cross-process advisory lock. After
either lock, the contender reloads the active manifest so threads/tasks and
processes coalesce to one request. With no generation a caller waits within its
deadline. With validated old data it may query the immutable generation while
another process refreshes and reports stale when due; readers never see temp
state.

Publication writes a unique private generation directory, fsyncs files and the
directory, and atomically replaces a current-generation pointer only after
hash/manifest revalidation. The prior valid generation remains until the new
pointer is durable. Startup ignores/removes abandoned temporaries, validates
the pointer, and falls back to the newest completely valid prior generation if
the pointer is missing/corrupt. It never chooses partially written timestamps.
Maintenance retains active and one prior generation and removes older invalid
state only under the cross-process lock.

Direct and batch requests share the existing configurable eight-second gene
optional-enrichment deadline across lock wait, download/revalidation,
validation, parsing, and query. HTTP streaming observes cancellation. Blocking
parse/index work receives a cancellation flag checked at least once per row;
on timeout/drop BioMCP requests cancellation and awaits worker settlement. No
detached request, worker, lock, publisher, or temp file survives completion.

## Resource and transport bounds

- Production uses only HTTPS host `thegencc.org`, exact path
  `/download/action/submissions-export-csv`, and exact query `format=new`.
  Fixture overrides are test-only. At most three redirects are followed, each
  staying HTTPS on that host and exact path/query; credentials/fragments are
  rejected. DNS/private-address and URL policy checks apply on every hop.
- A `200` requires `text/csv` (parameters allowed), valid ETag and Last-Modified,
  identity content encoding, and Content-Length <=64 MiB when present. Streaming
  cuts off at 64 MiB regardless. `206`, compressed/archive bodies, HTML, XLSX,
  TSV, and legacy CSV are rejected. `Content-Disposition` is optional and is
  ignored rather than being used as a local filename.
- CSV is UTF-8 (initial BOM allowed), RFC 4180, and has exactly one header plus
  at most 100,000 rows. It has the exact 31 columns above, once and in order.
  Trailing blank lines are allowed; ragged rows, duplicate headers, controls,
  invalid UTF-8/quoting, unknown/missing/reordered columns, or legacy `uuid`
  invalidate the generation.
- A raw field is <=16 KiB, normalized labels <=1,024 Unicode scalars, and each
  exposed link is <=2,048 bytes. An assertion has <=128 unique PMIDs. The first
  byte/row/field/publication above a fail-closed input bound is rejected; an
  invalid optional link follows the explicit null rule above. At most 100,000
  normalized assertions can enter a generation by the row bound, and the
  100-assertion response cap is separate.
- All errors are typed internally and mapped to stable status. No body excerpt,
  note, OMIM-original field, local path, or credential appears in outputs,
  stderr, or debug logs.

## Freshness, outcome, and provenance truth table

`GenCC` is the exact source label. `section_outcomes.gencc` and its matching
`_meta.section_sources` entry use this complete mapping:

| generation | action | identity | matches | status (`freshness/result/operation`) | section outcome | sources | message |
|---|---|---|---:|---|---|---|---|
| none | valid first `200` | conclusive | >0 | `fresh/data/initial_download` | `data` | `["GenCC"]` | null |
| none | valid first `200` | conclusive | 0 | `fresh/empty/initial_download` | `empty` | `["GenCC"]` | null |
| none | failure/timeout | n/a | n/a | `unavailable/unknown/initial_download` | `unavailable` | `[]` | unavailable |
| fresh | local read | conclusive | >0 | `fresh/data/local_query` | `data` | `["GenCC"]` | null |
| fresh | local read | conclusive | 0 | `fresh/empty/local_query` | `empty` | `["GenCC"]` | null |
| due | `304` | conclusive | >0 | `fresh/data/conditional_refresh` | `data` | `["GenCC"]` | null |
| due | `304` | conclusive | 0 | `fresh/empty/conditional_refresh` | `empty` | `["GenCC"]` | null |
| due | valid replacement `200` | conclusive | >0 | `fresh/data/conditional_refresh` | `data` | `["GenCC"]` | null |
| due | valid replacement `200` | conclusive | 0 | `fresh/empty/conditional_refresh` | `empty` | `["GenCC"]` | null |
| due | refresh fails, old retained | conclusive | >0 | `stale/data/conditional_refresh` | `degraded` | `["GenCC"]` | stale |
| due | refresh fails, old retained | conclusive | 0 | `stale/empty/conditional_refresh` | `unavailable` | `[]` | stale |
| any valid | retained/local | inconclusive | n/a | `unavailable/unknown/identity_match` | `unavailable` | `[]` | identity |

The stale-zero row is intentional: old positive assertions may be shown with a
warning, but zero matches in old data cannot establish current absence. For
`data`, `empty`, and `degraded`, provenance includes the GenCC download page and
retained source-record URLs. `unavailable` has no successful source attribution
or assertion evidence URL. Validators/timestamps come only from the manifest
and agree across all surfaces.

## Surfaces and coexistence

- Add `gencc` to the canonical Gene section enum/names, outcomes/source
  registry, direct and parallel/all dispatch, and renderer. It is included by
  `get gene <symbol> all` and accepted by
  `batch gene <ids> --sections gencc` without reordering batch results.
- Markdown renders `## GenCC gene-disease validity`, warning before stale data,
  separate submitter rows, IDs, dates, publications and links, and `Showing 100
  of N assertions` when capped. It makes no unqualified match/consensus/strongest
  claim.
- CLI JSON, batch JSON, raw MCP `biomcp` text/JSON, and typed MCP `get` for
  `entity: gene` accept/expose `gencc`. Update the typed section enum/schema; add
  no MCP tool. All surfaces preserve identical status, nulls, order, cap,
  outcome, provenance, and warnings.
- Update `biomcp list gene`, get/batch help, MCP help/list,
  `skills/schemas/gene.json`, registry/metadata, gene/CLI guides,
  licensing/data-source/troubleshooting references, checked llms artifacts, and
  a GenCC source page with the exact command, schema, lifecycle, CC0 attribution
  request, quota, stale-zero rule, and non-diagnostic boundary.
- Ticket 1159's `clingen.validity_status`/`dosage_status` remain ClinGen-only.
  `Gene.clingen`, ClinGen outcomes/provenance/requests/timeouts/rendering do not
  change. A combined `clingen gencc` fixture proves either section's
  data/empty/failure cannot overwrite the other's result or provenance.

## Acceptance

- A receipt-backed minimized new-format capture contains the three ODC1 rows
  and multiple submitters. Its inventory records endpoint, request
  method/headers, capture date, response status/content type/validators, full
  original length/hash, minimized hash, and deterministic minimization. Failure
  variants are labeled derived.
- Pure tests pin the schemas, enums, nulls, dates/PMIDs, all nine
  classifications, order, version/duplicate rules, 100/101 assertion and
  128/129 PMID boundaries, and identity rules. ODC1 returns all three separate
  Strong/autosomal-dominant assertions and submitters.
- Adversarial tests cover every first-over-bound value; header/schema/legacy,
  quote/UTF-8, content type/archive/compression, validator/status, CURIE/date/
  PMID/classification, duplicate, URL, cache path/link, redirect, leak, and
  timeout failures. Bad optional report/criteria URLs become null; structural or
  required-field failures invalidate the generation.
- An injected-clock HTTP fixture proves initial download, fresh reuse, exact
  seven-day edges, both conditional headers, valid/invalid `304`, replacement
  `200`, daily retry suppression, rollback, sync, `--no-cache`, and health HEAD.
  Its log rejects unplanned routes and proves normal reuse consumes no download.
- Thread/task and subprocess tests prove one GET for concurrent first use and
  refresh, immutable reads, bounded waits, private permissions, and no live work
  or temp files after success/timeout/cancellation. Crash injection before each
  fsync, pointer replacement, and after replacement recovers only a complete
  old or new generation.
- CLI direct/all/batch Markdown/JSON, raw MCP text/JSON, typed MCP, help/list,
  schema, provenance, docs, and 1159 coexistence are fixture-tested without live
  provider access.
- New Rust modules remain <=1,000 lines. Do not raise any existing exact
  source-size baseline or CLI 700-line cap. `cargo package --list --allow-dirty`
  remains exactly 1,300 files; additions require package-neutral consolidation.
- Focused tests pass, then `make lint`, `make test`, `make spec`, and
  `make full-feature-check`. Live verification is optional and must not spend a
  GenCC successful-download quota merely to close the ticket.

## Boundaries

This ticket adds public GenCC validity evidence and one explicit sync command.
It does not merge GenCC/ClinGen, compute consensus or strongest assertion,
infer diagnosis/treatment, add disease-side search, expose notes/original
disease fields, redistribute OMIM content, add an MCP tool, change ClinGen
semantics, or confuse source availability with validity.

Dependencies: none. Ticket 1159 is independent and may land before or after
this ticket; coexistence tests adapt to the additive ClinGen status schema on
the implementation base.

## Review

- Design review: pending re-review after this revision.
- Code review: pending.
