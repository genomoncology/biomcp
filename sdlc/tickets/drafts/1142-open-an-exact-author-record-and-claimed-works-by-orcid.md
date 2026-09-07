---
flow: build
priority: 6
deps: [1143]
---

# Open an exact author record and claimed works by ORCID

## Goal and boundary decision

`biomcp get author orcid:<id>` opens one public ORCID person record, and
`biomcp author papers orcid:<id>` returns a bounded local page of the public
works claimed on that record. Both remain exact **ORCID provider records**.
BioMCP does not infer that an ORCID and a Semantic Scholar author ID identify
the same person.

The user explicitly approved reversing the network boundary in record 0581.
When this ticket closes, its durable record must say exactly that it supersedes
record 0581's “no ORCID API calls; citation-supplied evidence only” prohibition.
It does not reinstate the deleted orphan client or supersede record 0581's
dead-code cleanup policy. This is a newly reviewed client with bounded public
surfaces. Semantic Scholar `externalIds.ORCID` remains untrusted and excluded;
all existing Semantic Scholar search/detail/byline warnings
`orcid_link_not_established` and all non-linkage behavior remain byte-for-byte
unchanged.

Current `origin/main` at `e95bb7a4` rejects every `orcid:` author ID and has no
ORCID source module or source-inventory row. The ORCID v3 Public API exposes
the public person and works resources needed here. Anonymous responses are not
the supported BioMCP contract: callers supply a pre-issued public-read bearer
token.

## Exact identity grammar

Extend `ProviderAuthorId` with `AuthorIdProvider::Orcid`. The only accepted
spelling is the 25-byte ASCII string
`orcid:dddd-dddd-dddd-dddC`: lowercase prefix, four hyphenated groups, fifteen
decimal payload digits, and a final decimal digit or uppercase `X`. Do not trim,
case-fold, accept an `https://orcid.org/` URL, accept bare IDs, or accept Unicode
lookalikes.

Validate the ISO/IEC 7064 MOD 11-2 checksum over the first 15 digits:

```text
total = 0
for digit in first_15: total = (total + digit) * 2
check = (12 - (total mod 11)) mod 11
render 10 as X, otherwise render the decimal digit
```

`orcid:0000-0002-1825-0097` and `orcid:0000-0002-1694-233X` are positive
controls. Wrong checksum, lowercase `x`, wrong hyphen positions/count, leading
or trailing whitespace, missing prefix, uppercase prefix, extra suffix, path,
query, control, overlong, and non-ASCII inputs return one static
`BioMcpError::InvalidArgument` with this exact inner message:

```text
author ID must use the exact form orcid:dddd-dddd-dddd-dddC with a valid ISO/IEC 7064 MOD 11-2 checksum.
```

Its public code is `invalid_argument`, human diagnostic is `Invalid argument: `
plus that message, source/recovery are absent, and its CLI exit is 2,
before client construction, cache access, semaphore/rate admission, or network
work. The diagnostic never echoes the rejected input. Existing exact
`semanticscholar:<nonempty ASCII decimal>` parsing and serialization are
unchanged.

## ORCID transport contract

Add one source client with production base `https://pub.orcid.org/v3.0` and a
test-only `BIOMCP_ORCID_BASE` loopback override. Its only feature requests are:

```text
GET /v3.0/<canonical-id-without-prefix>/person
GET /v3.0/<canonical-id-without-prefix>/works
Accept: application/vnd.orcid+json
Authorization: Bearer <ORCID_ACCESS_TOKEN>
```

The base includes `/v3.0`; request-plan paths are respectively
`<id>/person` and `<id>/works`, so the resulting paths above contain no double
slash. The already validated ID is safe path data and is never accepted from a
provider link. Accept a successful response only when its media type, ignoring
case and parameters, is `application/vnd.orcid+json` or `application/json`.
Never follow a provider-returned URL or an HTTP redirect.

