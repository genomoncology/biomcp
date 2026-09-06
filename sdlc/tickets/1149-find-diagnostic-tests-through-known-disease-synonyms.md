---
flow: build
priority: 6
deps: []
---

# Find diagnostic tests through known disease synonyms

## Goal and authoritative resolver

`search diagnostic --disease "Bachmann-Bupp syndrome" --source gtr` must find
`GTR000596648.2` when GTR stores the longer equivalent condition name. Expand
only through exact names and synonyms of one authoritative MyDisease identity;
never use parents, descendants, fuzzy scoring, substring candidate selection,
or the discover fallback.

Add one shared owner in `entities::disease::resolution`,
`resolve_exact_disease_terms`, used by diagnostic search rather than copying
MyDisease parsing or disease ranking into `entities::diagnostic`. It accepts
the already validated disease filter and returns requested term, optional
canonical ID/name, bounded exact synonyms, and one public resolution outcome.

Normalize only for equality/deduplication: Rust `str::trim`, ASCII lowercase,
and collapse each Unicode-whitespace run to one ASCII space. Preserve all
punctuation and non-ASCII code points; never apply the disease searcher's
`carcinoma` rewrite. Provider requests retain the original trimmed bytes and
the existing Lucene escaping. Reject a disease filter over 512 bytes, fewer
than three alphanumeric characters, or containing controls before resolver or
local-source work. The entity messages are exactly `--disease must be at most
512 UTF-8 bytes`, the existing `--disease must contain at least 3 alphanumeric
characters for diagnostic disease matching`, and `--disease must not contain
control characters`.

The resolver has one five-second total deadline, at most two logical MyDisease
requests, the existing shared HTTP cache policy, and an 8 MiB body cap per
response:

- A canonical MONDO/DOID ID performs exactly one existing detail GET.
- MESH/OMIM/ICD10CM performs one existing xref query with `size=5`, then one
  detail GET only when exactly one distinct canonical identity remains. The
  selected detail must itself contain the requested kind/value in its
  kind-specific fields: MESH in `mondo.xrefs.mesh`,
  `disease_ontology.xrefs.mesh`, or `umls.mesh`; OMIM in
  `mondo.xrefs.omim` or `disease_ontology.xrefs.omim`; ICD10CM in
  `mondo.xrefs.icd10`, `disease_ontology.xrefs.icd10`, or `umls.icd10am`.
  Compare trimmed ASCII-case-insensitively after stripping at most one matching
  `MESH:`, `OMIM:`, `ICD10:`, or `ICD10CM:` prefix from both requested and
  provider values. An irrelevant kind, missing value, or conflicting value is
  an inconsistent `unavailable` result; never accept the query hit alone or
  spend a third request searching for a replacement.
- Free text performs one existing name/synonym query with `size=50`, `from=0`,
  and `MYDISEASE_SEARCH_FIELDS`. Locally retain only hits where the normalized
  query equals the primary MONDO/DO name or one declared MONDO/DO exact
  synonym. Zero retained identities is `absent`; more than one is `ambiguous`;
  exactly one receives one detail GET with `MYDISEASE_GET_FIELDS`.

If a query reports `total > returned hits`, do not call absence authoritative:
classify it unavailable/incomplete. A detail 404 after a selected search hit,
ID disagreement, invalid JSON/type shape, missing/blank ID or canonical name,
oversize body, transport/HTTP failure, or deadline expiry is `unavailable`.
For the detail record, accept synonym as a string or array of strings only;
any other non-null shape is malformed. Reject controls and strings over 256
bytes, deduplicate by the equality normalization, omit the requested and
canonical terms, preserve MONDO then DO provider order, and retain at most 20
synonyms. Truncation at 20 is successful and exposed as `synonyms_truncated`.

## Applicability and exact match/rank/page algorithm

MyDisease expansion applies only when GTR is selected (`gtr` or `all`) and a
disease filter exists. WHO IVD's `Pathogen/Disease/Marker` is not an ontology
disease field and continues to use only the literal requested phrase. A
WHO-only disease search performs zero MyDisease requests and reports resolution
`inapplicable`. Existing `--source who-ivd --gene` rejection remains unchanged;
with `source=all` plus a gene, WHO remains inapplicable and only GTR is loaded.

Construct the GTR term list in this order: requested, distinct canonical name,
then distinct synonyms. Match each term with the existing Unicode-safe
alphanumeric phrase-boundary predicate against every condition. WHO uses that
same predicate with requested term only. A row records the first matching term:

