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
Last-Modified is the validated HTTP-date. GenCC documents no dataset-version
response header for this export, so this ticket does not read one:
`upstream_version` is always JSON `null` in public status and immutable
manifests. It is never synthesized from an undocumented header, validators,
row versions, or dates. Adding a provider version later requires a separate
contract change backed by GenCC documentation.

`message` is null for fresh data/empty. Stale results use exactly `GenCC refresh
failed; results come from the last validated dataset.` Unavailable acquisition
uses exactly `GenCC data is unavailable; no GenCC absence can be concluded.` A
failed identity match uses exactly `GenCC gene identity is inconclusive; no
GenCC absence can be concluded.` A stale read after this request's lock budget
expires uses exactly `GenCC refresh is still in progress; results come from the
last validated dataset.` A no-generation read after this request's lock budget
expires uses exactly `GenCC refresh is still in progress; no GenCC absence can
be concluded.` Upstream bodies, URLs, local paths, parser details, and lock
errors never enter public messages.

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
`GENCC:[0-9]{6}`, and `HP:[0-9]{7}`. Required labels are Unicode-trimmed,
nonblank, control-free strings under the exact normalization rules below.
`evaluated_date` is the date component of a valid `submitted_as_date`;
`submitted_date` is the date component of a valid `submitted_run_date`. Each
accepts exactly one of: ten ASCII bytes `YYYY-MM-DD`; nineteen ASCII bytes
`YYYY-MM-DD HH:MM:SS`, where the separator is one U+0020 SPACE and the time is
interpreted as UTC; or an RFC 3339 date-time with an explicit `Z` or numeric
offset. Calendar dates and hours/minutes/seconds must be real (`00:00:00`
through `23:59:59`); the exact space form has no fraction or offset, while RFC
3339 fraction and offset rules are those of the repository's pinned date-time
parser. Date-only and space-form inputs retain their parsed calendar date; an
RFC 3339 instant is converted to UTC before its calendar date is selected. The
public value is formatted canonically as `YYYY-MM-DD`. A
Unicode-whitespace-only optional date becomes null; any other malformed value
invalidates the row and therefore the generation.

`version_number` is parsed from an ASCII-decimal CSV scalar into `u32` and is
valid only in `1..=4_294_967_295`; signs, decimal points, exponent notation,
zero, and overflow invalidate the generation. The public `version` is a JSON
integer backed by that `u32`, and `id` is the validated `sgc_id`, one period,
and its canonical base-10 version with no leading zero. The authoritative
closed mapping treats `GENCC:100007` paired exactly with `Animal Model Only` as
`animal_model_only`; this pair is fixture-tested even though it was absent from
the 2026-09-05 provider capture, and is never inferred from neighboring CURIEs.

Only absolute `http` or `https` public-report and criteria URLs without user
information and no more than 2,048 UTF-8 bytes after Unicode trimming are
exposed; a blank, oversized, or malformed optional URL becomes null and is not
fetched. URL parsing is validation only: the public value preserves the exact
trimmed provider spelling byte-for-byte, including scheme/host case, explicit
default port, path spelling, query ordering, and percent-escape case. BioMCP
does not apply URL-library serialization or Unicode normalization. Each
assertion has
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

All decoded CSV fields first undergo the common byte/control validation below;
normalization then uses these exact rules. Unicode trimming removes the maximal
leading and trailing sequence for which Rust `char::is_whitespace` is true.
The remaining Unicode scalar sequence and its UTF-8 bytes are preserved: there
is no NFC/NFKC normalization, Unicode case folding, whitespace collapse, or
interior trimming. Namespace syntax and decimal parsing are ASCII-only. The
only case-insensitive operations are the explicitly named ASCII comparisons
for `HGNC:`, `PMID:`, gene symbols, and submitter sort keys. Numeric versions,
MyGene HGNC values, and PMIDs serialize to canonical base-10 as specified;
validated GenCC CSV CURIEs otherwise retain their exact required uppercase
namespace spelling. Optional URLs use the preserved trimmed spelling above,
while constructed GenCC/PubMed URLs use their fixed canonical templates.

For duplicate comparison, first parse one row into the canonical normalized
cache tuple in this exact field order: canonical `sgc_id`, `u32` version,
canonical gene CURIE, trimmed gene label, canonical disease CURIE, trimmed
disease label, the closed classification CURIE/label/code triple, canonical
inheritance CURIE plus trimmed label, canonical submitter CURIE plus trimmed
label, nullable canonical evaluated/submitted dates, nullable preserved
trimmed/validated public-report and criteria URL spellings, and the ordered
canonical PMID vector. Two rows
with one `(sgc_id, version)` are byte-equivalent only when every scalar field is
equal by its stated value and every retained string/vector element is equal by
its exact UTF-8 bytes in order; implementations need not invent a second wire
serialization merely to compare the tuple. Canonically equivalent but
byte-distinct Unicode labels differ because no Unicode normalization occurs.
Optional-null state, preserved URL spelling, PMID value, and PMID order also
matter. Thus two valid URL spellings that a URL library might serialize alike
remain different and invalidate a duplicate pair, while identical trimmed URL
spellings compare equal. Equivalent
duplicates become one row in first-row position; any tuple difference
invalidates the generation. Columns excluded from the normalized cache
(`disease_original_*`, notes, and the other unused `submitted_as_*` columns)
are decoded only for the global CSV/UTF-8/control/raw-field bounds: their
contents are neither normalized nor compared, so rows differing only in an
excluded column still deduplicate. When several versions of an SGC ID exist,
only its greatest positive version is current. This never combines different
SGC IDs, even when their other fields match. Current assertions are
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