`ORCID_ACCESS_TOKEN` is the sole auth environment variable. Trim outer ASCII
space only; require 1-4096 visible ASCII bytes and reject control/non-ASCII
bytes without displaying the value. The caller obtains a bearer token with
ORCID's `/read-public` scope outside BioMCP. BioMCP does not accept client ID or
secret variables, perform OAuth/token refresh, persist the token, or log it.
Missing/ASCII-space-only token is `api_key_required`; invalid nonblank token is
the new `BioMcpError::ApiCredentialInvalid { api, env_var }`; 401/403 is
`api_key_rejected`.

Freeze these credential projections rather than overloading provider rejection:

| token state | variant/code | exact message | source | recovery | CLI exit/work |
| --- | --- | --- | --- | --- | --- |
| missing or ASCII-space-only | `ApiKeyRequired { api:"ORCID", env_var:"ORCID_ACCESS_TOKEN", docs_url:"https://info.orcid.org/documentation/features/public-api/" }` / `api_key_required` | `API key required: ORCID requires ORCID_ACCESS_TOKEN environment variable.\n\nTo set:\n  export ORCID_ACCESS_TOKEN=your-key\n\nMore info: https://info.orcid.org/documentation/features/public-api/` | absent | absent | 1 / zero |
| nonblank but over 4,096 bytes or containing space/control/non-ASCII after outer-ASCII-space trim | `ApiCredentialInvalid { api:"ORCID", env_var:"ORCID_ACCESS_TOKEN" }` / `api_credential_invalid` | `ORCID credential in ORCID_ACCESS_TOKEN is invalid.` | `ORCID` | `Set ORCID_ACCESS_TOKEN to 1-4096 visible ASCII bytes and retry.` | 1 / zero |
| provider 401/403 after a valid token | existing `ApiKeyRejected { api:"ORCID", env_var:"ORCID_ACCESS_TOKEN", docs_url:<same URL> }` / `api_key_rejected` | existing `ApiKeyRejected` message | absent | absent | 1 / one physical GET, no retry |

`ApiCredentialInvalid`'s human display is its message, one space, then recovery;
the executable prefix is therefore exact
`Error: ORCID credential in ORCID_ACCESS_TOKEN is invalid. Set ORCID_ACCESS_TOKEN to 1-4096 visible ASCII bytes and retry.`.
Offset additions elsewhere in `error.rs` keep its line baseline unchanged.

The pure request plan stores only `auth_mode: bearer_env`, never the token or
an Authorization value; the source attaches the validated token after the URL
policy check when constructing the request. `Debug`, tracing, retry, observer,
and error values therefore cannot format the secret. Fixture assertions inspect
the received header and otherwise redact it.

Authorization may be attached only to the canonical HTTPS ORCID origin or the
exact loopback test origin selected at construction. Extend the provider URL
policy so userinfo, fragments, noncanonical ports/schemes/hosts, non-loopback
overrides, redirect targets, and origin changes fail before a credential can be
sent. Request/debug/error values and fixture failure bodies must never expose
the bearer token, Authorization header, requested URL query, email, or hostile
provider sentinel.

Each feature command has one absolute monotonic 35-second deadline covering
the source-local semaphore, rate admission, all attempts/sleeps, body read,
decode, projection, and rendering. Permit at most two concurrent logical ORCID
feature calls per process and pace every physical attempt at least one second
after the prior ORCID attempt start (the first admitted attempt may start
immediately). Use the shared transient retry policy:
one initial GET plus at most three retries (four physical GETs total), the
existing capped `Retry-After`/15-second sleep budget, and no retry for 400,
401, 403, 404, any other 4xx except 429, content-type, body-limit, decode, or
contract errors. No attempt
may start after the deadline; cancellation drops the request and permit.

Both authenticated feature requests force `CacheMode::NoStore`, including
under `BIOMCP_CACHE_MODE=infinite`; no response, credential, or credentialed
cache key reaches disk. `--no-cache` therefore does not change ORCID request
counts. Person bodies are capped at 512 KiB and works bodies at 8 MiB, checking
both `Content-Length` and streamed bytes before JSON decode. Use the shared
sanitized source-error projection with a new `SourceProvider::ORCID`.

## Person projection and privacy

