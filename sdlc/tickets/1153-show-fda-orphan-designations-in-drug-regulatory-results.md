---
flow: build
priority: 6
deps: [1151]
---

# Show FDA orphan designations in drug regulatory results

## Goal and source boundary

`get drug eflornithine regulatory` must include FDA's March 11, 2024
designation of eflornithine hydrochloride for Bachmann-Bupp syndrome without
calling that use approved. This is a new, U.S.-only regulatory overlay from
FDA's **Orphan Drug Designations and Approvals** service. It is not an
OpenFDA/Drugs@FDA response and has its own client, source identity, outcome,
fixtures, and health/inventory entry.

The production base is
`https://www.accessdata.fda.gov/scripts/opdlisting/oopd/`; test-only
`BIOMCP_FDA_ORPHAN_BASE` replaces that prefix. POST form-urlencoded to
`OOPD_Results.cfm` with exactly, in this order:

```
Product_name=<candidate>
sponsor_name=
Designation=
Designation_Start_Date=
Designation_End_Date=
Search_param=DESDATE
Output_Format=Excel
Sort_order=GENERIC_NAME
RecordsPerPage=25
newSearch=Run Search
```

The advertised Excel response is an HTML table. Header normalization decodes
HTML entities, trims, and collapses ASCII whitespace. Accept HTTP 200 and one
table with exactly these normalized headers in order: `Generic Name`, `Trade Name`,
`Date Designated`, `Orphan Designation`, `Orphan Designation Status`, `Date
Designation Withdrawn or Revoked`, `FDA Orphan Approval Status`, `Approved
Labeled Indication`, `Marketing Approval Date`, `Exclusivity End Date`,
`Exclusivity Protected Indication * (Shown on labeling)`, `Sponsor Company`,
`Sponsor Address 1`, `Sponsor Address 2`, `Sponsor City`, `Sponsor State`,
`Sponsor Zip`, `Sponsor Country`, `CF Grid Key`. Reject a redirect/non-200,
login/error page, missing/duplicate/reordered header, nested row/table
ambiguity, invalid UTF-8, over-limit body, or malformed admitted row as that
query's source failure. Never follow or emit provider HTML links.

## Bounded acquisition and identity

Attempt this source only when `regulatory` is selected (including `all`) and
the effective region is `us` or `all`. Do not call it for `approvals` alone or
for `eu`/`who`; then the new field is absent and no orphan source is claimed.
Existing Drugs@FDA, EMA, WHO, label, and approval acquisition is unchanged.

Build candidates from the already resolved drug. Caller spelling and resolved
`Drug.name` are always candidates. A selected MyChem hit may contribute UNII
display name, DrugBank name and synonyms, ChEMBL preferred name, NDC
nonproprietary names, and MyChem OpenFDA generic/brand names only when one of
its typed DrugBank, ChEMBL, or UNII identifiers exactly matches the
corresponding nonblank selected-drug anchor, and every other nonblank typed
anchor present on both the hit and selected drug also agrees. Thus at least one
typed anchor must match and any comparable nonblank conflicting anchor rejects
the hit. An anchorless or conflicting hit contributes nothing; an unrelated fallback hit never contributes merely
because it has attractive aliases. The UNII value itself is not sent because
FDA's form has no UNII input. Collect categories in the stated category order,
hits in selected-hit order, and provider arrays in provider order. Then trim,
collapse ASCII whitespace, deduplicate ASCII-case-insensitively, retain the
first six, and issue at most six logical POSTs with concurrency at most two.

Admit a row only when its normalized generic or trade name exactly equals one
of those candidates after ASCII-case folding and whitespace collapse. There is
no prefix, substring, token-removal, or salt-stripping fallback: the existing
salt transform is owned by WHO product matching, while this FDA path already
receives salt-qualified names from anchored MyChem aliases. Require a nonblank
generic name and designation, valid `MM/DD/YYYY` designation date, and
ASCII-decimal CF Grid Key.