Identity has two explicit phases. The pre-index phase validates the canonical
MyGene symbol and HGNC wire union. A missing/invalid symbol or malformed/
multiple HGNC values is conclusively inconclusive before every GenCC root
check, lock, lifecycle decision, HEAD/GET, and local index query. That case
always returns the exact `identity_match` row, consumes zero GenCC quota,
creates no root/lock, and leaves every durable timestamp and last-attempt byte
unchanged; it wins over a hypothetical root failure or suppression state.

A valid canonical symbol with zero or one HGNC value is only an identity
candidate: the symbol-to-CURIE consistency rules require a validated GenCC
index. Lifecycle acquisition/refresh therefore runs first, with its normal GET
and timestamp effects, and identity matching runs against the resulting fresh
index or a leased stale index. If no index is available because initial
download/root/lock/suppression failed, that lifecycle failure wins and the
operation is not mislabeled `identity_match`. If an index is queryable and the
symbol/HGNC check is inconclusive, `identity_match` wins in the public section
over stale-failed, retry-suppressed, or refresh-deferred presentation, while
the already completed lifecycle attempt/suppression timestamps remain exactly
as recorded. A failed base MyGene request wins before constructing
`Gene.gencc`, preserving the existing whole-card/section failure behavior; no
GenCC status, evidence source, or URL is manufactured.

Tests cover every scalar/array wire shape, missing/null/empty, numeric
normalization, exact and first-over bounds, equivalent-value deduplication,
mixed malformed arrays, multiple distinct IDs, case, alias input resolved to
the canonical symbol, matching symbol with wrong HGNC, matching HGNC with wrong
symbol, duplicate GenCC identities, and failed base identity.

## Dataset lifecycle and concurrency

The GenCC store selection is exact. If `BIOMCP_GENCC_DIR` is absent, the root
is `dirs::data_dir()/biomcp/gencc` (for example
`$XDG_DATA_HOME/biomcp/gencc` on Linux under the `dirs` crate contract). A set
override is Unicode-trimmed once and must be a nonblank absolute path with no
`.` or `..` lexical component; blank or relative overrides are configuration
errors and never fall back to the default. If the override is absent and
`dirs::data_dir()` returns `None`, ordinary `gencc` returns unavailable and
explicit sync exits nonzero. Every one of those selection failures makes zero
GenCC requests and creates no directory.

First-root creation has one deterministic bootstrap anchor whose pathname does
not depend on whether the selected root exists. For the default root, the
anchor directory is exactly the already-existing `dirs::data_dir()` and the
selected root is its `biomcp/gencc` descendant. For an override, the anchor
directory is exactly the selected root's lexical parent and only that final
root component may be absent. The anchor directory itself must already exist,
must be opened component-by-component without following symlinks/reparse
points, and must satisfy the existing private-path ownership rules. An absent
or unsafe default anchor directory, or an override whose lexical parent is
absent or unsafe, fails closed with zero GETs and creates nothing; BioMCP does
not recursively bootstrap an unanchored hierarchy.

Inside that invariant directory the anchor is the regular current-user-owned
`0600` file `.biomcp-gencc-root-<sha256-of-selected-absolute-path>.lock`.
Every operation derives the same pathname from the selected absolute path,
opens it with no-follow semantics (create-new when absent, otherwise open),
verifies one link plus pathname-to-open-file inode/file-ID equality, and takes
its exclusive advisory lock before validating or creating the selected root.
Under that lock it re-walks the selected root, creates only its permitted
missing components with `0700`/the existing Windows private ACL, creates and
verifies the distinct root files `.refresh.lock` and `.store.lock` as regular
current-user-owned `0600` files, and fsyncs each changed parent. The bootstrap
lock is released only after the needed root lock file descriptions have been
opened and `.store.lock` is held shared for the initial state snapshot. The
anchor is outside the selected root and GenCC cleanup never unlinks it, so a
missing root becoming present and a later deletion/recreation of the selected
root both rediscover the same anchor pathname and inode. Default and override
subprocess tests record the anchor device/inode (or Windows file ID) before
creation, after creation, and across root deletion/recreation and prove two
processes still serialize on that one file.

This guarantee covers BioMCP creation and deletion/recreation of the selected
root while its invariant anchor directory remains intact. Concurrent hostile
deletion/replacement of the anchor file or anchor directory by the owning OS
user cannot be made safe with advisory locks and is explicitly outside the
guarantee; pathname/file-ID revalidation fails closed when detected. Tests do
not claim protection from an owner that can unlink a currently locked file.
Failure to locate/open/lock/revalidate the anchor, ownership/link/type failure,
root or root-lock creation/fsync failure, or a changed component makes zero
GETs. Tests cover a missing selected default root beneath an existing default
anchor, a missing override root beneath an existing override parent,
already-created roots, two-process first creation, both root
delete/recreate cases, absent anchor directories, blank/relative overrides,
unsafe ancestors/root/lock files, `dirs::data_dir() == None`, permissions, and
exact zero-request logs. This durable source never falls back to a process
temporary directory. It is not an ordinary HTTP-response cache entry, and
metadata contains no absolute local paths.