The `/person` root must be an object whose `path` is exactly
`/<id>/person`. A mismatch, missing/wrong type, malformed JSON, wrong content
type, or over-limit body fails the whole command without partial output. From a
public name object, choose the first nonblank value in this order:
`credit-name.value`; joined nonblank `given-names.value` plus
`family-name.value`; given name alone; family name alone. Each component is
Unicode-trimmed, rejects controls, and is at most 1,024 UTF-8 bytes. No usable
public name is a provider-contract error, not `not_found`. The name object must
have exact `visibility:"PUBLIC"`; missing, null, or non-public names are not
projected.

Successful ORCID detail reuses the provider-neutral `AuthorDetail` envelope
with this exact logical JSON projection (the example values are fixtures):

```json
{
  "identity":{"kind":"exact_provider","id":"orcid:0000-0002-1825-0097"},
  "display_name":"Josiah Carberry",
  "provider_records":[{"id":"orcid:0000-0002-1825-0097","source":"orcid","status":"available"}],
  "affiliations":[],
  "paper_count":null,
  "citation_count":null,
  "h_index":null,
  "conflicts":[],
  "warnings":[],
  "_meta":{
    "source_status":[{"source":"orcid","status":"available"}],
    "evidence_urls":[{"source":"orcid","url":"https://orcid.org/0000-0002-1825-0097"}],
    "next_commands":["biomcp author papers orcid:0000-0002-1825-0097"]
  }
}
```

Do not deserialize into or serialize arbitrary person fields. Biography,
emails, addresses, keywords, researcher URLs, other external/person IDs,
employment, education, peer review, memberships, funding, inferred
demographics, and provider `source` objects are never public. `/person` does
not establish affiliations or work/count metrics, so their empty/null values
above are facts about this projection, not healthy-empty claims about other
ORCID endpoints.

## Claimed-works algorithm

`author papers` makes exactly one logical `/works` request per invocation and
never calls `/person`, `/work/<put-code>`, article detail, Semantic Scholar, or
another provider. Because authenticated ORCID responses are no-store, a later
CLI continuation makes one new `/works` request; within one invocation the
decoded corpus is mapped once and never refetched or prefetched.

Require a root object with exact `path = "/<id>/works"` and a present `group`
array of at most 10,000 entries. Each group must have 1-64 `work-summary`
entries; each summary may have at most 64 external IDs. Group-level
`external-ids` are neither deserialized nor consulted. Only exact string
`visibility:"PUBLIC"` is eligible; missing,
null, or another string is privacy-filtered before reading other fields, while a
wrong-typed visibility is a contract error. A public summary must have an
unsigned nonzero 64-bit `put-code`, a canonical
ASCII-decimal-string `display-index` (`0` or a nonzero digit followed by digits,
with no sign or leading zero) fitting `u64`, and a nonblank main
`title.title.value` of at most 4,096 UTF-8 bytes without controls. A malformed
public summary or structural/bound violation fails the complete response; a
group with no public summary is privacy-filtered out.

Choose one representative per group by greatest numeric `display-index`, then
lowest `put-code`, then original summary order. Use only its main title (do not
concatenate subtitle/translated title), Unicode-trimmed `journal-title.value`,
and a four-digit publication `year.value` in 1000-9999. A present journal is
nonblank, control-free, and at most 1,024 UTF-8 bytes after trimming. Missing
optional journal/year is null; wrong-typed or invalid present data is a contract error.
Its stable provider work ID is
`orcid:<16-digit-hyphenated-id>/work:<put-code>` (for example,
`orcid:0000-0002-1825-0097/work:42`), and its evidence URL is constructed only
as `https://orcid.org/<16-digit-hyphenated-id>/work/<put-code>`.