```
"disease_match": {
  "kind": "requested|canonical|synonym",
  "term": "Bachmann-Bupp syndrome",
  "resolved_id": "MONDO:0033642"
}
```

Whenever resolution is `resolved`, `resolved_id` is that ID for **every** match
kind, including a requested-term match. It is null for requested matches under
absent, ambiguous, unavailable, or WHO-only inapplicable resolution;
canonical/synonym kinds cannot occur in those states. Omit `disease_match` only
when no disease filter exists. Provider condition strings remain unchanged.

Within disease-filtered results, rank requested before canonical before
synonym; synonyms tie by their resolver order. Then retain the current
case-insensitive trimmed result-name order, accession order, and finally source
key. Without a disease filter, ordering is byte-for-byte unchanged. Deduplicate
after matching/ranking and before slicing by `(source, accession)`, each trimmed
and ASCII-case-folded; best-ranked first wins, while equal accessions from GTR
and WHO remain distinct.

The disease predicate is one OR over its applicable terms. It remains ANDed
with every requested gene, exact test type, manufacturer/lab, and source
predicate; expansion cannot resurrect a row rejected by another filter. Load
all selected bounded local rows, match, rank, and deduplicate before applying
`[offset, min(offset + limit, len))` with checked arithmetic. When all required
inputs are complete, `total` is the exact deduplicated count even for
`source=all`, and `has_more` is `offset + returned < total`. An unavailable
selected local source or unavailable disease resolution makes `total: null`
and `has_more: true`; retained rows still return in deterministic order. Empty
and ambiguous resolution are complete literal-only searches and may produce a
confirmed zero.

## Exact provenance and failure schema

Add these exact objects under JSON `_meta`; CLI JSON, raw MCP JSON, and typed
MCP JSON use the same serializer:

```
"disease_resolution": {
  "outcome": "resolved|absent|ambiguous|unavailable|inapplicable",
  "source": "MyDisease.info" | null,
  "query": "Bachmann-Bupp syndrome",
  "resolved_id": "MONDO:0033642" | null,
  "canonical_name": "..." | null,
  "synonyms_returned": 0,
  "synonyms_truncated": false,
  "message": "..." | null
},
"source_status": [{
  "source": "gtr|who-ivd",
  "outcome": "data|empty|unavailable|inapplicable",
  "message": null
}]
```

Omit `disease_resolution` when no disease filter exists. Otherwise serialize
every key shown above; no key uses omission. The exact outcome table is:

| outcome | `source` | `resolved_id` / `canonical_name` | `synonyms_returned` / `synonyms_truncated` | `message` |
| --- | --- | --- | --- | --- |
| `resolved` | `"MyDisease.info"` | nonblank strings | exact retained count `0..20` / whether additional valid unique synonyms existed | null |
| `absent` | `"MyDisease.info"` | null / null | `0` / `false` | `No exact MyDisease identity matched; the literal disease term was searched.` |
| `ambiguous` | `"MyDisease.info"` | null / null | `0` / `false` | `Multiple exact MyDisease identities matched; synonyms were not expanded.` |
| `unavailable` | null | null / null | `0` / `false` | `Disease synonym resolution is temporarily unavailable; the literal disease term was searched.` |
| `inapplicable` | null | null / null | `0` / `false` | `WHO IVD uses the literal disease term; ontology synonyms are not applied.` |

`query` is always the caller's trimmed, otherwise unchanged disease string.
Malformed detail, requested-ID mismatch, requested-xref mismatch, incomplete
query page, HTTP/transport/body-limit error, and deadline all use the one
unavailable row without leaking their internal cause. Do not expose provider
text, URLs, paths, or credentials.

Emit one source-status row for each selected source in `gtr`, then `who-ivd`
order. A source that loaded and retained rows is data; loaded with none is
empty; a load/sync/index failure is unavailable with exactly `GTR diagnostic
data is temporarily unavailable.` or `WHO IVD diagnostic data is temporarily
unavailable.`; WHO suppressed by a gene is inapplicable with `WHO IVD does not
support gene filtering.` Data/empty messages are null. Unavailable sources
contribute no provenance. For `source=all`,
one unavailable source no longer discards rows from the other; both unavailable
returns an in-band incomplete empty page, never a confirmed zero. Explicit
single-source failure uses the same in-band state. Input errors still fail
before any request.

Markdown adds a `Disease match` column using `Requested:`, `Canonical:`, or
`Synonym:` plus escaped term, followed by bounded resolution/source notes.
Only a complete `total: 0` prints `No diagnostic tests found.` and filter
recovery. Incomplete zero prints `Diagnostic search is incomplete.` and the
fixed unavailable notes. Existing row `source`, JSON field nullability, compact
gene/condition arrays, `_meta.next_commands`, and exact first-row command
behavior otherwise remain.