Durable state has two deliberately different layers:

- `generations/<generation-id>/manifest.json` and its normalized bounded index
  are immutable after publication. The manifest records schema version,
  endpoint identity, body and index SHA-256, row/assertion counts, the
  successful `200` `retrieved_at`, the validated ETag and Last-Modified, and
  an `upstream_version` slot fixed to null by this ticket. A generation ID is derived from the index hash
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

- First use with no valid generation takes the process mutex and exclusive
  `.refresh.lock`, rechecks disk under a brief shared `.store.lock`, releases
  the store lock, performs one unconditional `GET`, validates completely, then
  publishes under exclusive `.store.lock` and queries. Failure is unavailable;
  partial data is not retained.
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
  stored values. No response header is interpreted as a dataset version, on
  either `200` or `304`; `upstream_version` therefore remains null and an
  unrelated/header-extension appearance or change cannot mutate it. A valid
  `304` atomically replaces `state.json` with advanced
  `checked_at`/`attempted_at`, retaining `retrieved_at`, validators, version,
  and index. A valid `200` publishes a new generation. Any other response, or
  any storage failure before the state-rename linearization point, including a
  one-byte body, invalid/mismatched validator, invalid header/body/schema,
  timeout, or pre-rename publication failure, keeps the old generation and
  makes this lookup stale after durably recording the failed attempt when that
  failure-state update itself commits. Post-rename root-fsync failure follows
  the separate namespace-authority matrix below.
- `biomcp gencc sync` always waits for the process mutex and exclusive
  `.refresh.lock`, uses `.store.lock` only for its recheck and publication
  phases, and performs the same conditional revalidation even when fresh.
  `304` and valid `200` succeed. Refresh failure
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

Refresh leadership uses two locks with distinct purposes: one process-local
async mutex plus the cross-process exclusive advisory `.refresh.lock` elect a
single refresh leader, while `.store.lock` protects only short state,
publication, lease-acquisition, recovery, and cleanup critical sections. The
blocking acquisition order for leadership/publication is bootstrap anchor
(initialization only, released before leadership), process-local refresh mutex,
`.refresh.lock`, then `.store.lock`. Lease acquisition is separately
`.store.lock` shared then generation `lease.lock` shared. After releasing the
store lock, an ordinary due caller may deliberately retain that already-held
fallback lease while later acquiring the mutex, refresh lock, and store lock;
this cannot form a wait cycle because no path waits for a generation-exclusive
lock while holding any of those locks. Cleanup holds `.store.lock` exclusive
and only tries generation locks nonblocking, skipping contention. No path
acquires the bootstrap anchor while holding a root lock, and no path performs a
blocking generation-exclusive acquisition. Readers that do not seek refresh
leadership take only `.store.lock` shared and then the generation lease. The
leader holds the mutex and
`.refresh.lock` continuously from election through its final durable state
decision, but never holds `.store.lock` during HTTP, streaming, parsing, or
index construction.

An ordinary caller first uses a shared `.store.lock` phase to read state and,
when present, acquire the active generation snapshot lease by the protocol
below. It releases `.store.lock` while retaining that lease as its exact
timeout/failure fallback. If refresh is due or no generation exists, it then
waits for the process mutex and `.refresh.lock` within its one-gene deadline.
After both leadership locks are held it briefly reacquires `.store.lock`
shared, reloads state and any generation, and decides again whether work is
due. A completed predecessor therefore produces `local_query` after success or
`retry_suppressed` after a durably recorded failure, with no second GET. A true
leader releases `.store.lock`, performs the one HTTP attempt and complete
validation while retaining both leadership locks, then takes `.store.lock`
exclusive to publish the generation/state or the failed-attempt state. It
releases `.store.lock`, then `.refresh.lock`, then the process mutex.

This separation deliberately lets a follower that starts after the leader has
entered HTTP take `.store.lock` shared and lease the still-active old
generation. If its deadline expires before leadership becomes available, it
queries that lease as stale with `refresh_deferred`. A follower that observes
no generation likewise waits without issuing a GET; on deadline it returns the
exact unavailable `refresh_deferred` row defined below rather than claiming
`initial_download`. Lock wait/timeout is not an upstream attempt and never
changes `attempted_at`. These rules apply identically to concurrent first use:
one leader performs the unconditional GET, successful followers query its
generation, and failed followers observe its persisted suppression state.
Readers open only finalized immutable generation files and therefore never see
temporary or partially published state. Opening a finalized file alone is not
the cleanup-safety protocol; every query holds the generation lease below.

Deterministic thread and two-process barriers pause a leader only after it has
released `.store.lock` and entered the fixture HTTP handler. A late follower is
then started. With an old generation it must acquire and retain that exact
lease, make zero GETs, preserve its state timestamps/validators, and return the
stale `refresh_deferred` projection on timeout. Without a generation it must
also make zero GETs and return the unavailable `refresh_deferred` projection;
the shared request log contains exactly the leader's one GET. Companion tests
release the leader before the follower deadline and prove reload-after-lock
returns `local_query` or `retry_suppressed` without a follower GET.