Read identifiers only from the selected PUBLIC representative's
`external-ids.external-id` array. Each entry must have string
`external-id-type`, string `external-id-value`, and string
`external-id-relationship`; admit only the exact relationship `SELF`. Normalize
a case-insensitive ASCII type to lowercase
`[a-z0-9._-]{1,32}` and Unicode-trim its control-free value to 1-512 UTF-8
bytes. Preserve other valid publication-ID types in `identifiers[]`; never
project provider external-ID URLs. A structurally wrong-typed external ID is a
contract error; a correctly shaped non-`SELF` or value-invalid ID is ignored.
Absent or null `external-ids` means an empty list; if present it must be an
object with an absent/null-or-array `external-id`, subject to the caps above.
Identifiers present only at group level, or only on a private/nonselected
summary, are absent from `identifiers[]`, flattened IDs, evidence, and follow-up
commands and do not participate in cross-group deduplication. A hostile fixture
puts a unique PMID/DOI exclusively in each of those three excluded locations
and freezes their absence from CLI and raw-MCP works JSON/Markdown; typed detail
continues to expose no works collection at all.
Canonicalize recognized types to `doi`, `pmid`, `pmcid`, or `arxiv` and their
values as follows:

- DOI: remove one case-insensitive `doi:` or `https://doi.org/` prefix, trim,
  ASCII-lowercase, and require 3-255 visible bytes containing `/`;
- PMID: require 1-12 ASCII digits, numeric value greater than zero, and render
  without leading zeroes;
- PMCID (`pmc` or `pmcid`): accept case-insensitive `PMC` plus 1-12 digits,
  require a nonzero number, and render uppercase without leading zeroes; and
- arXiv: remove one case-insensitive `arxiv:` prefix and require 1-64 visible
  ASCII `[A-Za-z0-9./-]` bytes.

Deduplicate normalized identifiers by `(type,value)`, preserving first
occurrence. Flatten the lexically first normalized value of each recognized
kind into `doi`, `pmid`, `pmcid`, and `arxiv_id`. Process eligible groups in
provider order. Drop a later group if any of its recognized canonical IDs was
already present in a retained group; otherwise retain it and register all its
recognized IDs. Groups with no recognized ID are never title-deduplicated.

After privacy filtering, mapping, and stable deduplication, slice locally by
the requested offset and limit. `limit` stays 1-100; offset is at most 10,000
and arithmetic is checked. After existing `next`, `pagination` appends skip-when-null
`total: Option<u64>` and `truncated: Option<bool>` fields so existing Semantic
Scholar compact JSON stays byte-identical with both set to null internally.
ORCID sets `offset` to the caller value, `limit` to
the caller value, `total` to the complete retained count, `next` to
`offset + returned` exactly when that value is below total, and
`truncated:false`. Offset at/after total is a successful available empty page.
More than 10,000 groups or any other hard bound is an error, never silent
truncation.

Replace the author-papers result's use of shared `ArticleRelatedPaper` with an
author-owned compact paper type whose first fields, in order, are the existing
`paper_id`, `pmid`, `doi`, `arxiv_id`, `title`, `journal`, and `year`; append
skip-when-absent `work_id`, `pmcid`, and nonempty `identifiers`. This preserves
Semantic Scholar JSON key order and omissions without changing the shared
article type. Each identifier is exactly
`{"type":"<normalized-type>","value":"<normalized-value>"}` in preserved
deduplicated order. `paper_id` is absent for ORCID. These additions are absent
for Semantic Scholar, whose compact JSON and Markdown remain byte-for-byte
unchanged. For each visible ORCID row, emit at most one article follow-up using
PMID, then PMCID, DOI, then arXiv priority and existing `NextCommand` shell
quoting. A work lacking those IDs gets no invented article command. Row evidence
URLs follow row order; the continuation follows all row commands.

## Status and failure truth table

| condition | public outcome | requests |
| --- | --- | --- |
| invalid ORCID grammar/checksum or bad limit/offset | static `invalid_argument`, exit 2; no card | 0 |
| missing/ASCII-space-only token | exact `api_key_required` projection above, exit 1 | 0 |
| invalid nonblank token | exact `api_credential_invalid` projection above, exit 1 | 0 |
| valid 200 person with matching path/name | available ORCID detail; nullable fields exactly as above | 1 logical |
| valid 200 works, zero groups or offset beyond total | available page, `papers:[]`, exact total, `next:null`, `truncated:false` | 1 logical |
| valid 200 works with rows | available page and exact local continuation | 1 logical |
| 404 | `not_found` for the validated requested ORCID; no success/null card | 1 logical, no retry |
| 401/403 | sanitized `api_key_rejected`; no body/token | 1 logical, no retry |
| 429/5xx/transport | sanitized ORCID unavailable/retry error | 1 logical, at most 4 physical |
| timeout, body/content-type/JSON/path/contract/bound failure | sanitized failure; no rows, metadata, or continuation leak | 1 logical; contract failures are not retried |