Collect every completed query result before merging. For a repeated CF Grid
Key, byte-identical normalized records collapse. If any public field differs,
discard that key and add one merge failure regardless of which POST completed
first; retain other nonconflicting records and reduce the envelope normally
(`degraded` when at least one query parsed, `unavailable` otherwise). Only then
sort by designation date descending and numeric key ascending. Parse at most
500 wire rows per response and return at most 100 unique matches. Cap each body
at 2 MiB. One eight-second deadline starts before cache initialization and
covers cache lookup/deletion, queueing, provider retry, body reading, parsing,
cache publication, and permission finalization. Work may be cancelled before
cache publication. Once publication is armed, await its finalization and then
report timeout or failure if the deadline expired. Cancellation stops
outstanding requests, retries, cleanup, and writes; no detached work continues.
Total deadline expiry reduces the complete orphan lookup to `unavailable`, even
when an earlier alias succeeded.

Raw form POST responses always use HTTP `NoStore`; provider HTML is never
cached. Add `ManagedResultCache` under the existing `src/cache/mod.rs`, backed
by the managed private cache so normal permissions, locking, capacity, clean,
and clear behavior apply. Construct one cache instance for the whole orphan
lookup, not once per alias. Store only validated normalized successes,
including confirmed empty results, in a private envelope that denies unknown
fields and contains exactly schema version `1`, a stored Unix timestamp, and
the normalized success payload.

The key is `fda-orphan:v1:sha256:<lowercase hex>`, whose digest uses this exact
preimage and concatenation order:

```text
b"biomcp:fda-orphan-result:v1\0"
|| u64_be(base.len) || base.trim_end_matches('/').as_bytes()
|| u64_be(form.len) || exact_ordered_form_body
```

`base.len` is the UTF-8 byte length after removing every trailing ASCII `/`;
`form.len` is the byte length of the exact form body. Both are unsigned 64-bit
big-endian integers. The private envelope uses `deny_unknown_fields` and has
exactly these field names and types:

```json
{
  "schema": 1,
  "stored_at_unix_seconds": 0,
  "success": {}
}
```

`schema` is `u8` and must equal 1, `stored_at_unix_seconds` is `u64`, and
`success` is the normalized source-owned success payload.

Request-local no-cache wins; then global cache `off` maps to `NoStore`,
`infinite` to `Force`, and every other configuration to `Default`. `NoStore`
does not initialize, read, delete, or write this result cache. `Default` uses a
valid entry only when `0 <= age < 86400`; equality is stale, so a stale entry is
deleted before provider fallback. `Force` uses every valid non-future entry
regardless of age; a miss calls the provider and stores a successful result.
In both `Default` and `Force`, a future timestamp, corrupt payload, or wrong
schema is deleted and fails that alias without provider fallback. One shared
initialization failure makes every planned alias a failure.

Cache initialization, get, required delete, decode, put, and permission-
finalization failures count as alias failures. A provider success is not
exposed until cache publication and secure finalization finish. Never cache
transport, HTTP, size, HTML-shape, or row-validation failures.

## Exact public schema and truth semantics

Add optional `Drug.fda_orphan_designations`. It is omitted when not attempted;
when attempted it has exactly this shape (shown with a data row):

```
{
  "outcome": "data|empty|degraded|unavailable",
  "sources": ["FDA Orphan Drug Designations and Approvals"],
  "records": [{
    "record_id": "992323",
    "generic_name": "Eflornithine hydrochloride",
    "trade_name": null,
    "designation_date": "2024-03-11",
    "designation": "...",
    "designation_status": "Designated",
    "designation_withdrawn_or_revoked_date": null,
    "orphan_approval": "not_approved|approved|unknown",
    "orphan_approval_status_text": "Not FDA Approved for Orphan Indication",
    "approved_labeled_indication": null,
    "marketing_approval_date": null,
    "exclusivity_end_date": null,
    "exclusivity_protected_indication": null,
    "sponsor": "...",
    "source_url": "https://www.accessdata.fda.gov/scripts/opdlisting/oopd/detailedIndex.cfm?cfgridkey=992323"
  }],
  "total_matching": 1,
  "truncated": false
}
```

Blank cells and `&nbsp;` become null; optional dates are ISO dates or null.
Required strings never serialize blank. Construct `source_url` only from the
validated decimal key and production FDA origin, even with a fixture override.