Each finalized generation contains a private regular `0600` `lease.lock`.
To start a query, a process takes `.store.lock` shared, reads and validates
`state.json`, obtains a process-local reference-counted `GenerationLease` for
that generation, takes a shared advisory lock on its `lease.lock`, opens the
manifest/index handles, rechecks that state still names the same generation,
and then releases `.store.lock`. A changed state restarts this sequence. The
generation shared lock, open handles, and in-process reference remain held
through the complete index query/render projection. Same-process leases share
one locked file description and release it only when the last reference drops,
avoiding per-process advisory-lock semantics accidentally unlocking another
reader.

Publication/state replacement/maintenance take `.store.lock` exclusively.
Cleanup may delete a non-retained generation only after a nonblocking exclusive
lock on that generation's `lease.lock` succeeds and the process-local reference
count is zero; contention means skip, not wait or unlink. It revalidates the
directory under that lock immediately before deletion. Thus POSIX unlink and
Windows deletion semantics cannot race an active snapshot. An adversarial
three-generation test holds a G1 reader lease while G2 is active, publishes G3,
proves cleanup retains leased G1 even though the normal retained pair is G3/G2,
finishes the G1 query unchanged, releases the lease, and proves the next locked
maintenance removes G1 while preserving G3/G2. A subprocess variant proves the
same cross-process behavior.

Publication creates a unique private sibling temporary generation, writes the
index and manifest with create-new semantics, fsyncs each file and then that
directory, renames it to its final generation name, fsyncs the generations
directory, and reopens/revalidates both hashes and the manifest. It next writes
a unique private `state.json` temporary, fsyncs it, atomically renames it over
`state.json`, and fsyncs the root. The successful state rename—not the later
root fsync—is the namespace linearization point: before it, the old state is
authoritative and remains name-visible; after it, the new state is
name-visible and authoritative to every non-crashed current/subsequent reader,
and BioMCP never claims that the inaccessible old pathname is still active.
The following root fsync establishes crash durability.

Failure before the state rename leaves the old state authoritative and the
current ordinary call uses its already-held old-generation lease (stale on a
due refresh), while explicit sync fails nonzero. Failure of the root fsync
after a successful rename cannot be rolled back. The ordinary leader discards
its pre-refresh fallback for presentation, reloads and leases the generation
named by the namespace-visible new state, and reports that new state's normal
success row. In the `304` case it likewise reports the advanced visible state.
It must not render old validators, timestamps, assertions, or status after the
rename. Explicit sync still exits nonzero because that command promises a
durable update, and the root fsync did not establish it. A subsequent
non-crashed call reloads the namespace-visible new state and uses it. After a
crash at that point, startup may observe the complete old or new state; it
validates whichever is visible and never combines them. A later retry of root
fsync may make the rename durable, but it does not alter the already truthful
ordinary response or turn the failed explicit-sync invocation into success.

A `304` and a failed-attempt update use the same temporary-file fsync,
rename-linearization, and root-fsync rules for state only. If a failure-state
rename did not occur, no suppression is claimed. If its rename occurred but
the root fsync failed, the current call reports the underlying failure without
claiming durable suppression; subsequent non-crashed calls that reload the
valid namespace-visible failure state use `retry_suppressed`, while a crash is
allowed to reveal the prior state and retry. Timestamps always come wholly
from the single visible state record. Every exit removes only its owned
raw/state/unpublished-generation temporaries after worker settlement; a
finalized generation or renamed state is never treated as a temporary.

The state-update failure matrix is normative:

| last completed step | namespace authority now | current ordinary call | next non-crashed call | restart |
|---|---|---|---|---|
| before generation rename | old state/generation, or none | stale old result / unavailable | reloads old/none; retry follows persisted attempt state | old/none |
| generation renamed+directory fsynced; before state rename | old state; new generation is inactive | stale old result / unavailable | reloads old; orphan is cleanup/recovery input, never active by implication | valid old state wins; scan only if state requires recovery |
| state temporary fsynced; before state rename | old state | stale old result / unavailable | reloads old | old |
| state rename succeeded; root fsync failed | new namespace-visible state | reloads and reports the new visible state; explicit sync nonzero | reloads and uses new state | complete old or new state, never mixed |
| state rename and root fsync succeeded | new durable state | new result | new result | new |
| failure-attempt state update failed before rename | prior state | underlying stale/unavailable failure, no suppression claim | prior state decides eligibility | prior state |
| failure-attempt state renamed; root fsync failed | new namespace-visible failure state | underlying stale/unavailable failure, no durable-suppression claim | suppresses from new `attempted_at` | old or new failure state; whichever validates decides |

“Stale old result” includes its prior assertions only when positive and uses
the exact stale-failed mapping; zero remains unavailable. None of these storage
failures issues a second provider GET inside the same call.

For root-fsync failure after state rename, this public-status matrix is also
normative. `D` and `E` mean conclusive positive and zero matches respectively;
the source/message columns are those of the referenced complete truth-table
row. “Next” and “restart” mean an immediate call at the same injected wall-clock
instant. A restart genuinely may expose either directory entry because the
root fsync failed, so the last column names both deterministic branches instead
of pretending one is durable. A branch retaining an older state performs the
ordinary due decision and may make one GET. When a first `200` had no older
state, however, its generation rename and generations-directory fsync already
made the finalized generation durable: loss of the later state rename triggers
missing-state recovery, which selects that generation and makes zero additional
provider requests.