Successful `_meta.source_status` contains exactly one available ORCID row;
command failures return the existing structured CLI/MCP error envelopes rather
than an unavailable/empty author card. No ORCID failure mutates or falls back to
a Semantic Scholar record.

## CLI, MCP, health, and documentation

The public commands are exactly:

```text
biomcp get author orcid:0000-0002-1825-0097
biomcp --json get author orcid:0000-0002-1825-0097
biomcp author papers orcid:0000-0002-1825-0097 --limit 10 --offset 0
biomcp --json author papers orcid:0000-0002-1825-0097 --limit 10 --offset 0
```

The provider-specific Markdown branches are exactly these templates (`?` lines
are omitted when their value is null/empty). Extend the established
`src/render/human.rs` sanitizer with `sanitize_provider_inline`; do not build a
second author-local escape policy. It performs, in order:

1. NFC normalization with direct dependency `unicode-normalization = "0.1.25"`
   (already locked at that version; the package path count does not change).
2. Replace each U+2028 LINE SEPARATOR or U+2029 PARAGRAPH SEPARATOR with one
   ASCII space. Replace each maximal run in the Unicode 16.0
   `Default_Ignorable_Code_Point` property **or** General_Category `Cf` with one
   visible U+FFFD, using checked-in scalar-range match tables named with that
   Unicode version; this includes bidi controls, soft hyphen, variation/tag
   selectors, joiners, and zero-width spaces.
3. Before a remaining scalar whose Unicode canonical combining class is
   nonzero, insert U+25CC DOTTED CIRCLE when no retained non-space scalar has
   occurred since the start or last whitespace. Combining marks following a
   base remain attached; NFC-composable sequences are already composed.
4. Pass the result through existing `sanitize_inline` for ANSI/C0/C1 handling.
   Then preserve every non-ASCII scalar plus ASCII letters, digits, and spaces,
   while encoding every other ASCII graphic as decimal HTML
   `&#<codepoint>;` with no leading zeroes.

Apply this operation only to provider display name, title, journal, and
displayed identifier type/value. JSON retains the validated, Unicode-trimmed
provider value; trusted canonical IDs and numeric fields remain literal in
Markdown. Exact sanitizer controls are:

```text
provider input:  "e\u{0301}" | "\u{0301}A" | "A\u{2028}B\u{2029}C"
Markdown output: "é" | "◌́A" | "A B C"

provider input:  "A\u{202E}B\u{200B}\u{200D}C 👩\u{200D}🔬 <x&`$()>"
Markdown output: "A�B�C 👩�🔬 &#60;x&#38;&#96;&#36;&#40;&#41;&#62;"
```

The second output's leading mark includes an inserted U+25CC. Fixture assertions compare
UTF-8 bytes for isolated/attached combining marks, consecutive line separators,
every table boundary, bidi isolates/overrides, zero-width/joiner/variation/tag
characters, and NFC composition. The operation is called exactly once at the
Markdown leaf. Provider text is single-line visible Markdown and
never an HTML/link target or shell argument.

```text
# <safe display_name>

Source: ORCID

Identity: exact provider

- ID: `<canonical prefixed ID>`
- Status: available

See also:
  biomcp author papers <canonical prefixed ID>
```

```text
# Papers for `<canonical prefixed ID>`

Source: ORCID

Identity: exact provider

Status: available
Total: <total>; offset: <offset>; returned: <returned>; has more: <true|false>; truncated: false

## Work <1-based visible-row number>

