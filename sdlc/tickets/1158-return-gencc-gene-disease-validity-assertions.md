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
`conditional_refresh`, `retry_suppressed`, `refresh_deferred`, or
`identity_match`. `retry_suppressed` means the durable previous failed attempt
is still inside the one-day automatic retry window; `refresh_deferred` means
this request exhausted its lock/deadline budget while another owner could be
refreshing. Neither value claims that this request contacted GenCC. Timestamps
are RFC 3339 UTC.
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
GenCC absence can be concluded.` A stale read after this request's lock budget
expires uses exactly `GenCC refresh is still in progress; results come from the
last validated dataset.` Upstream bodies, URLs, local paths, parser details,
and lock errors never enter public messages.

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

`version_number` is parsed from an ASCII-decimal CSV scalar into `u32` and is
valid only in `1..=4_294_967_295`; signs, decimal points, exponent notation,
zero, and overflow invalidate the generation. The public `version` is a JSON
integer backed by that `u32`, and `id` is the validated `sgc_id`, one period,
and its canonical base-10 version with no leading zero. The authoritative
closed mapping treats `GENCC:100007` paired exactly with `Animal Model Only` as
`animal_model_only`; this pair is fixture-tested even though it was absent from
the 2026-09-05 provider capture, and is never inferred from neighboring CURIEs.

Only absolute `http` or `https` public-report and criteria URLs without user
information and no more than 2,048 bytes are exposed; a blank, oversized, or
malformed optional URL becomes null and is not fetched. Each assertion has
exactly the three link slots in its schema (source record, public report, and
assertion criteria), of which only the source record is required. The
source-record URL is constructed from validated SGC identity and version.
PMIDs accept comma-separated ASCII-decimal identifiers with an optional
ASCII-case-insensitive `PMID:` prefix and surrounding Unicode whitespace. The
digits parse as `u64` in `1..=18_446_744_073_709_551_615`; leading zeroes are
accepted and removed by canonical base-10 serialization, so `PMID:00042` and
`42` deduplicate to `42`, while an all-zero token, sign, non-ASCII digit,
decimal point, overflow, empty interior token, or prefix without digits
invalidates the row. Canonical PMIDs are de-duplicated in first-seen order and
receive the fixed PubMed URL. A malformed nonblank PMID field invalidates the
row. `disease_original_*`, every
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
from its current exact field list by appending `HGNC`, and extend the private
response to retain that wire value without changing a top-level public `Gene`
field. MyGene documents `HGNC` as the annotation key and shows a decimal string,
but the decoder deliberately accepts the provider shapes already encountered
for identifier fields: one JSON string, one unsigned JSON integer, or a flat
array containing strings and/or unsigned integers. Missing, `null`, or an empty
array means no HGNC ID. Strings accept, ASCII-case-insensitively, either bare
ASCII digits or one `HGNC:` prefix followed by ASCII digits; values parse as
`u32` in `1..=4_294_967_295` and normalize to `HGNC:<canonical decimal>`, so
`8109`, `"008109"`, and `"hgnc:8109"` are equivalent. Preserve wire order and
deduplicate normalized equivalents. A nested array, object, boolean, signed or
floating number, blank/non-ASCII digit, zero, overflow, another prefix, or a
mixed valid-and-invalid array makes resolved identity inconclusive rather than
silently becoming “no HGNC ID.” Exactly one distinct normalized ID is usable;
two or more are the existing multiple-value identity conflict.

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

Tests cover every scalar/array wire shape, missing/null/empty, numeric
normalization, exact and first-over bounds, equivalent-value deduplication,
mixed malformed arrays, multiple distinct IDs, case, alias input resolved to
the canonical symbol, matching symbol with wrong HGNC, matching HGNC with wrong
symbol, duplicate GenCC identities, and failed base identity.

## Dataset lifecycle and concurrency

The GenCC store lives at the trimmed nonblank `BIOMCP_GENCC_DIR` exactly when
set. Otherwise its exact root is `dirs::data_dir()/biomcp/gencc` (for example
`$XDG_DATA_HOME/biomcp/gencc` on Linux under the `dirs` crate contract). If the
environment override is absent and `dirs::data_dir()` returns `None`, ordinary
`gencc` returns unavailable and explicit sync exits nonzero without making a
network request; this durable source never falls back to a process temporary
directory. A missing selected root is created only while holding the private
parent/root setup lock, before any GET. Failure to create, permission, and
unsafe-path failures likewise make zero GETs because BioMCP could not durably
record quota-consuming work. It is a dedicated durable source dataset, not an
ordinary HTTP-response cache entry. Directories/lock files are current-user
private (`0700`/`0600` on Unix and the existing ACL equivalent on Windows).
Existing private-path helpers reject symlinks/reparse points, non-regular files,
unexpected hard links, and paths escaping the root. Metadata has no absolute
local paths.

Durable state has two deliberately different layers:

- `generations/<generation-id>/manifest.json` and its normalized bounded index
  are immutable after publication. The manifest records schema version,
  endpoint identity, body and index SHA-256, row/assertion counts, the
  successful `200` `retrieved_at`, the validated ETag and Last-Modified, and
  nullable upstream version. A generation ID is derived from the index hash
  plus an unguessable creation suffix and is used only as a local opaque name.
- Root `state.json` is the single mutable control record. It contains nullable
  `active_generation`, `checked_at`, `attempted_at`, and an exact last-attempt
  outcome (`success_200`, `success_304`, or `failure`). Its validators/version
  are read from the referenced immutable manifest. Only the lock owner replaces
  this record atomically. A failed attempt advances only `attempted_at` and
  `last_attempt=failure`; a `304` advances `checked_at` and `attempted_at` while
  retaining the generation and its `retrieved_at`; a `200` points all three
  event times at the newly published generation. Followers and lock contention
  never fabricate or advance any timestamp.

Raw CSV exists only as a private temporary file while it is bounded, hashed,
and parsed; it is removed after publication/failure and never becomes queryable
state. Thus original/notes/OMIM-only columns are not redistributed through
BioMCP state.

Freshness is exactly 604,800 seconds (7 x 24 hours) from `checked_at`; equality
is refresh-due. A wall clock earlier than a stored timestamp is refresh-due,
not indefinitely fresh. Failed ordinary refresh records `attempted_at` and
suppresses another automatic attempt for 86,400 seconds, consistent with the
official once-daily polling guidance. This failure state is durably recorded
whether or not a generation exists; the generation remains stale when present.
Equality is retry-eligible. A wall clock earlier than `attempted_at` remains
suppressed (rather than issuing repeated requests after rollback) until stored
time plus 86,400 seconds; explicit sync ignores suppression. An injected clock
tests one second before, exactly at, and one second after both boundaries and
clock rollback.

- First use with no valid generation takes the refresh lock, rechecks disk,
  performs one unconditional `GET`, validates completely, publishes, and
  queries. Failure is unavailable; partial data is not retained.
- A generation younger than 604,800 seconds is queried with no HTTP request.
- A due generation sends one conditional `GET` with `If-None-Match` and
  `If-Modified-Since`, because every accepted `200` must supply both validators;
  ETag remains the preferred validator semantically but neither stored value is
  omitted. A `304` is valid only for that conditional request backed by the
  same fully validated active generation. It must have no `Transfer-Encoding`,
  no `Content-Encoding` other than an explicit `identity`, Content-Length
  absent or exactly zero, and exactly zero streamed body bytes. Content-Type is
  ignored on the bodyless response. ETag and Last-Modified may each be absent;
  when supplied they must be syntactically valid and byte-for-byte equal to the
  stored values. A supplied documented dataset-version header must likewise
  equal the stored nullable version; appearance, disappearance, or change is a
  refresh failure requiring a `200`, never a metadata-only mutation. A valid
  `304` atomically replaces `state.json` with advanced
  `checked_at`/`attempted_at`, retaining `retrieved_at`, validators, version,
  and index. A valid `200` publishes a new generation. Any other response,
  one-byte body, invalid/mismatched validator, invalid header/body/schema,
  timeout, or publication failure keeps the old generation and makes this
  lookup stale after durably recording the failed attempt.
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

Refresh uses a process-local async mutex and cross-process advisory lock. An
ordinary leader obtains both within its one-gene deadline, reloads `state.json`
and the active generation after each acquisition, and decides again whether a
request is due. Same-process followers wait on the mutex; cross-process
followers wait on the advisory lock. A follower whose leader completed reloads
the durable result: it reports `local_query` after a successful leader refresh,
or `retry_suppressed` after the leader durably records failure, and never sends
a second GET. At deadline, a follower with a valid old generation queries it as
stale with `refresh_deferred`; without one it reports unavailable
`initial_download`. Lock wait/timeout is not an upstream attempt and never
changes `attempted_at`. These rules apply identically to concurrent first use:
one leader performs the unconditional GET, successful followers query its
generation, and failed followers observe its persisted suppression state.
Readers open only finalized immutable generation files and therefore never see
temporary or partially published state.

Publication creates a unique private sibling temporary generation, writes the
index and manifest with create-new semantics, fsyncs each file and then that
directory, renames it to its final generation name, fsyncs the generations
directory, and reopens/revalidates both hashes and the manifest. It next writes
a unique private `state.json` temporary, fsyncs it, atomically renames it over
`state.json`, and fsyncs the root. The old state remains authoritative until
that last rename is durable. A `304` and a failed attempt use the same
write/fsync/rename/root-fsync sequence for state only; if persisting a failed
attempt itself fails, the request still fails/stales but retry suppression is
not claimed. Every exit removes its owned raw/state/generation temporaries
after worker settlement.

Startup validates `state.json`, then validates its referenced finalized
generation by manifest and hashes. If state is missing/corrupt or references an
invalid generation, recovery scans only finalized generation directories and
chooses the greatest valid immutable `retrieved_at`, breaking ties by bytewise
generation ID. It reconstructs `checked_at` and `attempted_at` from that
generation's immutable `retrieved_at` and writes a fresh state record; it never
recovers a later 304/failure timestamp from a partial state file. If no valid
generation exists, acquisition starts from none (while an independently valid
failure-only `state.json` still enforces retry suppression). Under the
cross-process lock, post-publication/startup maintenance retains the active and
newest other valid generation, removes abandoned temporaries and all other
invalid/old generations, and fsyncs every parent directory whose entries
changed. Cleanup failure is logged and leaves extra private files; it never
invalidates the already durable active state or deletes either retained copy.
Crash injection before/after every fsync and rename proves recovery selects
only a complete old or new generation.

One direct gene request gives its GenCC section one existing configurable
eight-second optional-enrichment deadline, starting when that section future is
first polled and covering mutex/advisory-lock wait, HTTP, streaming,
validation, publication, identity resolution, and index query. Batch retains
the existing maximum of ten input IDs and `join_all` behavior: all at-most-ten
gene futures are polled concurrently, each has its own eight-second GenCC
deadline, and there is no additional shared batch deadline. They share the
same store mutex and therefore at most one GenCC GET; local immutable queries
may proceed concurrently after reload. Explicit sync instead has one
120-second end-to-end deadline and waits for both locks; timeout exits nonzero
with or without a generation, preserves old data when present, publishes
nothing, and is not recorded as an upstream attempt if no request began. HTTP
streaming observes cancellation. Blocking parse/index work receives a
cancellation flag checked at least once per row; on timeout/drop BioMCP requests
cancellation and awaits worker settlement. No detached request, worker, lock,
publisher, or temp file survives completion.

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

`GenCC` is the exact source label. In the table, “query” means the immutable
local index may be queried after lifecycle resolution; it never means a GenCC
network query. `stale-failed`, `stale-progress`, `unavailable`, and `identity`
mean the four exact messages defined above. Same-process mutex followers and
cross-process advisory-lock followers use the same rows. The lock owner alone
updates timestamps. `section_outcomes.gencc` and its matching
`_meta.section_sources` entry use this complete mapping:

| generation at entry | lifecycle decision for this call | local query | identity / matches | status (`freshness/result/operation`) | outcome | sources | message |
|---|---|---|---|---|---|---|---|
| none | root unavailable/unsafe before GET | no | n/a | `unavailable/unknown/initial_download` | `unavailable` | `[]` | unavailable |
| none | leader valid first `200` | yes | conclusive / >0 | `fresh/data/initial_download` | `data` | `["GenCC"]` | null |
| none | leader valid first `200` | yes | conclusive / 0 | `fresh/empty/initial_download` | `empty` | `["GenCC"]` | null |
| none | leader GET/validation/publication fails or times out | no | n/a | `unavailable/unknown/initial_download` | `unavailable` | `[]` | unavailable |
| none | durable failed attempt still suppresses retry | no | n/a | `unavailable/unknown/retry_suppressed` | `unavailable` | `[]` | unavailable |
| none | first-use follower reloads leader's valid `200` | yes | conclusive / >0 | `fresh/data/local_query` | `data` | `["GenCC"]` | null |
| none | first-use follower reloads leader's valid `200` | yes | conclusive / 0 | `fresh/empty/local_query` | `empty` | `["GenCC"]` | null |
| none | first-use follower reloads leader's persisted failure | no | n/a | `unavailable/unknown/retry_suppressed` | `unavailable` | `[]` | unavailable |
| none | mutex/advisory-lock wait reaches per-gene deadline | no | n/a | `unavailable/unknown/initial_download` | `unavailable` | `[]` | unavailable |
| fresh | no refresh due | yes | conclusive / >0 | `fresh/data/local_query` | `data` | `["GenCC"]` | null |
| fresh | no refresh due | yes | conclusive / 0 | `fresh/empty/local_query` | `empty` | `["GenCC"]` | null |
| due | leader valid `304` | yes | conclusive / >0 | `fresh/data/conditional_refresh` | `data` | `["GenCC"]` | null |
| due | leader valid `304` | yes | conclusive / 0 | `fresh/empty/conditional_refresh` | `empty` | `["GenCC"]` | null |
| due | leader valid replacement `200` | yes | conclusive / >0 | `fresh/data/conditional_refresh` | `data` | `["GenCC"]` | null |
| due | leader valid replacement `200` | yes | conclusive / 0 | `fresh/empty/conditional_refresh` | `empty` | `["GenCC"]` | null |
| due | leader refresh fails; old retained | yes | conclusive / >0 | `stale/data/conditional_refresh` | `degraded` | `["GenCC"]` | stale-failed |
| due | leader refresh fails; old retained | yes | conclusive / 0 | `stale/empty/conditional_refresh` | `unavailable` | `[]` | stale-failed |
| due | failed attempt still suppresses retry | yes | conclusive / >0 | `stale/data/retry_suppressed` | `degraded` | `["GenCC"]` | stale-failed |
| due | failed attempt still suppresses retry | yes | conclusive / 0 | `stale/empty/retry_suppressed` | `unavailable` | `[]` | stale-failed |
| due | follower reloads leader's successful `200`/`304` | yes | conclusive / >0 | `fresh/data/local_query` | `data` | `["GenCC"]` | null |
| due | follower reloads leader's successful `200`/`304` | yes | conclusive / 0 | `fresh/empty/local_query` | `empty` | `["GenCC"]` | null |
| due | follower reloads leader's persisted failure | yes | conclusive / >0 | `stale/data/retry_suppressed` | `degraded` | `["GenCC"]` | stale-failed |
| due | follower reloads leader's persisted failure | yes | conclusive / 0 | `stale/empty/retry_suppressed` | `unavailable` | `[]` | stale-failed |
| due | mutex/advisory-lock wait reaches deadline; old retained | yes | conclusive / >0 | `stale/data/refresh_deferred` | `degraded` | `["GenCC"]` | stale-progress |
| due | mutex/advisory-lock wait reaches deadline; old retained | yes | conclusive / 0 | `stale/empty/refresh_deferred` | `unavailable` | `[]` | stale-progress |
| any valid | lifecycle permits local read | yes, identity index only | inconclusive / n/a | `unavailable/unknown/identity_match` | `unavailable` | `[]` | identity |

The stale-zero rows are intentional: old positive assertions may be shown with
a warning, but zero matches in old data cannot establish current absence.

GenCC evidence URLs are appended after the existing NCBI Gene, UniProt,
Ensembl, and OMIM evidence in this exact order when the GenCC outcome is
`data`, `empty`, or `degraded`: `(GenCC dataset,
https://thegencc.org/download)` once; then, for each returned assertion in
public assertion order, its `(GenCC submission, source_record_url)`, optional
`(GenCC public report, public_report_url)`, optional `(GenCC assertion
criteria, assertion_criteria_url)`, and each `(PubMed, publication.url)` in
publication order. Deduplicate globally by exact final URL string in first-seen
order, retaining the first label; this includes collisions with an existing
base-gene URL and across assertions/link roles. The download URL remains for a
healthy empty because the validated dataset is its evidence. Any `unavailable`
GenCC outcome contributes no GenCC or assertion evidence URL. Status validators
come from the active immutable manifest and event timestamps from durable
`state.json`; every output surface must agree.

Explicit sync does not manufacture a `Gene.gencc` status row. On a valid `200`
its existing sync envelope uses `source: "gencc"`, `status: "synchronized"`,
and `changed: true`; on a valid `304` it uses the same envelope with
`changed: false`. Human output distinguishes “updated” from “already current.”
Any GET, validation, publication, root, or 120-second lock/deadline failure
uses the existing typed command error and nonzero exit, with no success
envelope. With an old generation that generation remains active; without one
none is invented. These outcomes are tested for lock timeout both with and
without a generation.

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
- Pure tests pin the schemas, enums, nulls, dates, all nine classifications
  (including the authoritative `GENCC:100007` Animal Model Only pair), order,
  version/duplicate rules, public `u32` version exact maximum and maximum-plus-
  one, PMID zero/leading-zero/`u64` maximum/overflow behavior, 100/101 assertion
  and 128/129 unique-PMID boundaries, and identity rules. ODC1 returns all three
  separate Strong/autosomal-dominant assertions and submitters.
- Adversarial tests cover every first-over-bound value; header/schema/legacy,
  quote/UTF-8, content type/archive/compression, validator/status, CURIE/date/
  PMID/classification, duplicate, URL, cache path/link, redirect, leak, and
  timeout failures. Bad optional report/criteria URLs become null; structural or
  required-field failures invalidate the generation.
- An injected-clock HTTP fixture proves initial download, fresh reuse, exact
  seven-day edges, both conditional headers, zero-body `304`, Content-Length
  zero/one, transfer/content encoding, absent/equal/mismatched validators and
  version on `304`, replacement `200`, durable daily retry suppression with and
  without old data, rollback, sync, `--no-cache`, and health HEAD. Its log
  rejects unplanned routes and proves normal/suppressed/follower reuse consumes
  no download.
- MyGene fixtures prove the appended `HGNC` projection and every accepted and
  rejected scalar/string/flat-array shape, normalization bound, deduplication,
  and conflict before exercising the combined identity index.
- Thread/task and subprocess tests separately prove one GET for concurrent
  first use and refresh, same-process successful/failed followers,
  cross-process successful/failed followers, old-generation contention,
  ordinary lock timeout with and without a generation, explicit-sync lock
  timeout with and without a generation, exact operation/message/timestamp
  preservation, per-gene eight-second batch deadlines for ten concurrent items,
  no shared batch deadline, immutable concurrent reads, private permissions,
  and no live work or temp files after success/timeout/cancellation. Crash
  injection before and after each file/directory fsync, generation rename,
  state rename, and cleanup recovers only a complete old or new generation and
  proves failure-attempt persistence plus deterministic fallback ordering.
- CLI direct/all/batch Markdown/JSON, raw MCP text/JSON, typed MCP, help/list,
  schema, exact evidence-URL labels/order/global deduplication for report,
  criteria, duplicate PubMed and capped assertions, docs, and 1159 coexistence
  are fixture-tested without live provider access.
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

- Initial design review: rejected; the first revision established the public
  submission schema and broad lifecycle but did not fully specify follower and
  lock outcomes, mutable refresh state versus immutable generations, MyGene's
  HGNC wire union, evidence-URL projection, bodyless 304 validation, direct
  versus batch deadlines, or several numeric/root boundaries.
- Design revision: resolves those gaps with the exhaustive lifecycle table,
  two-layer durable state protocol and crash ordering, exact HGNC/PMID/version
  parsing, authoritative Animal Model Only mapping, exact GenCC evidence URL
  labels/order/deduplication, strict 304 rules, and the repository's existing
  per-gene eight-second/max-ten-concurrent batch behavior. Pending fresh
  independent design re-review.
- Code review: pending.