| renamed state update | state before update | current ordinary call | next non-crashed call | first call after restart |
|---|---|---|---|---|
| replacement `200`, D | old generation | `fresh/data/conditional_refresh`, `data`, `["GenCC"]`, null | `fresh/data/local_query`, `data`, `["GenCC"]`, null | new visible: `fresh/data/local_query`; old visible: old generation is due and follows a due conditional-request row |
| replacement `200`, E | old generation | `fresh/empty/conditional_refresh`, `empty`, `["GenCC"]`, null | `fresh/empty/local_query`, `empty`, `["GenCC"]`, null | new visible: `fresh/empty/local_query`; old visible: old generation is due and follows a due conditional-request row |
| first `200`, D | none | `fresh/data/initial_download`, `data`, `["GenCC"]`, null | `fresh/data/local_query`, `data`, `["GenCC"]`, null | new visible or state rename lost: validate/recover the already durable generation, then `fresh/data/local_query`, `data`, `["GenCC"]`, null; zero GETs |
| first `200`, E | none | `fresh/empty/initial_download`, `empty`, `["GenCC"]`, null | `fresh/empty/local_query`, `empty`, `["GenCC"]`, null | new visible or state rename lost: validate/recover the already durable generation, then `fresh/empty/local_query`, `empty`, `["GenCC"]`, null; zero GETs |
| valid `304`, D | due generation | `fresh/data/conditional_refresh`, `data`, `["GenCC"]`, null | `fresh/data/local_query`, `data`, `["GenCC"]`, null | new visible: `fresh/data/local_query`; old visible: same generation is due and follows a due conditional-request row |
| valid `304`, E | due generation | `fresh/empty/conditional_refresh`, `empty`, `["GenCC"]`, null | `fresh/empty/local_query`, `empty`, `["GenCC"]`, null | new visible: `fresh/empty/local_query`; old visible: same generation is due and follows a due conditional-request row |
| failed-attempt update | old generation, D | `stale/data/conditional_refresh`, `degraded`, `["GenCC"]`, stale-failed | `stale/data/retry_suppressed`, `degraded`, `["GenCC"]`, stale-failed | new visible: the retry-suppressed row; old visible: eligible due refresh and its resulting due row |
| failed-attempt update | old generation, E | `stale/empty/conditional_refresh`, `unavailable`, `[]`, stale-failed | `stale/empty/retry_suppressed`, `unavailable`, `[]`, stale-failed | new visible: the retry-suppressed row; old visible: eligible due refresh and its resulting due row |
| failed-attempt update | none | `unavailable/unknown/initial_download`, `unavailable`, `[]`, unavailable | `unavailable/unknown/retry_suppressed`, `unavailable`, `[]`, unavailable | new visible: the retry-suppressed row; old/missing visible: eligible no-generation first use and its resulting row |

Every current and next-call row above reads `checked_at`, `attempted_at`,
`retrieved_at`, validators, and assertions wholly from the named visible state
and manifest. In particular, the current failed-attempt row reads the newly
visible `attempted_at` even though its operation truthfully says that this call
performed `conditional_refresh` or `initial_download`; only a later call says
`retry_suppressed`.

Startup validates `state.json`, then validates its referenced finalized
generation by manifest and hashes. If state is missing/corrupt or references an
invalid generation, recovery scans only finalized generation directories and
chooses the greatest valid immutable `retrieved_at`, breaking ties by bytewise
generation ID. It reconstructs `checked_at` and `attempted_at` from that
generation's immutable `retrieved_at` and writes a fresh state record; it never
recovers a later 304/failure timestamp from a partial state file. If no valid
generation exists, acquisition starts from none (while an independently valid
failure-only `state.json` still enforces retry suppression). Under the
exclusive cross-process store lock and generation-lease protocol,
post-publication/startup maintenance retains the active and newest other valid
generation, removes abandoned temporaries and all other unleased invalid/old
generations, and fsyncs every parent directory whose entries changed. Cleanup
failure or a leased candidate is logged and leaves extra private files; it
never invalidates the already durable active state or deletes either retained
copy or an actively leased older copy.
Crash/failure injection before and after every fsync and rename separately
asserts the current call, the next non-crashed call, and restart outcome. It
proves that state is old before rename, new after rename, restart selects only
a complete old or new state/generation, and no output mixes timestamps,
validators, or indexes across versions.

One direct gene request gives its GenCC section one existing configurable
eight-second optional-enrichment deadline, starting when that section future is
first polled and covering refresh-mutex/advisory-lock wait, short store-lock
phases, HTTP, streaming,
validation, publication, identity resolution, and index query. Batch retains
the existing maximum of ten input IDs and `join_all` behavior: all at-most-ten
gene futures are polled concurrently, each has its own eight-second GenCC
deadline, and there is no additional shared batch deadline. They share the
same process refresh mutex and `.refresh.lock` and therefore at most one GenCC
GET; local immutable queries may proceed concurrently after reload. Explicit
sync instead has one 120-second end-to-end deadline and waits for the process
mutex and `.refresh.lock`, then for each required short `.store.lock` phase;
timeout exits nonzero
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
- After RFC 4180 unquoting and UTF-8 decoding, every field is at most exactly
  16,384 UTF-8 bytes before trimming or any other normalization. CSV delimiter,
  quote, and record-terminator bytes are not part of the decoded field; an
  embedded CR/LF in a quoted field is part of it and is rejected by the control
  rule. The parser checks `field.as_bytes().len()`, not Unicode scalar count or
  storage capacity. Every decoded field, including excluded/unused columns and
  optional URLs that would otherwise become null, is rejected if it contains a
  Unicode General Category `Cc` control scalar: exactly U+0000-U+001F and
  U+007F-U+009F. Other categories such as `Cf` are not controls under this
  predicate. The optional initial U+FEFF BOM is accepted only before the first
  header byte and is never field content.