- Title: <safe title>
- Work ID: `<work_id>`
- Journal: <safe journal>                                      ?
- Year: <year>                                                 ?
- PMID: `<pmid>`                                               ?
- PMCID: `<pmcid>`                                             ?
- DOI: `<doi>`                                                 ?
- arXiv ID: `<arxiv_id>`                                       ?
- Identifier: <safe type>:<safe value>                         ? (one per identifier)

See also:                                                     ?
  <row article command in row order>                          ?
  biomcp author papers <ID> --limit <limit> --offset <next>    ?
```

There is one blank line exactly where shown, a final newline, no row block on an
empty page, no `See also:` block when neither a row command nor continuation
exists, and no prose descriptions after ORCID commands. The matching ORCID works JSON is the existing
`AuthorPapersResult` envelope with `author`, `papers`, `pagination`, and `_meta`;
goldens freeze every key/order/null omission, including the identifier object
above, `total`, `truncated`, row evidence URLs, row commands, then continuation.
One representative nonterminal fixture is exactly:

```json
{"author":{"kind":"exact_provider","id":"orcid:0000-0002-1825-0097"},"papers":[{"pmid":"123","doi":"10.1/example","title":"A claimed work","journal":"A Journal","year":2024,"work_id":"orcid:0000-0002-1825-0097/work:42","pmcid":"PMC456","identifiers":[{"type":"pmid","value":"123"},{"type":"doi","value":"10.1/example"},{"type":"pmcid","value":"PMC456"}]}],"pagination":{"offset":0,"limit":1,"next":1,"total":2,"truncated":false},"_meta":{"source_status":[{"source":"orcid","status":"available"}],"evidence_urls":[{"source":"orcid","url":"https://orcid.org/0000-0002-1825-0097/work/42"}],"next_commands":["biomcp get article 123","biomcp author papers orcid:0000-0002-1825-0097 --limit 1 --offset 1"]}}
```

Complete golden Markdown and JSON fixtures—not substring-only tests—freeze
detail plus first, middle, terminal, and empty pages. JSON preserves hostile
normalized provider strings; Markdown neutralizes pipes, angle brackets,
backticks, newlines, quotes, backslashes, `$()`, semicolons, and ampersands. No
provider value becomes a heading, HTML/link target, or command argument.

Raw MCP uses the same CLI paths, for example
`{"command":"biomcp author papers orcid:0000-0002-1825-0097 --limit 2 --offset 0","json":true}`.
Its parsed JSON/Markdown, request log, errors, and continuation bytes equal
direct CLI. Typed MCP supports detail through the existing request
`{"entity":"author","id":"orcid:0000-0002-1825-0097","json":true}`;
post-mapping argv is
`[biomcp,get,author,orcid:0000-0002-1825-0097,--json]` and the runtime ID is
unchanged. There is no typed author-papers operation.

Invalid-ID and invalid-nonblank-token surfaces freeze the same projections:

| surface | exact disposition |
| --- | --- |
| human CLI | stderr is `Error: ` plus the exact human diagnostic above and one newline; exit 2 for ID, 1 for credential; stdout empty |
| JSON CLI detail | stdout is the exact logical object below plus final newline; stderr empty; exit 2 or 1 |
| JSON CLI papers | same object plus the existing top-level `"papers":[]`; no other partial field; exit 2 or 1 |
| raw MCP and typed `get`, default Markdown mode | `isError:true`, one text content equal to the exact human CLI line without its final newline; no process exit |
| raw MCP and typed `get`, `json:true` | preserve the existing generic CLI/MCP wrapper: `isError:false`, one text content byte-equal to the JSON CLI object without its final newline; no process exit |

```json
{"error":{"code":"invalid_argument","message":"Invalid argument: author ID must use the exact form orcid:dddd-dddd-dddd-dddC with a valid ISO/IEC 7064 MOD 11-2 checksum."},"_meta":{"not_found":false}}
{"error":{"code":"api_credential_invalid","message":"ORCID credential in ORCID_ACCESS_TOKEN is invalid.","source":"ORCID","recovery":"Set ORCID_ACCESS_TOKEN to 1-4096 visible ASCII bytes and retry."},"_meta":{"not_found":false}}
```

These are logical one-line fixtures; CLI pretty-print whitespace/key order is
separately frozen byte-for-byte. Missing and blank-token MCP text is the existing
inline sanitization of its exact multiline diagnostic:
`Error: API key required: ORCID requires ORCID_ACCESS_TOKEN environment variable. To set:   export ORCID_ACCESS_TOKEN=your-key More info: https://info.orcid.org/documentation/features/public-api/`.
No diagnostic contains the invalid ID, token, token length, or provider body.
Tests exercise invalid ID and each credential state through both feature
commands, human/JSON CLI, raw MCP, and typed detail, then make a healthy MCP call
to prove the session remains usable.