| FDA facts | `orphan_approval` |
| --- | --- |
| marketing date present, or splitting designation status on `/`, trimming, and ASCII-case-folding yields a segment exactly equal to `Approved` | `approved` |
| no positive fact and approval text equals `Not FDA Approved for Orphan Indication` after trim/case folding | `not_approved` |
| neither fact | `unknown` |
| positive and negative facts coexist, or an approval/exclusivity date is malformed | query failure |

Designation, withdrawal/revocation, marketing approval, and exclusivity dates
remain separate. Never infer approval from designation, current exclusivity
from its end date, or `not_approved` from missing data.

Every record field is required in JSON. `record_id`, `generic_name`,
`designation_date`, `designation`, `orphan_approval`, and `source_url` are
nonblank strings (`orphan_approval` is the stated enum); `trade_name`,
`designation_status`, `designation_withdrawn_or_revoked_date`,
`orphan_approval_status_text`, `approved_labeled_indication`,
`marketing_approval_date`, `exclusivity_end_date`,
`exclusivity_protected_indication`, and `sponsor` are `string|null`. Dates are
ISO strings when nonnull. No record field is omitted.

The envelope follows existing `SectionOutcome` serialization. `outcome` is the
four-value string enum; `sources` and `records` are always arrays; `message` is
omitted for data/empty and is the exact string below for degraded/unavailable;
`total_matching` is `integer|null`; and `truncated` is always boolean.
`total_matching` counts nonconflicting admitted records across successfully
parsed aliases before the 100-record cap, not hypothetical records from failed
aliases. Therefore data is `N>=1` with `truncated == (N>100)`; empty is `0` and
false; degraded (including degraded with no records) is the observed `N>=0`
and `truncated == (N>100)`; unavailable is null and false. Records contain the
first `min(N,100)` sorted rows for data/degraded and are empty for
empty/unavailable.

All successful queries with no rows reduce to `empty`; rows and no failures to
`data`; at least one success plus any query or merge failure to `degraded`; all
query failures or total timeout to `unavailable`. Data/empty have the one
source and no message; degraded has that source and `Some FDA
orphan-designation aliases were unavailable.`; unavailable has no sources and
`FDA orphan-designation data is temporarily unavailable.` Data, empty, and
degraded contribute this source to regulatory provenance; unavailable does not
falsely claim it.

## Health contract

Add `ProbeKind::FdaOrphan` rather than pretending this form endpoint is GET or
JSON. Its source-owned health method honors `BIOMCP_FDA_ORPHAN_BASE` and sends
one uncached POST to `OOPD_Results.cfm` with the same ordered form, except
`Product_name=eflornithine hydrochloride`; this read-only search is harmless
and known to exercise a result table. It follows no redirect,
accepts only HTTP 200 plus the exact header-only/result-table shape, reads at
most 2 MiB, and relies on the existing 12-second per-health-probe timeout. A
valid empty or nonempty table is healthy; transport, status, size, or shape
failure is the existing `HealthStatus::Error`/`ProbeClass::Error` with no API
key fields.

Register the canonical health name `FDA Orphan Drug Designations` immediately
after `OpenFDA`, with affects text `get drug regulatory --region us|all`.
Fixture-backed runner tests pin the override URL, exact method/path/form,
healthy and error rows, latency/count reconciliation, and `--fail-on-error`.
Catalog and CLI tests pin inventory order, exact case-insensitive `--api`
selection/deduplication, JSON and Markdown row labels, the repeatable `--api`
help contract, and the same canonical name/example in CLI reference/list help.
No live FDA health assertion enters routine tests.

## Production surfaces and acceptance

Add `### FDA orphan designations` to existing regulatory Markdown. Rows print
separate Designated, Designation status, Orphan approval, Marketing approval,
Exclusivity end, Sponsor, indication, and `FDA record` link fields; null prints
`-`. Empty prints `No matching FDA orphan designations found.` Degraded prints
its warning before rows; unavailable prints its message and the existing
bounded retry form for `get drug <identity> regulatory --region us`. All
provider text uses shared Markdown escaping. A hostile fixture with pipes,
brackets, parentheses, backticks, HTML, CR/LF, controls, and shell
metacharacters cannot create a column, link, heading, HTML node, terminal
escape, or command.