- After the common raw-field check and Unicode trimming, normalized labels are
  <=1,024 Unicode scalar values and exposed links are <=2,048 UTF-8 bytes. An
  assertion has <=128 unique canonical PMIDs. Boundary tests accept 16,384
  ASCII bytes and 8,192 two-byte scalars, reject 16,385 bytes including a
  multibyte scalar that crosses the boundary, accept/reject 1,024/1,025 label
  scalars and 2,048/2,049 URL bytes, and exercise leading/trailing NBSP and EM
  SPACE, preserved interior whitespace, composed versus decomposed labels,
  allowed `Cf`, and rejected C0/C1 controls in required, optional, quoted, and
  excluded fields. URL tests prove preserved spelling and exact-string evidence
  deduplication; duplicate-row tests prove same parsed URL spelling deduplicates
  while distinct spellings invalidate the generation. The first
  byte/row/field/publication above a fail-closed input bound is rejected; an
  invalid optional link follows the explicit null rule only after the common
  raw bound/control checks pass. At most 100,000 normalized assertions can
  enter a generation by the row bound, and the 100-assertion response cap is
  separate.
- All errors are typed internally and mapped to stable status. No body excerpt,
  note, OMIM-original field, local path, or credential appears in outputs,
  stderr, or debug logs.

## Freshness, outcome, and provenance truth table

`GenCC` is the exact source label. In the table, “query” means the immutable
local index may be queried after lifecycle resolution; it never means a GenCC
network query. `stale-failed`, `stale-progress`, `unavailable`, `identity`, and
`no-generation-progress` mean respectively the five exact failure/warning
messages defined above. Same-process mutex followers and
cross-process advisory-lock followers use the same rows. The lock owner alone
updates timestamps. `section_outcomes.gencc` and its matching
`_meta.section_sources` entry use this complete mapping:

| generation at entry | lifecycle decision for this call | local query | identity / matches | status (`freshness/result/operation`) | outcome | sources | message |
|---|---|---|---|---|---|---|---|
| any/none | pre-index MyGene identity is inconclusive; lifecycle is not consulted | no | inconclusive / n/a | `unavailable/unknown/identity_match` | `unavailable` | `[]` | identity |
| none | candidate identity; valid first `200`, then index identity inconclusive | yes | inconclusive / n/a | `unavailable/unknown/identity_match` | `unavailable` | `[]` | identity |
| fresh | candidate identity; no refresh due, index identity inconclusive | yes | inconclusive / n/a | `unavailable/unknown/identity_match` | `unavailable` | `[]` | identity |
| due | candidate identity; valid `200`/`304`, then index identity inconclusive | yes | inconclusive / n/a | `unavailable/unknown/identity_match` | `unavailable` | `[]` | identity |
| due | candidate identity; refresh fails, stale index identity inconclusive | yes | inconclusive / n/a | `unavailable/unknown/identity_match` | `unavailable` | `[]` | identity |
| due | candidate identity; retry suppressed, stale index identity inconclusive | yes | inconclusive / n/a | `unavailable/unknown/identity_match` | `unavailable` | `[]` | identity |
| due | candidate identity; lock deadline, leased stale index identity inconclusive | yes | inconclusive / n/a | `unavailable/unknown/identity_match` | `unavailable` | `[]` | identity |
| none | root unavailable/unsafe before GET | no | n/a | `unavailable/unknown/initial_download` | `unavailable` | `[]` | unavailable |
| none | leader valid first `200` | yes | conclusive / >0 | `fresh/data/initial_download` | `data` | `["GenCC"]` | null |
| none | leader valid first `200` | yes | conclusive / 0 | `fresh/empty/initial_download` | `empty` | `["GenCC"]` | null |
| none | leader GET/validation/publication fails or times out | no | n/a | `unavailable/unknown/initial_download` | `unavailable` | `[]` | unavailable |
| none | durable failed attempt still suppresses retry | no | n/a | `unavailable/unknown/retry_suppressed` | `unavailable` | `[]` | unavailable |
| none | first-use follower reloads leader's valid `200` | yes | conclusive / >0 | `fresh/data/local_query` | `data` | `["GenCC"]` | null |
| none | first-use follower reloads leader's valid `200` | yes | conclusive / 0 | `fresh/empty/local_query` | `empty` | `["GenCC"]` | null |
| none | first-use follower reloads leader's persisted failure | no | n/a | `unavailable/unknown/retry_suppressed` | `unavailable` | `[]` | unavailable |
| none | mutex/advisory refresh-lock wait reaches per-gene deadline; this follower sends no GET | no | n/a | `unavailable/unknown/refresh_deferred` | `unavailable` | `[]` | no-generation-progress |
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