Ticket 1143 lands first and retains `--full` only for Semantic Scholar's rich
author-papers endpoint. `author papers orcid:<id> --full` returns this exact
static `invalid_argument` guidance:

```text
--full is available only for semanticscholar: author IDs; omit --full for ORCID claimed works
```

It performs zero ORCID/Semantic Scholar requests. Every accepted 1143 compact/rich S2 byte,
one-request/deadline rule, and typed-tool exclusion remains unchanged. Ticket
1145's graph/JATS behavior is independent; it is only a transitive prerequisite
through 1143, and ORCID makes zero graph, JATS, citation, or full-text requests.

Do not add a tool or change any MCP JSON Schema, tool/catalog description,
seven-tool inventory, catalog byte/token ceiling, or raw authorization policy.
Final schema/catalog snapshots and a base-to-HEAD byte comparison of the MCP
catalog/schema owners prove this.

Add an `ORCID` authenticated API health row affecting exact ORCID author detail
and works. It performs one non-retried GET of
`https://pub.orcid.org/v3.0/0000-0002-1825-0097/person`, with the same Accept
and bearer headers, under the existing 12-second per-probe deadline and
16-probe global concurrency. Missing token yields exact `excluded (set
ORCID_ACCESS_TOKEN)`, `required_env_var:"ORCID_ACCESS_TOKEN"`,
`status:"excluded"`, `latency:"n/a"`, `affects:"get author and author papers for
ORCID IDs"`, `key_configured:false`, and zero GETs; ASCII-space-only is identical.
The selected row's exact logical JSON is
`{"api":"ORCID","status":"excluded","latency":"n/a","affects":"get author and author papers for ORCID IDs","key_configured":false,"required_env_var":"ORCID_ACCESS_TOKEN"}`.
The health probe uses the same token validator. Invalid nonblank input yields
exact Markdown `error (key configured)` and the JSON row
`{"api":"ORCID","status":"error","latency":"n/a","affects":"get author and author papers for ORCID IDs","key_configured":true,"required_env_var":"ORCID_ACCESS_TOKEN"}`
with zero GETs. Health rows have no error variant/code/source/recovery fields;
report-only exits 0, while `--fail-on-error` exits 1 for this invalid row (the
missing/blank excluded row is not an error). Configured 2xx is
`ok (key configured)`; 401/403/timeout/other failure is an error row without
token/body leakage.
`--apis-only` includes the row and `--api ORCID` selects it.

Update `biomcp get author --help`, `biomcp author papers --help`,
`biomcp list author`, `docs/user-guide/author.md`, the CLI reference,
configuration/API-key guide, data-source matrix, MCP server key list, source
index, `docs/reference/sources.json`, and generated source-licensing reference.
The ORCID inventory row is direct API, required env, and names only these two
surfaces; it must document Public API access/terms and public-data reuse without
claiming that all ORCID record data or downstream works are identically
licensed. Replace the obsolete test that forbids all direct ORCID markers with
positive inventory/module/surface/auth assertions while retaining its PubMed
citation-ORCID assertion.

## Acceptance and ownership

Test first at the owning layers:

1. Exhaustive parser/checksum tables cover canonical numeric/X examples and
   every malformed category above through entity, CLI, raw MCP, and typed get;
   counting fixtures prove zero client/cache/network work and fixed no-echo
   errors for hostile invalid IDs.