Pin the same fixture through production CLI Markdown/JSON, raw MCP `biomcp`
Markdown/JSON, and typed MCP `get` Markdown/JSON. Assert exact schema, nulls,
order, links, outcomes, and CLI/MCP equivalence, including eflornithine key
992323, an approved row, empty, degraded with and without records, unavailable,
malformed/oversize HTML, cap boundaries, exact generic/trade aliases,
salt-qualified candidate aliases, rejected prefix/substring matches, anchored
UNII-selected hits, and all regions. Exact JSON assertions pin every record
key/null and the state table for `total_matching`/`truncated`. Completion-order
permutations pin identical-key collapse and conflicting-key discard/degradation
byte-for-byte. Fixture logs prove exact forms, six/two request/concurrency caps,
one total deadline, cancellation, cache hit/expiry/no-cache, and zero
inapplicable requests. Raw and typed MCP delegate to production CLI; typed
request schema, section enum, tool count, and tool inventory remain
byte-for-byte unchanged.

All implementation and acceptance evidence is deterministic. The eflornithine
observation is historical evidence from commit `f8ff2a78`; tests use only
authored inline/generated HTML and loopback fixture responses. Do not contact
live FDA, edit `testdata/sources/capture-receipts.json`, add
`SPEC_LIVE_PATHS`, run `make verify`, or run `make release-live-smoke` for this
ticket. A fixture with an unrelated fallback hit containing attractive aliases
must prove that an anchorless or conflicting hit contributes no POST candidate.
Deterministic fault injection and paused time must prove initialization, get,
delete, decode, put, and secure-finalization failures; ages 86,399 and exactly
86,400; future timestamps, corrupt envelopes, and wrong schemas; request-local
no-cache precedence; global `off`, `infinite`, and default hit/miss behavior;
cancellation before publication; deadline expiry after publication where
finalization completes but the result remains unavailable and no task remains;
and total deadline expiry after an earlier alias success still yielding overall
`unavailable`.

## Ownership, ratchets, and boundaries

Own transport/parser/types and inline tests in one bounded
`src/sources/fda_orphan.rs`;
orchestration/identity under `src/entities/drug/`; Markdown in the existing
regulatory renderer; and provenance/schema, source inventory/licensing/docs,
MCP assertions, and existing fixtures/specs with their current owners. No
provider parsing belongs in CLI/renderers and MCP gets no duplicate model.
Routine gates gain deterministic fixtures, not live FDA assertions. Existing
source-size ceilings and CLI 700-line caps may not rise.

On the post-1151 base, `src/entities/drug/get.rs` must remain at or below 1,094
lines, `src/sources/mod.rs` at or below 2,274 lines,
`src/cli/system/dispatch.rs` at or below 700 lines, and
`src/render/markdown/related.rs` exactly 1,123 lines. Keep
`src/sources/fda_orphan.rs` below 1,000 lines and the regulatory renderer at or
below 700 lines.

Balance that one new packaged path by folding the two declarations from the
one-line `src/sources/openfda/tests/mod.rs` index into its existing parent test
module and deleting only that index; its two test owners and behavior remain.
No other deletion, filler, or package exclusion is authorized. `cargo package
--list --allow-dirty --locked` remains exactly 1,300 files. Run focused tests,
then `make lint`, `make test`, `make spec`, `make full-feature-check`, exact
package inventory, and `git diff --check`.

Ticket 1151 is a dependency. Implement from its accepted branch after it lands
and preserve `DrugCommandDiscovery`, `_meta.next_commands`, ordering and cap,
typed schemas, and MCP inventory byte-for-byte. This work does not add a
requestable section, change region parsing/`all`, alter Drugs@FDA/label
semantics, offer clinical advice, or add non-FDA orphan programs.

The external-project coupling check must remain green:

`uv run --no-project python tools/check-zero-coupling.py --root .`

## Review

The previous design review accepted an underspecified cache and removed the
1151 dependency. The 2026-09-09 freshness review rejected that state. This
revision restores the dependency and freezes the shared command owner;
replaces raw-response caching with the exact managed normalized-success cache
contract; places every cache/provider/finalization action under one eight-
second deadline; defines fail-closed publication, corruption, and timestamp
behavior; restricts aliases to typed selected-identity anchors; requires
deterministic fixtures only; and records exact post-1151 source/package/
coupling ratchets. Independent re-review is required before implementation.