## Typed, raw, and executable acceptance

Add `diagnostic` to the existing typed MCP `search` union with fields `gene`,
`disease`, `test_type`, `manufacturer` (strings), `source` enum
`gtr|who-ivd|all`, `full` boolean, and existing typed `limit` 1-25, `offset`
0-1000, and `json`. Require at least one of the four filters; map `test_type`
to `--type`. The diagnostic `disease` schema is `type:string`, `minLength:1`,
`maxLength:512`; JSON Schema length is Unicode scalar count, so the mapper also
trims and enforces **UTF-8 byte length** `1..=512` before constructing CLI args.
It returns typed invalid-params `disease must contain 1-512 UTF-8 bytes after
trimming` outside that byte range. CLI/entity validation independently uses
the same byte boundary. Controls and the fewer-than-three-alphanumeric case
reach entity validation and use the exact entity messages above. All these
failures occur before MyDisease, GTR readiness, WHO readiness, cache, or
rendering.
Other typed diagnostic text fields retain the shared 1-256-character schema
and runtime rule. This changes only that schema branch: tool count, other
branches, raw allowlist, CLI arguments, and typed execution owner remain
unchanged.

Extend the existing GTR/WHO/MyDisease fixture family and diagnostic spec.
Through executable CLI Markdown/JSON, raw MCP `biomcp` Markdown/JSON, and typed
MCP `search` Markdown/JSON, pin Bachmann-Bupp by requested and long synonym,
canonical-name and synonym-only rows, precedence/ties/dedupe, every source
mode, conjunctive filters, offsets before/at/after the end, exact totals and
`has_more`, resolution absent/ambiguous/unavailable/malformed/timeout, each
local-source failure combination, and 0/1/2 resolver request paths. Crosswalk
fixtures include the requested value in the correct detail field, only under
an irrelevant xref kind, absent, and a conflicting same-kind value; the last
three produce unavailable after exactly two requests. Typed/CLI boundary
fixtures accept 512 ASCII bytes and a multibyte string of exactly 512 bytes,
reject 513 ASCII bytes and the next multibyte scalar, and prove every rejection
logs zero resolver/local-source requests. MCP bodies are byte-equal to their
CLI format after the existing MCP footer rules.

Fixture logs independently assert request path, fields, size, cache reuse,
deadline and request/body/synonym caps, zero resolver calls for WHO-only or no
disease, and zero extra calls from rendering. Parse emitted detail commands
with the real CLI parser. Provider names, synonyms, conditions, accessions, and
manufacturer values containing pipe, brackets, parentheses, Markdown/HTML,
quotes, backslash, dollar, backtick, semicolon, ampersand, CR/LF, and terminal
controls must not create a table cell, link, heading, HTML node, terminal
escape, extra argument, command, or provider request. Invalid controls in
resolver identity data produce the specified unavailable state; hostile local
row text is safely rendered and round-trips unchanged in JSON.

## Ownership, ratchets, dependency, and gates

Keep the resolver/result owner in existing `entities/disease/resolution.rs`,
matching/paging and its tests in existing `entities/diagnostic/search.rs` and
`mod.rs`, projection in existing diagnostic CLI/Markdown owners, and typed
mapping/tests in existing MCP owners. Extend existing fixtures/docs/specs; add
no parallel parser, renderer, fixture family, dependency, or file. Absorb added
lines through cohesive consolidation in the same touched owners: no existing
source-size baseline or CLI 700-line cap may increase. Package inventory stays
exactly 1,300 without filler, exclusion changes, or unrelated deletion.

Run focused disease-resolution, diagnostic entity/render/CLI/MCP, fixture, and
mustmatch tests, then `make lint`, `make test`, `make spec`,
`make full-feature-check`, exact package inventory, and `git diff --check`.

Ticket 1148 ranks gene search and shares no code or public field with this
work. It is a scheduling preference, not a dependency; neither ticket may
absorb or wait on the other. This ticket adds no ontology traversal, fuzzy
matching, diagnostic interpretation, or new GTR/WHO ingestion.

## Review

Accepted after independent design review. The reviewer confirmed exact
kind-specific xref proof within the two-request bound, the exhaustive public
resolution/nullability table, typed 512-byte runtime boundaries with zero-work
rejection, and all previously accepted ranking, source, surface, ownership,
package, and dependency contracts.