2. Pure source-plan and URL-policy tests pin exact methods, paths, headers,
   token-origin restriction, NoStore mode, body limits, and content types.
   Paused-time/adversarial tests prove the 35-second deadline, two-call
   concurrency, one-second physical admission, cancellation, retry classes,
   four-attempt cap, and no permit/task leak.
3. Person wire tables cover requested-path match, every name precedence/null
   combination, byte/control boundaries, all status/error rows, privacy
   allowlisting, and exact public JSON/Markdown. Shared human-renderer tables
   freeze the Unicode-16 predicate boundaries and every sanitizer byte example
   above without changing JSON values.
4. Works tables cover 0/1/64/65 summaries and selected-summary external IDs,
   ignored hostile group/private/nonselected IDs, 10,000/10,001
   groups, representative ties, private filtering, identifier normalization and
   conflicts, stable dedupe, groups without IDs, first/middle/terminal/empty
   slices, offset/limit arithmetic, article-command priority/quoting, and one
   `/works` request with no per-work/person/page prefetch.
5. Execute complete CLI Markdown/JSON, raw MCP Markdown/JSON, and typed-detail
   Markdown/JSON against the strict fixture. Assert headers and exact request
   counts; parse every emitted command with the real CLI; preserve hostile
   strings safely; prove malformed/provider failures expose no partial page,
   credentials, emails, private fields, URLs, or fixture sentinels.
6. Re-run all Semantic Scholar author search/detail/compact/rich and accepted
   ticket-1143 contracts, including untrusted ORCID warnings and `--full` mode;
   run ticket-1145 graph/JATS regression tests and assert the ORCID log has no
   such routes. Run final typed schema/tool inventory snapshots unchanged.
7. Extend `spec/entity/author.md` and `spec/surface/mcp.md` using the existing
   prepared author fixture, plus focused Rust/Python help, health, docs,
   licensing, provider-network, source-registry, and package tests. Then run
   `make lint`, `make test`, and `make spec`, `git diff --check`, and the locked
   offline package list at exactly 1,300 paths. AlphaGenome is untouched, so no
   full-feature gate is required.

`src/sources/orcid.rs` owns request plans, wire structs, response validation,
and the bounded transport. `src/entities/author/mod.rs` owns qualified identity
and detail projection; `src/entities/author/papers.rs` owns group mapping,
dedupe, paging, metadata, and follow-ups. Existing CLI and Markdown author
modules only select/render; health catalog/runner, rate limit, provider URL
policy, `SourceProvider`, inventory/docs, and existing fixture/test sidecars
receive their narrow additions. Do not put ORCID behavior in Semantic Scholar,
MCP mapping, or fixture-only branches.

The package has exactly 1,300 paths at the design base. Add
`src/sources/orcid.rs`, merge the directly owned 174-line
`src/entities/author/detail.rs` implementation/tests into the 251-line
`src/entities/author/mod.rs`, and delete `detail.rs`: one real owner replaces
one real owner, so the package remains exactly 1,300 with no filler or Cargo
exclusion change. Add the already locked `unicode-normalization = "0.1.25"` as a
direct dependency without changing its lock resolution or package path count.
Keep the merged author module and new source below 700 lines;
`src/cli/commands.rs` (686), CLI author/list modules, health modules,
`src/sources/rate_limit.rs` (605), and `src/sources/provider_url_policy.rs`
(945) remain below 700/1,000 as applicable. Keep `src/sources/mod.rs` exactly
1,884 lines and `src/error.rs` at or below its 1,125-line over-cap baseline by
offsetting their registry additions locally; do not raise an allowance.
`src/mcp/shell.rs` stays byte-identical at 2,136 lines. Net production `src/`
growth must stay at or below 800 lines and all existing quality/source/CLI
ratchets remain enforced.

## Exclusions

No ORCID name search, cross-provider resolution/merge, trust in Semantic
Scholar ORCID strings, member/private API, non-public fields, OAuth issuance,
client-secret handling, write/update operation, webhook, peer-review/funding
surface, whole-corpus export/local index, per-work detail fetch, paper metadata
enrichment, citation/full-text/JATS traversal, or claim that ORCID authorship is
independent proof of identity. The output reports a public claim from one exact
provider and nothing more.