The stale-zero rows are intentional: old positive assertions may be shown with
a warning, but zero matches in old data cannot establish current absence. The
pre-index identity row makes zero GenCC HEAD/GET requests, creates no root, and
leaves state byte-for-byte unchanged. Index-identity rows inherit the lifecycle
effects that occurred before the index could be checked: first/successful due
refresh consumes exactly one GET and advances the success timestamps; a failed
due refresh consumes one GET and advances only its failed `attempted_at`;
fresh, suppressed, and lock-deadline reads consume zero GETs and change no
timestamp. Root/no-generation acquisition failure remains the corresponding
`initial_download`/`retry_suppressed` lifecycle row because no index existed to
prove an identity conflict.

The no-generation `refresh_deferred` follower projection is exact:
`assertions: []`, `total_matching_assertions: 0`, `truncated: false`, section
outcome `{outcome: "unavailable", sources: [], message:
"GenCC refresh is still in progress; no GenCC absence can be concluded."}`,
the matching `_meta.section_sources` entry with `key: "gencc"`, outcome
`unavailable`, and `sources: []`, and no GenCC evidence URLs. `checked_at`,
`retrieved_at`, ETag, and Last-Modified are null. If there
was no earlier completed attempt, `attempted_at` is also null; if a valid
failure-only state predated the in-flight leader, its completed `attempted_at`
is preserved byte-for-byte. The leader's unfinished attempt changes no durable
timestamp. The follower performs exactly zero HEAD/GET requests and no state
write; a concurrent fixture log therefore contains only the leader's one GET.

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

## Implementation ownership and package neutrality

The implementation adds exactly these nine repository paths, of which eight
are package-visible:
`src/sources/gencc.rs` (transport/facade),
`src/sources/gencc/model.rs` (bounded CSV parsing and normalized model),
`src/sources/gencc/store.rs` (root, state, generation, lease, and recovery),
`src/sources/gencc/tests.rs` (source/store adversarial tests),
`src/entities/gene/gencc.rs` (resolved identity/index query and section status),
`src/entities/gene/gencc/tests.rs` (identity/lifecycle truth-table tests),
`docs/sources/gencc.md`, one minimized
`testdata/sources/gencc/submissions-new-odc1.csv`, and one focused
`tests/test_gencc_docs_contract.py`. `Cargo.toml` deliberately excludes the
entire `testdata/` tree, so the minimized CSV is a repository test fixture but
not a Cargo package path; it must remain under the existing source-fixture
owner and must not be moved or copied merely to affect the package count.
Existing Gene dispatch/rendering, CLI, MCP, schemas, fixture setup,
receipts/inventory, and executable specs are modified in place rather than
gaining more files. Each new Rust production or test module remains at or
below 1,000 lines.

Before adding those paths, perform these behavior-neutral consolidations and
delete exactly eight package files: inline
`src/sources/mygene/tests/{mod.rs,construction.rs,parsing.rs}` into a single
`#[cfg(test)] mod tests` in `src/sources/mygene.rs`, but retain
`src/sources/mygene/tests/live.rs` as that module's `mod live;` child because
the ignored real-provider checks have a distinct Tier-4 owner; inline
`src/sources/clingen_cspec/tests/{mod.rs,construction.rs}` into
`src/sources/clingen_cspec.rs`; and inline
`src/sources/clingen_erepo/tests/{mod.rs,construction.rs,parsing.rs}` into
`src/sources/clingen_erepo.rs`. Preserve every test name, ignored/live marker,
fixture path, and behavior, and run the three consolidated test filters before
GenCC work. These owners remain below 1,000 lines after consolidation. No
source-size baseline is raised. `cargo package --list --allow-dirty --locked
--offline` must be exactly 1,292 immediately after the eight deletions and
exactly 1,300 after the eight package-visible additions. The excluded GenCC
CSV does not change either Cargo package count. Any different implementation
file plan returns the ticket to design review rather than deleting an
opportunistic unrelated file, broadening `Cargo.toml` inclusion, or adding a
filler path.

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
  and 128/129 unique-PMID boundaries, exact date-only/19-byte-space/RFC-3339
  grammars and UTC-date projection, Unicode trim-without-normalization rules,
  and identity rules. ODC1 returns all three separate
  Strong/autosomal-dominant assertions and submitters.
- Adversarial tests cover every first-over-bound value; header/schema/legacy,
  quote/UTF-8, content type/archive/compression, validator/status, CURIE/date/
  PMID/classification, duplicate, exact URL spelling, raw UTF-8 byte versus
  scalar bounds, Unicode whitespace/normalization, the exact Cc control
  predicate in included and excluded fields, cache path/link, redirect, leak,
  and timeout failures. Bad optional report/criteria URLs become null only after
  their common raw-field/control validation passes; structural or required-field
  failures invalidate the generation.
- An injected-clock HTTP fixture proves initial download, fresh reuse, exact
  seven-day edges, both conditional headers, zero-body `304`, Content-Length
  zero/one, transfer/content encoding, absent/equal/mismatched validators and
  always-null/ignored-undocumented upstream-version headers, replacement `200`,
  durable daily retry suppression with and without old data, rollback, sync,
  `--no-cache`, and health HEAD. Its log
  rejects unplanned routes and proves normal/suppressed/follower reuse consumes
  no download.
- MyGene fixtures prove the appended `HGNC` projection and every accepted and
  rejected scalar/string/flat-array shape, normalization bound, deduplication,
  and conflict before exercising the combined identity index. Pre-index and
  index-dependent inconclusive identity are then tested against no/fresh/due/
  failed/retry-suppressed/lock-deferred states and root failures, proving the
  exact precedence, zero-request pre-index behavior, lifecycle-dependent GET
  count, and exact timestamp/state effects in the truth table.
- Thread/task and subprocess tests separately prove one GET for concurrent
  first use and refresh, same-process successful/failed followers,
  cross-process successful/failed followers, old-generation contention,
  ordinary lock timeout with and without a generation, explicit-sync lock
  timeout with and without a generation, exact operation/message/timestamp
  preservation, per-gene eight-second batch deadlines for ten concurrent items,
  no shared batch deadline, immutable concurrent reads, private permissions,
  and no live work or temp files after success/timeout/cancellation. Barrier
  cases start followers only after the elected leader has entered HTTP and
  released `.store.lock`; with an old generation they prove a late lease and
  stale `refresh_deferred`, while without a generation they prove exact
  unavailable/unknown/`refresh_deferred`, unchanged/null state fields, zero
  follower GETs, and one total leader GET. Default and override subprocess
  cases prove the invariant out-of-root bootstrap anchor path and inode/file ID
  across missing-to-existing and selected-root delete/recreate, and explicitly
  bound the guarantee to an intact anchor directory. Crash
  injection before and after each file/directory fsync, generation rename,
  state rename, and cleanup recovers only a complete old or new generation and
  proves the exact current/next/restart matrix—including root-fsync failure
  after replacement `200` with old data or no generation, `304`, and
  failure-state rename—failure-attempt persistence, deterministic fallback
  ordering, and the shared/refcounted generation lease. The no-prior-generation
  restart cases specifically prove recovery selects the finalized, fsynced
  first-`200` generation with zero additional GETs.
  The three-generation in-process and subprocess cases keep a G1 snapshot
  readable across G2/G3 publication and defer its deletion until lease release.
- CLI direct/all/batch Markdown/JSON, raw MCP text/JSON, typed MCP, help/list,
  schema, exact evidence-URL labels/order/global deduplication for report,
  criteria, duplicate PubMed and capped assertions, docs, and 1159 coexistence
  are fixture-tested without live provider access.
- New Rust modules remain <=1,000 lines. Do not raise any existing exact
  source-size baseline or CLI 700-line cap. The named eight-package-file
  deletion/eight-package-visible-addition plan is followed exactly, the ninth
  repository addition remains the excluded GenCC capture fixture, and `cargo
  package --list --allow-dirty` remains exactly 1,300 files. The package-list
  assertion names all eight new included paths, excludes
  `testdata/sources/gencc/submissions-new-odc1.csv`, and retains
  `src/sources/mygene/tests/live.rs`; the affected MyGene, ClinGen CSpec, and
  ClinGen ERepo filters prove consolidation did not lose or rename a test.
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
  per-gene eight-second/max-ten-concurrent batch behavior.
- Second design review: rejected because identity-failure precedence and quota
  effects, post-rename authority, cleanup-safe reader leases, upstream-version
  handling, first-root locking, package-neutral file ownership, and duplicate
  equivalence remained underspecified. This revision adds exact truth-table and
  crash outcomes, shared/refcounted per-generation leases with a
  three-generation test, always-null version semantics, deterministic
  default/override bootstrap locking and zero-request failures, a named
  package-neutral file plan, and canonical duplicate comparison.
- Third design review rejected one remaining contradiction: state rename was
  the declared namespace authority, but the current ordinary caller still
  rendered its old lease after a post-rename root-fsync failure. This revision
  consistently makes the namespace-visible state authoritative immediately
  after rename, keeps explicit sync's durability failure nonzero, and gives
  exact current/next/restart public-status rows for replacement `200`, `304`,
  and failed-attempt updates with and without old data.
- The next independent design review rejected the existence-dependent
  bootstrap anchor, conflated refresh leadership with store serialization,
  mislabeled a no-generation lock follower as `initial_download`, and left
  CSV Unicode/byte normalization underspecified. This revision fixes the
  bootstrap anchor outside the selected root and explicitly narrows its threat
  boundary, separates `.refresh.lock` from `.store.lock` with one acquisition
  order and late-lease tests, defines the exact no-generation
  `refresh_deferred` projection and zero-GET/timestamp behavior, and pins the
  timestamp, UTF-8-byte, control, Unicode-trim, URL-preservation, and duplicate
  rules.
- Final independent design re-review: accepted. The reviewer confirmed the
  invariant external bootstrap anchor, distinct refresh/store/generation-lock
  roles and ordering, zero-request no-generation follower outcome, exact CSV
  parsing boundaries, additive coexistence with ticket 1159, package-neutral
  file plan, source-size constraints, and bounded optional-enrichment behavior.
- Implementation observation returned the ticket to design: the previously
  named nine deletions plus nine repository additions produced 1,299 Cargo
  package paths because `Cargo.toml` excludes `testdata/`. This revision keeps
  the minimized capture in its real fixture owner, retains the independently
  owned MyGene Tier-4 `live.rs`, and freezes eight package deletions against
  eight package-visible additions for the existing 1,300-file ceiling without
  an unrelated deletion, filler file, or package-exclusion change.
- Code review: pending.
